import type { IncomingMessage } from 'http';

/**
 * Client-IP resolution shared by the HTTP and WebSocket paths.
 *
 * The two paths used to disagree: Express was never told about a reverse proxy,
 * so `req.ip` was the proxy address and every user shared one rate-limit bucket,
 * while the WebSocket server trusted the first `X-Forwarded-For` entry from
 * anyone — a header any client can set, which let a caller bypass the per-IP
 * connection cap by inventing a new value on every connection.
 *
 * Both paths now read the same setting, `TRUST_PROXY`:
 *
 * - unset / `0` / `false` (default): forwarded headers are **ignored**, the
 *   socket address is the client. A spoofed `X-Forwarded-For` changes nothing.
 * - a number: the number of proxies in front of the app (hops). Only the last
 *   N hops are skipped; the first untrusted address in the chain is the client.
 * - a comma-separated list of IPs, CIDRs or named ranges (`loopback`,
 *   `linklocal`, `uniquelocal`): addresses matching any entry are proxies.
 *
 * A malformed setting fails closed (treated as "no trusted proxy") rather than
 * throwing at boot or, worse, trusting headers.
 */
export type TrustProxySetting = false | number | string[];

/** Named ranges, matching the ones Express itself understands. */
const NAMED_RANGES: Record<string, string[]> = {
   loopback: ['127.0.0.0/8', '::1/128'],
   linklocal: ['169.254.0.0/16', 'fe80::/10'],
   uniquelocal: ['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16', 'fc00::/7'],
};

/** Strip the IPv4-mapped IPv6 prefix so `::ffff:10.0.0.1` matches `10.0.0.0/8`. */
function normalizeIp(ip: string): string {
   const trimmed = ip.trim();
   const mapped = /^::ffff:(\d+\.\d+\.\d+\.\d+)$/i.exec(trimmed);
   return mapped ? mapped[1] : trimmed.toLowerCase();
}

function ipv4ToInt(ip: string): number | null {
   const parts = ip.split('.');
   if (parts.length !== 4) {
      return null;
   }
   let value = 0;
   for (const part of parts) {
      if (!/^\d{1,3}$/.test(part)) {
         return null;
      }
      const octet = Number(part);
      if (octet > 255) {
         return null;
      }
      value = (value << 8) + octet;
   }
   return value >>> 0;
}

/** Parse an IPv6 address into its 128-bit value. Returns null when unparseable. */
function ipv6ToBigInt(ip: string): bigint | null {
   const address = normalizeIp(ip);
   if (!address.includes(':')) {
      return null;
   }

   const parts = address.split('::');
   const head = parts[0] ?? '';
   const tail = parts.length > 1 ? parts.slice(1).join('::') : undefined;
   const headGroups = head ? head.split(':').filter(Boolean) : [];
   const tailGroups = tail ? tail.split(':').filter(Boolean) : [];
   if (!address.includes('::') && headGroups.length !== 8) {
      return null;
   }

   const missing = 8 - headGroups.length - tailGroups.length;
   if (address.includes('::') && missing < 0) {
      return null;
   }

   const groups = [
      ...headGroups,
      ...Array.from({ length: address.includes('::') ? missing : 0 }, () => '0'),
      ...tailGroups,
   ];
   if (groups.length !== 8) {
      return null;
   }

   let value = 0n;
   for (const group of groups) {
      if (!/^[0-9a-f]{1,4}$/.test(group)) {
         return null;
      }
      value = (value << 16n) + BigInt(parseInt(group, 16));
   }
   return value;
}

function matchesCidr(ip: string, cidr: string): boolean {
   const [network, prefixRaw] = cidr.split('/');
   if (prefixRaw === undefined) {
      return normalizeIp(ip) === normalizeIp(network);
   }

   const prefix = Number(prefixRaw);
   if (!Number.isInteger(prefix)) {
      return false;
   }

   const v4Address = ipv4ToInt(normalizeIp(ip));
   const v4Network = ipv4ToInt(normalizeIp(network));
   if (v4Address !== null && v4Network !== null) {
      if (prefix < 0 || prefix > 32) {
         return false;
      }
      if (prefix === 0) {
         return true;
      }
      const mask = prefix === 32 ? 0xffffffff : (0xffffffff << (32 - prefix)) >>> 0;
      return (v4Address & mask) >>> 0 === (v4Network & mask) >>> 0;
   }

   const v6Address = ipv6ToBigInt(ip);
   const v6Network = ipv6ToBigInt(network);
   if (v6Address !== null && v6Network !== null) {
      if (prefix < 0 || prefix > 128) {
         return false;
      }
      if (prefix === 0) {
         return true;
      }
      const shift = BigInt(128 - prefix);
      return (v6Address >> shift) === (v6Network >> shift);
   }

   return false;
}

function isTrustedAddress(ip: string, entries: string[]): boolean {
   return entries.some(entry => {
      const expanded = NAMED_RANGES[entry.toLowerCase()];
      if (expanded) {
         return expanded.some(cidr => matchesCidr(ip, cidr));
      }
      return matchesCidr(ip, entry);
   });
}

/**
 * Parse `TRUST_PROXY` into the value Express expects for `app.set('trust proxy')`.
 * Anything unrecognised resolves to `false` (trust nobody) instead of throwing.
 */
export function parseTrustProxy(raw: string | undefined): TrustProxySetting {
   const value = (raw ?? '').trim();
   if (value === '' || value === '0' || /^(false|no|off)$/i.test(value)) {
      return false;
   }

   if (/^\d+$/.test(value)) {
      return Number(value);
   }

   const entries = value
      .split(',')
      .map(entry => entry.trim())
      .filter(Boolean);

   if (entries.length === 0) {
      return false;
   }

   const isValidEntry = (entry: string): boolean => {
      if (NAMED_RANGES[entry.toLowerCase()]) {
         return true;
      }
      if (/^[^\s/]+\/\d{1,3}$/.test(entry)) {
         const [network] = entry.split('/');
         return ipv4ToInt(normalizeIp(network)) !== null || ipv6ToBigInt(network) !== null;
      }
      return ipv4ToInt(normalizeIp(entry)) !== null || ipv6ToBigInt(entry) !== null;
   };

   if (!entries.every(isValidEntry)) {
      console.warn(
         `[trusted-proxy] Ignoring malformed TRUST_PROXY=${value}; treating the socket address as the client IP.`,
      );
      return false;
   }

   return entries;
}

/** The configured trust setting, resolved once at import time. */
export const trustProxySetting: TrustProxySetting = parseTrustProxy(
   process.env.TRUST_PROXY,
);

function forwardedChain(req: Pick<IncomingMessage, 'headers'>): string[] {
   const header = req.headers['x-forwarded-for'];
   const raw = Array.isArray(header) ? header.join(',') : header;
   if (!raw) {
      return [];
   }
   return raw
      .split(',')
      .map(entry => entry.trim())
      .filter(Boolean);
}

/**
 * The client IP for a raw request or WebSocket upgrade.
 *
 * With no trusted proxy configured the socket address is returned and
 * `X-Forwarded-For` is ignored entirely, so the header cannot be spoofed into
 * changing the identity used for rate or connection limits.
 */
export function resolveClientIp(
   req: Pick<IncomingMessage, 'headers' | 'socket'>,
   setting: TrustProxySetting = trustProxySetting,
): string {
   const socketAddress = normalizeIp(req.socket?.remoteAddress ?? '');
   if (!socketAddress) {
      return 'unknown';
   }

   if (setting === false) {
      return socketAddress;
   }

   // Walk the chain from the closest hop outwards: the socket address first,
   // then the forwarded entries right-to-left. Everything trusted is skipped;
   // the first untrusted address is the client.
   const chain = [socketAddress, ...forwardedChain(req).reverse()];

   if (typeof setting === 'number') {
      // chain[0] is the socket (the closest hop), so skipping `setting` trusted
      // hops lands on the first untrusted address.
      const index = Math.min(Math.max(setting, 0), chain.length - 1);
      return chain[index] ?? socketAddress;
   }

   let index = 0;
   while (index < chain.length - 1 && isTrustedAddress(chain[index], setting)) {
      index += 1;
   }
   return chain[index] ?? 'unknown';
}
