#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::Address as _, testutils::Ledger as _, token::StellarAssetClient, Address, Env,
};

// ============================================================
// Test Helpers
// ============================================================

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn balance(env: &Env, token: &Address, addr: &Address) -> i128 {
    token::Client::new(env, token).balance(addr)
}

fn approve_spender(env: &Env, token: &Address, owner: &Address, spender: &Address, amount: i128) {
    let live_until = env.ledger().sequence() + 1_000;
    token::Client::new(env, token).approve(owner, spender, &amount, &live_until);
}

fn record_campaign_from_orchestrator(
    env: &Env,
    _orchestrator: &Address,
    client: &SubscriptionManagerContractClient<'_>,
    subscriber: &Address,
) {
    let _ = env;
    client.record_campaign_used(subscriber);
}

fn record_impression_from_orchestrator(
    env: &Env,
    _orchestrator: &Address,
    client: &SubscriptionManagerContractClient<'_>,
    subscriber: &Address,
    count: u64,
) {
    let _ = env;
    client.record_impression(subscriber, &count);
}

/// Returns (client, admin, token_admin_addr, token_addr, treasury_addr, orchestrator_addr, contract_addr)
fn setup(
    env: &Env,
) -> (
    SubscriptionManagerContractClient<'_>,
    Address,
    Address,
    Address,
    Address,
    Address,
    Address,
) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let treasury = Address::generate(env);
    let orchestrator = Address::generate(env);
    let id = env.register(SubscriptionManagerContract, ());
    let c = SubscriptionManagerContractClient::new(env, &id);
    c.initialize(&admin, &token, &treasury, &orchestrator);
    (c, admin, token_admin, token, treasury, orchestrator, id)
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
    let (c, admin, _, token, _, orchestrator, ..) = setup(&env);
    c.initialize(&admin, &token, &Address::generate(&env), &orchestrator);
}

// ============================================================
// subscribe()
// ============================================================

#[test]
fn test_subscribe_new_subscriber() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
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
    let (c, _, _, token, treasury, ..) = setup(&env);
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
    let (c, _, _, token, treasury, ..) = setup(&env);
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
    let (c, _, _, token, ..) = setup(&env);
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
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);

    // Advance time past expiry
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + MONTHLY_SECS + 1);

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
    let (c, _, _, token, treasury, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    // Subscribe to Growth (monthly)
    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);

    // Advance 10 days (of 30) — 20 days remaining
    let ten_days = 10 * 24 * 3600u64;
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + ten_days);

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
    let (c, _, _, token, _, orchestrator, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);

    // Simulate some usage
    record_campaign_from_orchestrator(&env, &orchestrator, &c, &subscriber);
    record_campaign_from_orchestrator(&env, &orchestrator, &c, &subscriber);
    record_impression_from_orchestrator(&env, &orchestrator, &c, &subscriber, 5_000);

    c.change_tier(&subscriber, &SubscriptionTier::Business, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.campaigns_used, 2, "campaigns_used must be preserved");
    assert_eq!(
        sub.impressions_used, 5_000,
        "impressions_used must be preserved"
    );
}

#[test]
fn test_upgrade_new_expiry_is_full_period_from_now() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    fund_subscriber(&env, &token, &subscriber);

    c.subscribe(&subscriber, &SubscriptionTier::Growth, &false, &true);
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 10 * 24 * 3600);
    let now = env.ledger().timestamp();

    c.change_tier(&subscriber, &SubscriptionTier::Business, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.expires_at, now + MONTHLY_SECS);
}

#[test]
fn test_upgrade_zero_net_charge_when_credit_exceeds_new_price() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, treasury, ..) = setup(&env);
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
    let (c, _, _, token, ..) = setup(&env);
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
    let (c, _, _, token, ..) = setup(&env);
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
    let (c, _, _, token, ..) = setup(&env);
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
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 3);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let first_expiry = c.get_subscription(&subscriber).unwrap().expires_at;

    // Renew with 15 days still remaining
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 15 * 24 * 3600);
    c.renew(&subscriber, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    // New expiry = first_expiry + MONTHLY_SECS (stacks on top)
    assert_eq!(sub.expires_at, first_expiry + MONTHLY_SECS);
}

#[test]
fn test_renew_after_expiry_starts_from_now() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
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
    let (c, _, _, token, _, orchestrator, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    record_campaign_from_orchestrator(&env, &orchestrator, &c, &subscriber);
    record_impression_from_orchestrator(&env, &orchestrator, &c, &subscriber, 1_234);

    c.renew(&subscriber, &false, &true);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert_eq!(sub.campaigns_used, 1);
    assert_eq!(sub.impressions_used, 1_234);
}

#[test]
fn test_renew_preserves_original_started_at() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let original_started_at = c.get_subscription(&subscriber).unwrap().started_at;

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 15 * 24 * 3600);
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
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    c.cancel(&subscriber);

    let sub = c.get_subscription(&subscriber).unwrap();
    assert!(!sub.auto_renew);
    assert!(
        c.is_active(&subscriber),
        "subscription must still be active until expiry"
    );
}

// ============================================================
// Usage Tracking
// ============================================================

#[test]
fn test_record_campaign_used_increments_counter() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _, orchestrator, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    record_campaign_from_orchestrator(&env, &orchestrator, &c, &subscriber);
    record_campaign_from_orchestrator(&env, &orchestrator, &c, &subscriber);

    assert_eq!(c.get_subscription(&subscriber).unwrap().campaigns_used, 2);
}

#[test]
fn test_record_impression_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, _, orchestrator, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    record_impression_from_orchestrator(&env, &orchestrator, &c, &subscriber, 10_000);
    record_impression_from_orchestrator(&env, &orchestrator, &c, &subscriber, 5_000);

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
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + MONTHLY_SECS + 1);
    assert!(!c.is_active(&subscriber));
}

// ============================================================
// auto_renew_subscription()
// ============================================================

#[test]
fn test_auto_renew_extends_expiry_from_old_expires_at() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (c, _, _, token, _, _, contract_id) = setup(&env);
    let subscriber = Address::generate(&env);
    // Fund for initial subscription + renewal
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let original_expiry = c.get_subscription(&subscriber).unwrap().expires_at;
    approve_spender(&env, &token, &subscriber, &contract_id, STARTER_MONTHLY);

    // Advance just past expiry
    env.ledger().set_timestamp(original_expiry + 1);

    c.auto_renew_subscription(&subscriber);

    let sub = c.get_subscription(&subscriber).unwrap();
    // New expiry = original_expiry.max(now) + MONTHLY_SECS = original_expiry + 1 + MONTHLY_SECS
    // since now (original_expiry + 1) > original_expiry
    assert_eq!(sub.expires_at, original_expiry + 1 + MONTHLY_SECS);
    assert!(c.is_active(&subscriber));
}

#[test]
fn test_auto_renew_charges_correct_amount() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (c, _, _, token, treasury, _, contract_id) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let original_expiry = c.get_subscription(&subscriber).unwrap().expires_at;
    approve_spender(&env, &token, &subscriber, &contract_id, STARTER_MONTHLY);
    env.ledger().set_timestamp(original_expiry + 1);

    let treasury_before = balance(&env, &token, &treasury);
    c.auto_renew_subscription(&subscriber);
    let charged = balance(&env, &token, &treasury) - treasury_before;

    assert_eq!(charged, STARTER_MONTHLY);
}

#[test]
#[should_panic(expected = "insufficient allowance for auto-renewal")]
fn test_auto_renew_panics_when_allowance_missing() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let expiry = c.get_subscription(&subscriber).unwrap().expires_at;
    env.ledger().set_timestamp(expiry + 1);

    c.auto_renew_subscription(&subscriber);
}

#[test]
#[should_panic(expected = "auto renew not enabled")]
fn test_auto_renew_panics_when_flag_is_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    // Subscribe with auto_renew = false
    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &false);
    let expiry = c.get_subscription(&subscriber).unwrap().expires_at;
    env.ledger().set_timestamp(expiry + 1);

    c.auto_renew_subscription(&subscriber);
}

#[test]
#[should_panic(expected = "subscription not yet expired")]
fn test_auto_renew_panics_when_still_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    // Don't advance time — subscription is still active
    c.auto_renew_subscription(&subscriber);
}

#[test]
#[should_panic(expected = "insufficient balance for auto-renewal")]
fn test_auto_renew_panics_when_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    // Fund only enough for the initial subscription; nothing left for renewal
    mint(&env, &token, &subscriber, STARTER_MONTHLY);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    let expiry = c.get_subscription(&subscriber).unwrap().expires_at;
    env.ledger().set_timestamp(expiry + 1);

    c.auto_renew_subscription(&subscriber);
}

#[test]
#[should_panic(expected = "no subscription found")]
fn test_auto_renew_panics_when_no_subscription() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, ..) = setup(&env);
    c.auto_renew_subscription(&Address::generate(&env));
}

#[test]
#[should_panic(expected = "auto renew not enabled")]
fn test_cancel_then_auto_renew_is_blocked() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token, ..) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, STARTER_MONTHLY * 2);

    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    c.cancel(&subscriber);

    let expiry = c.get_subscription(&subscriber).unwrap().expires_at;
    env.ledger().set_timestamp(expiry + 1);

    // cancel() sets auto_renew = false, so auto_renew_subscription must panic
    c.auto_renew_subscription(&subscriber);
}
