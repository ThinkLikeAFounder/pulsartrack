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
    LifecycleCounter,
    Lifecycle(u64),
    TransitionCount(u64),
    Transition(u64, u32), // campaign_id, transition_index
}

#[contract]
pub struct CampaignLifecycleContract;

#[contractimpl]
impl CampaignLifecycleContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::LifecycleCounter, &0u64);
    }

    pub fn register_campaign(
        env: Env,
        advertiser: Address,
        campaign_id: u64,
        end_ledger: u32,
    ) {
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

        env.storage()
            .persistent()
            .set(&DataKey::Lifecycle(campaign_id), &lifecycle);

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
        actor.require_auth();

        let mut lifecycle: CampaignLifecycle = env
            .storage()
            .persistent()
            .get(&DataKey::Lifecycle(campaign_id))
            .expect("lifecycle not found");

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

        // Only advertiser or admin can transition
        if actor != lifecycle.advertiser && actor != admin {
            panic!("unauthorized");
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

        env.storage()
            .persistent()
            .set(&DataKey::Lifecycle(campaign_id), &lifecycle);

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
        env.storage()
            .persistent()
            .set(&DataKey::Transition(campaign_id, count), &transition);
        env.storage()
            .persistent()
            .set(&DataKey::TransitionCount(campaign_id), &(count + 1));

        env.events().publish(
            (symbol_short!("lifecycle"), symbol_short!("transit")),
            campaign_id,
        );
    }

    pub fn extend_campaign(env: Env, advertiser: Address, campaign_id: u64, extra_ledgers: u32) {
        advertiser.require_auth();

        let mut lifecycle: CampaignLifecycle = env
            .storage()
            .persistent()
            .get(&DataKey::Lifecycle(campaign_id))
            .expect("lifecycle not found");

        if lifecycle.advertiser != advertiser {
            panic!("unauthorized");
        }

        lifecycle.current_end_ledger += extra_ledgers;
        lifecycle.extension_count += 1;

        env.storage()
            .persistent()
            .set(&DataKey::Lifecycle(campaign_id), &lifecycle);
    }

    pub fn get_lifecycle(env: Env, campaign_id: u64) -> Option<CampaignLifecycle> {
        env.storage()
            .persistent()
            .get(&DataKey::Lifecycle(campaign_id))
    }

    pub fn get_transition(env: Env, campaign_id: u64, index: u32) -> Option<StateTransition> {
        env.storage()
            .persistent()
            .get(&DataKey::Transition(campaign_id, index))
    }

    pub fn get_transition_count(env: Env, campaign_id: u64) -> u32 {
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
