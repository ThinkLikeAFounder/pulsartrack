#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (OracleIntegrationContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, OracleIntegrationContract);
    let c = OracleIntegrationContractClient::new(env, &id);
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
    let id = env.register_contract(None, OracleIntegrationContract);
    let c = OracleIntegrationContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, OracleIntegrationContract);
    let c = OracleIntegrationContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_add_oracle() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let oracle = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    assert!(c.is_oracle_authorized(&oracle));
}

#[test]
fn test_remove_oracle() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let oracle = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.remove_oracle(&admin, &oracle);
    assert!(!c.is_oracle_authorized(&oracle));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_oracle_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.add_oracle(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_update_price() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let oracle = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.update_price(&oracle, &s(&env, "XLM"), &1_500_000i128, &95u32, &s(&env, "external"));
    let price = c.get_price(&s(&env, "XLM")).unwrap();
    assert_eq!(price.price_usd, 1_500_000);
}

#[test]
fn test_update_performance() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let oracle = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.update_performance(&oracle, &1u64, &1000u64, &50u64, &5u64, &10u32);
    let perf = c.get_performance(&1u64).unwrap();
    assert_eq!(perf.impressions, 1000);
}

#[test]
fn test_get_price_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_price(&s(&env, "BTC")).is_none());
}

#[test]
fn test_is_oracle_authorized_false() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.is_oracle_authorized(&Address::generate(&env)));
}
