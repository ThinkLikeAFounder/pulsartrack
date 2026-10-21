import { test, expect } from '@playwright/test';

// A syntactically valid (StrKey-checksummed) Stellar keypair used only to
// drive the mocked Freighter wallet below. It isn't funded or real — no
// signing happens in this test (the mocked RPC call below fails before the
// app ever reaches the signing step).
const MOCK_WALLET_ADDRESS = 'GD2TNOV4MFL2IWBTIWRMGB5RJR7X44M23XVELVHO2B45MCHIN4STEI35';
const TESTNET_PASSPHRASE = 'Test SDF Network ; September 2015';
const SOROBAN_RPC_HOST = 'soroban-testnet.stellar.org';

/**
 * Exercises the real advertiser flow end-to-end: connect a (mocked)
 * Freighter wallet, fill out the campaign creation form, and submit it.
 *
 * PulsarTrack's contracts aren't deployed anywhere reachable from CI (see
 * the E2E job's NEXT_PUBLIC_CONTRACT_* placeholders), so a real on-chain
 * flow isn't possible here. Instead, the two things an installed wallet
 * extension and a live network provide are mocked at their natural
 * boundaries:
 *   - Freighter: real wallet extensions inject a page-world global (content
 *     scripts run in an isolated JS world and can't hand the page live
 *     functions any other way), so `window.freighterApi` is stubbed the same
 *     way the real extension would appear.
 *   - Soroban RPC: every JSON-RPC call is answered with a well-formed
 *     JSON-RPC error over the real HTTP endpoint.
 *
 * Everything else — wallet state, route gating, form validation, and the
 * contract-id/RPC-url wiring in useContract/soroban-client — is the app's
 * real code, so a regression anywhere in that chain (e.g. a broken submit
 * handler, a misconfigured contract id, a wallet-state bug that never
 * unlocks the dashboard) will fail this test.
 */
test.describe('campaign creation flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(
      ({ address, passphrase }) => {
        (window as unknown as { freighterApi: Record<string, unknown> }).freighterApi = {
          isConnected: async () => ({ isConnected: true }),
          isAllowed: async () => ({ isAllowed: true }),
          requestAccess: async () => ({ address }),
          getAddress: async () => ({ address }),
          getNetworkDetails: async () => ({ network: 'TESTNET', networkPassphrase: passphrase }),
          signTransaction: async () => ({ error: 'signing is not exercised by this test' }),
        };
      },
      { address: MOCK_WALLET_ADDRESS, passphrase: TESTNET_PASSPHRASE },
    );

    await page.route(`https://${SOROBAN_RPC_HOST}/**`, async (route) => {
      let id: unknown = null;
      try {
        id = route.request().postDataJSON()?.id ?? null;
      } catch {
        // Non-JSON body — fall through with id: null.
      }
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          jsonrpc: '2.0',
          id,
          error: { code: -32000, message: 'mocked: no live Soroban RPC in this environment' },
        }),
      });
    });
  });

  test('connect wallet, fill out, and submit a campaign', async ({ page }) => {
    const pageErrors: Error[] = [];
    page.on('pageerror', (err) => pageErrors.push(err));

    await page.goto('/advertiser');

    // The dashboard gates everything behind a wallet connection. Scope to
    // <main> — the site header also renders its own connect button(s).
    const main = page.locator('main');
    await main.getByRole('button', { name: /connect/i }).click();

    // Real app state: the wallet store picks up the mocked connection and
    // the dashboard replaces the "connect your wallet" gate.
    await expect(main.getByRole('heading', { name: 'Advertiser Dashboard' })).toBeVisible();
    await expect(main.getByText(MOCK_WALLET_ADDRESS)).toBeVisible();

    // Switch to the real campaign creation form. `.first()` because the
    // empty "My Campaigns" tab also shows its own "Create Campaign" CTA.
    await main.getByRole('button', { name: 'Create Campaign' }).first().click();

    await page.getByLabel('Campaign Title').fill('E2E Test Campaign');
    await page
      .getByLabel('Content ID (IPFS Hash)')
      .fill('QmE2ETestContentHashXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX');
    await page.getByLabel('Total Budget (XLM)').fill('1000');
    await page.getByLabel('Daily Budget (XLM)').fill('50');
    await page.getByLabel('Duration (days)').fill('30');

    const submit = page.locator('form button[type="submit"]');
    await expect(submit).toHaveText('Create Campaign');

    // Submitting drives the app's real create-campaign flow, which resolves
    // the configured contract id/RPC url and issues a real network request.
    // Prove that request actually happens, against the right endpoint.
    const rpcRequest = page.waitForRequest(
      (req) => req.url().includes(SOROBAN_RPC_HOST) && req.method() === 'POST',
    );
    await submit.click();
    await rpcRequest;

    // The mutation must settle (not hang) and hand control back to the form.
    await expect(submit).toBeEnabled({ timeout: 15_000 });
    await expect(submit).toHaveText('Create Campaign');

    expect(pageErrors).toEqual([]);
  });
});
