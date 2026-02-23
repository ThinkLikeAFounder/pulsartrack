#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, token::StellarAssetClient, Address, Env};

// ============================================================
// Test Helpers
// ============================================================

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn balance(env: &Env, token: &Address, addr: &Address) -> i128 {
    token::Client::new(env, token).balance(addr)
}

/// Returns (client, admin, token_admin_addr, token_addr, treasury_addr)
fn setup(env: &Env) -> (
    SubscriptionManagerContractClient,
    Address,
    Address,
    Address,
    Address,
) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let treasury = Address::generate(env);
    let id = env.register_contract(None, SubscriptionManagerContract);
    let c = SubscriptionManagerContractClient::new(env, &id);
    c.initialize(&admin, &token, &treasury);
    (c, admin, token_admin, token, treasury)
}

/// Mint enough tokens for the subscriber to cover a Business annual plan (worst case).
fn fund_subscriber(env: &Env, token: &Address, subscriber: &Address) {
    // Enterprise annual: 19_990_000_000 stroops — mint 2× for safety
    mint(env, token, subscriber, 40_000_000_000);
}

// Plan prices (stroops) — must mirror `_init_plans`
const STARTER_MONTHLY: i128 = 99_000_000;
const GROWTH_MONTHLY: i128 = 299_000_000;
const BUSINESS_MONTHLY: i128 = 799_000_000;
const GROWTH_ANNUAL: i128 = 2_990_000_000;

const MONTHLY_SECS: u64 = 30 * 24 * 3600;
const ANNUAL_SECS: u64 = 365 * 24 * 3600;

// ============================================================
// Initialization
// ============================================================

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    setup(&env);
    // Plans must be queryable after init
    let (c, ..) = setup(&env);
    assert!(c.get_plan(&SubscriptionTier::Starter).is_some());
    assert!(c.get_plan(&SubscriptionTier::Enterprise).is_some());
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, token, _) = setup(&env);
    c.initialize(&admin, &token, &Address::generate(&env));
}

// ============================================================
// subscribe()
// ============================================================

#[test]
fn test_subscribe_new_subscriber() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);

    assert!(c.is_active(&subscriber));
    let sub = c.get_subscription(&subscriber).unwrap();
    assert!(matches!(sub.tier, SubscriptionTier::Starter));
    assert_eq!(sub.campaigns_used, 0);
    assert_eq!(sub.impressions_used, 0);
    assert_eq!(sub.amount_paid, STARTER_MONTHLY);
}

#[test]
fn test_subscribe_charges_full_price() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, treasury) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, GROWTH_MONTHLY);

    let before = balance(&env, &token, &treasury);
    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);
    let after = balance(&env, &token, &treasury);

    assert_eq!(after - before, GROWTH_MONTHLY);
}

#[test]
fn test_subscribe_annual_charges_annual_price() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, treasury) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, GROWTH_ANNUAL);

    let before = balance(&env, &token, &treasury);
    c.subscribe(&subscriber, &SubscriptionTier::Growth, &true, &false);
    let after = balance(&env, &token, &treasury);

    assert_eq!(after - before, GROWTH_ANNUAL);
    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.expires_at - sub.started_at, ANNUAL_SECS);
}

#[test]
#[should_panic(expected = "already active")]
fn test_subscribe_panics_if_active_subscription_exists() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    // Second call must panic
    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
}

#[test]
fn test_subscribe_after_expiry_is_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);

    // Advance time past expiry
    env.ledger().set_timestamp(env.ledger().timestamp() + MONTHLY_SECS + 1);

    assert!(!c.is_active(&subscriber));
    // Should succeed now
    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    assert!(c.is_active(&subscriber));
}

// ============================================================
// change_tier()  — Upgrades
// ============================================================

#[test]
fn test_upgrade_charges_prorated_delta() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, treasury) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    // Subscribe to Growth (monthly)
    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);

    // Advance 10 days (of 30) — 20 days remaining
    let ten_days = 10 * 24 * 3600u64;
    env.ledger().set_timestamp(env.ledger().timestamp() + ten_days);

    let treasury_before = balance(&env, &token, &treasury);

    c.change_tier(&subscriber, &SubscriptionTier::Business, &false, &true);

    let net_charged = balance(&env, &token, &treasury) - treasury_before;

    // remaining = 20 days out of 30; credit = Growth_monthly * 20/30
    let remaining = MONTHLY_SECS - ten_days;
    let credit = (GROWTH_MONTHLY * remaining as i128) / MONTHLY_SECS as i128;
    let expected_net = (BUSINESS_MONTHLY - credit).max(0);

    assert_eq!(net_charged, expected_net);
}

#[test]
fn test_upgrade_preserves_usage_counters() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);

    // Simulate some usage
    c.record_campaign_used(&subscriber);
    c.record_campaign_used(&subscriber);
    c.record_impression(&subscriber, &5_000);

    c.change_tier(&subscriber, &SubscriptionTier::Business, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.campaigns_used, 2, "campaigns_used must be preserved");
    assert_eq!(sub.impressions_used, 5_000, "impressions_used must be preserved");
}

#[test]
fn test_upgrade_new_expiry_is_full_period_from_now() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);
    env.ledger().set_timestamp(env.ledger().timestamp() + 10 * 24 * 3600);
    let now = env.ledger().timestamp();

    c.change_tier(&subscriber, &SubscriptionTier::Business, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.expires_at, now + MONTHLY_SECS);
}

#[test]
fn test_upgrade_zero_net_charge_when_credit_exceeds_new_price() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, treasury) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    // Subscribe to Business annual (expensive) then immediately upgrade to Enterprise.
    // Credit from Business annual covers entire Enterprise monthly.
    c.subscribe(&subscriber, &SubscriptionTier::Business, &true, &true);

    let treasury_before = balance(&env, &token, &treasury);
    c.change_tier(&subscriber, &SubscriptionTier::Enterprise, &false, &true);
    let net_charged = balance(&env, &token, &treasury) - treasury_before;

    // credit = 7_990_000_000 (BUSINESS_ANNUAL, full since no time has passed)
    // net = max(1_999_000_000 - 7_990_000_000, 0) = 0
    assert_eq!(net_charged, 0, "credit should fully absorb the new price");
}

#[test]
fn test_upgrade_stores_full_price_for_future_proration() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);
    c.change_tier(&subscriber, &SubscriptionTier::Business, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    // amount_paid must reflect the new plan's full price, not the net charged
    assert_eq!(sub.amount_paid, BUSINESS_MONTHLY);
}

// ============================================================
// change_tier()  — Blocked Operations
// ============================================================

#[test]
#[should_panic(expected = "no active subscription")]
fn test_change_tier_panics_with_no_subscription() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, ..) = setup(&env);
    c.change_tier(
        &Address::generate(&env),
        &SubscriptionTier::Business,
        &false,
        &true,
    );
}

#[test]
#[should_panic(expected = "downgrade not allowed")]
fn test_change_tier_blocks_downgrade() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    c.subscribe(&subscriber, &SubscriptionTier::Business, &false, &true);
    c.change_tier(&subscriber, &SubscriptionTier::Growth, &false, &true);
}

#[test]
#[should_panic(expected = "same tier")]
fn test_change_tier_blocks_same_tier() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);
    c.change_tier(&subscriber, &SubscriptionTier::Growth, &false, &true);
}

// ============================================================
// renew()
// ============================================================

#[test]
fn test_renew_extends_expiry_beyond_current_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 3);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let first_expiry = c.get_subscription(&subscriber).unwrap().expires_at;

    // Renew with 15 days still remaining
    env.ledger().set_timestamp(env.ledger().timestamp() + 15 * 24 * 3600);
    c.renew(&subscriber, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    // New expiry = first_expiry + MONTHLY_SECS (stacks on top)
    assert_eq!(sub.expires_at, first_expiry + MONTHLY_SECS);
}

#[test]
fn test_renew_after_expiry_starts_from_now() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);

    // Advance past expiry
    let jump = MONTHLY_SECS + 5 * 24 * 3600;
    env.ledger().set_timestamp(env.ledger().timestamp() + jump);
    let now = env.ledger().timestamp();

    c.renew(&subscriber, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.expires_at, now + MONTHLY_SECS);
}

#[test]
fn test_renew_preserves_usage_counters() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    c.record_campaign_used(&subscriber);
    c.record_impression(&subscriber, &1_234);

    c.renew(&subscriber, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.campaigns_used, 1);
    assert_eq!(sub.impressions_used, 1_234);
}

#[test]
fn test_renew_preserves_original_started_at() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let original_started_at = c.get_subscription(&subscriber).unwrap().started_at;

    env.ledger().set_timestamp(env.ledger().timestamp() + 15 * 24 * 3600);
    c.renew(&subscriber, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.started_at, original_started_at);
}

#[test]
#[should_panic]
fn test_renew_panics_with_no_subscription() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, ..) = setup(&env);
    c.renew(&Address::generate(&env), &false, &true);
}

// ============================================================
// cancel()
// ============================================================

#[test]
fn test_cancel_disables_auto_renew_but_stays_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    c.cancel(&subscriber);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert!(!sub.auto_renew);
    assert!(c.is_active(&subscriber), "subscription must still be active until expiry");
}

// ============================================================
// Usage Tracking
// ============================================================

#[test]
fn test_record_campaign_used_increments_counter() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    c.record_campaign_used(&subscriber);
    c.record_campaign_used(&subscriber);

    assert_eq!(c.get_subscription(&subscriber).unwrap().campaigns_used, 2);
}

#[test]
fn test_record_impression_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    c.record_impression(&subscriber, &10_000);
    c.record_impression(&subscriber, &5_000);

    assert_eq!(
        c.get_subscription(&subscriber).unwrap().impressions_used,
        15_000
    );
}

// ============================================================
// Read-Only Views
// ============================================================

#[test]
fn test_is_active_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, ..) = setup(&env);
    assert!(!c.is_active(&Address::generate(&env)));
}

#[test]
fn test_get_subscription_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, ..) = setup(&env);
    assert!(c.get_subscription(&Address::generate(&env)).is_none());
}

#[test]
fn test_is_active_returns_false_after_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    env.ledger().set_timestamp(env.ledger().timestamp() + MONTHLY_SECS + 1);
    assert!(!c.is_active(&subscriber));
}
