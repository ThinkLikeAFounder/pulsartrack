#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, vec, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // This should panic because admin didn't authorize it and we haven't mocked it
    client.initialize(&admin);
}

#[test]
fn test_fraud_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.initialize(&admin);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    // 1. Test scaling fraud flags -> Publisher suspension
    // We'll set the threshold low for testing
    client.set_threshold(&admin, &90); // Verification threshold

    // Normally we'd call flag_suspicious multiple times
    // For this test, let's just verify it can be called without panic
    // (Actual cross-contract verification would require registering the other contracts too)
    client.flag_suspicious(&publisher);
}
