//! PulsarTrack - Rewards Distributor (Soroban)
//! Distributes PULSAR governance token rewards to ecosystem participants on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct RewardProgram {
    pub program_id: u32,
    pub name: String,
    pub total_budget: i128,
    pub distributed: i128,
    pub reward_per_unit: i128,
    pub start_ledger: u32,
    pub end_ledger: u32,
    pub is_active: bool,
}

use soroban_sdk::String;

#[contracttype]
#[derive(Clone)]
pub struct UserRewards {
    pub user: Address,
    pub total_earned: i128,
    pub total_claimed: i128,
    pub pending: i128,
    pub last_earned: u64,
    pub vesting_start: u64,    // timestamp when vesting begins
    pub vesting_duration: u64, // total vesting period in seconds
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    RewardToken,
    ProgramCounter,
    Program(u32),
    UserRewards(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct RewardsDistributorContract;

#[contractimpl]
impl RewardsDistributorContract {
    pub fn initialize(env: Env, admin: Address, reward_token: Address) {
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
            .set(&DataKey::RewardToken, &reward_token);
        env.storage()
            .instance()
            .set(&DataKey::ProgramCounter, &0u32);
    }

    pub fn create_program(
        env: Env,
        admin: Address,
        name: String,
        budget: i128,
        reward_per_unit: i128,
        duration_ledgers: u32,
    ) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        if budget <= 0 {
            panic!("budget must be positive");
        }
        if reward_per_unit <= 0 {
            panic!("reward_per_unit must be positive");
        }
        if duration_ledgers == 0 {
            panic!("duration_ledgers must be positive");
        }

        let counter: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProgramCounter)
            .unwrap_or(0);
        let program_id = counter + 1;

        let start = env.ledger().sequence();
        let program = RewardProgram {
            program_id,
            name,
            total_budget: budget,
            distributed: 0,
            reward_per_unit,
            start_ledger: start,
            end_ledger: start + duration_ledgers,
            is_active: true,
        };

        let _ttl_key = DataKey::Program(program_id);
        env.storage().persistent().set(&_ttl_key, &program);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::ProgramCounter, &program_id);

        program_id
    }

    pub fn distribute_rewards(
        env: Env,
        admin: Address,
        recipient: Address,
        amount: i128,
        program_id: u32,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        if amount <= 0 {
            panic!("distribution amount must be positive");
        }

        let mut program: RewardProgram = env
            .storage()
            .persistent()
            .get(&DataKey::Program(program_id))
            .expect("program not found");

        if !program.is_active {
            panic!("program not active");
        }

        if program.distributed + amount > program.total_budget {
            panic!("exceeds budget");
        }

        if env.ledger().sequence() > program.end_ledger {
            panic!("program ended");
        }

        program.distributed += amount;
        let _ttl_key = DataKey::Program(program_id);
        env.storage().persistent().set(&_ttl_key, &program);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Update user rewards with vesting schedule
        let key = DataKey::UserRewards(recipient.clone());
        let now = env.ledger().timestamp();
        let ledger_duration = program.end_ledger - program.start_ledger;
        let vesting_duration: u64 = (ledger_duration as u64)
            .checked_mul(5)
            .expect("vesting_duration overflow"); // ~5 seconds per ledger
        let mut rewards: UserRewards =
            env.storage().persistent().get(&key).unwrap_or(UserRewards {
                user: recipient.clone(),
                total_earned: 0,
                total_claimed: 0,
                pending: 0,
                last_earned: 0,
                vesting_start: 0,
                vesting_duration,
            });

        // Initialize vesting_start on first distribution using vesting_start == 0 as sentinel
        if rewards.vesting_start == 0 {
            rewards.vesting_start = now;
            rewards.vesting_duration = vesting_duration;
        }

        rewards.total_earned += amount;
        rewards.pending += amount;
        rewards.last_earned = now;
        env.storage().persistent().set(&key, &rewards);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("rewards"), symbol_short!("earned")),
            (recipient, amount),
        );
    }

    pub fn claim_rewards(env: Env, user: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        user.require_auth();

        let key = DataKey::UserRewards(user.clone());
        let mut rewards: UserRewards = env.storage().persistent().get(&key).expect("no rewards");

        if rewards.pending <= 0 {
            panic!("no pending rewards to claim");
        }

        // Calculate vested amount based on linear vesting schedule
        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(rewards.vesting_start);
        let vesting_fraction = elapsed.min(rewards.vesting_duration);
        let vested_total = if rewards.vesting_duration == 0 {
            rewards.total_earned
        } else {
            // Use a 10_000 bps scaling factor to preserve precision for small rewards
            // with long vesting durations, preventing integer division truncation to 0.
            let vested_bps = rewards.total_earned as u128
                * vesting_fraction as u128
                * 10_000
                / rewards.vesting_duration as u128;
            (vested_bps / 10_000) as i128
        };
        let claimable = vested_total.saturating_sub(rewards.total_claimed);

        if claimable <= 0 {
            panic!("no vested rewards available to claim");
        }

        // --- CEI: Effects before Interactions ---
        rewards.total_claimed += claimable;
        rewards.pending = rewards.pending.saturating_sub(claimable);
        env.storage().persistent().set(&key, &rewards);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // External interaction LAST
        let token: Address = env.storage().instance().get(&DataKey::RewardToken).unwrap();
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &user, &claimable);

        env.events().publish(
            (symbol_short!("rewards"), symbol_short!("claimed")),
            (user, claimable),
        );

        claimable
    }

    pub fn get_program(env: Env, program_id: u32) -> Option<RewardProgram> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Program(program_id))
    }

    pub fn get_user_rewards(env: Env, user: Address) -> Option<UserRewards> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::UserRewards(user))
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
