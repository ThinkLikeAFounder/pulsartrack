#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

fn setup(env: &Env) -> (TreasuryManagerContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();

    let id = env.register(TreasuryManagerContract, ());
    let c = TreasuryManagerContractClient::new(env, &id);
    env.mock_all_auths();
    c.initialize(&admin, &token_addr);
    (c, admin, token_addr)
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _admin, token) = setup(&env);
    assert_eq!(c.get_token(), token);
    let state = c.get_state();
    assert_eq!(state.balance, 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, token) = setup(&env);
    c.initialize(&admin, &token);
}

#[test]
fn test_deposit_and_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, token) = setup(&env);
    let contract_id = c.address.clone();

    mint(&env, &token, &admin, 1_000);

    c.deposit(&admin, &1_000i128);
    assert_eq!(c.get_state().balance, 1_000);

    let recipient = Address::generate(&env);
    c.withdraw(&admin, &recipient, &400i128);

    let state = c.get_state();
    assert_eq!(state.balance, 600);
    assert_eq!(state.total_withdrawn, 400);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&recipient), 400);
    assert_eq!(token_client.balance(&contract_id), 600);
}

#[test]
#[should_panic(expected = "insufficient on-chain token balance")]
fn test_withdraw_rejects_overdraft() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, token) = setup(&env);
    mint(&env, &token, &admin, 500);
    c.deposit(&admin, &500i128);
    // Try to withdraw more than what's actually on-chain
    c.withdraw(&admin, &admin, &600i128);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_withdraw_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _admin, token) = setup(&env);
    let stranger = Address::generate(&env);
    mint(&env, &token, &stranger, 100);
    c.withdraw(&stranger, &stranger, &50i128);
}

#[test]
fn test_sync_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _admin, token) = setup(&env);
    let contract_id = c.address.clone();

    mint(&env, &token, &contract_id, 1_000);
    assert_eq!(c.get_state().balance, 0);

    c.sync_balance();
    assert_eq!(c.get_state().balance, 1_000);
}

#[test]
fn test_sync_balance_permissionless() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _admin, token) = setup(&env);
    let contract_id = c.address.clone();

    mint(&env, &token, &contract_id, 500);
    c.sync_balance();
    assert_eq!(c.get_state().balance, 500);
}

