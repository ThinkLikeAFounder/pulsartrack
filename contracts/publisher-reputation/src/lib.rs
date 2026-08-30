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
    pub uptime_contribution: u32, // Track the current uptime contribution to score
    pub last_uptime_ledger: Option<u32>,
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
    ReviewRecord(Address, Address, u64), // reviewer, publisher, campaign_id - to prevent duplicates per campaign
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;
const SLASH_COOLDOWN_LEDGERS: u32 = 17_280; // ~1 day at 5 s/ledger

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
        publisher.require_auth();
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
            uptime_contribution: 0,
            last_uptime_ledger: None,
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

        if !(1..=5).contains(&rating) {
            panic!("invalid rating");
        }

        // Check if reviewer has already submitted a review for this publisher in this campaign
        let review_key_check =
            DataKey::ReviewRecord(advertiser.clone(), publisher.clone(), campaign_id);
        if env.storage().persistent().has(&review_key_check) {
            panic!("reviewer has already submitted a review for this publisher in this campaign");
        }

        let mut rep: ReputationScore = env
            .storage()
            .persistent()
            .get(&DataKey::Reputation(publisher.clone()))
            .expect("publisher not registered");

        let review = ReviewEntry {
            reviewer: advertiser.clone(),
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

        // Mark that this reviewer has reviewed this publisher for this campaign
        let review_key = DataKey::ReviewRecord(advertiser.clone(), publisher.clone(), campaign_id);
        env.storage().persistent().set(&review_key, &true);
        env.storage().persistent().extend_ttl(
            &review_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        rep.total_reviews += 1;
        if positive {
            rep.positive_reviews += 1;
            // Increase score (max 1000)
            rep.score = (rep.score + rating * 2).min(1000);
        } else {
            rep.negative_reviews += 1;
            // Decrease score (min 0)
            rep.score = rep.score.saturating_sub(rating * 3);
        }
        rep.last_updated = env.ledger().timestamp();

        let _ttl_key = DataKey::Reputation(publisher.clone());
        env.storage().persistent().set(&_ttl_key, &rep);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("review"), symbol_short!("submitted")),
            (advertiser, publisher, rating, rep.score),
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
        if current_ledger <= rep.last_slash_ledger + SLASH_COOLDOWN_LEDGERS {
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

        let current_ledger = env.ledger().sequence();
        if let Some(last_ledger) = rep.last_uptime_ledger {
            if current_ledger <= last_ledger + 17_280 {
                panic!("uptime update cooldown active");
            }
        }

        // Remove the previous uptime contribution from the score
        rep.score = rep.score.saturating_sub(rep.uptime_contribution);

        // Calculate new uptime contribution
        let new_contribution = if uptime >= 90 {
            uptime / 5 // up to 20 points for uptime >= 90
        } else {
            0 // no bonus for uptime < 90
        };

        // Apply the new uptime contribution
        rep.score = (rep.score + new_contribution).min(1000);
        rep.uptime_score = uptime;
        rep.uptime_contribution = new_contribution;
        rep.last_updated = env.ledger().timestamp();
        rep.last_uptime_ledger = Some(current_ledger);

        let _ttl_key = DataKey::Reputation(publisher.clone());
        env.storage().persistent().set(&_ttl_key, &rep);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("uptime"), symbol_short!("updated")),
            (publisher, uptime),
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
