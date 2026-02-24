//! PulsarTrack - Oracle Integration (Soroban)
//! Price feeds and external data oracle integration on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct PriceFeed {
    pub asset: String,
    pub price_usd: i128, // price in USD * 1e7
    pub confidence: u32, // 0-100
    pub timestamp: u64,
    pub source: String,
}

#[contracttype]
#[derive(Clone)]
pub struct PerformanceData {
    pub campaign_id: u64,
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub fraud_score: u32, // 0-100, lower is better
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    PriceFeed(String),    // asset symbol
    PerformanceData(u64), // campaign_id
    OracleCount,
    AuthorizedOracle(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct OracleIntegrationContract;

#[contractimpl]
impl OracleIntegrationContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::OracleCount, &0u32);
    }

    pub fn add_oracle(env: Env, admin: Address, oracle: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        let _ttl_key = DataKey::AuthorizedOracle(oracle.clone());
        env.storage().persistent().set(&_ttl_key, &true);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::OracleCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::OracleCount, &(count + 1));
    }

    pub fn remove_oracle(env: Env, admin: Address, oracle: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage()
            .persistent()
            .remove(&DataKey::AuthorizedOracle(oracle));
    }

    pub fn update_price(
        env: Env,
        oracle: Address,
        asset: String,
        price_usd: i128,
        confidence: u32,
        source: String,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        Self::_require_oracle(&env, &oracle);

        let feed = PriceFeed {
            asset: asset.clone(),
            price_usd,
            confidence,
            timestamp: env.ledger().timestamp(),
            source,
        };

        let _ttl_key = DataKey::PriceFeed(asset.clone());
        env.storage().persistent().set(&_ttl_key, &feed);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("price")),
            (asset, price_usd),
        );
    }

    pub fn update_performance(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        impressions: u64,
        clicks: u64,
        conversions: u64,
        fraud_score: u32,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        Self::_require_oracle(&env, &oracle);

        let data = PerformanceData {
            campaign_id,
            impressions,
            clicks,
            conversions,
            fraud_score,
            timestamp: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::PerformanceData(campaign_id);
        env.storage().persistent().set(&_ttl_key, &data);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("perf")),
            campaign_id,
        );
    }

    pub fn get_price(env: Env, asset: String) -> Option<PriceFeed> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::PriceFeed(asset))
    }

    pub fn get_performance(env: Env, campaign_id: u64) -> Option<PerformanceData> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::PerformanceData(campaign_id))
    }

    pub fn is_oracle_authorized(env: Env, oracle: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::AuthorizedOracle(oracle))
            .unwrap_or(false)
    }

    fn _require_oracle(env: &Env, oracle: &Address) {
        let is_auth: bool = env
            .storage()
            .persistent()
            .get(&DataKey::AuthorizedOracle(oracle.clone()))
            .unwrap_or(false);
        if !is_auth {
            panic!("not authorized oracle");
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
