#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (GovernanceTokenContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, GovernanceTokenContract);
    let c = GovernanceTokenContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

#[test]
fn test_initialize() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.total_supply(), 0);
    assert_eq!(c.decimals(), 7);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, GovernanceTokenContract);
    let c = GovernanceTokenContractClient::new(&env, &id);
    let a = Address::generate(&env); c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, GovernanceTokenContract);
    let c = GovernanceTokenContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_mint() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000_000i128);
    assert_eq!(c.balance(&user), 1_000_000);
    assert_eq!(c.total_supply(), 1_000_000);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_mint_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.mint(&Address::generate(&env), &Address::generate(&env), &1_000i128);
}

#[test]
fn test_transfer() {
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000i128);
    c.burn(&user, &300i128);
    assert_eq!(c.balance(&user), 700);
    assert_eq!(c.total_supply(), 700);
}

#[test]
fn test_delegate() {
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000i128);
    assert_eq!(c.voting_power(&user), 1_000);
}

#[test]
fn test_approve_and_allowance() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    c.approve(&owner, &spender, &500i128, &1000u32);
    assert_eq!(c.allowance(&owner, &spender), 500);
}

#[test]
fn test_balance_zero() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.balance(&Address::generate(&env)), 0);
}
