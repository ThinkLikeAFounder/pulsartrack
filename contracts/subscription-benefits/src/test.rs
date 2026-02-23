#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (SubscriptionBenefitsContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, SubscriptionBenefitsContract);
    let c = SubscriptionBenefitsContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, SubscriptionBenefitsContract);
    let c = SubscriptionBenefitsContractClient::new(&env, &id);
    let a = Address::generate(&env); c.initialize(&a); c.initialize(&a);
}

#[test]
fn test_add_benefit() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let bid = c.add_benefit(&admin, &s(&env, "Premium Access"), &s(&env, "Full API access"), &1u32);
    let benefit = c.get_benefit(&bid).unwrap();
    assert_eq!(benefit.min_tier, 1);
}

#[test]
fn test_check_benefit_access() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let sub = Address::generate(&env);
    let bid = c.add_benefit(&admin, &s(&env, "Premium"), &s(&env, "Access"), &1u32);
    assert!(c.check_benefit_access(&sub, &bid, &2u32));  // tier 2 >= min_tier 1
    assert!(!c.check_benefit_access(&sub, &bid, &0u32)); // tier 0 < min_tier 1
}

#[test]
fn test_use_benefit() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let sub = Address::generate(&env);
    let bid = c.add_benefit(&admin, &s(&env, "Premium"), &s(&env, "Access"), &1u32);
    c.use_benefit(&sub, &bid, &2u32);
    let usage = c.get_usage(&sub, &bid).unwrap();
    assert_eq!(usage.uses_this_period, 1);
}

#[test]
fn test_get_benefit_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_benefit(&999u32).is_none());
}
