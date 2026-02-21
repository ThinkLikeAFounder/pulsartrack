#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, vec, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, OracleIntegrationContract);
    let client = OracleIntegrationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, OracleIntegrationContract);
    let client = OracleIntegrationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, OracleIntegrationContract);
    let client = OracleIntegrationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // This should panic because admin didn't authorize it and we haven't mocked it
    client.initialize(&admin);
}

#[test]
fn test_oracle_authorization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, OracleIntegrationContract);
    let client = OracleIntegrationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let asset = String::from_str(&env, "BTC");

    client.initialize(&admin);

    // Initial update_price should fail (unauthorized)
    let res = client.try_update_price(&oracle, &asset, &5000000000000, &95, &String::from_str(&env, "binance"));
    assert!(res.is_err());

    // Admin adds oracle
    client.add_oracle(&admin, &oracle);

    // Now update_price should succeed
    client.update_price(&oracle, &asset, &5000000000000, &95, &String::from_str(&env, "binance"));

    // Verify price was set
    let price = client.get_price(&asset).unwrap();
    assert_eq!(price.price_usd, 5000000000000);

    // Admin removes oracle
    client.remove_oracle(&admin, &oracle);

    // update_price should fail again
    let res = client.try_update_price(&oracle, &asset, &5100000000000, &95, &String::from_str(&env, "binance"));
    assert!(res.is_err());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_non_admin_cannot_add_oracle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, OracleIntegrationContract);
    let client = OracleIntegrationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    client.initialize(&admin);

    // non_admin tries to add oracle
    client.add_oracle(&non_admin, &oracle);
}
