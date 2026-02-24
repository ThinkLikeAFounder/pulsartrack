//! PulsarTrack - Subscription Benefits (Soroban)
//! Manages benefits, perks, and feature access tied to subscription tiers on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct Benefit {
    pub benefit_id: u32,
    pub name: String,
    pub description: String,
    pub min_tier: u32, // 0=Starter, 1=Growth, 2=Business, 3=Enterprise
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct BenefitUsage {
    pub subscriber: Address,
    pub benefit_id: u32,
    pub uses_this_period: u32,
    pub max_uses_per_period: u32,
    pub period_reset_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    BenefitCounter,
    Benefit(u32),
    BenefitUsage(Address, u32), // subscriber, benefit_id
    TierBenefits(u32),          // tier -> list of benefit IDs
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct SubscriptionBenefitsContract;

#[contractimpl]
impl SubscriptionBenefitsContract {
    pub fn initialize(env: Env, admin: Address) {
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
            .set(&DataKey::BenefitCounter, &0u32);
    }

    pub fn add_benefit(
        env: Env,
        admin: Address,
        name: String,
        description: String,
        min_tier: u32,
    ) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let counter: u32 = env
            .storage()
            .instance()
            .get(&DataKey::BenefitCounter)
            .unwrap_or(0);
        let benefit_id = counter + 1;

        let benefit = Benefit {
            benefit_id,
            name,
            description,
            min_tier,
            is_active: true,
        };

        let _ttl_key = DataKey::Benefit(benefit_id);
        env.storage().persistent().set(&_ttl_key, &benefit);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::BenefitCounter, &benefit_id);

        benefit_id
    }

    pub fn check_benefit_access(
        env: Env,
        _subscriber: Address,
        benefit_id: u32,
        subscriber_tier: u32,
    ) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(benefit) = env
            .storage()
            .persistent()
            .get::<DataKey, Benefit>(&DataKey::Benefit(benefit_id))
        {
            benefit.is_active && subscriber_tier >= benefit.min_tier
        } else {
            false
        }
    }

    pub fn use_benefit(env: Env, subscriber: Address, benefit_id: u32, subscriber_tier: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        subscriber.require_auth();

        if !Self::check_benefit_access(env.clone(), subscriber.clone(), benefit_id, subscriber_tier)
        {
            panic!("access denied");
        }

        let now = env.ledger().timestamp();
        let period_secs = 30 * 24 * 3600u64;

        let key = DataKey::BenefitUsage(subscriber.clone(), benefit_id);
        let mut usage: BenefitUsage =
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(BenefitUsage {
                    subscriber: subscriber.clone(),
                    benefit_id,
                    uses_this_period: 0,
                    max_uses_per_period: 100,
                    period_reset_at: now + period_secs,
                });

        // Reset period if expired
        if now > usage.period_reset_at {
            usage.uses_this_period = 0;
            usage.period_reset_at = now + period_secs;
        }

        if usage.uses_this_period >= usage.max_uses_per_period {
            panic!("usage limit reached");
        }

        usage.uses_this_period += 1;
        env.storage().persistent().set(&key, &usage);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_benefit(env: Env, benefit_id: u32) -> Option<Benefit> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Benefit(benefit_id))
    }

    pub fn get_usage(env: Env, subscriber: Address, benefit_id: u32) -> Option<BenefitUsage> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::BenefitUsage(subscriber, benefit_id))
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
