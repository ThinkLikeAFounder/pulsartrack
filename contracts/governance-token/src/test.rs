#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env,
};

fn setup(env: &Env) -> (GovernanceTokenContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(GovernanceTokenContract, ());
    let c = GovernanceTokenContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.total_supply(), 0);
    assert_eq!(c.decimals(), 7);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(GovernanceTokenContract, ());
    let c = GovernanceTokenContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register(GovernanceTokenContract, ());
    let c = GovernanceTokenContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_mint() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000_000i128);
    assert_eq!(c.balance(&user), 1_000_000);
    assert_eq!(c.total_supply(), 1_000_000);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_mint_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    c.mint(
        &Address::generate(&env),
        &Address::generate(&env),
        &1_000i128,
    );
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    c.mint(&admin, &from, &1_000i128);
    c.transfer(&from, &to, &400i128);
    assert_eq!(c.balance(&from), 600);
    assert_eq!(c.balance(&to), 400);
}

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000i128);
    c.burn(&user, &300i128);
    assert_eq!(c.balance(&user), 700);
    assert_eq!(c.total_supply(), 700);
}

#[test]
fn test_delegate() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    c.mint(&admin, &delegator, &1_000i128);
    c.delegate(&delegator, &delegate);
    let d = c.get_delegation(&delegator).unwrap();
    assert_eq!(d.delegate, delegate);
    // Delegator loses voting power after delegating
    assert_eq!(c.voting_power(&delegator), 0);
}

#[test]
fn test_revoke_delegation() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    c.mint(&admin, &delegator, &1_000i128);
    c.delegate(&delegator, &delegate);
    c.revoke_delegation(&delegator);
    assert!(c.get_delegation(&delegator).is_none());
}

#[test]
fn test_voting_power_self() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000i128);
    assert_eq!(c.voting_power(&user), 1_000);
}

#[test]
fn test_approve_and_allowance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    c.approve(&owner, &spender, &500i128, &1000u32);
    assert_eq!(c.allowance(&owner, &spender), 500);
}

#[test]
fn test_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let to = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    // expiry well above default ledger sequence (0)
    c.approve(&owner, &spender, &500i128, &1000u32);
    c.transfer_from(&spender, &owner, &to, &200i128);
    assert_eq!(c.balance(&owner), 800);
    assert_eq!(c.balance(&to), 200);
    assert_eq!(c.allowance(&owner, &spender), 300);
}

#[test]
#[should_panic(expected = "allowance expired")]
fn test_transfer_from_expired_allowance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let to = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    // approve with expiry = 5, then advance ledger past it
    c.approve(&owner, &spender, &500i128, &5u32);
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: 6,
        timestamp: env.ledger().timestamp(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3_110_400,
    });
    c.transfer_from(&spender, &owner, &to, &100i128);
}

#[test]
fn test_balance_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.balance(&Address::generate(&env)), 0);
}

#[test]
#[should_panic(expected = "expiry must be a future ledger sequence")]
fn test_approve_expired_expiry_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    // ledger sequence starts at 0; expiry=0 must be rejected
    c.approve(&owner, &spender, &500i128, &0u32);
}

#[test]
#[should_panic(expected = "exceeds max supply")]
fn test_mint_exceeds_max_supply_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    // Any single mint above MAX_SUPPLY is rejected by the cap check.
    c.mint(&admin, &user, &(MAX_SUPPLY + 1));
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_mint_zero_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &0);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_mint_negative_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &-100);
}

#[test]
#[should_panic(expected = "supply overflow")]
fn test_mint_supply_overflow_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    // Reach the cap, then mint a value large enough that
    // current_supply + amount overflows i128 — checked_add must catch this
    // before the cap comparison.
    c.mint(&admin, &user, &MAX_SUPPLY);
    c.mint(&admin, &user, &i128::MAX);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_mint_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &0i128);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_mint_negative_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &-1i128);
}
