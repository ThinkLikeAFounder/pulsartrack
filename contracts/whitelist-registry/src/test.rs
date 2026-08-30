#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env,
};

fn setup(env: &Env) -> (WhitelistRegistryContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(WhitelistRegistryContract, ());
    let c = WhitelistRegistryContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
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
    let (c, admin) = setup(&env);
    c.initialize(&admin);
}

#[test]
fn test_add_and_is_whitelisted() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let addr = Address::generate(&env);
    c.add_to_whitelist(&admin, &addr, &ListType::Publisher);
    assert!(c.is_whitelisted(&addr, &ListType::Publisher));
}

#[test]
fn test_remove_schedules_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let addr = Address::generate(&env);
    c.add_to_whitelist(&admin, &addr, &ListType::Publisher);

    c.remove_from_whitelist(&admin, &addr, &ListType::Publisher);

    // Still whitelisted during grace period
    assert!(c.is_whitelisted(&addr, &ListType::Publisher));

    let entry = c.get_entry(&addr, &ListType::Publisher).unwrap();
    assert!(entry.removal_scheduled_at.is_some());
}

#[test]
fn test_is_whitelisted_false_after_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let addr = Address::generate(&env);
    c.add_to_whitelist(&admin, &addr, &ListType::Publisher);
    c.remove_from_whitelist(&admin, &addr, &ListType::Publisher);

    // Advance ledger time past the grace period
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + GRACE_PERIOD_SECS + 1,
        protocol_version: env.ledger().protocol_version(),
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 6_312_000,
    });

    assert!(!c.is_whitelisted(&addr, &ListType::Publisher));
}

#[test]
fn test_purge_after_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let addr = Address::generate(&env);
    c.add_to_whitelist(&admin, &addr, &ListType::Publisher);
    c.remove_from_whitelist(&admin, &addr, &ListType::Publisher);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + GRACE_PERIOD_SECS + 1,
        protocol_version: env.ledger().protocol_version(),
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 6_312_000,
    });

    c.purge_entry(&addr, &ListType::Publisher);
    assert!(c.get_entry(&addr, &ListType::Publisher).is_none());
}

#[test]
#[should_panic(expected = "grace period has not elapsed yet")]
fn test_purge_before_grace_period_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let addr = Address::generate(&env);
    c.add_to_whitelist(&admin, &addr, &ListType::Publisher);
    c.remove_from_whitelist(&admin, &addr, &ListType::Publisher);
    // Attempt purge immediately — should panic
    c.purge_entry(&addr, &ListType::Publisher);
}

#[test]
fn test_remove_is_idempotent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let addr = Address::generate(&env);
    c.add_to_whitelist(&admin, &addr, &ListType::Publisher);
    c.remove_from_whitelist(&admin, &addr, &ListType::Publisher);
    let first = c
        .get_entry(&addr, &ListType::Publisher)
        .unwrap()
        .removal_scheduled_at;

    // Second call must not update the scheduled timestamp
    c.remove_from_whitelist(&admin, &addr, &ListType::Publisher);
    let second = c
        .get_entry(&addr, &ListType::Publisher)
        .unwrap()
        .removal_scheduled_at;

    assert_eq!(first, second);
}
