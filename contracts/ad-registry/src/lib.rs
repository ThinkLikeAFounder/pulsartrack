//! PulsarTrack - Ad Content Registry (Soroban)
//! Manages ad creative assets, validation, and performance tracking on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

// ============================================================
// Data Types
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum ContentStatus {
    Pending,
    Approved,
    Rejected,
    Suspended,
    Archived,
}

#[contracttype]
#[derive(Clone)]
pub enum ContentFormat {
    Image,
    Video,
    Text,
    Native,
}

#[contracttype]
#[derive(Clone)]
pub struct AdContent {
    pub campaign_id: u64,
    pub owner: Address,
    pub ipfs_hash: String,
    pub format: ContentFormat,
    pub size: u64,
    pub status: ContentStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub flags_count: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct ContentMetadata {
    pub title: String,
    pub description: String,
    pub call_to_action: String,
    pub landing_url: String,
}

#[contracttype]
#[derive(Clone)]
pub struct ContentPerformance {
    pub total_views: u64,
    pub total_clicks: u64,
    pub unique_viewers: u64,
    pub click_through_rate: u64, // Multiplied by 10000 for precision (e.g., 525 = 5.25%)
    pub last_shown: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct FlagRecord {
    pub reason: String,
    pub timestamp: u64,
    pub verified: bool,
}

// ============================================================
// Storage Keys
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    ContentNonce,
    MinContentSize,
    MaxContentSize,
    FlagThreshold,
    Content(u64),
    Metadata(u64),
    Performance(u64),
    Flag(u64, Address),
    CampaignContents(u64),
    ContentVariants(u64),
}

// ============================================================
// Error Codes
// ============================================================

pub const ERR_UNAUTHORIZED: u32 = 1;
pub const ERR_NOT_FOUND: u32 = 2;
pub const ERR_ALREADY_EXISTS: u32 = 3;
pub const ERR_INVALID_CONTENT: u32 = 4;
pub const ERR_INVALID_STATUS: u32 = 5;
pub const ERR_CONTENT_FLAGGED: u32 = 6;
pub const ERR_INVALID_FORMAT: u32 = 7;
pub const ERR_NOT_INITIALIZED: u32 = 8;

// ============================================================
// Contract
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct AdRegistryContract;

#[contractimpl]
impl AdRegistryContract {
    /// Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ContentNonce, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::MinContentSize, &100u64);
        env.storage()
            .instance()
            .set(&DataKey::MaxContentSize, &10_485_760u64); // 10MB
        env.storage().instance().set(&DataKey::FlagThreshold, &5u32);
    }

    /// Register new ad content
    pub fn register_content(
        env: Env,
        campaign_id: u64,
        ipfs_hash: String,
        format: ContentFormat,
        size: u64,
        title: String,
        description: String,
        call_to_action: String,
        landing_url: String,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let caller = env.current_contract_address();
        let _ = caller; // will be overridden by auth
                        // Use invoker auth pattern
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let _ = admin;

        let min_size: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MinContentSize)
            .unwrap_or(100);
        let max_size: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MaxContentSize)
            .unwrap_or(10_485_760);

        if size < min_size || size > max_size {
            panic!("invalid content size");
        }

        let nonce: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ContentNonce)
            .unwrap_or(0);
        let content_id = nonce + 1;

        let content = AdContent {
            campaign_id,
            owner: env.current_contract_address(),
            ipfs_hash,
            format,
            size,
            status: ContentStatus::Pending,
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            flags_count: 0,
        };

        let metadata = ContentMetadata {
            title,
            description,
            call_to_action,
            landing_url,
        };

        let performance = ContentPerformance {
            total_views: 0,
            total_clicks: 0,
            unique_viewers: 0,
            click_through_rate: 0,
            last_shown: 0,
        };

        let _ttl_key = DataKey::Content(content_id);
        env.storage().persistent().set(&_ttl_key, &content);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::Metadata(content_id);
        env.storage().persistent().set(&_ttl_key, &metadata);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::Performance(content_id);
        env.storage().persistent().set(&_ttl_key, &performance);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::ContentNonce, &content_id);

        env.events().publish(
            (symbol_short!("register"), symbol_short!("content")),
            (content_id, campaign_id),
        );

        content_id
    }

    /// Update content status (admin only)
    pub fn update_status(env: Env, admin: Address, content_id: u64, new_status: ContentStatus) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut content: AdContent = env
            .storage()
            .persistent()
            .get(&DataKey::Content(content_id))
            .expect("content not found");
        content.status = new_status;
        content.updated_at = env.ledger().timestamp();
        let _ttl_key = DataKey::Content(content_id);
        env.storage().persistent().set(&_ttl_key, &content);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Flag content for review
    pub fn flag_content(env: Env, reporter: Address, content_id: u64, reason: String) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        reporter.require_auth();

        let mut content: AdContent = env
            .storage()
            .persistent()
            .get(&DataKey::Content(content_id))
            .expect("content not found");

        if content.owner == reporter {
            panic!("cannot flag own content");
        }

        let flag = FlagRecord {
            reason,
            timestamp: env.ledger().timestamp(),
            verified: false,
        };

        let _ttl_key = DataKey::Flag(content_id, reporter.clone());
        env.storage().persistent().set(&_ttl_key, &flag);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        content.flags_count += 1;

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::FlagThreshold)
            .unwrap_or(5);

        if content.flags_count >= threshold {
            content.status = ContentStatus::Suspended;
        }

        content.updated_at = env.ledger().timestamp();
        let _ttl_key = DataKey::Content(content_id);
        env.storage().persistent().set(&_ttl_key, &content);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Track a content view
    pub fn track_view(env: Env, content_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let content: AdContent = env
            .storage()
            .persistent()
            .get(&DataKey::Content(content_id))
            .expect("content not found");

        match content.status {
            ContentStatus::Approved => {}
            _ => panic!("content not approved"),
        }

        let mut perf: ContentPerformance = env
            .storage()
            .persistent()
            .get(&DataKey::Performance(content_id))
            .expect("performance not found");

        perf.total_views += 1;
        perf.unique_viewers += 1;
        perf.last_shown = env.ledger().timestamp();

        if perf.total_views > 0 {
            perf.click_through_rate = (perf.total_clicks * 10_000) / perf.total_views;
        }

        let _ttl_key = DataKey::Performance(content_id);
        env.storage().persistent().set(&_ttl_key, &perf);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Track a content click
    pub fn track_click(env: Env, content_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let mut perf: ContentPerformance = env
            .storage()
            .persistent()
            .get(&DataKey::Performance(content_id))
            .expect("performance not found");

        perf.total_clicks += 1;

        if perf.total_views > 0 {
            perf.click_through_rate = (perf.total_clicks * 10_000) / perf.total_views;
        }

        let _ttl_key = DataKey::Performance(content_id);
        env.storage().persistent().set(&_ttl_key, &perf);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Archive content (owner only)
    pub fn archive_content(env: Env, owner: Address, content_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        owner.require_auth();

        let mut content: AdContent = env
            .storage()
            .persistent()
            .get(&DataKey::Content(content_id))
            .expect("content not found");

        if content.owner != owner {
            panic!("unauthorized");
        }

        content.status = ContentStatus::Archived;
        content.updated_at = env.ledger().timestamp();
        let _ttl_key = DataKey::Content(content_id);
        env.storage().persistent().set(&_ttl_key, &content);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    // ============================================================
    // Read-Only Functions
    // ============================================================

    pub fn get_content(env: Env, content_id: u64) -> Option<AdContent> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Content(content_id))
    }

    pub fn get_metadata(env: Env, content_id: u64) -> Option<ContentMetadata> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Metadata(content_id))
    }

    pub fn get_performance(env: Env, content_id: u64) -> Option<ContentPerformance> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Performance(content_id))
    }

    pub fn is_approved(env: Env, content_id: u64) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(content) = env
            .storage()
            .persistent()
            .get::<DataKey, AdContent>(&DataKey::Content(content_id))
        {
            matches!(content.status, ContentStatus::Approved)
        } else {
            false
        }
    }

    pub fn get_nonce(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::ContentNonce)
            .unwrap_or(0)
    }

    pub fn set_flag_threshold(env: Env, admin: Address, threshold: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage()
            .instance()
            .set(&DataKey::FlagThreshold, &threshold);
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
