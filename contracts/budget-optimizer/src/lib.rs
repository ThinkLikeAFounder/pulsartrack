//! PulsarTrack - Budget Optimizer (Soroban)
//! Automated campaign budget optimization and allocation on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct BudgetAllocation {
    pub campaign_id: u64,
    pub total_budget: i128,
    pub daily_budget: i128,
    pub hourly_budget: i128,
    pub spent_today: i128,
    pub spent_total: i128,
    pub optimization_mode: OptimizationMode,
    pub target_cpa: i128, // Target cost per acquisition
    pub target_ctr: u32,  // Target CTR * 10000
    pub last_optimized: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum OptimizationMode {
    ManualCpc,      // Manual cost per click
    AutoCpm,        // Automatic CPM
    TargetCpa,      // Target cost per acquisition
    MaxConversions, // Maximize conversions
    MaxReach,       // Maximize reach
}

#[contracttype]
#[derive(Clone)]
pub struct OptimizationLog {
    pub campaign_id: u64,
    pub old_daily_budget: i128,
    pub new_daily_budget: i128,
    pub reason: String,
    pub optimized_at: u64,
}

use soroban_sdk::String;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    OracleAddress,
    Allocation(u64),
    OptLog(u64, u32), // campaign_id, log_index
    OptLogCount(u64),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct BudgetOptimizerContract;

#[contractimpl]
impl BudgetOptimizerContract {
    pub fn initialize(env: Env, admin: Address, oracle: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::OracleAddress, &oracle);
    }

    pub fn set_budget_allocation(
        env: Env,
        advertiser: Address,
        campaign_id: u64,
        total_budget: i128,
        daily_budget: i128,
        optimization_mode: OptimizationMode,
        target_cpa: i128,
        target_ctr: u32,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        if daily_budget > total_budget {
            panic!("daily budget exceeds total");
        }

        let allocation = BudgetAllocation {
            campaign_id,
            total_budget,
            daily_budget,
            hourly_budget: daily_budget / 24,
            spent_today: 0,
            spent_total: 0,
            optimization_mode,
            target_cpa,
            target_ctr,
            last_optimized: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Allocation(campaign_id);
        env.storage().persistent().set(&_ttl_key, &allocation);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn optimize_budget(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        new_daily_budget: i128,
        reason: String,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::OracleAddress)
            .unwrap();
        if oracle != stored_oracle {
            panic!("unauthorized");
        }

        let mut allocation: BudgetAllocation = env
            .storage()
            .persistent()
            .get(&DataKey::Allocation(campaign_id))
            .expect("allocation not found");

        let old_daily = allocation.daily_budget;

        // Ensure new daily budget doesn't exceed total remaining
        let remaining = allocation.total_budget - allocation.spent_total;
        let capped_daily = new_daily_budget.min(remaining);

        allocation.daily_budget = capped_daily;
        allocation.hourly_budget = capped_daily / 24;
        allocation.last_optimized = env.ledger().timestamp();

        let _ttl_key = DataKey::Allocation(campaign_id);
        env.storage().persistent().set(&_ttl_key, &allocation);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Log the optimization
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::OptLogCount(campaign_id))
            .unwrap_or(0);

        let log = OptimizationLog {
            campaign_id,
            old_daily_budget: old_daily,
            new_daily_budget: capped_daily,
            reason,
            optimized_at: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::OptLog(campaign_id, count);
        env.storage().persistent().set(&_ttl_key, &log);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::OptLogCount(campaign_id);
        env.storage().persistent().set(&_ttl_key, &(count + 1));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("budget"), symbol_short!("optimized")),
            (campaign_id, capped_daily),
        );
    }

    pub fn record_spend(env: Env, admin: Address, campaign_id: u64, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();

        let mut allocation: BudgetAllocation = env
            .storage()
            .persistent()
            .get(&DataKey::Allocation(campaign_id))
            .expect("allocation not found");

        allocation.spent_today += amount;
        allocation.spent_total += amount;

        // Reset daily spend if new day
        let hour = env.ledger().timestamp() / 86_400;
        let _ = hour; // Could be used for daily tracking

        let _ttl_key = DataKey::Allocation(campaign_id);
        env.storage().persistent().set(&_ttl_key, &allocation);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_allocation(env: Env, campaign_id: u64) -> Option<BudgetAllocation> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Allocation(campaign_id))
    }

    pub fn can_spend(env: Env, campaign_id: u64, amount: i128) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(alloc) = env
            .storage()
            .persistent()
            .get::<DataKey, BudgetAllocation>(&DataKey::Allocation(campaign_id))
        {
            alloc.spent_today + amount <= alloc.daily_budget
                && alloc.spent_total + amount <= alloc.total_budget
        } else {
            false
        }
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        pulsar_common_admin::propose_admin(
            &env,
            &DataKey::Admin,
            &DataKey::PendingAdmin,
            current_admin,
            new_admin,
        );
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        pulsar_common_admin::accept_admin(&env, &DataKey::Admin, &DataKey::PendingAdmin, new_admin);
    }
}

mod test;
