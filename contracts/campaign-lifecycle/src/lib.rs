//! PulsarTrack - Campaign Lifecycle (Soroban)
//! Manages ad campaign state transitions and lifecycle events on Stellar.
//!
//! Events:
//! - ("lifecycle", "transit"): [campaign_id: u64]
//! - ("campaign", "pause"): [campaign_id: u64, actor: Address]
//! - ("campaign", "resume"): [campaign_id: u64, actor: Address]

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String,
};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum LifecycleState {
    Draft,
    PendingReview,
    Active,
    Paused,
    Completed,
    Cancelled,
    Expired,
    Archived,
    Rejected,
}

#[contracttype]
#[derive(Clone)]
pub struct CampaignLifecycle {
    pub campaign_id: u64,
    pub advertiser: Address,
    pub state: LifecycleState,
    pub created_at: u64,
    pub activated_at: Option<u64>,
    pub paused_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub cancelled_at: Option<u64>,
    pub pause_count: u32,
    pub extension_count: u32,
    pub original_end_ledger: u32,
    pub current_end_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct StateTransition {
    pub from_state: LifecycleState,
    pub to_state: LifecycleState,
    pub actor: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    FraudContract,
    LifecycleCounter,
    Lifecycle(u64),
    TransitionCount(u64),
    Transition(u64, u32), // campaign_id, transition_index
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

/// Maximum number of times a campaign can be extended.
const MAX_EXTENSIONS: u32 = 10;
/// Maximum total duration multiplier relative to original_end_ledger.
const MAX_DURATION_MULTIPLIER: u32 = 3;

#[contract]
pub struct CampaignLifecycleContract;

#[contractimpl]
impl CampaignLifecycleContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::LifecycleCounter, &0u64);
    }

    pub fn set_fraud_contract(env: Env, admin: Address, fraud_contract: Address) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().instance().set(&DataKey::FraudContract, &fraud_contract);
    }

    pub fn pause_for_fraud(env: Env, fraud_contract: Address, campaign_id: u64) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        fraud_contract.require_auth();
        
        let stored_fraud_contract: Address = env.storage().instance().get(&DataKey::FraudContract).expect("fraud contract not set");
        if fraud_contract != stored_fraud_contract {
            panic!("unauthorized fraud contract");
        }

        Self::transition(
            env.clone(),
            fraud_contract,
            campaign_id,
            LifecycleState::Paused,
            String::from_str(&env, "paused for fraud detection"),
        );
    }

    pub fn register_campaign(
        env: Env,
        advertiser: Address,
        campaign_id: u64,
        end_ledger: u32,
    ) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        let lifecycle = CampaignLifecycle {
            campaign_id,
            advertiser: advertiser.clone(),
            state: LifecycleState::Draft,
            created_at: env.ledger().timestamp(),
            activated_at: None,
            paused_at: None,
            completed_at: None,
            cancelled_at: None,
            pause_count: 0,
            extension_count: 0,
            original_end_ledger: end_ledger,
            current_end_ledger: end_ledger,
        };

        let _ttl_key = DataKey::Lifecycle(campaign_id);
        env.storage()
            .persistent()
            .set(&_ttl_key, &lifecycle);
        env.storage()
            .persistent()
            .extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LifecycleCounter)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::LifecycleCounter, &(count + 1));
    }

    pub fn transition(
        env: Env,
        actor: Address,
        campaign_id: u64,
        new_state: LifecycleState,
        reason: String,
    ) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        actor.require_auth();

        let mut lifecycle: CampaignLifecycle = env
            .storage()
            .persistent()
            .get(&DataKey::Lifecycle(campaign_id))
            .expect("lifecycle not found");

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let fraud_contract: Option<Address> = env.storage().instance().get(&DataKey::FraudContract);

        // Only advertiser, admin or authorized fraud contract can transition
        if actor != lifecycle.advertiser && actor != admin {
            if let Some(fraud_addr) = fraud_contract {
                if actor != fraud_addr {
                    panic!("unauthorized");
                }
            } else {
                panic!("unauthorized");
            }
        }

        // Validate state transition
        let old_state = lifecycle.state.clone();
        Self::_validate_transition(&old_state, &new_state);

        // Apply state
        let now = env.ledger().timestamp();
        match new_state {
            LifecycleState::Active => {
                lifecycle.activated_at = Some(now);
                if old_state == LifecycleState::Paused {
                    env.events().publish(
                        (symbol_short!("campaign"), symbol_short!("resume")),
                        (campaign_id, actor.clone()),
                    );
                }
            }
            LifecycleState::Paused => {
                lifecycle.paused_at = Some(now);
                lifecycle.pause_count += 1;
                env.events().publish(
                    (symbol_short!("campaign"), symbol_short!("pause")),
                    (campaign_id, actor.clone()),
                );
            }
            LifecycleState::Completed => {
                lifecycle.completed_at = Some(now);
            }
            LifecycleState::Cancelled => {
                lifecycle.cancelled_at = Some(now);
            }
            _ => {}
        }
        lifecycle.state = new_state.clone();

        let _ttl_key = DataKey::Lifecycle(campaign_id);
        env.storage()
            .persistent()
            .set(&_ttl_key, &lifecycle);
        env.storage()
            .persistent()
            .extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        // Record transition
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::TransitionCount(campaign_id))
            .unwrap_or(0);
        let transition = StateTransition {
            from_state: old_state,
            to_state: new_state,
            actor,
            reason,
            timestamp: now,
        };
        let _ttl_key = DataKey::Transition(campaign_id, count);
        env.storage()
            .persistent()
            .set(&_ttl_key, &transition);
        env.storage()
            .persistent()
            .extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        let _ttl_key = DataKey::TransitionCount(campaign_id);
        env.storage()
            .persistent()
            .set(&_ttl_key, &(count + 1));
        env.storage()
            .persistent()
            .extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("lifecycle"), symbol_short!("transit")),
            campaign_id,
        );
    }

    pub fn extend_campaign(env: Env, advertiser: Address, campaign_id: u64, extra_ledgers: u32) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        let mut lifecycle: CampaignLifecycle = env
            .storage()
            .persistent()
            .get(&DataKey::Lifecycle(campaign_id))
            .expect("lifecycle not found");

        if lifecycle.advertiser != advertiser {
            panic!("unauthorized");
        }

        // Only active campaigns can be extended
        if lifecycle.state != LifecycleState::Active {
            panic!("campaign not active");
        }

        // Reject zero-ledger extensions
        if extra_ledgers == 0 {
            panic!("extra_ledgers must be greater than zero");
        }

        // Enforce maximum extension count
        if lifecycle.extension_count >= MAX_EXTENSIONS {
            panic!("max extensions reached");
        }

        // Enforce maximum total duration (original_end_ledger * MAX_DURATION_MULTIPLIER)
        let max_end = lifecycle.original_end_ledger.saturating_mul(MAX_DURATION_MULTIPLIER);
        let new_end = lifecycle.current_end_ledger.saturating_add(extra_ledgers);
        if new_end > max_end {
            panic!("extension exceeds max campaign duration");
        }

        lifecycle.current_end_ledger = new_end;
        lifecycle.extension_count += 1;

        let _ttl_key = DataKey::Lifecycle(campaign_id);
        env.storage()
            .persistent()
            .set(&_ttl_key, &lifecycle);
        env.storage()
            .persistent()
            .extend_ttl(&_ttl_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }

    pub fn get_lifecycle(env: Env, campaign_id: u64) -> Option<CampaignLifecycle> {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Lifecycle(campaign_id))
    }

    pub fn get_transition(env: Env, campaign_id: u64, index: u32) -> Option<StateTransition> {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Transition(campaign_id, index))
    }

    pub fn get_transition_count(env: Env, campaign_id: u64) -> u32 {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::TransitionCount(campaign_id))
            .unwrap_or(0)
    }

    fn _validate_transition(from: &LifecycleState, to: &LifecycleState) {
        let valid = match from {
            LifecycleState::Draft => matches!(to, LifecycleState::PendingReview | LifecycleState::Cancelled),
            LifecycleState::PendingReview => matches!(to, LifecycleState::Active | LifecycleState::Rejected | LifecycleState::Cancelled),
            LifecycleState::Active => matches!(to, LifecycleState::Paused | LifecycleState::Completed | LifecycleState::Cancelled | LifecycleState::Expired),
            LifecycleState::Paused => matches!(to, LifecycleState::Active | LifecycleState::Cancelled),
            _ => false,
        };
        if !valid {
            panic!("invalid state transition");
        }
    }
}

mod test;
