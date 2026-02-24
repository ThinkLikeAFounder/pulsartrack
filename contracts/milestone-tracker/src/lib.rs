//! PulsarTrack - Milestone Tracker (Soroban)
//! Campaign milestone tracking and performance-based payment releases on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum MilestoneStatus {
    Pending,
    InProgress,
    Achieved,
    Missed,
    Disputed,
}

#[contracttype]
#[derive(Clone)]
pub struct Milestone {
    pub milestone_id: u64,
    pub campaign_id: u64,
    pub description: String,
    pub target_metric: String,
    pub target_value: u64,
    pub current_value: u64,
    pub reward_amount: i128,
    pub status: MilestoneStatus,
    pub deadline_ledger: u32,
    pub achieved_at: Option<u64>,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    OracleAddress,
    MilestoneCounter,
    Milestone(u64),
    CampaignMilestoneCount(u64),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct MilestoneTrackerContract;

#[contractimpl]
impl MilestoneTrackerContract {
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
        env.storage()
            .instance()
            .set(&DataKey::MilestoneCounter, &0u64);
    }

    pub fn create_milestone(
        env: Env,
        advertiser: Address,
        campaign_id: u64,
        description: String,
        target_metric: String,
        target_value: u64,
        reward_amount: i128,
        deadline_ledger: u32,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        advertiser.require_auth();

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MilestoneCounter)
            .unwrap_or(0);
        let milestone_id = counter + 1;

        let milestone = Milestone {
            milestone_id,
            campaign_id,
            description,
            target_metric,
            target_value,
            current_value: 0,
            reward_amount,
            status: MilestoneStatus::Pending,
            deadline_ledger,
            achieved_at: None,
            created_at: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Milestone(milestone_id);
        env.storage().persistent().set(&_ttl_key, &milestone);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::MilestoneCounter, &milestone_id);

        let m_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::CampaignMilestoneCount(campaign_id))
            .unwrap_or(0);
        let _ttl_key = DataKey::CampaignMilestoneCount(campaign_id);
        env.storage().persistent().set(&_ttl_key, &(m_count + 1));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        milestone_id
    }

    pub fn update_progress(env: Env, oracle: Address, milestone_id: u64, current_value: u64) {
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

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .expect("milestone not found");

        if milestone.status == MilestoneStatus::Achieved {
            return; // Already achieved, no update needed
        }

        milestone.current_value = current_value;

        if current_value >= milestone.target_value {
            milestone.status = MilestoneStatus::Achieved;
            milestone.achieved_at = Some(env.ledger().timestamp());

            env.events().publish(
                (symbol_short!("milestone"), symbol_short!("achieved")),
                (milestone_id, milestone.campaign_id),
            );
        } else if env.ledger().sequence() > milestone.deadline_ledger {
            milestone.status = MilestoneStatus::Missed;
        } else {
            milestone.status = MilestoneStatus::InProgress;
        }

        let _ttl_key = DataKey::Milestone(milestone_id);
        env.storage().persistent().set(&_ttl_key, &milestone);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn dispute_milestone(env: Env, caller: Address, milestone_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .expect("milestone not found");

        milestone.status = MilestoneStatus::Disputed;
        let _ttl_key = DataKey::Milestone(milestone_id);
        env.storage().persistent().set(&_ttl_key, &milestone);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn resolve_dispute(env: Env, admin: Address, milestone_id: u64, achieved: bool) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .expect("milestone not found");

        milestone.status = if achieved {
            MilestoneStatus::Achieved
        } else {
            MilestoneStatus::Missed
        };

        let _ttl_key = DataKey::Milestone(milestone_id);
        env.storage().persistent().set(&_ttl_key, &milestone);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_milestone(env: Env, milestone_id: u64) -> Option<Milestone> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
    }

    pub fn get_campaign_milestone_count(env: Env, campaign_id: u64) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::CampaignMilestoneCount(campaign_id))
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
