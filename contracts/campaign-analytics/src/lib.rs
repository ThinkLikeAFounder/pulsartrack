//! PulsarTrack - Campaign Analytics V4 (Soroban)
//! Advanced campaign analytics with real-time metrics on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct CampaignSnapshot {
    pub campaign_id: u64,
    pub ledger_sequence: u32,
    pub timestamp: u64,
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub spend: i128,
    pub reach: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct RetentionMetrics {
    pub campaign_id: u64,
    pub day_1_retention: u32, // percentage * 100
    pub day_7_retention: u32,
    pub day_30_retention: u32,
    pub avg_session_duration: u64, // seconds
    pub bounce_rate: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct ConversionFunnel {
    pub campaign_id: u64,
    pub impressions: u64,
    pub engagements: u64,
    pub clicks: u64,
    pub sign_ups: u64,
    pub conversions: u64,
    pub conversion_value: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    OracleAddress,
    SnapshotCount(u64),
    Snapshot(u64, u32), // campaign_id, snapshot_index
    RetentionMetrics(u64),
    Funnel(u64),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct CampaignAnalyticsContract;

#[contractimpl]
impl CampaignAnalyticsContract {
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

    pub fn record_snapshot(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        impressions: u64,
        clicks: u64,
        conversions: u64,
        spend: i128,
        reach: u64,
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

        let snapshot = CampaignSnapshot {
            campaign_id,
            ledger_sequence: env.ledger().sequence(),
            timestamp: env.ledger().timestamp(),
            impressions,
            clicks,
            conversions,
            spend,
            reach,
        };

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::SnapshotCount(campaign_id))
            .unwrap_or(0);
        let _ttl_key = DataKey::Snapshot(campaign_id, count);
        env.storage().persistent().set(&_ttl_key, &snapshot);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::SnapshotCount(campaign_id);
        env.storage().persistent().set(&_ttl_key, &(count + 1));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn update_funnel(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        impressions: u64,
        engagements: u64,
        clicks: u64,
        sign_ups: u64,
        conversions: u64,
        conversion_value: i128,
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

        let funnel = ConversionFunnel {
            campaign_id,
            impressions,
            engagements,
            clicks,
            sign_ups,
            conversions,
            conversion_value,
        };

        let _ttl_key = DataKey::Funnel(campaign_id);
        env.storage().persistent().set(&_ttl_key, &funnel);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn update_retention(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        day1: u32,
        day7: u32,
        day30: u32,
        avg_session: u64,
        bounce_rate: u32,
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

        let metrics = RetentionMetrics {
            campaign_id,
            day_1_retention: day1,
            day_7_retention: day7,
            day_30_retention: day30,
            avg_session_duration: avg_session,
            bounce_rate,
        };

        let _ttl_key = DataKey::RetentionMetrics(campaign_id);
        env.storage().persistent().set(&_ttl_key, &metrics);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_snapshot(env: Env, campaign_id: u64, index: u32) -> Option<CampaignSnapshot> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Snapshot(campaign_id, index))
    }

    pub fn get_snapshot_count(env: Env, campaign_id: u64) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::SnapshotCount(campaign_id))
            .unwrap_or(0)
    }

    pub fn get_funnel(env: Env, campaign_id: u64) -> Option<ConversionFunnel> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Funnel(campaign_id))
    }

    pub fn get_retention(env: Env, campaign_id: u64) -> Option<RetentionMetrics> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::RetentionMetrics(campaign_id))
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
