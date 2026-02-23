#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup(env: &Env) -> (GovernanceCoreContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, GovernanceCoreContract);
    let c = GovernanceCoreContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, GovernanceCoreContract);
    let c = GovernanceCoreContractClient::new(&env, &id);
    let a = Address::generate(&env); c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, GovernanceCoreContract);
    let c = GovernanceCoreContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_grant_role() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let account = Address::generate(&env);
    c.grant_role(&admin, &account, &Role::Operator, &None);
    assert!(c.has_role(&account, &Role::Operator));
}

#[test]
fn test_grant_role_with_expiry() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let account = Address::generate(&env);
    c.grant_role(&admin, &account, &Role::Operator, &Some(86_400u64));
    let grant = c.get_role_grant(&account, &Role::Operator).unwrap();
    assert!(grant.expires_at.is_some());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_grant_role_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.grant_role(&Address::generate(&env), &Address::generate(&env), &Role::Operator, &None);
}

#[test]
fn test_revoke_role() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let account = Address::generate(&env);
    c.grant_role(&admin, &account, &Role::Operator, &None);
    assert!(c.has_role(&account, &Role::Operator));
    c.revoke_role(&admin, &account, &Role::Operator);
    assert!(!c.has_role(&account, &Role::Operator));
}

#[test]
fn test_has_role_false() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.has_role(&Address::generate(&env), &Role::Operator));
}

#[test]
fn test_update_params() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let params = GovernanceParams {
        min_proposal_threshold: 1000,
        voting_period_ledgers: 86_400,
        quorum_pct: 30,
        pass_threshold_pct: 60,
        timelock_ledgers: 3600,
        max_active_proposals: 10,
    };
    c.update_params(&admin, &params);
    let stored = c.get_params();
    assert_eq!(stored.voting_period_ledgers, 86_400);
}

#[test]
fn test_get_role_grant_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_role_grant(&Address::generate(&env), &Role::Operator).is_none());
}
