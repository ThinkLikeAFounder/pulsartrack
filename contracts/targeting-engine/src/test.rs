#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (TargetingEngineContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, TargetingEngineContract);
    let c = TargetingEngineContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

fn default_params(env: &Env) -> TargetingParams {
    TargetingParams {
        geographic_targets: s(env, "US,EU"),
        interest_segments: s(env, "tech,news"),
        excluded_segments: s(env, ""),
        min_age: 18,
        max_age: 65,
        device_types: s(env, "mobile,desktop"),
        languages: s(env, "en"),
        min_reputation: 50,
        exclude_fraud: true,
        require_kyc: false,
        max_cpm: 10_000,
    }
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, TargetingEngineContract);
    let c = TargetingEngineContractClient::new(&env, &id);
    let a = Address::generate(&env); c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, TargetingEngineContract);
    let c = TargetingEngineContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_add_oracle() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.add_oracle(&admin, &Address::generate(&env));
}

#[test]
fn test_set_targeting() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let params = default_params(&env);
    c.set_targeting(&advertiser, &1u64, &params);
    let config = c.get_targeting(&1u64).unwrap();
    assert_eq!(config.geographic_targets, s(&env, "US,EU"));
}

#[test]
fn test_compute_score() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let oracle = Address::generate(&env);
    let pub1 = Address::generate(&env);
    let advertiser = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.set_targeting(&advertiser, &1u64, &default_params(&env));
    c.compute_score(&oracle, &1u64, &pub1, &750u32, &s(&env, "geo_match,interest_match"));
    let score = c.get_targeting_score(&1u64, &pub1).unwrap();
    assert_eq!(score.score, 750);
}

#[test]
fn test_is_publisher_targeted() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let oracle = Address::generate(&env);
    let pub1 = Address::generate(&env);
    let advertiser = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.set_targeting(&advertiser, &1u64, &default_params(&env));
    c.compute_score(&oracle, &1u64, &pub1, &750u32, &s(&env, "match"));
    assert!(c.is_publisher_targeted(&1u64, &pub1, &600u32));
    assert!(!c.is_publisher_targeted(&1u64, &pub1, &800u32));
}

#[test]
fn test_get_targeting_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_targeting(&999u64).is_none());
}
