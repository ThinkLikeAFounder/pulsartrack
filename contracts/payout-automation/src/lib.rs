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
    Payout(u64),
    PublisherEarnings(Address),
}

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

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PayoutCounter)
            .unwrap_or(0);
        let payout_id = counter + 1;

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
            scheduled_at: env.ledger().timestamp(),
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

        env.events().publish(
            (symbol_short!("payout"), symbol_short!("schedule")),
            (payout_id, recipient, amount),
        );

        payout_id
    }

    pub fn execute_payout(env: Env, payout_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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

        let token_client = token::Client::new(&env, &payout.token);
        token_client.transfer(
            &env.current_contract_address(),
            &payout.recipient,
            &payout.amount,
        );

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

        earnings.total_paid += payout.amount;
        earnings.pending_amount = earnings.pending_amount.saturating_sub(payout.amount);
        earnings.last_payout = env.ledger().timestamp();
        env.storage().persistent().set(&key, &earnings);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("payout"), symbol_short!("execute")),
            (payout_id, payout.amount),
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

        earnings.pending_amount += amount;
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
