#![cfg(test)]
use super::{
    accept_admin as accept_admin_impl, cancel_admin_proposal as cancel_admin_proposal_impl,
    propose_admin as propose_admin_impl,
};
use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    Address, Env,
};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PendingAdmin,
}

const MIN_ADMIN_DELAY_LEDGERS: u32 = 17_280;

#[contract]
pub struct AdminTransferTest;

#[contractimpl]
impl AdminTransferTest {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        propose_admin_impl(
            &env,
            &DataKey::Admin,
            &DataKey::PendingAdmin,
            current_admin,
            new_admin,
        );
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        accept_admin_impl(&env, &DataKey::Admin, &DataKey::PendingAdmin, new_admin);
    }

    pub fn cancel_admin_proposal(env: Env, current_admin: Address) {
        cancel_admin_proposal_impl(&env, &DataKey::Admin, &DataKey::PendingAdmin, current_admin);
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn pending(env: Env) -> Option<(Address, u32)> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    pub fn extend_instance_ttl(env: Env, extend_to: u32) {
        env.storage().instance().extend_ttl(extend_to, extend_to);
    }
}

fn setup(env: &Env) -> (AdminTransferTestClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(AdminTransferTest, ());
    let client = AdminTransferTestClient::new(env, &id);
    client.initialize(&admin);
    client.extend_instance_ttl(&(MIN_ADMIN_DELAY_LEDGERS + 100));
    (client, admin)
}

#[test]
fn test_propose_then_accept_after_timelock() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    client.propose_admin(&admin, &new_admin);

    let pending = client.pending().unwrap();
    assert_eq!(pending.0, new_admin);
    assert_eq!(pending.1, env.ledger().sequence());
    assert_eq!(client.admin(), admin);

    let proposed_at = pending.1;
    env.ledger()
        .with_mut(|li| li.sequence_number = proposed_at + MIN_ADMIN_DELAY_LEDGERS);

    client.accept_admin(&new_admin);

    assert_eq!(client.admin(), new_admin);
    assert!(client.pending().is_none());
}

#[test]
#[should_panic(expected = "admin transfer time lock not elapsed")]
fn test_accept_before_timelock_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    client.propose_admin(&admin, &new_admin);
    let proposed_at = client.pending().unwrap().1;
    env.ledger()
        .with_mut(|li| li.sequence_number = proposed_at + MIN_ADMIN_DELAY_LEDGERS - 1);

    client.accept_admin(&new_admin);
}

#[test]
fn test_cancel_pending_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    client.propose_admin(&admin, &new_admin);
    assert!(client.pending().is_some());

    client.cancel_admin_proposal(&admin);

    assert!(client.pending().is_none());
    assert_eq!(client.admin(), admin);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_non_admin_propose_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.propose_admin(&stranger, &new_admin);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_non_admin_cancel_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let stranger = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.propose_admin(&admin, &new_admin);
    client.cancel_admin_proposal(&stranger);
}

#[test]
#[should_panic(expected = "no pending admin")]
fn test_accept_without_pending_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let new_admin = Address::generate(&env);

    client.accept_admin(&new_admin);
}

#[test]
#[should_panic(expected = "pending admin proposal already exists")]
fn test_duplicate_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    client.propose_admin(&admin, &new_admin);
    client.propose_admin(&admin, &new_admin);
}

#[test]
#[should_panic(expected = "not pending admin")]
fn test_accept_by_non_pending_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let pending_admin = Address::generate(&env);
    let other = Address::generate(&env);

    client.propose_admin(&admin, &pending_admin);
    let proposed_at = client.pending().unwrap().1;
    env.ledger()
        .with_mut(|li| li.sequence_number = proposed_at + MIN_ADMIN_DELAY_LEDGERS);

    client.accept_admin(&other);
}
