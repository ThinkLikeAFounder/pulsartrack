#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup(env: &Env) -> (AccessControlContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(AccessControlContract, ());
    let c = AccessControlContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    assert!(c.has_role(&admin, &Role::SuperAdmin));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(AccessControlContract, ());
    let c = AccessControlContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
fn test_grant_role() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let account = Address::generate(&env);
    c.grant_role(&admin, &account, &Role::Operator);
    assert!(c.has_role(&account, &Role::Operator));
}

#[test]
#[should_panic(expected = "cannot grant a role equal to or higher than your own")]
fn test_grant_role_cannot_grant_equal() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, super_admin) = setup(&env);
    let admin_account = Address::generate(&env);
    c.grant_role(&super_admin, &admin_account, &Role::Admin);

    // Admin tries to grant another Admin role — should fail
    let other = Address::generate(&env);
    c.grant_role(&admin_account, &other, &Role::Admin);
}

#[test]
fn test_revoke_role_lower_rank() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, super_admin) = setup(&env);
    let operator = Address::generate(&env);
    c.grant_role(&super_admin, &operator, &Role::Operator);
    assert!(c.has_role(&operator, &Role::Operator));
    c.revoke_role(&super_admin, &operator, &Role::Operator);
    assert!(!c.has_role(&operator, &Role::Operator));
}

#[test]
#[should_panic(expected = "cannot revoke a role equal to or higher than your own")]
fn test_admin_cannot_revoke_super_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, super_admin) = setup(&env);
    let admin_account = Address::generate(&env);
    c.grant_role(&super_admin, &admin_account, &Role::Admin);

    // Admin tries to revoke SuperAdmin — must panic
    c.revoke_role(&admin_account, &super_admin, &Role::SuperAdmin);
}

#[test]
#[should_panic(expected = "cannot revoke a role equal to or higher than your own")]
fn test_admin_cannot_revoke_peer_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, super_admin) = setup(&env);
    let admin_a = Address::generate(&env);
    let admin_b = Address::generate(&env);
    c.grant_role(&super_admin, &admin_a, &Role::Admin);
    c.grant_role(&super_admin, &admin_b, &Role::Admin);

    // admin_a tries to revoke admin_b (same rank) — must panic
    c.revoke_role(&admin_a, &admin_b, &Role::Admin);
}

#[test]
#[should_panic(expected = "cannot revoke a role equal to or higher than your own")]
fn test_self_revocation_blocked() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, super_admin) = setup(&env);
    let admin_account = Address::generate(&env);
    c.grant_role(&super_admin, &admin_account, &Role::Admin);

    // Admin tries to revoke their own role — must panic (equal rank)
    c.revoke_role(&admin_account, &admin_account, &Role::Admin);
}
