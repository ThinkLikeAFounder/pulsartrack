//! PulsarTrack - Analytics Aggregator (Soroban)
//! On-chain analytics aggregation for ad campaigns on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct CampaignAnalytics {
    pub campaign_id: u64,
    pub total_impressions: u64,
    pub total_clicks: u64,
    pub total_conversions: u64,
    pub unique_viewers: u64,
    pub total_spend: i128,
    pub ctr: u32,  // click-through rate * 10000
    pub cvr: u32,  // conversion rate * 10000
    pub cpm: i128, // cost per 1000 impressions
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct HourlyStats {
    pub impressions: u64,
    pub clicks: u64,
    pub spend: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    OracleAddress,
    CampaignAnalytics(u64),
    HourlyStats(u64, u64), // campaign_id, hour
    GlobalStats,
}

#[contracttype]
#[derive(Clone)]
pub struct GlobalStats {
    pub total_campaigns: u64,
    pub total_impressions: u64,
    pub total_clicks: u64,
    pub total_spend: i128,
    pub last_updated: u64,
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct AnalyticsAggregatorContract;

#[contractimpl]
impl AnalyticsAggregatorContract {
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

        let global = GlobalStats {
            total_campaigns: 0,
            total_impressions: 0,
            total_clicks: 0,
            total_spend: 0,
            last_updated: env.ledger().timestamp(),
        };
        env.storage().instance().set(&DataKey::GlobalStats, &global);
    }

    pub fn record_impression(env: Env, caller: Address, campaign_id: u64, spend: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let _stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::OracleAddress)
            .unwrap();
        caller.require_auth();

        let mut analytics: CampaignAnalytics = env
            .storage()
            .persistent()
            .get(&DataKey::CampaignAnalytics(campaign_id))
            .unwrap_or(CampaignAnalytics {
                campaign_id,
                total_impressions: 0,
                total_clicks: 0,
                total_conversions: 0,
                unique_viewers: 0,
                total_spend: 0,
                ctr: 0,
                cvr: 0,
                cpm: 0,
                last_updated: 0,
            });

        analytics.total_impressions += 1;
        analytics.total_spend += spend;
        analytics.last_updated = env.ledger().timestamp();

        if analytics.total_impressions > 0 {
            analytics.ctr = (analytics.total_clicks * 10_000 / analytics.total_impressions) as u32;
            analytics.cpm = (analytics.total_spend * 1_000) / analytics.total_impressions as i128;
        }

        let _ttl_key = DataKey::CampaignAnalytics(campaign_id);
        env.storage().persistent().set(&_ttl_key, &analytics);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Update hourly stats
        let hour = env.ledger().timestamp() / 3600;
        let hourly_key = DataKey::HourlyStats(campaign_id, hour);
        let mut hourly: HourlyStats =
            env.storage()
                .temporary()
                .get(&hourly_key)
                .unwrap_or(HourlyStats {
                    impressions: 0,
                    clicks: 0,
                    spend: 0,
                    timestamp: env.ledger().timestamp(),
                });
        hourly.impressions += 1;
        hourly.spend += spend;
        env.storage().temporary().set(&hourly_key, &hourly);

        // Update global stats
        let mut global: GlobalStats = env.storage().instance().get(&DataKey::GlobalStats).unwrap();
        global.total_impressions += 1;
        global.total_spend += spend;
        global.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::GlobalStats, &global);
    }

    pub fn record_click(env: Env, caller: Address, campaign_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let mut analytics: CampaignAnalytics = env
            .storage()
            .persistent()
            .get(&DataKey::CampaignAnalytics(campaign_id))
            .expect("analytics not found");

        analytics.total_clicks += 1;
        if analytics.total_impressions > 0 {
            analytics.ctr = (analytics.total_clicks * 10_000 / analytics.total_impressions) as u32;
        }
        analytics.last_updated = env.ledger().timestamp();

        let _ttl_key = DataKey::CampaignAnalytics(campaign_id);
        env.storage().persistent().set(&_ttl_key, &analytics);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let mut global: GlobalStats = env.storage().instance().get(&DataKey::GlobalStats).unwrap();
        global.total_clicks += 1;
        env.storage().instance().set(&DataKey::GlobalStats, &global);
    }

    pub fn get_campaign_analytics(env: Env, campaign_id: u64) -> Option<CampaignAnalytics> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::CampaignAnalytics(campaign_id))
    }

    pub fn get_global_stats(env: Env) -> GlobalStats {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::GlobalStats)
            .unwrap_or(GlobalStats {
                total_campaigns: 0,
                total_impressions: 0,
                total_clicks: 0,
                total_spend: 0,
                last_updated: 0,
            })
    }

    pub fn get_hourly_stats(env: Env, campaign_id: u64, hour: u64) -> Option<HourlyStats> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .temporary()
            .get(&DataKey::HourlyStats(campaign_id, hour))
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
