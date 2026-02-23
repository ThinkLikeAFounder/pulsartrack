#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (TokenBridgeContractClient, Address) {
    let admin = Address::generate(env);
    let relayer = Address::generate(env);
    let id = env.register_contract(None, TokenBridgeContract);
    let c = TokenBridgeContractClient::new(env, &id);
    c.initialize(&admin, &relayer);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, TokenBridgeContract);
    let c = TokenBridgeContractClient::new(&env, &id);
    let a = Address::generate(&env); let r = Address::generate(&env);
    c.initialize(&a, &r); c.initialize(&a, &r);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, TokenBridgeContract);
    let c = TokenBridgeContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_add_supported_chain() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.add_supported_chain(&admin, &s(&env, "ethereum"), &1_000_000i128);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_supported_chain_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.add_supported_chain(&Address::generate(&env), &s(&env, "ethereum"), &1_000_000i128);
}

#[test]
fn test_get_deposit_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_deposit(&999u64).is_none());
}
