//! PulsarTrack - Recurring Payment (Soroban)
//! Automated recurring payment subscriptions for ad campaigns on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum RecurringStatus {
    Active,
    Paused,
    Cancelled,
    Failed,
}

#[contracttype]
#[derive(Clone)]
pub struct RecurringPayment {
    pub payment_id: u64,
    pub payer: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub interval_secs: u64, // payment interval
    pub max_payments: Option<u32>,
    pub total_payments: u32,
    pub status: RecurringStatus,
    pub created_at: u64,
    pub last_payment: u64,
    pub next_payment: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    PaymentCounter,
    Payment(u64),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct RecurringPaymentContract;

#[contractimpl]
impl RecurringPaymentContract {
    pub fn initialize(env: Env, admin: Address) {
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
            .set(&DataKey::PaymentCounter, &0u64);
    }

    pub fn create_recurring(
        env: Env,
        payer: Address,
        recipient: Address,
        token: Address,
        amount: i128,
        interval_secs: u64,
        max_payments: Option<u32>,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        payer.require_auth();

        if amount <= 0 {
            panic!("invalid amount");
        }
        if interval_secs == 0 {
            panic!("invalid interval");
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PaymentCounter)
            .unwrap_or(0);
        let payment_id = counter + 1;

        let now = env.ledger().timestamp();
        let recurring = RecurringPayment {
            payment_id,
            payer: payer.clone(),
            recipient,
            token,
            amount,
            interval_secs,
            max_payments,
            total_payments: 0,
            status: RecurringStatus::Active,
            created_at: now,
            last_payment: now,
            next_payment: now + interval_secs,
        };

        let _ttl_key = DataKey::Payment(payment_id);
        env.storage().persistent().set(&_ttl_key, &recurring);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::PaymentCounter, &payment_id);

        payment_id
    }

    pub fn execute_payment(env: Env, payment_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let mut recurring: RecurringPayment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .expect("payment not found");

        if recurring.status != RecurringStatus::Active {
            panic!("payment not active");
        }

        let now = env.ledger().timestamp();
        if now < recurring.next_payment {
            panic!("too early");
        }

        if let Some(max) = recurring.max_payments {
            if recurring.total_payments >= max {
                recurring.status = RecurringStatus::Cancelled;
                let _ttl_key = DataKey::Payment(payment_id);
                env.storage().persistent().set(&_ttl_key, &recurring);
                env.storage().persistent().extend_ttl(
                    &_ttl_key,
                    PERSISTENT_LIFETIME_THRESHOLD,
                    PERSISTENT_BUMP_AMOUNT,
                );
                panic!("max payments reached");
            }
        }

        let token_client = token::Client::new(&env, &recurring.token);
        token_client.transfer(&recurring.payer, &recurring.recipient, &recurring.amount);

        recurring.total_payments += 1;
        recurring.last_payment = now;
        recurring.next_payment = now + recurring.interval_secs;

        let _ttl_key = DataKey::Payment(payment_id);
        env.storage().persistent().set(&_ttl_key, &recurring);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("recurring"), symbol_short!("paid")),
            (payment_id, recurring.amount),
        );
    }

    pub fn pause_payment(env: Env, payer: Address, payment_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        payer.require_auth();

        let mut recurring: RecurringPayment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .expect("payment not found");

        if recurring.payer != payer {
            panic!("unauthorized");
        }

        recurring.status = RecurringStatus::Paused;
        let _ttl_key = DataKey::Payment(payment_id);
        env.storage().persistent().set(&_ttl_key, &recurring);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn resume_payment(env: Env, payer: Address, payment_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        payer.require_auth();

        let mut recurring: RecurringPayment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .expect("payment not found");

        if recurring.payer != payer {
            panic!("unauthorized");
        }

        recurring.status = RecurringStatus::Active;
        recurring.next_payment = env.ledger().timestamp() + recurring.interval_secs;
        let _ttl_key = DataKey::Payment(payment_id);
        env.storage().persistent().set(&_ttl_key, &recurring);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn cancel_payment(env: Env, payer: Address, payment_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        payer.require_auth();

        let mut recurring: RecurringPayment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .expect("payment not found");

        if recurring.payer != payer {
            panic!("unauthorized");
        }

        recurring.status = RecurringStatus::Cancelled;
        let _ttl_key = DataKey::Payment(payment_id);
        env.storage().persistent().set(&_ttl_key, &recurring);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_payment(env: Env, payment_id: u64) -> Option<RecurringPayment> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
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
