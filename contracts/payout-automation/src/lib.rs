//! PulsarTrack - Payout Automation (Soroban)
//! Automated publisher payouts and scheduled payments on Stellar.
//!
//! Events:
//! - ("payout", "schedule"): [payout_id: u64, recipient: Address, amount: i128]
//! - ("payout", "execute"): [payout_id: u64, amount: i128]

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum PayoutStatus {
    Scheduled,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct ScheduledPayout {
    pub payout_id: u64,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub scheduled_at: u64,
    pub execute_after: u64,
    pub status: PayoutStatus,
    pub campaign_id: Option<u64>,
    pub executed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub struct PublisherEarnings {
    pub publisher: Address,
    pub pending_amount: i128,
    pub total_paid: i128,
    pub last_payout: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TokenAddress,
    PayoutCounter,
    MinPayoutAmount,
    MaxPendingAmount,
    Payout(u64),
    PublisherEarnings(Address),
    Paused,
}

const MAX_PENDING_AMOUNT: i128 = 1_000_000_000_000; // 100k XLM in stroops (assuming 7 decimals)

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct PayoutAutomationContract;

#[contractimpl]
impl PayoutAutomationContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage().instance().set(&DataKey::PayoutCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::MinPayoutAmount, &1_000_000i128);
    }

    pub fn schedule_payout(
        env: Env,
        admin: Address,
        recipient: Address,
        amount: i128,
        execute_after: u64,
        campaign_id: Option<u64>,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let now = env.ledger().timestamp();
        if execute_after < now {
            panic!("execute_after must be in the future");
        }

        let min_payout: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinPayoutAmount)
            .unwrap_or(1_000_000);
        if amount < min_payout {
            panic!("amount below minimum payout");
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PayoutCounter)
            .unwrap_or(0);
        let payout_id = counter.checked_add(1).expect("counter overflow");

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();

        let payout = ScheduledPayout {
            payout_id,
            recipient: recipient.clone(),
            token: token_addr,
            amount,
            scheduled_at: now,
            execute_after,
            status: PayoutStatus::Scheduled,
            campaign_id,
            executed_at: None,
        };

        let _ttl_key = DataKey::Payout(payout_id);
        env.storage().persistent().set(&_ttl_key, &payout);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.storage()
            .instance()
            .set(&DataKey::PayoutCounter, &payout_id);

        // Check if pending_amount exceeds cap after adding
        let max_pending: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MaxPendingAmount)
            .unwrap_or(MAX_PENDING_AMOUNT);
        let pending_amount = env
            .storage()
            .persistent()
            .get::<_, PublisherEarnings>(&DataKey::PublisherEarnings(recipient.clone()))
            .map(|earnings| earnings.pending_amount)
            .unwrap_or(0);
        if pending_amount > max_pending {
            // Trigger an event for the admin to notice the high pending amount
            env.events().publish(
                (symbol_short!("payout"), symbol_short!("alert")),
                (recipient.clone(), pending_amount),
            );
        }

        env.events().publish(
            (symbol_short!("payout"), symbol_short!("schedule")),
            (payout_id, recipient, amount),
        );

        payout_id
    }

    pub fn execute_payout(env: Env, caller: Address, payout_id: u64) {
        Self::require_not_paused(&env);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != stored_admin {
            panic!("unauthorized");
        }
        let mut payout: ScheduledPayout = env
            .storage()
            .persistent()
            .get(&DataKey::Payout(payout_id))
            .expect("payout not found");

        if payout.status != PayoutStatus::Scheduled {
            panic!("payout not scheduled");
        }

        if env.ledger().timestamp() < payout.execute_after {
            panic!("too early to execute");
        }

        payout.status = PayoutStatus::Completed;
        payout.executed_at = Some(env.ledger().timestamp());
        let _ttl_key = DataKey::Payout(payout_id);
        env.storage().persistent().set(&_ttl_key, &payout);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Update publisher earnings
        let key = DataKey::PublisherEarnings(payout.recipient.clone());
        let mut earnings: PublisherEarnings =
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(PublisherEarnings {
                    publisher: payout.recipient.clone(),
                    pending_amount: 0,
                    total_paid: 0,
                    last_payout: 0,
                });

        earnings.total_paid = earnings
            .total_paid
            .checked_add(payout.amount)
            .expect("total_paid overflow");
        earnings.pending_amount = earnings.pending_amount.saturating_sub(payout.amount);
        earnings.last_payout = env.ledger().timestamp();
        env.storage().persistent().set(&key, &earnings);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let token_client = token::Client::new(&env, &payout.token);
        token_client.transfer(
            &env.current_contract_address(),
            &payout.recipient,
            &payout.amount,
        );

        env.events().publish(
            (symbol_short!("payout"), symbol_short!("execute")),
            (payout_id, payout.amount),
        );
    }

    pub fn fund_payouts(env: Env, sender: Address, amount: i128) {
        Self::require_not_paused(&env);
        sender.require_auth();
        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        token::Client::new(&env, &token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );
    }

    pub fn add_publisher_earnings(env: Env, admin: Address, publisher: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let key = DataKey::PublisherEarnings(publisher.clone());
        let mut earnings: PublisherEarnings =
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(PublisherEarnings {
                    publisher: publisher.clone(),
                    pending_amount: 0,
                    total_paid: 0,
                    last_payout: 0,
                });

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let max_pending: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MaxPendingAmount)
            .unwrap_or(MAX_PENDING_AMOUNT);
        let new_pending = earnings
            .pending_amount
            .checked_add(amount)
            .expect("pending_amount overflow");

        if new_pending > max_pending {
            panic!("pending_amount exceeds maximum cap");
        }

        earnings.pending_amount = new_pending;
        env.storage().persistent().set(&key, &earnings);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_payout(env: Env, payout_id: u64) -> Option<ScheduledPayout> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Payout(payout_id))
    }

    pub fn get_publisher_earnings(env: Env, publisher: Address) -> Option<PublisherEarnings> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::PublisherEarnings(publisher))
    }

    pub fn set_max_pending_amount(env: Env, admin: Address, amount: i128) {
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
            .set(&DataKey::MaxPendingAmount, &amount);
    }

    pub fn get_max_pending_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::MaxPendingAmount)
            .unwrap_or(MAX_PENDING_AMOUNT)
    }

    /// Pause the contract. Only callable by the admin.
    ///
    /// While paused every fund-moving entrypoint panics. Read-only getters
    /// and administrative functions remain available so the contract can be
    /// inspected and unpaused once a fix is ready.
    pub fn set_paused(env: Env, admin: Address, paused: bool) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().instance().set(&DataKey::Paused, &paused);
        env.events()
            .publish((symbol_short!("pause"), symbol_short!("set")), paused);
    }

    /// Whether the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    fn require_not_paused(env: &Env) {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            panic!("contract is paused");
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
