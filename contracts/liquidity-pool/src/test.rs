#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::{Client as TokenClient, StellarAssetClient}, Address, Env};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (LiquidityPoolContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let contract_id = env.register_contract(None, LiquidityPoolContract);
    let c = LiquidityPoolContractClient::new(env, &contract_id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    c.initialize(&admin, &token);
}

#[test]
fn test_deposit() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    let shares = c.deposit(&provider, &100_000i128);
    assert!(shares > 0);
    let pos = c.get_provider_position(&provider).unwrap();
    assert_eq!(pos.shares, shares);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_liquidity, 100_000);
}

#[test]
fn test_withdraw() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    let shares = c.deposit(&provider, &100_000i128);
    let withdrawn = c.withdraw(&provider, &shares);
    assert_eq!(withdrawn, 100_000);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_liquidity, 0);
}

#[test]
fn test_borrow() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);
    let borrow = c.get_borrow(&1u64).unwrap();
    assert_eq!(borrow.borrowed, 100_000);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_borrowed, 100_000);
}

#[test]
fn test_repay() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);
    c.repay(&borrower, &1u64, &100_000i128);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
}

#[test]
fn test_get_provider_position_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_provider_position(&Address::generate(&env)).is_none());
}

#[test]
fn test_get_borrow_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_borrow(&999u64).is_none());
}
