#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (AnalyticsAggregatorContractClient, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, AnalyticsAggregatorContract);
    let c = AnalyticsAggregatorContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin)
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, AnalyticsAggregatorContract);
    let c = AnalyticsAggregatorContractClient::new(&env, &id);
    let a = Address::generate(&env); let o = Address::generate(&env);
    c.initialize(&a, &o); c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, AnalyticsAggregatorContract);
    let c = AnalyticsAggregatorContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_record_impression() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let caller = Address::generate(&env);
    c.record_impression(&caller, &1u64, &100i128);
    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.total_impressions, 1);
}

#[test]
fn test_record_click() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let caller = Address::generate(&env);
    c.record_impression(&caller, &1u64, &100i128);
    c.record_click(&caller, &1u64);
    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.total_clicks, 1);
}

#[test]
fn test_get_campaign_analytics_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_campaign_analytics(&999u64).is_none());
}

#[test]
fn test_get_global_stats() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let stats = c.get_global_stats();
    assert_eq!(stats.total_campaigns, 0);
}
