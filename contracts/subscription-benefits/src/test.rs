#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (SubscriptionBenefitsContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(SubscriptionBenefitsContract, ());
    let c = SubscriptionBenefitsContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String {
    String::from_str(env, v)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    setup(&env);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(SubscriptionBenefitsContract, ());
    let c = SubscriptionBenefitsContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
fn test_add_benefit() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let bid = c.add_benefit(
        &admin,
        &s(&env, "Premium Access"),
        &s(&env, "Full API access"),
        &1u32,
    );
    let benefit = c.get_benefit(&bid).unwrap();
    assert_eq!(benefit.min_tier, 1);
}

#[test]
fn test_check_benefit_access() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let sub_high = Address::generate(&env);
    let sub_low = Address::generate(&env);
    let bid = c.add_benefit(&admin, &s(&env, "Premium"), &s(&env, "Access"), &1u32);
    // Access is governed by the subscriber's stored tier, not the caller-supplied
    // value — so the third argument cannot be used to escalate access.
    c.update_subscriber_tier(&admin, &sub_high, &2u32);
    assert!(c.check_benefit_access(&sub_high, &bid, &0u32)); // stored tier 2 >= min_tier 1
    assert!(!c.check_benefit_access(&sub_low, &bid, &2u32)); // stored tier 0 < min_tier 1
}

#[test]
fn test_use_benefit() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let sub = Address::generate(&env);
    let bid = c.add_benefit(&admin, &s(&env, "Premium"), &s(&env, "Access"), &1u32);
    // The subscriber must hold a sufficient stored tier to use the benefit.
    c.update_subscriber_tier(&admin, &sub, &2u32);
    c.use_benefit(&sub, &bid, &2u32);
    let usage = c.get_usage(&sub, &bid).unwrap();
    assert_eq!(usage.uses_this_period, 1);
}

#[test]
fn test_get_benefit_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_benefit(&999u32).is_none());
}

// ── update_subscriber_tier tests (#785) ─────────────────────────────────────

#[test]
fn test_update_subscriber_tier() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let sub = Address::generate(&env);
    let bid = c.add_benefit(&admin, &s(&env, "Premium"), &s(&env, "Access"), &2u32);

    // Initially tier 0 — no access to tier-2 benefit
    assert!(!c.check_benefit_access(&sub, &bid, &0u32));

    // Upgrade to tier 2
    c.update_subscriber_tier(&admin, &sub, &2u32);
    assert!(c.check_benefit_access(&sub, &bid, &0u32));

    // Downgrade to tier 1 — loses access
    c.update_subscriber_tier(&admin, &sub, &1u32);
    assert!(!c.check_benefit_access(&sub, &bid, &0u32));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_update_subscriber_tier_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let sub = Address::generate(&env);
    c.update_subscriber_tier(&Address::generate(&env), &sub, &2u32);
}
