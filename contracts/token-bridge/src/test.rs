#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (TokenBridgeContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let relayer = Address::generate(env);
    let id = env.register(TokenBridgeContract, ());
    let c = TokenBridgeContractClient::new(env, &id);
    c.initialize(&admin, &relayer);
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
    let id = env.register(TokenBridgeContract, ());
    let c = TokenBridgeContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let r = Address::generate(&env);
    c.initialize(&a, &r);
    c.initialize(&a, &r);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register(TokenBridgeContract, ());
    let c = TokenBridgeContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_add_supported_chain() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.add_supported_chain(&admin, &s(&env, "ethereum"), &1_000_000i128);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_supported_chain_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    c.add_supported_chain(
        &Address::generate(&env),
        &s(&env, "ethereum"),
        &1_000_000i128,
    );
}

#[test]
#[should_panic(expected = "max_daily_limit must be positive")]
fn test_add_supported_chain_zero_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.add_supported_chain(&admin, &s(&env, "ethereum"), &0i128);
}

#[test]
#[should_panic(expected = "max_daily_limit must be positive")]
fn test_add_supported_chain_negative_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.add_supported_chain(&admin, &s(&env, "ethereum"), &-1i128);
}

#[test]
fn test_get_deposit_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_deposit(&999u64).is_none());
}
#[test]
#[should_panic(expected = "bridge fee calculation overflow")]
fn test_deposit_for_bridge_overflow() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);

    // Support the chain
    let chain = s(&env, "ethereum");
    c.add_supported_chain(&admin, &chain, &i128::MAX);

    // Attempt deposit with MAX i128 should overflow when multiplied by fee_bps (default 50)
    c.deposit_for_bridge(
        &Address::generate(&env),
        &token_id,
        &i128::MAX,
        &chain,
        &s(&env, "0x123"),
    );
}
#[test]
fn test_deposit_for_bridge_normal() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);

    let chain = s(&env, "ethereum");
    c.add_supported_chain(&admin, &chain, &1_000_000i128);

    let amount = 10_000i128;
    let sender = Address::generate(&env);

    // Mint tokens to sender (requires token_admin auth)
    let token_stellar_client = token::StellarAssetClient::new(&env, &token_id);
    token_stellar_client.mint(&sender, &amount);

    // fee_bps is 50 (0.5%), so fee should be 10,000 * 50 / 10,000 = 50
    // net_amount should be 10,000 - 50 = 9,950
    let deposit_id = c.deposit_for_bridge(&sender, &token_id, &amount, &chain, &s(&env, "0x123"));

    let deposit = c.get_deposit(&deposit_id).unwrap();
    assert_eq!(deposit.amount, 9950);
    assert_eq!(deposit.bridge_fee, 50);
}

#[test]
#[should_panic(expected = "daily transfer limit exceeded for chain")]
fn test_daily_limit_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);

    let chain = s(&env, "ethereum");
    c.add_supported_chain(&admin, &chain, &100i128);

    let sender = Address::generate(&env);
    let token_stellar_client = token::StellarAssetClient::new(&env, &token_id);
    token_stellar_client.mint(&sender, &200i128);

    // First deposit uses 80 (within limit of 100)
    c.deposit_for_bridge(&sender, &token_id, &80i128, &chain, &s(&env, "0x1"));

    // Second deposit of 30 would push total to 110, exceeding limit
    c.deposit_for_bridge(&sender, &token_id, &30i128, &chain, &s(&env, "0x2"));
}
