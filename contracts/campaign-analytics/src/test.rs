#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup(env: &Env) -> (CampaignAnalyticsContractClient, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, CampaignAnalyticsContract);
    let c = CampaignAnalyticsContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin, oracle)
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, CampaignAnalyticsContract);
    let c = CampaignAnalyticsContractClient::new(&env, &id);
    let a = Address::generate(&env); let o = Address::generate(&env);
    c.initialize(&a, &o); c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, CampaignAnalyticsContract);
    let c = CampaignAnalyticsContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_record_snapshot() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.record_snapshot(&oracle, &1u64, &1000u64, &50u64, &10u64, &5000i128, &800u64);
    let snap = c.get_snapshot(&1u64, &0u32).unwrap();
    assert_eq!(snap.impressions, 1000);
    assert_eq!(snap.clicks, 50);
}

#[test]
fn test_update_retention() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.update_retention(&oracle, &1u64, &80u32, &60u32, &40u32, &120u64, &25u32);
    let ret = c.get_retention(&1u64).unwrap();
    assert_eq!(ret.day_1_retention, 80);
    assert_eq!(ret.day_7_retention, 60);
    assert_eq!(ret.day_30_retention, 40);
}

#[test]
fn test_get_snapshot_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_snapshot(&999u64, &0u32).is_none());
}

#[test]
fn test_get_retention_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_retention(&999u64).is_none());
}

#[test]
fn test_snapshot_count() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.record_snapshot(&oracle, &1u64, &1000u64, &50u64, &10u64, &5000i128, &800u64);
    assert_eq!(c.get_snapshot_count(&1u64), 1);
}
