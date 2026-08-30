import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, afterEach } from 'vitest';
import { SubscriptionStatus } from './SubscriptionStatus';
import { Subscription } from '@/types/contracts';

const NOW_MS = 1_714_000_000_000; // fixed "now" for deterministic tests
const NOW_S = Math.floor(NOW_MS / 1000);

function makeSubscription(expiresOffsetMs: number): Subscription {
  const expiresS = BigInt(Math.floor((NOW_MS + expiresOffsetMs) / 1000));
  return {
    subscriber: 'GABC...123',
    tier: 'Growth',
    is_annual: false,
    amount_paid: 50_000_000n, // 5 XLM in stroops
    started_at: BigInt(NOW_S - 2_592_000), // 30 days ago
    expires_at: expiresS,
    auto_renew: true,
    campaigns_used: 2,
    impressions_used: 10_000n,
  };
}

describe('SubscriptionStatus expiry logic', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('shows "Expired" for a subscription that expired 2 days ago', () => {
    vi.useFakeTimers();
    vi.setSystemTime(NOW_MS);

    const sub = makeSubscription(-2 * 86_400_000); // expired 2 days ago
    render(<SubscriptionStatus subscription={sub} />);
    expect(screen.getByText('Expired')).toBeDefined();
  });

  it('shows "Expired" for a subscription that expired 1ms ago', () => {
    vi.useFakeTimers();
    vi.setSystemTime(NOW_MS);

    const sub = makeSubscription(-1);
    render(<SubscriptionStatus subscription={sub} />);
    expect(screen.getByText('Expired')).toBeDefined();
  });

  it('shows days remaining for a subscription expiring in 3 days', () => {
    vi.useFakeTimers();
    vi.setSystemTime(NOW_MS);

    const sub = makeSubscription(3 * 86_400_000);
    render(<SubscriptionStatus subscription={sub} />);
    expect(screen.getByText('3 days left')).toBeDefined();
  });

  it('shows days remaining for a subscription expiring today (5 hours left)', () => {
    vi.useFakeTimers();
    vi.setSystemTime(NOW_MS);

    const sub = makeSubscription(5 * 3_600_000); // 5 hours remaining
    render(<SubscriptionStatus subscription={sub} />);
    // Math.ceil(5h / 24h) = 1
    expect(screen.getByText('1 days left')).toBeDefined();
  });

  it('shows the expiry date (not a warning) for a subscription with > 7 days left', () => {
    vi.useFakeTimers();
    vi.setSystemTime(NOW_MS);

    const sub = makeSubscription(30 * 86_400_000);
    render(<SubscriptionStatus subscription={sub} />);
    // Should render the formatted expiry date, not "Expired" or "X days left"
    expect(screen.queryByText('Expired')).toBeNull();
    expect(screen.queryByText(/days left/)).toBeNull();
  });

  it('isExpired and isExpiringSoon are mutually exclusive — expired sub shows Renew, not Cancel', () => {
    vi.useFakeTimers();
    vi.setSystemTime(NOW_MS);

    const sub = makeSubscription(-1 * 86_400_000); // expired yesterday
    const onRenew = vi.fn();
    const onCancel = vi.fn();
    render(<SubscriptionStatus subscription={sub} onRenew={onRenew} onCancel={onCancel} />);

    expect(screen.getByRole('button', { name: 'Renew' })).toBeDefined();
    expect(screen.queryByRole('button', { name: 'Cancel' })).toBeNull();
  });

  it('renders null subscription with "No active subscription" message', () => {
    render(<SubscriptionStatus subscription={null} />);
    expect(screen.getByText('No active subscription')).toBeDefined();
  });
});
