//! PulsarTrack - Escrow Vault (Soroban)
//! Advanced escrow with time-locked funds, performance triggers, and multi-party approval.
//!
//! Events:
//! - ("escrow", "created"): [escrow_id: u64, campaign_id: u64, amount: i128]
//! - ("escrow", "release"): [escrow_id: u64, amount: i128]
//! - ("escrow", "release_partial"): [escrow_id: u64, amount: i128]
//! - ("escrow", "refund"): [escrow_id: u64, amount: i128]

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env, Vec};

// ============================================================
// Data Types
// ============================================================

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum EscrowState {
    Pending,
    Locked,
    Released,
    Refunded,
    PartiallyReleased,
    Disputed,
}

#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub campaign_id: u64,
    pub depositor: Address,
    pub beneficiary: Address,
    pub amount: i128,
    pub locked_amount: i128,
    pub released_amount: i128,
    pub refunded_amount: i128,
    pub state: EscrowState,
    pub time_lock_until: u64,       // Unix timestamp
    pub performance_threshold: u32, // percentage 0-100
    pub created_at: u64,
    pub locked_at: Option<u64>,
    pub released_at: Option<u64>,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct EscrowApproval {
    pub approved: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct PerformanceMetrics {
    pub current_performance: u32,
    pub views_delivered: u64,
    pub clicks_delivered: u64,
    pub last_updated: u64,
}

// ============================================================
// Storage Keys
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    FraudContract,
    TokenAddress,
    OracleAddress,
    MinApprovalThreshold,
    EscrowNonce,
    Escrow(u64),
    Approval(u64, Address),
    ApprovalCount(u64),
    RequiredApprover(u64, Address),
    Performance(u64),
}

// ============================================================
// Contract
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct EscrowVaultContract;

#[contractimpl]
impl EscrowVaultContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address, token_address: Address, oracle: Address) {
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
            .set(&DataKey::OracleAddress, &oracle);
        env.storage()
            .instance()
            .set(&DataKey::MinApprovalThreshold, &1u32);
        env.storage().instance().set(&DataKey::EscrowNonce, &0u64);
    }

    pub fn set_fraud_contract(env: Env, admin: Address, fraud_contract: Address) {
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
            .set(&DataKey::FraudContract, &fraud_contract);
    }

    pub fn hold_for_fraud(env: Env, fraud_contract: Address, escrow_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        fraud_contract.require_auth();

        let stored_fraud: Address = env
            .storage()
            .instance()
            .get(&DataKey::FraudContract)
            .expect("fraud contract not set");
        if fraud_contract != stored_fraud {
            panic!("unauthorized fraud contract");
        }

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        escrow.state = EscrowState::Disputed;

        let _ttl_key = DataKey::Escrow(escrow_id);
        env.storage().persistent().set(&_ttl_key, &escrow);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Create a new escrow
    pub fn create_escrow(
        env: Env,
        depositor: Address,
        campaign_id: u64,
        beneficiary: Address,
        amount: i128,
        time_lock_duration: u64,
        performance_threshold: u32,
        expires_in: u64,
        required_approvers: Vec<Address>,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        depositor.require_auth();

        if amount <= 0 {
            panic!("invalid amount");
        }
        if performance_threshold > 100 {
            panic!("invalid performance threshold");
        }

        // Transfer funds to escrow contract
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let nonce: u64 = env
            .storage()
            .instance()
            .get(&DataKey::EscrowNonce)
            .unwrap_or(0);
        let escrow_id = nonce + 1;

        let now = env.ledger().timestamp();
        let escrow = Escrow {
            campaign_id,
            depositor: depositor.clone(),
            beneficiary,
            amount,
            locked_amount: amount,
            released_amount: 0,
            refunded_amount: 0,
            state: EscrowState::Locked,
            time_lock_until: now + time_lock_duration,
            performance_threshold,
            created_at: now,
            locked_at: Some(now),
            released_at: None,
            expires_at: now + expires_in,
        };

        let _ttl_key = DataKey::Escrow(escrow_id);
        env.storage().persistent().set(&_ttl_key, &escrow);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::ApprovalCount(escrow_id);
        env.storage().persistent().set(&_ttl_key, &0u32);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Register required approvers
        for approver in required_approvers.iter() {
            let _ttl_key = DataKey::RequiredApprover(escrow_id, approver.clone());
            env.storage().persistent().set(&_ttl_key, &true);
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

        env.storage()
            .instance()
            .set(&DataKey::EscrowNonce, &escrow_id);

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("created")),
            (escrow_id, campaign_id, amount),
        );

        escrow_id
    }

    /// Approve escrow release
    pub fn approve_release(env: Env, approver: Address, escrow_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        approver.require_auth();

        let is_required: bool = env
            .storage()
            .persistent()
            .get(&DataKey::RequiredApprover(escrow_id, approver.clone()))
            .unwrap_or(false);

        if !is_required {
            panic!("not a required approver");
        }

        let approval_key = DataKey::Approval(escrow_id, approver.clone());
        if env.storage().persistent().has(&approval_key) {
            panic!("already approved");
        }

        let escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        if escrow.state == EscrowState::Released {
            panic!("already released");
        }

        let approval = EscrowApproval {
            approved: true,
            timestamp: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Approval(escrow_id, approver);
        env.storage().persistent().set(&_ttl_key, &approval);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ApprovalCount(escrow_id))
            .unwrap_or(0);
        let _ttl_key = DataKey::ApprovalCount(escrow_id);
        env.storage().persistent().set(&_ttl_key, &(count + 1));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Release full escrow to beneficiary
    pub fn release_escrow(env: Env, caller: Address, escrow_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        // Must be depositor or admin
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != escrow.depositor && caller != admin {
            panic!("unauthorized");
        }

        Self::_check_can_release(&env, &escrow, escrow_id);

        let locked = escrow.locked_amount;
        if locked <= 0 {
            panic!("nothing to release");
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.beneficiary,
            &locked,
        );

        escrow.locked_amount = 0;
        escrow.released_amount = escrow.amount;
        escrow.state = EscrowState::Released;
        escrow.released_at = Some(env.ledger().timestamp());

        let _ttl_key = DataKey::Escrow(escrow_id);
        env.storage().persistent().set(&_ttl_key, &escrow);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("release")),
            (escrow_id, locked),
        );
    }

    /// Partial release
    pub fn release_partial(env: Env, caller: Address, escrow_id: u64, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != escrow.depositor && caller != admin {
            panic!("unauthorized");
        }

        Self::_check_can_release(&env, &escrow, escrow_id);

        if amount <= 0 || amount > escrow.locked_amount {
            panic!("invalid amount");
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.beneficiary,
            &amount,
        );

        escrow.locked_amount -= amount;
        escrow.released_amount += amount;
        escrow.state = EscrowState::PartiallyReleased;

        let _ttl_key = DataKey::Escrow(escrow_id);
        env.storage().persistent().set(&_ttl_key, &escrow);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("release_p")), // "release_partial" is too long for symbol_short
            (escrow_id, amount),
        );
    }

    /// Refund escrow if expired
    pub fn refund_escrow(env: Env, caller: Address, escrow_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        let now = env.ledger().timestamp();
        if now < escrow.expires_at {
            panic!("escrow not yet expired");
        }

        if escrow.locked_amount <= 0 {
            panic!("nothing to refund");
        }

        let refund = escrow.locked_amount;
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &escrow.depositor, &refund);

        escrow.locked_amount = 0;
        escrow.refunded_amount = refund;
        escrow.state = EscrowState::Refunded;

        let _ttl_key = DataKey::Escrow(escrow_id);
        env.storage().persistent().set(&_ttl_key, &escrow);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("refund")),
            (escrow_id, refund),
        );
    }

    /// Update performance metrics (oracle only)
    pub fn update_performance(
        env: Env,
        oracle: Address,
        escrow_id: u64,
        performance: u32,
        views: u64,
        clicks: u64,
    ) {
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

        if performance > 100 {
            panic!("invalid performance");
        }

        let metrics = PerformanceMetrics {
            current_performance: performance,
            views_delivered: views,
            clicks_delivered: clicks,
            last_updated: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Performance(escrow_id);
        env.storage().persistent().set(&_ttl_key, &metrics);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    // ============================================================
    // Read-Only Functions
    // ============================================================

    pub fn get_escrow(env: Env, escrow_id: u64) -> Option<Escrow> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Escrow(escrow_id))
    }

    pub fn get_performance(env: Env, escrow_id: u64) -> Option<PerformanceMetrics> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Performance(escrow_id))
    }

    pub fn get_approval_count(env: Env, escrow_id: u64) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::ApprovalCount(escrow_id))
            .unwrap_or(0)
    }

    pub fn can_release(env: Env, escrow_id: u64) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(escrow) = env
            .storage()
            .persistent()
            .get::<DataKey, Escrow>(&DataKey::Escrow(escrow_id))
        {
            let now = env.ledger().timestamp();
            let time_ok = now >= escrow.time_lock_until;
            let min_threshold: u32 = env
                .storage()
                .instance()
                .get(&DataKey::MinApprovalThreshold)
                .unwrap_or(1);
            let approvals: u32 = env
                .storage()
                .persistent()
                .get(&DataKey::ApprovalCount(escrow_id))
                .unwrap_or(0);
            let approvals_ok = approvals >= min_threshold;

            let perf_ok = if let Some(perf) = env
                .storage()
                .persistent()
                .get::<DataKey, PerformanceMetrics>(&DataKey::Performance(escrow_id))
            {
                perf.current_performance >= escrow.performance_threshold
            } else {
                true
            };

            time_ok && approvals_ok && perf_ok
        } else {
            false
        }
    }

    // ============================================================
    // Internal Helpers
    // ============================================================

    fn _check_can_release(env: &Env, escrow: &Escrow, escrow_id: u64) {
        if escrow.state == EscrowState::Disputed {
            panic!("escrow is disputed due to fraud");
        }
        let now = env.ledger().timestamp();
        if now < escrow.time_lock_until {
            panic!("time lock active");
        }

        let min_threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MinApprovalThreshold)
            .unwrap_or(1);
        let approvals: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ApprovalCount(escrow_id))
            .unwrap_or(0);
        if approvals < min_threshold {
            panic!("approval required");
        }

        if let Some(perf) = env
            .storage()
            .persistent()
            .get::<DataKey, PerformanceMetrics>(&DataKey::Performance(escrow_id))
        {
            if perf.current_performance < escrow.performance_threshold {
                panic!("performance threshold not met");
            }
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
