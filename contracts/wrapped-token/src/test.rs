#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (WrappedTokenContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let relayer = Address::generate(env);
    let id = env.register_contract(None, WrappedTokenContract);
    let c = WrappedTokenContractClient::new(env, &id);
    c.initialize(&admin, &relayer);
    (c, admin, relayer)
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
    let id = env.register_contract(None, WrappedTokenContract);
    let c = WrappedTokenContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let r = Address::generate(&env);
    c.initialize(&a, &r);
    c.initialize(&a, &r);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, WrappedTokenContract);
    let c = WrappedTokenContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_register_wrapped_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let stellar_token = Address::generate(&env);
    c.register_wrapped_token(
        &admin,
        &s(&env, "wETH"),
        &s(&env, "Wrapped Ether"),
        &8u32,
        &s(&env, "ethereum"),
        &s(&env, "0xAddr"),
        &stellar_token,
    );
    let token = c.get_wrapped_token(&s(&env, "wETH")).unwrap();
    assert_eq!(token.decimals, 8);
}

#[test]
fn test_mint_wrapped() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = Address::generate(&env);
    c.register_wrapped_token(
        &admin,
        &s(&env, "wETH"),
        &s(&env, "Wrapped Ether"),
        &8u32,
        &s(&env, "ethereum"),
        &s(&env, "0xAddr"),
        &stellar_token,
    );
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash"),
    );
    assert_eq!(c.get_user_balance(&s(&env, "wETH"), &user), 1_000_000);
}

#[test]
fn test_burn_wrapped() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = Address::generate(&env);
    c.register_wrapped_token(
        &admin,
        &s(&env, "wETH"),
        &s(&env, "Wrapped Ether"),
        &8u32,
        &s(&env, "ethereum"),
        &s(&env, "0xAddr"),
        &stellar_token,
    );
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash"),
    );
    c.burn_wrapped(
        &user,
        &s(&env, "wETH"),
        &400_000i128,
        &s(&env, "0xTargetAddr"),
    );
    assert_eq!(c.get_user_balance(&s(&env, "wETH"), &user), 600_000);
}

#[test]
fn test_get_wrapped_token_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_wrapped_token(&s(&env, "NOPE")).is_none());
}

#[test]
fn test_get_user_balance_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert_eq!(
        c.get_user_balance(&s(&env, "wETH"), &Address::generate(&env)),
        0
    );
}

#[test]
#[should_panic(expected = "source transaction already processed")]
fn test_mint_wrapped_replay_attack_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = Address::generate(&env);
    
    c.register_wrapped_token(
        &admin,
        &s(&env, "wETH"),
        &s(&env, "Wrapped Ether"),
        &8u32,
        &s(&env, "ethereum"),
        &s(&env, "0xAddr"),
        &stellar_token,
    );
    
    // First mint succeeds
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash123"),
    );
    
    // Second mint with same source_tx should fail
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash123"),
    );
}

#[test]
fn test_mint_wrapped_different_tx_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = Address::generate(&env);
    
    c.register_wrapped_token(
        &admin,
        &s(&env, "wETH"),
        &s(&env, "Wrapped Ether"),
        &8u32,
        &s(&env, "ethereum"),
        &s(&env, "0xAddr"),
        &stellar_token,
    );
    
    // First mint
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash123"),
    );
    
    // Second mint with different source_tx should succeed
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &500_000i128,
        &s(&env, "0xTxHash456"),
    );
    
    assert_eq!(c.get_user_balance(&s(&env, "wETH"), &user), 1_500_000);
}

#[test]
#[should_panic(expected = "source transaction already processed")]
fn test_mint_wrapped_replay_different_recipient_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let stellar_token = Address::generate(&env);
    
    c.register_wrapped_token(
        &admin,
        &s(&env, "wETH"),
        &s(&env, "Wrapped Ether"),
        &8u32,
        &s(&env, "ethereum"),
        &s(&env, "0xAddr"),
        &stellar_token,
    );
    
    // First mint to user1
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user1,
        &1_000_000i128,
        &s(&env, "0xTxHash789"),
    );
    
    // Attempt to mint same source_tx to different user should fail
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user2,
        &1_000_000i128,
        &s(&env, "0xTxHash789"),
    );
}

#[test]
#[should_panic(expected = "source transaction already processed")]
fn test_mint_wrapped_replay_different_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = Address::generate(&env);
    
    c.register_wrapped_token(
        &admin,
        &s(&env, "wETH"),
        &s(&env, "Wrapped Ether"),
        &8u32,
        &s(&env, "ethereum"),
        &s(&env, "0xAddr"),
        &stellar_token,
    );
    
    // First mint
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHashABC"),
    );
    
    // Attempt to mint same source_tx with different amount should fail
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &2_000_000i128,
        &s(&env, "0xTxHashABC"),
    );
}
