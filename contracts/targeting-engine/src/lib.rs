//! PulsarTrack - Targeting Engine (Soroban)
//! Privacy-preserving on-chain targeting configuration for ad campaigns on Stellar.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String,
};

#[contracttype]
#[derive(Clone)]
pub struct TargetingParams {
    pub geographic_targets: String,   // JSON-encoded geo list
    pub interest_segments: String,    // Segment IDs
    pub excluded_segments: String,
    pub min_age: u32,
    pub max_age: u32,
    pub device_types: String,         // mobile, desktop, tablet
    pub languages: String,
    pub min_reputation: u32,
    pub exclude_fraud: bool,
    pub require_kyc: bool,
    pub max_cpm: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct TargetingConfig {
    pub campaign_id: u64,
    pub advertiser: Address,
    pub geographic_targets: String,
    pub interest_segments: String,
    pub excluded_segments: String,
    pub min_age: u32,
    pub max_age: u32,
    pub device_types: String,
    pub operating_systems: String,
    pub languages: String,
    pub min_reputation_score: u32,
    pub exclude_fraud: bool,
    pub require_kyc: bool,
    pub max_cpm: i128,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TargetingScore {
    pub campaign_id: u64,
    pub publisher: Address,
    pub score: u32,   // 0-1000, higher = better match
    pub match_reasons: String,
    pub computed_at: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    TargetingConfig(u64),   // campaign_id
    TargetingScore(u64, Address),  // campaign_id, publisher
    AuthorizedOracle(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct TargetingEngineContract;

#[contractimpl]
impl TargetingEngineContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn add_oracle(env: Env, admin: Address, oracle: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        let _ttl_key = DataKey::AuthorizedOracle(oracle.clone());
        env.storage().persistent().set(&_ttl_key, &true);
        env.storage().persistent().extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }

    pub fn remove_oracle(env: Env, admin: Address, oracle: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().persistent().remove(&DataKey::AuthorizedOracle(oracle));
    }

    pub fn set_targeting(
        env: Env,
        advertiser: Address,
        campaign_id: u64,
        params: TargetingParams,
    ) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        if params.min_age > params.max_age {
            panic!("invalid age range");
        }

        let config = TargetingConfig {
            campaign_id,
            advertiser: advertiser.clone(),
            geographic_targets: params.geographic_targets,
            interest_segments: params.interest_segments,
            excluded_segments: params.excluded_segments,
            min_age: params.min_age,
            max_age: params.max_age,
            device_types: params.device_types,
            operating_systems: String::from_str(&env, ""),
            languages: params.languages,
            min_reputation_score: params.min_reputation,
            exclude_fraud: params.exclude_fraud,
            require_kyc: params.require_kyc,
            max_cpm: params.max_cpm,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::TargetingConfig(campaign_id);
        env.storage().persistent().set(&_ttl_key, &config);
        env.storage().persistent().extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("targeting"), symbol_short!("set")),
            (campaign_id, advertiser),
        );
    }

    pub fn compute_score(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        publisher: Address,
        score: u32,
        match_reasons: String,
    ) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        
        if !env.storage().persistent().has(&DataKey::AuthorizedOracle(oracle.clone())) {
            panic!("unauthorized");
        }

        let targeting_score = TargetingScore {
            campaign_id,
            publisher: publisher.clone(),
            score,
            match_reasons,
            computed_at: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::TargetingScore(campaign_id, publisher);
        env.storage().persistent().set(&_ttl_key, &targeting_score);
        env.storage().persistent().extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }

    pub fn get_targeting(env: Env, campaign_id: u64) -> Option<TargetingConfig> {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::TargetingConfig(campaign_id))
    }

    pub fn get_targeting_score(env: Env, campaign_id: u64, publisher: Address) -> Option<TargetingScore> {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::TargetingScore(campaign_id, publisher))
    }

    pub fn is_publisher_targeted(env: Env, campaign_id: u64, publisher: Address, min_score: u32) -> bool {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(score) = env
            .storage()
            .persistent()
            .get::<DataKey, TargetingScore>(&DataKey::TargetingScore(campaign_id, publisher))
        {
            score.score >= min_score
        } else {
            false
        }
    }
}

mod test;
