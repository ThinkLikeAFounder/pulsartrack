#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (WrappedTokenContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let relayer = Address::generate(env);
    let id = env.register(WrappedTokenContract, ());
    let c = WrappedTokenContractClient::new(env, &id);
    c.initialize(&admin, &relayer);
    (c, admin, relayer)
}

/// Deploy a Stellar asset whose admin is the wrapped-token contract itself so that
/// mint_wrapped (StellarAssetClient::mint) and burn_wrapped (StellarAssetClient::clawback)
/// are authorised.
fn deploy_stellar_token(env: &Env, contract_admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(contract_admin.clone())
        .address()
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
    let id = env.register(WrappedTokenContract, ());
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
    let id = env.register(WrappedTokenContract, ());
    let c = WrappedTokenContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_register_wrapped_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let stellar_token = deploy_stellar_token(&env, &c.address);
    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
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
    let stellar_token = deploy_stellar_token(&env, &c.address);
    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash"),
    );
    // get_user_balance now reads from the real stellar token contract
    assert_eq!(c.get_user_balance(&s(&env, "wETH"), &user), 1_000_000);
}

#[test]
fn test_burn_wrapped() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = deploy_stellar_token(&env, &c.address);
    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
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
    // After clawback, the real token balance should be reduced
    assert_eq!(c.get_user_balance(&s(&env, "wETH"), &user), 600_000);
}

#[test]
fn test_set_relayer_rotates_mint_authority() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let new_relayer = Address::generate(&env);
    let user = Address::generate(&env);
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );
    c.set_relayer(&admin, &new_relayer);
    c.mint_wrapped(
        &new_relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xRotatedTx"),
    );

    assert_eq!(c.get_user_balance(&s(&env, "wETH"), &user), 1_000_000);
}

#[test]
#[should_panic(expected = "unauthorized relayer")]
fn test_old_relayer_after_rotation_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, old_relayer) = setup(&env);
    let new_relayer = Address::generate(&env);
    let user = Address::generate(&env);
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );
    c.set_relayer(&admin, &new_relayer);
    c.mint_wrapped(
        &old_relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xOldRelayerTx"),
    );
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_relayer_by_stranger_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let stranger = Address::generate(&env);
    let new_relayer = Address::generate(&env);

    c.set_relayer(&stranger, &new_relayer);
}

#[test]
#[should_panic(expected = "minting paused")]
fn test_mint_wrapped_paused_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );
    c.set_minting_paused(&admin, &true);
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xPausedTx"),
    );
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_minting_paused_by_stranger_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let stranger = Address::generate(&env);

    c.set_minting_paused(&stranger, &true);
}

#[test]
fn test_mint_wrapped_after_unpause_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );
    c.set_minting_paused(&admin, &true);
    c.set_minting_paused(&admin, &false);
    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xUnpausedTx"),
    );

    assert_eq!(c.get_user_balance(&s(&env, "wETH"), &user), 1_000_000);
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
    // Returns 0 when the token symbol is not registered
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
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );

    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash123"),
    );
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
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );

    c.mint_wrapped(
        &relayer,
        &s(&env, "wETH"),
        &user,
        &1_000_000i128,
        &s(&env, "0xTxHash123"),
    );
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
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );

    c.mint_wrapped(&relayer, &s(&env, "wETH"), &user1, &1_000_000i128, &s(&env, "0xTxHash789"));
    c.mint_wrapped(&relayer, &s(&env, "wETH"), &user2, &1_000_000i128, &s(&env, "0xTxHash789"));
}

#[test]
#[should_panic(expected = "source transaction already processed")]
fn test_mint_wrapped_replay_different_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, relayer) = setup(&env);
    let user = Address::generate(&env);
    let stellar_token = deploy_stellar_token(&env, &c.address);

    c.register_wrapped_token(
        &admin,
        &RegisterTokenParams {
            symbol: s(&env, "wETH"),
            name: s(&env, "Wrapped Ether"),
            decimals: 8u32,
            underlying_chain: s(&env, "ethereum"),
            underlying_address: s(&env, "0xAddr"),
            stellar_token: stellar_token.clone(),
        },
    );

    c.mint_wrapped(&relayer, &s(&env, "wETH"), &user, &1_000_000i128, &s(&env, "0xTxHashABC"));
    c.mint_wrapped(&relayer, &s(&env, "wETH"), &user, &2_000_000i128, &s(&env, "0xTxHashABC"));
}
