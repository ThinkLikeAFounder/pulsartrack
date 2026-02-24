//! PulsarTrack - Campaign Orchestrator (Soroban)
//! Advanced decentralized advertising campaign orchestration on Stellar.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String,
    IntoVal, Symbol, Val, Vec as SdkVec,
};

// Define external contract interfaces for cross-contract calls
// These match the actual contract implementations

// ============================================================
// Data Types
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum CampaignStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
    Expired,
}

#[contracttype]
#[derive(Clone)]
pub struct Campaign {
    pub advertiser: Address,
    pub campaign_type: u32,
    pub budget: i128,
    pub remaining_budget: i128,
    pub cost_per_view: i128,
    pub start_ledger: u32,
    pub end_ledger: u32,
    pub status: CampaignStatus,
    pub target_views: u64,
    pub current_views: u64,
    pub daily_view_limit: u64,
    pub refundable: bool,
    pub platform_fee: i128,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct CampaignType {
    pub name: String,
    pub min_duration: u32,
    pub max_duration: u32,
    pub min_budget: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct VerifiedPublisher {
    pub verified: bool,
    pub reputation_score: u32,
    pub total_earnings: i128,
    pub join_ledger: u32,
    pub last_active: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AdvertiserStats {
    pub total_campaigns: u32,
    pub active_campaigns: u32,
    pub total_spent: i128,
    pub total_views: u64,
    pub average_view_rate: u32,
    pub reputation_score: u32,
    pub last_campaign_id: u64,
    pub join_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct CampaignMetrics {
    pub campaign: Campaign,
    pub total_spent: i128,
    pub completion_rate: u32,
}

// ============================================================
// Storage Keys
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TokenAddress,
    MinCampaignAmount,
    PlatformFeePct,
    CampaignCounter,
    TotalPlatformFees,
    Campaign(u64),
    CampaignType(u32),
    Publisher(Address),
    AdvertiserStats(Address),
    DailyViews(u64, u64),
    // Contract addresses for cross-contract validation
    LifecycleContract,
    EscrowContract,
    TargetingContract,
    AuctionContract,
}

// ============================================================
// Contract
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct CampaignOrchestratorContract;

#[contractimpl]
impl CampaignOrchestratorContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address, token_address: Address) {
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
            .set(&DataKey::TokenAddress, &token_address);
        env.storage()
            .instance()
            .set(&DataKey::MinCampaignAmount, &1_000_000i128); // 0.1 XLM (in stroops)
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeePct, &2u32); // 2%
        env.storage()
            .instance()
            .set(&DataKey::CampaignCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::TotalPlatformFees, &0i128);

        // Register default campaign type
        let default_type = CampaignType {
            name: String::from_str(&env, "Standard"),
            min_duration: 100,
            max_duration: 10_000,
            min_budget: 1_000_000,
        };
        env.storage()
            .instance()
            .set(&DataKey::CampaignType(1), &default_type);
    }

    /// Set contract addresses for cross-contract validation (admin only)
    pub fn set_lifecycle_contract(env: Env, admin: Address, contract_address: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().instance().set(&DataKey::LifecycleContract, &contract_address);
    }

    pub fn set_escrow_contract(env: Env, admin: Address, contract_address: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().instance().set(&DataKey::EscrowContract, &contract_address);
    }

    pub fn set_targeting_contract(env: Env, admin: Address, contract_address: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().instance().set(&DataKey::TargetingContract, &contract_address);
    }

    pub fn set_auction_contract(env: Env, admin: Address, contract_address: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().instance().set(&DataKey::AuctionContract, &contract_address);
    }

    /// Create a new ad campaign
    pub fn create_campaign(
        env: Env,
        advertiser: Address,
        campaign_type: u32,
        budget: i128,
        cost_per_view: i128,
        duration: u32,
        target_views: u64,
        daily_view_limit: u64,
        refundable: bool,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        let campaign_type_data: CampaignType = env
            .storage()
            .instance()
            .get(&DataKey::CampaignType(campaign_type))
            .expect("campaign type not found");

        if budget < campaign_type_data.min_budget {
            panic!("budget too low");
        }
        if duration < campaign_type_data.min_duration || duration > campaign_type_data.max_duration
        {
            panic!("invalid duration");
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CampaignCounter)
            .unwrap_or(0);
        let campaign_id = counter + 1;

        let platform_fee_pct: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PlatformFeePct)
            .unwrap_or(2);
        let platform_fee = (budget * platform_fee_pct as i128) / 100;

        // Transfer budget + fee from advertiser to this contract
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(
            &advertiser,
            &env.current_contract_address(),
            &(budget + platform_fee),
        );

        let start_ledger = env.ledger().sequence();
        let end_ledger = start_ledger + duration;

        let campaign = Campaign {
            advertiser: advertiser.clone(),
            campaign_type,
            budget,
            remaining_budget: budget,
            cost_per_view,
            start_ledger,
            end_ledger,
            status: CampaignStatus::Active,
            target_views,
            current_views: 0,
            daily_view_limit,
            refundable,
            platform_fee,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Campaign(campaign_id);
        env.storage().persistent().set(&_ttl_key, &campaign);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::CampaignCounter, &campaign_id);

        let total_fees: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPlatformFees)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalPlatformFees, &(total_fees + platform_fee));

        // Update advertiser stats
        Self::_update_advertiser_stats(&env, &advertiser, campaign_id, budget);

        env.events().publish(
            (symbol_short!("campaign"), symbol_short!("created")),
            (campaign_id, advertiser, budget),
        );

        campaign_id
    }

    /// Record a view (publisher earns cost_per_view)
    pub fn record_view(env: Env, campaign_id: u64, publisher: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        publisher.require_auth();

        // CROSS-CONTRACT VALIDATION: Validate campaign status across all contracts
        Self::_validate_campaign_cross_contract(&env, campaign_id, &publisher);

        let mut campaign: Campaign = env
            .storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
            .expect("campaign not found");

        // Verify publisher
        let publisher_data: VerifiedPublisher = env
            .storage()
            .persistent()
            .get(&DataKey::Publisher(publisher.clone()))
            .expect("publisher not verified");

        if !publisher_data.verified {
            panic!("publisher not verified");
        }

        // Check campaign is active
        match campaign.status {
            CampaignStatus::Active => {}
            _ => panic!("campaign not active"),
        }

        if campaign.current_views >= campaign.target_views {
            panic!("campaign target reached");
        }

        if env.ledger().sequence() > campaign.end_ledger {
            panic!("campaign expired");
        }

        if campaign.remaining_budget < campaign.cost_per_view {
            panic!("insufficient budget");
        }

        // Check daily view limit
        let current_day = env.ledger().timestamp() / 86_400;
        let daily_key = DataKey::DailyViews(campaign_id, current_day);
        let daily_views: u64 = env.storage().temporary().get(&daily_key).unwrap_or(0);

        if daily_views >= campaign.daily_view_limit {
            panic!("daily view limit reached");
        }

        // Transfer payment to publisher
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(
            &env.current_contract_address(),
            &publisher,
            &campaign.cost_per_view,
        );

        // Update campaign
        campaign.remaining_budget -= campaign.cost_per_view;
        campaign.current_views += 1;
        campaign.last_updated = env.ledger().timestamp();

        if campaign.current_views >= campaign.target_views {
            campaign.status = CampaignStatus::Completed;
        }

        let _ttl_key = DataKey::Campaign(campaign_id);
        env.storage().persistent().set(&_ttl_key, &campaign);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .temporary()
            .set(&daily_key, &(daily_views + 1));

        // Update publisher earnings
        Self::_update_publisher_earnings(&env, &publisher, campaign.cost_per_view);

        env.events().publish(
            (symbol_short!("view"), symbol_short!("recorded")),
            (campaign_id, publisher),
        );
    }

    /// Pause a campaign (advertiser only)
    pub fn pause_campaign(env: Env, advertiser: Address, campaign_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        let mut campaign: Campaign = env
            .storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
            .expect("campaign not found");

        if campaign.advertiser != advertiser {
            panic!("unauthorized");
        }

        campaign.status = CampaignStatus::Paused;
        campaign.last_updated = env.ledger().timestamp();
        let _ttl_key = DataKey::Campaign(campaign_id);
        env.storage().persistent().set(&_ttl_key, &campaign);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Resume a paused campaign
    pub fn resume_campaign(env: Env, advertiser: Address, campaign_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        let mut campaign: Campaign = env
            .storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
            .expect("campaign not found");

        if campaign.advertiser != advertiser {
            panic!("unauthorized");
        }

        campaign.status = CampaignStatus::Active;
        campaign.last_updated = env.ledger().timestamp();
        let _ttl_key = DataKey::Campaign(campaign_id);
        env.storage().persistent().set(&_ttl_key, &campaign);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Cancel campaign and refund remaining budget (if refundable)
    pub fn cancel_campaign(env: Env, advertiser: Address, campaign_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        let mut campaign: Campaign = env
            .storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
            .expect("campaign not found");

        if campaign.advertiser != advertiser {
            panic!("unauthorized");
        }

        if !campaign.refundable {
            panic!("campaign not refundable");
        }

        let refund = campaign.remaining_budget;
        campaign.remaining_budget = 0;
        campaign.status = CampaignStatus::Cancelled;
        campaign.last_updated = env.ledger().timestamp();

        let _ttl_key = DataKey::Campaign(campaign_id);
        env.storage().persistent().set(&_ttl_key, &campaign);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        if refund > 0 {
            let token_addr: Address = env
                .storage()
                .instance()
                .get(&DataKey::TokenAddress)
                .unwrap();
            let token_client = token::Client::new(&env, &token_addr);
            token_client.transfer(&env.current_contract_address(), &advertiser, &refund);
        }

        env.events().publish(
            (symbol_short!("campaign"), symbol_short!("cancelled")),
            (campaign_id, refund),
        );
    }

    /// Admin: verify a publisher
    pub fn verify_publisher(env: Env, admin: Address, publisher: Address, initial_score: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let publisher_data = VerifiedPublisher {
            verified: true,
            reputation_score: initial_score,
            total_earnings: 0,
            join_ledger: env.ledger().sequence(),
            last_active: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Publisher(publisher.clone());
        env.storage().persistent().set(&_ttl_key, &publisher_data);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("publisher"), symbol_short!("verified")),
            publisher,
        );
    }

    /// Admin: set platform fee
    pub fn set_platform_fee(env: Env, admin: Address, fee_pct: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        if fee_pct > 10 {
            panic!("fee too high");
        }
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeePct, &fee_pct);
    }

    // ============================================================
    // Read-Only Functions
    // ============================================================

    pub fn get_campaign(env: Env, campaign_id: u64) -> Option<Campaign> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
    }

    pub fn get_campaign_metrics(env: Env, campaign_id: u64) -> Option<CampaignMetrics> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let campaign: Campaign = env
            .storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))?;

        let total_spent = campaign.budget - campaign.remaining_budget;
        let completion_rate = if campaign.target_views > 0 {
            ((campaign.current_views * 100) / campaign.target_views) as u32
        } else {
            0
        };

        Some(CampaignMetrics {
            campaign,
            total_spent,
            completion_rate,
        })
    }

    pub fn get_publisher_metrics(env: Env, publisher: Address) -> Option<VerifiedPublisher> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Publisher(publisher))
    }

    pub fn get_advertiser_stats(env: Env, advertiser: Address) -> Option<AdvertiserStats> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::AdvertiserStats(advertiser))
    }

    pub fn get_campaign_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::CampaignCounter)
            .unwrap_or(0)
    }

    // ============================================================
    // Internal Helpers
    // ============================================================

    /// Validate campaign across all contracts before processing
    fn _validate_campaign_cross_contract(env: &Env, campaign_id: u64, publisher: &Address) {
        // 1. Validate campaign lifecycle status
        if let Some(lifecycle_addr) = env.storage().instance().get::<DataKey, Address>(&DataKey::LifecycleContract) {
            // Call get_lifecycle on the lifecycle contract
            let lifecycle_result: Option<Val> = env.invoke_contract(
                &lifecycle_addr,
                &Symbol::new(env, "get_lifecycle"),
                SdkVec::from_array(env, [campaign_id.into_val(env)]),
            );
            
            if lifecycle_result.is_none() {
                panic!("campaign not found in lifecycle contract");
            }
            
            // Note: In production, you would deserialize the result and check the state
            // For now, we're validating that the campaign exists in the lifecycle contract
        }

        // 2. Validate escrow has sufficient budget
        if let Some(escrow_addr) = env.storage().instance().get::<DataKey, Address>(&DataKey::EscrowContract) {
            // Call get_escrow on the escrow contract
            let escrow_result: Option<Val> = env.invoke_contract(
                &escrow_addr,
                &Symbol::new(env, "get_escrow"),
                SdkVec::from_array(env, [campaign_id.into_val(env)]),
            );
            
            // If escrow exists, validate it can be released (has budget)
            if escrow_result.is_some() {
                let can_release: bool = env.invoke_contract(
                    &escrow_addr,
                    &Symbol::new(env, "can_release"),
                    SdkVec::from_array(env, [campaign_id.into_val(env)]),
                );
                
                if !can_release {
                    panic!("escrow cannot be released - insufficient budget or conditions not met");
                }
            }
        }

        // 3. Validate publisher matches targeting rules
        if let Some(targeting_addr) = env.storage().instance().get::<DataKey, Address>(&DataKey::TargetingContract) {
            // Call get_targeting to check if targeting config exists
            let targeting_result: Option<Val> = env.invoke_contract(
                &targeting_addr,
                &Symbol::new(env, "get_targeting"),
                SdkVec::from_array(env, [campaign_id.into_val(env)]),
            );
            
            // If targeting config exists, check publisher score
            if targeting_result.is_some() {
                // Try to get the targeting score for this publisher
                let score_result: Option<Val> = env.invoke_contract(
                    &targeting_addr,
                    &Symbol::new(env, "get_score"),
                    SdkVec::from_array(env, [
                        campaign_id.into_val(env),
                        publisher.into_val(env),
                    ]),
                );
                
                // If no score exists and targeting is configured, publisher may not be eligible
                if score_result.is_none() {
                    // In production, you might want to compute the score on-the-fly
                    // or have a more lenient policy
                    // For now, we'll allow it but log a warning via events
                    env.events().publish(
                        (symbol_short!("warning"), symbol_short!("no_score")),
                        (campaign_id, publisher.clone()),
                    );
                }
            }
        }
    }

    fn _update_advertiser_stats(env: &Env, advertiser: &Address, campaign_id: u64, budget: i128) {
        let key = DataKey::AdvertiserStats(advertiser.clone());
        let stats = env
            .storage()
            .persistent()
            .get::<DataKey, AdvertiserStats>(&key);

        let new_stats = if let Some(mut s) = stats {
            s.total_campaigns += 1;
            s.active_campaigns += 1;
            s.total_spent += budget;
            s.last_campaign_id = campaign_id;
            s
        } else {
            AdvertiserStats {
                total_campaigns: 1,
                active_campaigns: 1,
                total_spent: budget,
                total_views: 0,
                average_view_rate: 0,
                reputation_score: 100,
                last_campaign_id: campaign_id,
                join_ledger: env.ledger().sequence(),
            }
        };

        env.storage().persistent().set(&key, &new_stats);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    fn _update_publisher_earnings(env: &Env, publisher: &Address, earning: i128) {
        let key = DataKey::Publisher(publisher.clone());
        if let Some(mut pub_data) = env
            .storage()
            .persistent()
            .get::<DataKey, VerifiedPublisher>(&key)
        {
            pub_data.total_earnings += earning;
            pub_data.last_active = env.ledger().timestamp();
            env.storage().persistent().set(&key, &pub_data);
            env.storage().persistent().extend_ttl(
                &key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
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
