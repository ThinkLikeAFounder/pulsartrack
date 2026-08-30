#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(env: &Env) -> (VestingScheduleContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);
    let contract_id = env.register(VestingScheduleContract, ());
    let client = VestingScheduleContractClient::new(env, &contract_id);
    client.initialize(&admin);
    (client, admin, token_addr)
}

#[test]
fn test_upsert_schedule_preserves_claimed_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token_addr) = setup(&env);
    let beneficiary = Address::generate(&env);
    let token_client = TokenClient::new(&env, &token_addr);

    mint(&env, &token_addr, &client.address, 200_000);
    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &100_000i128,
        &0u64,
        &100u64,
        &0u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 40;
    });
    assert_eq!(client.claim(&beneficiary), 40_000);

    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &200_000i128,
        &0u64,
        &200u64,
        &0u64,
    );

    let schedule = client.get_schedule(&beneficiary).unwrap();
    assert_eq!(schedule.claimed_amount, 40_000);

    env.ledger().with_mut(|li| {
        li.timestamp = 80;
    });
    assert_eq!(client.claim(&beneficiary), 40_000);
    assert_eq!(token_client.balance(&beneficiary), 80_000);
}

#[test]
fn test_claim_full_schedule() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token_addr) = setup(&env);
    let beneficiary = Address::generate(&env);

    mint(&env, &token_addr, &client.address, 100_000);
    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &100_000i128,
        &0u64,
        &100u64,
        &0u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    let claimed = client.claim(&beneficiary);
    assert_eq!(claimed, 100_000);
    assert_eq!(TokenClient::new(&env, &token_addr).balance(&beneficiary), 100_000);
}

#[test]
#[should_panic(expected = "cliff period has not ended")]
fn test_claim_before_cliff_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token_addr) = setup(&env);
    let beneficiary = Address::generate(&env);

    mint(&env, &token_addr, &client.address, 100_000);
    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &100_000i128,
        &0u64,
        &100u64,
        &50u64, // 50s cliff
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 30; // before cliff
    });
    client.claim(&beneficiary);
}

#[test]
#[should_panic(expected = "no tokens claimable")]
fn test_double_claim_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token_addr) = setup(&env);
    let beneficiary = Address::generate(&env);

    mint(&env, &token_addr, &client.address, 100_000);
    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &100_000i128,
        &0u64,
        &100u64,
        &0u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 40;
    });
    client.claim(&beneficiary);
    // Try to claim again at same timestamp — nothing new vested
    client.claim(&beneficiary);
}

#[test]
#[should_panic(expected = "vesting schedule not found")]
fn test_claim_wrong_beneficiary_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token_addr) = setup(&env);
    let beneficiary = Address::generate(&env);
    let wrong = Address::generate(&env);

    mint(&env, &token_addr, &client.address, 100_000);
    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &100_000i128,
        &0u64,
        &100u64,
        &0u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 50;
    });
    client.claim(&wrong);
}
