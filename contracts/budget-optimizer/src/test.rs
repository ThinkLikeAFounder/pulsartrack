#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup(env: &Env) -> (BudgetOptimizerContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register(BudgetOptimizerContract, ());
    let c = BudgetOptimizerContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin, oracle)
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
    let id = env.register(BudgetOptimizerContract, ());
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
    let id = env.register(BudgetOptimizerContract, ());
    let c = BudgetOptimizerContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_set_budget_allocation() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.total_budget, 100_000);
    assert_eq!(alloc.daily_budget, 10_000);
}

#[test]
fn test_record_spend() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });
    c.record_spend(&admin, &1u64, &5_000i128);
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.spent_today, 5_000);
}

#[test]
fn test_record_spend_resets_on_new_day() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });

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
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });
    assert!(c.can_spend(&1u64, &5_000i128));
    assert!(!c.can_spend(&1u64, &110_000i128));
}

#[test]
fn test_can_spend_resets_on_new_day_without_write() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });

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
    let (c, _, _) = setup(&env);
    assert!(c.get_allocation(&999u64).is_none());
}

#[test]
fn test_can_spend_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(!c.can_spend(&999u64, &100i128));
}
#[test]
fn test_optimization_does_not_break_reset() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });

    // Spend full budget on Day 0
    c.record_spend(&admin, &1u64, &10_000i128);
    assert!(!c.can_spend(&1u64, &1i128));

    // Advance to Day 1
    env.ledger().with_mut(|li| {
        li.timestamp = 86_400;
    });

    // Optimize budget on Day 1
    c.optimize_budget(
        &oracle,
        &1u64,
        &12_000i128,
        &soroban_sdk::String::from_str(&env, "better performance"),
    );

    // Check if we can spend (should be reset to 0, so 12,000 available)
    assert!(c.can_spend(&1u64, &12_000i128));

    // Record spend on Day 1
    c.record_spend(&admin, &1u64, &5_000i128);
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.spent_today, 5_000);
    assert_eq!(alloc.spent_total, 15_000);
    assert_eq!(alloc.last_reset_day, 1);
}

#[test]
fn test_hourly_budget_remainder_tracked() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);

    // 100 stroops / 24 = 4 per hour, remainder 4
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 1_000i128,
        daily_budget: 100i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.hourly_budget, 4); // 100 / 24 = 4
    assert_eq!(alloc.budget_remainder, 4); // 100 % 24 = 4
                                           // First 4 hours get 5 stroops, remaining 20 hours get 4 = 4*5 + 20*4 = 100
    assert_eq!(
        alloc.budget_remainder * (alloc.hourly_budget + 1)
            + (24 - alloc.budget_remainder) * alloc.hourly_budget,
        alloc.daily_budget
    );
}

#[test]
fn test_hourly_budget_remainder_after_optimization() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });

    // Optimize to a value that doesn't divide evenly by 24
    c.optimize_budget(
        &oracle,
        &1u64,
        &100i128,
        &soroban_sdk::String::from_str(&env, "test remainder"),
    );
    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.hourly_budget, 4);
    assert_eq!(alloc.budget_remainder, 4);
    assert_eq!(
        alloc.budget_remainder * (alloc.hourly_budget + 1)
            + (24 - alloc.budget_remainder) * alloc.hourly_budget,
        alloc.daily_budget
    );
}

// ─── record_spend — stored-admin validation (#481) ───────────────────────────

#[test]
#[should_panic(expected = "unauthorized")]
fn test_record_spend_stranger_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let stranger = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });
    c.record_spend(&stranger, &1u64, &5_000i128);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_record_spend_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });
    c.record_spend(&admin, &1u64, &0i128);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_record_spend_negative_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });
    c.record_spend(&admin, &1u64, &-1_000i128);
}

// ─── set_budget_allocation — ownership check (#482) ──────────────────────────

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_budget_allocation_wrong_advertiser_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let attacker = Address::generate(&env);

    // Original advertiser creates the allocation
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });

    // Attacker tries to overwrite it
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: attacker.clone(),
        campaign_id: 1u64,
        total_budget: 1i128,
        daily_budget: 1i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 0i128,
        target_ctr: 0u32,
    });
}

#[test]
fn test_set_budget_allocation_owner_can_update() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);

    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 100_000i128,
        daily_budget: 10_000i128,
        optimization_mode: OptimizationMode::ManualCpc,
        target_cpa: 500i128,
        target_ctr: 100u32,
    });

    // Same advertiser updates their own allocation
    c.set_budget_allocation(&SetBudgetAllocationArgs {
        advertiser: advertiser.clone(),
        campaign_id: 1u64,
        total_budget: 200_000i128,
        daily_budget: 20_000i128,
        optimization_mode: OptimizationMode::AutoCpm,
        target_cpa: 0i128,
        target_ctr: 0u32,
    });

    let alloc = c.get_allocation(&1u64).unwrap();
    assert_eq!(alloc.total_budget, 200_000);
    assert_eq!(alloc.daily_budget, 20_000);
}
