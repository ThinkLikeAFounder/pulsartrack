#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (TimelockExecutorContractClient, Address, Address) {
    let admin = Address::generate(env);
    let executor = Address::generate(env);

    let contract_id = env.register_contract(None, TimelockExecutorContract);
    let client = TimelockExecutorContractClient::new(env, &contract_id);
    // min_delay=100, max_delay=86400
    client.initialize(&admin, &executor, &100u64, &86_400u64);

    (client, admin, executor)
}

fn make_fn(env: &Env) -> String {
    String::from_str(env, "upgrade")
}

fn make_desc(env: &Env) -> String {
    String::from_str(env, "upgrade contract")
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let executor = Address::generate(&env);

    let contract_id = env.register_contract(None, TimelockExecutorContract);
    let client = TimelockExecutorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &executor, &3600u64, &86_400u64);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let executor = Address::generate(&env);

    let contract_id = env.register_contract(None, TimelockExecutorContract);
    let client = TimelockExecutorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &executor, &3600u64, &86_400u64);
    client.initialize(&admin, &executor, &3600u64, &86_400u64);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let executor = Address::generate(&env);

    let contract_id = env.register_contract(None, TimelockExecutorContract);
    let client = TimelockExecutorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &executor, &3600u64, &86_400u64);
}

// ─── queue ───────────────────────────────────────────────────────────────────

#[test]
fn test_queue_entry() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);
    assert_eq!(entry_id, 1);

    let entry = client.get_entry(&entry_id).unwrap();
    assert_eq!(entry.proposer, admin);
    assert!(matches!(entry.status, TimelockStatus::Queued));
    assert_eq!(entry.eta, 500); // timestamp=0 + delay=500
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_queue_by_non_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    let stranger = Address::generate(&env);
    let target = Address::generate(&env);

    client.queue(&stranger, &target, &make_fn(&env), &make_desc(&env), &500u64);
}

#[test]
#[should_panic(expected = "invalid delay")]
fn test_queue_delay_too_short() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);

    // min_delay=100, so 50 is too short
    client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &50u64);
}

#[test]
#[should_panic(expected = "invalid delay")]
fn test_queue_delay_too_long() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);

    // max_delay=86_400, so 100_000 is too long
    client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &100_000u64);
}

// ─── execute ─────────────────────────────────────────────────────────────────

#[test]
fn test_execute_entry() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, executor) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);

    // Advance past ETA but within grace period
    env.ledger().with_mut(|li| { li.timestamp = 600; });

    client.execute(&executor, &entry_id);

    let entry = client.get_entry(&entry_id).unwrap();
    assert!(matches!(entry.status, TimelockStatus::Executed));
    assert_eq!(entry.executed_at, Some(600));
}

#[test]
#[should_panic(expected = "timelock not expired")]
fn test_execute_too_early() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, executor) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);

    // timestamp is still 0, ETA is 500
    client.execute(&executor, &entry_id);
}

#[test]
#[should_panic(expected = "unauthorized executor")]
fn test_execute_wrong_executor() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);
    let stranger = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);
    env.ledger().with_mut(|li| { li.timestamp = 600; });

    client.execute(&stranger, &entry_id);
}

#[test]
#[should_panic(expected = "grace period expired")]
fn test_execute_after_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, executor) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);

    // grace_period = 172_800 (2 days), ETA = 500
    // Advance way beyond grace period
    env.ledger().with_mut(|li| { li.timestamp = 500 + 172_800 + 1; });

    client.execute(&executor, &entry_id);
}

// ─── cancel ──────────────────────────────────────────────────────────────────

#[test]
fn test_cancel_entry() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);
    client.cancel(&admin, &entry_id);

    let entry = client.get_entry(&entry_id).unwrap();
    assert!(matches!(entry.status, TimelockStatus::Cancelled));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_cancel_by_non_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);
    let stranger = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);
    client.cancel(&stranger, &entry_id);
}

#[test]
#[should_panic(expected = "entry not queued")]
fn test_cancel_already_cancelled() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);
    client.cancel(&admin, &entry_id);
    client.cancel(&admin, &entry_id); // already cancelled
}

// ─── is_ready ────────────────────────────────────────────────────────────────

#[test]
fn test_is_ready_before_eta() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);
    assert!(!client.is_ready(&entry_id)); // timestamp=0 < eta=500
}

#[test]
fn test_is_ready_at_eta() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    let target = Address::generate(&env);

    let entry_id = client.queue(&admin, &target, &make_fn(&env), &make_desc(&env), &500u64);
    env.ledger().with_mut(|li| { li.timestamp = 500; });

    assert!(client.is_ready(&entry_id));
}

#[test]
fn test_is_ready_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    assert!(!client.is_ready(&999u64));
}

// ─── get_entry ───────────────────────────────────────────────────────────────

#[test]
fn test_get_entry_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    assert!(client.get_entry(&999u64).is_none());
}
