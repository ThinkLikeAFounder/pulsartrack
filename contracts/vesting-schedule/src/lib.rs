//! PulsarTrack - Vesting Schedule (Soroban)
//! Enforces cliff + linear vesting for token allocations.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct VestingSchedule {
    pub beneficiary: Address,
    pub token: Address,
    pub total_amount: i128,
    pub claimed_amount: i128,
    pub start_time: u64,
    pub duration: u64,
    pub cliff_duration: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    Schedule(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct VestingScheduleContract;

#[contractimpl]
impl VestingScheduleContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_schedule(
        env: Env,
        admin: Address,
        beneficiary: Address,
        token: Address,
        total_amount: i128,
        start_time: u64,
        duration: u64,
        cliff_duration: u64,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        if total_amount <= 0 {
            panic!("invalid amount");
        }
        if duration == 0 {
            panic!("invalid duration");
        }
        if cliff_duration > duration {
            panic!("invalid cliff");
        }

        let now = env.ledger().timestamp();
        const MAX_BACKDATE_SECS: u64 = 7 * 24 * 3600; // allow up to 7 days backdating
        if start_time + MAX_BACKDATE_SECS < now {
            panic!("start_time too far in the past");
        }

        let key = DataKey::Schedule(beneficiary.clone());
        let existing_claimed = env
            .storage()
            .persistent()
            .get::<DataKey, VestingSchedule>(&key)
            .map(|schedule| schedule.claimed_amount)
            .unwrap_or(0);
        let schedule = VestingSchedule {
            beneficiary,
            token,
            total_amount,
            claimed_amount: existing_claimed,
            start_time,
            duration,
            cliff_duration,
        };
        env.storage().persistent().set(&key, &schedule);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn claim(env: Env, beneficiary: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        beneficiary.require_auth();

        let key = DataKey::Schedule(beneficiary.clone());
        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&key)
            .expect("vesting schedule not found");

        let now = env.ledger().timestamp();

        // Explicit cliff gate: no claims are allowed before cliff ends.
        let cliff_end = schedule
            .start_time
            .checked_add(schedule.cliff_duration)
            .expect("cliff end overflows u64");
        if now < cliff_end {
            panic!("cliff period has not ended");
        }

        let elapsed = now.saturating_sub(schedule.start_time).min(schedule.duration);
        let vested = (schedule.total_amount * elapsed as i128) / schedule.duration as i128;
        let claimable = vested.saturating_sub(schedule.claimed_amount);

        if claimable <= 0 {
            panic!("no tokens claimable");
        }

        schedule.claimed_amount = schedule
            .claimed_amount
            .checked_add(claimable)
            .expect("claimed amount overflows i128");

        env.storage().persistent().set(&key, &schedule);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let token_client = token::Client::new(&env, &schedule.token);
        token_client.transfer(&env.current_contract_address(), &beneficiary, &claimable);

        env.events().publish(
            (symbol_short!("vesting"), symbol_short!("claim")),
            (beneficiary, claimable),
        );

        claimable
    }

    pub fn get_schedule(env: Env, beneficiary: Address) -> Option<VestingSchedule> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary))
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
