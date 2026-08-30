#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(
    env: &Env,
) -> (
    RewardsDistributorContractClient<'_>,
    Address,
    Address,
    Address,
) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let id = env.register(RewardsDistributorContract, ());
    let c = RewardsDistributorContractClient::new(env, &id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
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
    let (c, admin, _, token) = setup(&env);
    c.initialize(&admin, &token);
}

#[test]
fn test_create_program() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let pid = c.create_program(
        &admin,
        &s(&env, "Staking"),
        &1_000_000i128,
        &100i128,
        &10_000u32,
    );
    let prog = c.get_program(&pid).unwrap();
    assert_eq!(prog.total_budget, 1_000_000);
}

#[test]
fn test_distribute_rewards() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    let recipient = Address::generate(&env);
    let contract_addr = c.address.clone();
    mint(&env, &token, &contract_addr, 10_000_000);
    c.create_program(
        &admin,
        &s(&env, "Staking"),
        &1_000_000i128,
        &100i128,
        &10_000u32,
    );
    c.distribute_rewards(&admin, &recipient, &5_000i128, &1u32);
    let rewards = c.get_user_rewards(&recipient).unwrap();
    assert_eq!(rewards.total_earned, 5_000);
}

#[test]
#[should_panic(expected = "distribution amount must be positive")]
fn test_distribute_rewards_rejects_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let recipient = Address::generate(&env);
    c.create_program(
        &admin,
        &s(&env, "Staking"),
        &1_000_000i128,
        &100i128,
        &10_000u32,
    );
    c.distribute_rewards(&admin, &recipient, &-5_000i128, &1u32);
}

#[test]
#[should_panic(expected = "distribution amount must be positive")]
fn test_distribute_rewards_rejects_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let recipient = Address::generate(&env);
    c.create_program(
        &admin,
        &s(&env, "Staking"),
        &1_000_000i128,
        &100i128,
        &10_000u32,
    );
    c.distribute_rewards(&admin, &recipient, &0i128, &1u32);
}

#[test]
fn test_get_program_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_program(&999u32).is_none());
}

#[test]
fn test_get_user_rewards_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_user_rewards(&Address::generate(&env)).is_none());
}

#[test]
fn test_claim_rewards_caps_accrual_at_vesting_end() {
    let env = Env::default();
    env.mock_all_auths();

    let (c, admin, _, token) = setup(&env);
    let recipient = Address::generate(&env);

    mint(&env, &token, &c.address, 10_000_000);
    c.create_program(
        &admin,
        &s(&env, "Staking"),
        &1_000_000i128,
        &100i128,
        &10_000u32,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    c.distribute_rewards(&admin, &recipient, &5_000i128, &1u32);

    // Move beyond vesting end (2 years); claim should still cap at total earned.
    env.ledger().with_mut(|li| {
        li.timestamp = 100 + (365 * 24 * 3600 * 2);
    });
    let claimed = c.claim_rewards(&recipient);
    assert_eq!(claimed, 5_000);

    let rewards = c.get_user_rewards(&recipient).unwrap();
    assert_eq!(rewards.total_claimed, 5_000);
    assert_eq!(rewards.total_earned, 5_000);
}

#[test]
#[should_panic(expected = "no pending rewards to claim")]
fn test_claim_rewards_cannot_exceed_total_earned_after_vesting_end() {
    let env = Env::default();
    env.mock_all_auths();

    let (c, admin, _, token) = setup(&env);
    let recipient = Address::generate(&env);

    mint(&env, &token, &c.address, 10_000_000);
    c.create_program(
        &admin,
        &s(&env, "Staking"),
        &1_000_000i128,
        &100i128,
        &10_000u32,
    );
    c.distribute_rewards(&admin, &recipient, &1_000i128, &1u32);

    env.ledger().with_mut(|li| {
        li.timestamp += 365 * 24 * 3600 * 2;
    });

    let first_claim = c.claim_rewards(&recipient);
    assert_eq!(first_claim, 1_000);

    // Any additional claim attempt after full vesting is exhausted must fail.
    c.claim_rewards(&recipient);
}
