import { describe, it, expect, vi } from 'vitest';
import type { IncomingMessage } from 'http';
import { parseTrustProxy, resolveClientIp, type TrustProxySetting } from './trusted-proxy';

type FakeRequest = Pick<IncomingMessage, 'headers' | 'socket'>;

function request(
   remoteAddress: string,
   forwardedFor?: string | string[],
): FakeRequest {
   const headers: Record<string, string | string[] | undefined> = {};
   if (forwardedFor !== undefined) {
      headers['x-forwarded-for'] = forwardedFor;
   }
   return {
      headers,
      socket: { remoteAddress },
   } as unknown as FakeRequest;
}

describe('parseTrustProxy', () => {
   it('trusts nobody by default', () => {
      expect(parseTrustProxy(undefined)).toBe(false);
      expect(parseTrustProxy('')).toBe(false);
      expect(parseTrustProxy('   ')).toBe(false);
      expect(parseTrustProxy('0')).toBe(false);
      expect(parseTrustProxy('false')).toBe(false);
   });

   it('reads a hop count', () => {
      expect(parseTrustProxy('1')).toBe(1);
      expect(parseTrustProxy('2')).toBe(2);
   });

   it('reads addresses, CIDRs and named ranges', () => {
      expect(parseTrustProxy('10.0.0.0/8,::1')).toEqual(['10.0.0.0/8', '::1']);
      expect(parseTrustProxy('loopback')).toEqual(['loopback']);
      expect(parseTrustProxy('203.0.113.7')).toEqual(['203.0.113.7']);
   });

   it('fails closed on a malformed setting instead of trusting headers', () => {
      const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);

      expect(parseTrustProxy('yes-please')).toBe(false);
      expect(parseTrustProxy('10.0.0.0/8,not-an-ip')).toBe(false);
      expect(warn).toHaveBeenCalled();

      warn.mockRestore();
   });
});

describe('resolveClientIp', () => {
   it('ignores a spoofed x-forwarded-for when no proxy is configured', () => {
      // The reported vulnerability: the header used to win unconditionally.
      const req = request('203.0.113.9', '1.2.3.4');

      expect(resolveClientIp(req, false)).toBe('203.0.113.9');
      expect(resolveClientIp(req, false)).not.toBe('1.2.3.4');
   });

   it('ignores the header even when a client sends several entries', () => {
      const req = request('203.0.113.9', ['1.2.3.4', '5.6.7.8']);

      expect(resolveClientIp(req, false)).toBe('203.0.113.9');
   });

   it('returns the socket address when the header is absent', () => {
      expect(resolveClientIp(request('198.51.100.4'), 1)).toBe('198.51.100.4');
   });

   it('skips the configured number of hops', () => {
      // One proxy: the socket is the proxy, the rightmost entry is the client.
      expect(resolveClientIp(request('10.0.0.1', '203.0.113.9'), 1)).toBe(
         '203.0.113.9',
      );
      // Two proxies: only the second entry from the right is reached.
      expect(
         resolveClientIp(request('10.0.0.1', '198.51.100.4, 203.0.113.9'), 2),
      ).toBe('198.51.100.4');
      // A spoofed value appended by the attacker stays to the left, out of reach.
      expect(
         resolveClientIp(request('10.0.0.1', '1.2.3.4, 203.0.113.9'), 1),
      ).toBe('203.0.113.9');
   });

   it('stops at the nearest untrusted address for CIDR lists', () => {
      const setting: TrustProxySetting = ['10.0.0.0/8'];

      expect(resolveClientIp(request('10.0.0.5', '203.0.113.9'), setting)).toBe(
         '203.0.113.9',
      );
      // Two trusted hops, then the client.
      expect(
         resolveClientIp(request('10.0.0.5', '198.51.100.4, 10.1.2.3'), setting),
      ).toBe('198.51.100.4');
      // Socket outside the trusted range is itself the client.
      expect(resolveClientIp(request('203.0.113.9', '1.2.3.4'), setting)).toBe(
         '203.0.113.9',
      );
   });

   it('supports named ranges and IPv6 CIDRs', () => {
      expect(resolveClientIp(request('127.0.0.1', '203.0.113.9'), ['loopback'])).toBe(
         '203.0.113.9',
      );
      expect(
         resolveClientIp(request('fd00::1', '2001:db8::5'), ['fc00::/7']),
      ).toBe('2001:db8::5');
      expect(
         resolveClientIp(request('::ffff:10.0.0.1', '203.0.113.9'), ['10.0.0.0/8']),
      ).toBe('203.0.113.9');
   });

   it('reports unknown when the socket has no address', () => {
      const req = { headers: {}, socket: {} } as unknown as FakeRequest;

      expect(resolveClientIp(req, false)).toBe('unknown');
   });
});
