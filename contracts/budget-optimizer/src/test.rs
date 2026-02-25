#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup(env: &Env) -> (BudgetOptimizerContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, BudgetOptimizerContract);
    let c = BudgetOptimizerContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
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
    let id = env.register_contract(None, BudgetOptimizerContract);
    let c = BudgetOptimizerContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let o = Address::generate(&env);
    c.initialize(&a, &o);
    c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, BudgetOptimizerContract);
    let c = BudgetOptimizerContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_set_budget_allocation() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(
        &advertiser,
        &1u64,
        &100_000i128,
        &10_000i128,
        &OptimizationMode::ManualCpc,
        &500i128,
        &100u32,
    );
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.total_budget, 100_000);
    assert_eq!(alloc.daily_budget, 10_000);
}

#[test]
fn test_record_spend() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(
        &advertiser,
        &1u64,
        &100_000i128,
        &10_000i128,
        &OptimizationMode::ManualCpc,
        &500i128,
        &100u32,
    );
    c.record_spend(&admin, &1u64, &5_000i128);
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.spent_today, 5_000);
}

#[test]
fn test_record_spend_resets_on_new_day() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(
        &advertiser,
        &1u64,
        &100_000i128,
        &10_000i128,
        &OptimizationMode::ManualCpc,
        &500i128,
        &100u32,
    );

    c.record_spend(&admin, &1u64, &10_000i128);
    assert!(!c.can_spend(&1u64, &1i128));

    env.ledger().with_mut(|li| {
        li.timestamp = 86_400; // next day
    });

    c.record_spend(&admin, &1u64, &2_000i128);
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.spent_today, 2_000);
    assert_eq!(alloc.spent_total, 12_000);
}

#[test]
fn test_can_spend() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(
        &advertiser,
        &1u64,
        &100_000i128,
        &10_000i128,
        &OptimizationMode::ManualCpc,
        &500i128,
        &100u32,
    );
    assert!(c.can_spend(&1u64, &5_000i128));
    assert!(!c.can_spend(&1u64, &110_000i128));
}

#[test]
fn test_can_spend_resets_on_new_day_without_write() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(
        &advertiser,
        &1u64,
        &100_000i128,
        &10_000i128,
        &OptimizationMode::ManualCpc,
        &500i128,
        &100u32,
    );

    c.record_spend(&admin, &1u64, &10_000i128);
    assert!(!c.can_spend(&1u64, &1i128));

    env.ledger().with_mut(|li| {
        li.timestamp = 86_400; // next day
    });

    assert!(c.can_spend(&1u64, &10_000i128));
}

#[test]
fn test_get_allocation_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_allocation(&999u64).is_none());
}

#[test]
fn test_can_spend_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.can_spend(&999u64, &100i128));
}
