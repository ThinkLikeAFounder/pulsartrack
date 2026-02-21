#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, vec, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TargetingEngineContract);
    let client = TargetingEngineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TargetingEngineContract);
    let client = TargetingEngineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, TargetingEngineContract);
    let client = TargetingEngineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // This should panic because admin didn't authorize it and we haven't mocked it
    client.initialize(&admin);
}

#[test]
fn test_oracle_authorization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TargetingEngineContract);
    let client = TargetingEngineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.initialize(&admin);

    // Initial compute_score should fail (unauthorized)
    let res = client.try_compute_score(&oracle, &1, &publisher, &500, &String::from_str(&env, "match"));
    assert!(res.is_err());

    // Admin adds oracle
    client.add_oracle(&admin, &oracle);

    // Now compute_score should succeed
    client.compute_score(&oracle, &1, &publisher, &500, &String::from_str(&env, "match"));

    // Verify score was set
    let score = client.get_targeting_score(&1, &publisher).unwrap();
    assert_eq!(score.score, 500);

    // Admin removes oracle
    client.remove_oracle(&admin, &oracle);

    // compute_score should fail again
    let res = client.try_compute_score(&oracle, &1, &publisher, &600, &String::from_str(&env, "match"));
    assert!(res.is_err());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_non_admin_cannot_add_oracle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TargetingEngineContract);
    let client = TargetingEngineContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    client.initialize(&admin);

    // non_admin tries to add oracle
    client.add_oracle(&non_admin, &oracle);
}
