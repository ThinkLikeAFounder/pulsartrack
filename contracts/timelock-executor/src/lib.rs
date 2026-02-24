//! PulsarTrack - Timelock Executor (Soroban)
//! Time-locked execution of governance decisions on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum TimelockStatus {
    Queued,
    Executed,
    Cancelled,
    Expired,
}

#[contracttype]
#[derive(Clone)]
pub struct TimelockEntry {
    pub entry_id: u64,
    pub proposer: Address,
    pub target_contract: Address,
    pub function_name: String,
    pub description: String,
    pub eta: u64,          // Earliest time of execution (timestamp)
    pub grace_period: u64, // How long after ETA it can still be executed
    pub status: TimelockStatus,
    pub queued_at: u64,
    pub executed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    ExecutorAddress,
    MinDelay,
    MaxDelay,
    GracePeriod,
    EntryCounter,
    Entry(u64),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct TimelockExecutorContract;

#[contractimpl]
impl TimelockExecutorContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        executor: Address,
        min_delay_secs: u64,
        max_delay_secs: u64,
    ) {
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
            .set(&DataKey::ExecutorAddress, &executor);
        env.storage()
            .instance()
            .set(&DataKey::MinDelay, &min_delay_secs);
        env.storage()
            .instance()
            .set(&DataKey::MaxDelay, &max_delay_secs);
        env.storage()
            .instance()
            .set(&DataKey::GracePeriod, &172_800u64); // 2 days
        env.storage().instance().set(&DataKey::EntryCounter, &0u64);
    }

    pub fn queue(
        env: Env,
        proposer: Address,
        target_contract: Address,
        function_name: String,
        description: String,
        delay_secs: u64,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        proposer.require_auth();

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if proposer != admin {
            panic!("unauthorized");
        }

        let min_delay: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MinDelay)
            .unwrap_or(86_400);
        let max_delay: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MaxDelay)
            .unwrap_or(2_592_000);

        if delay_secs < min_delay || delay_secs > max_delay {
            panic!("invalid delay");
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::EntryCounter)
            .unwrap_or(0);
        let entry_id = counter + 1;

        let now = env.ledger().timestamp();
        let grace: u64 = env.storage().instance().get(&DataKey::GracePeriod).unwrap();

        let entry = TimelockEntry {
            entry_id,
            proposer: proposer.clone(),
            target_contract,
            function_name,
            description,
            eta: now + delay_secs,
            grace_period: grace,
            status: TimelockStatus::Queued,
            queued_at: now,
            executed_at: None,
        };

        let _ttl_key = DataKey::Entry(entry_id);
        env.storage().persistent().set(&_ttl_key, &entry);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::EntryCounter, &entry_id);

        env.events().publish(
            (symbol_short!("timelock"), symbol_short!("queued")),
            (entry_id, proposer),
        );

        entry_id
    }

    pub fn execute(env: Env, executor: Address, entry_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        executor.require_auth();

        let stored_executor: Address = env
            .storage()
            .instance()
            .get(&DataKey::ExecutorAddress)
            .unwrap();
        if executor != stored_executor {
            panic!("unauthorized executor");
        }

        let mut entry: TimelockEntry = env
            .storage()
            .persistent()
            .get(&DataKey::Entry(entry_id))
            .expect("entry not found");

        if entry.status != TimelockStatus::Queued {
            panic!("entry not queued");
        }

        let now = env.ledger().timestamp();

        if now < entry.eta {
            panic!("timelock not expired");
        }

        if now > entry.eta + entry.grace_period {
            entry.status = TimelockStatus::Expired;
            let _ttl_key = DataKey::Entry(entry_id);
            env.storage().persistent().set(&_ttl_key, &entry);
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
            panic!("grace period expired");
        }

        entry.status = TimelockStatus::Executed;
        entry.executed_at = Some(now);
        let _ttl_key = DataKey::Entry(entry_id);
        env.storage().persistent().set(&_ttl_key, &entry);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("timelock"), symbol_short!("executed")),
            entry_id,
        );
    }

    pub fn cancel(env: Env, admin: Address, entry_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut entry: TimelockEntry = env
            .storage()
            .persistent()
            .get(&DataKey::Entry(entry_id))
            .expect("entry not found");

        if entry.status != TimelockStatus::Queued {
            panic!("entry not queued");
        }

        entry.status = TimelockStatus::Cancelled;
        let _ttl_key = DataKey::Entry(entry_id);
        env.storage().persistent().set(&_ttl_key, &entry);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_entry(env: Env, entry_id: u64) -> Option<TimelockEntry> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Entry(entry_id))
    }

    pub fn is_ready(env: Env, entry_id: u64) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(entry) = env
            .storage()
            .persistent()
            .get::<DataKey, TimelockEntry>(&DataKey::Entry(entry_id))
        {
            let now = env.ledger().timestamp();
            entry.status == TimelockStatus::Queued
                && now >= entry.eta
                && now <= entry.eta + entry.grace_period
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
