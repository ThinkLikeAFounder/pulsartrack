//! PulsarTrack - Publisher Reputation (Soroban)
//! On-chain reputation scoring system for publishers on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct ReputationScore {
    pub publisher: Address,
    pub score: u32, // 0-1000
    pub total_reviews: u64,
    pub positive_reviews: u64,
    pub negative_reviews: u64,
    pub slashes: u32,
    pub uptime_score: u32,  // 0-100
    pub quality_score: u32, // 0-100
    pub last_slash_ledger: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct ReviewEntry {
    pub reviewer: Address,
    pub campaign_id: u64,
    pub positive: bool,
    pub rating: u32, // 1-5
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    ReputationOracle,
    Reputation(Address),
    Review(Address, u64), // publisher, review_index
    ReviewCount(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct PublisherReputationContract;

#[contractimpl]
impl PublisherReputationContract {
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
            .set(&DataKey::ReputationOracle, &oracle);
    }

    pub fn init_publisher(env: Env, publisher: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env
            .storage()
            .persistent()
            .has(&DataKey::Reputation(publisher.clone()))
        {
            panic!("already initialized");
        }

        let score = ReputationScore {
            publisher: publisher.clone(),
            score: 500,
            total_reviews: 0,
            positive_reviews: 0,
            negative_reviews: 0,
            slashes: 0,
            uptime_score: 100,
            quality_score: 100,
            last_slash_ledger: 0,
            last_updated: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Reputation(publisher);
        env.storage().persistent().set(&_ttl_key, &score);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn submit_review(
        env: Env,
        advertiser: Address,
        publisher: Address,
        campaign_id: u64,
        positive: bool,
        rating: u32,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        if rating < 1 || rating > 5 {
            panic!("invalid rating");
        }

        let mut rep: ReputationScore = env
            .storage()
            .persistent()
            .get(&DataKey::Reputation(publisher.clone()))
            .expect("publisher not registered");

        let review = ReviewEntry {
            reviewer: advertiser,
            campaign_id,
            positive,
            rating,
            timestamp: env.ledger().timestamp(),
        };

        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::ReviewCount(publisher.clone()))
            .unwrap_or(0);
        let _ttl_key = DataKey::Review(publisher.clone(), count);
        env.storage().persistent().set(&_ttl_key, &review);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::ReviewCount(publisher.clone());
        env.storage().persistent().set(&_ttl_key, &(count + 1));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        rep.total_reviews += 1;
        if positive {
            rep.positive_reviews += 1;
            // Increase score (max 1000)
            rep.score = (rep.score + rating as u32 * 2).min(1000);
        } else {
            rep.negative_reviews += 1;
            // Decrease score (min 0)
            rep.score = rep.score.saturating_sub(rating as u32 * 3);
        }
        rep.last_updated = env.ledger().timestamp();

        let _ttl_key = DataKey::Reputation(publisher);
        env.storage().persistent().set(&_ttl_key, &rep);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn slash_publisher(env: Env, oracle: Address, publisher: Address, penalty: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::ReputationOracle)
            .unwrap();
        if oracle != stored_oracle {
            panic!("unauthorized");
        }

        let mut rep: ReputationScore = env
            .storage()
            .persistent()
            .get(&DataKey::Reputation(publisher.clone()))
            .expect("publisher not registered");

        let current_ledger = env.ledger().sequence();
        if current_ledger <= rep.last_slash_ledger + 100 {
            panic!("slash cooldown active");
        }

        let penalty = penalty.min(100);

        rep.slashes += 1;
        rep.score = rep.score.saturating_sub(penalty);
        rep.last_slash_ledger = current_ledger;
        rep.last_updated = env.ledger().timestamp();

        let _ttl_key = DataKey::Reputation(publisher.clone());
        env.storage().persistent().set(&_ttl_key, &rep);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("publisher"), symbol_short!("slashed")),
            (publisher, penalty),
        );
    }

    pub fn update_uptime(env: Env, oracle: Address, publisher: Address, uptime: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::ReputationOracle)
            .unwrap();
        if oracle != stored_oracle {
            panic!("unauthorized");
        }

        if uptime > 100 {
            panic!("invalid uptime");
        }

        let mut rep: ReputationScore = env
            .storage()
            .persistent()
            .get(&DataKey::Reputation(publisher.clone()))
            .expect("publisher not registered");

        rep.uptime_score = uptime;
        if uptime < 90 {
            let penalty = (90 - uptime) * 2;
            rep.score = rep.score.saturating_sub(penalty);
        } else {
            let uptime_weight = uptime / 5; // up to 20 points
            rep.score = (rep.score + uptime_weight).min(1000);
        }
        rep.last_updated = env.ledger().timestamp();

        let _ttl_key = DataKey::Reputation(publisher);
        env.storage().persistent().set(&_ttl_key, &rep);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_reputation(env: Env, publisher: Address) -> Option<ReputationScore> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Reputation(publisher))
    }

    pub fn get_review(env: Env, publisher: Address, index: u64) -> Option<ReviewEntry> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Review(publisher, index))
    }

    pub fn get_review_count(env: Env, publisher: Address) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::ReviewCount(publisher))
            .unwrap_or(0)
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
