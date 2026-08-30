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
    pub interval_secs: u64,
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

/// Maximum allowed payment interval: 1 year in seconds.
/// Prevents u64 overflow when computing `now + interval_secs`.
const MAX_INTERVAL_SECS: u64 = 365 * 24 * 3_600;

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

        // FIX: enforce both lower and upper bounds to prevent u64 overflow
        // when computing next_payment = now + interval_secs.
        if interval_secs == 0 || interval_secs > MAX_INTERVAL_SECS {
            panic!("interval out of valid range");
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PaymentCounter)
            .unwrap_or(0);
        let payment_id = counter + 1;

        let now = env.ledger().timestamp();

        // FIX: use checked_add as a runtime safety net against any residual overflow.
        let next_payment = now
            .checked_add(interval_secs)
            .expect("next_payment timestamp overflow");

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
            next_payment,
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

    pub fn execute_payment(env: Env, caller: Address, payment_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        caller.require_auth();

        let mut recurring: RecurringPayment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .expect("payment not found");

        // Only payer, recipient, or admin can execute
        if caller != recurring.payer && caller != recurring.recipient {
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .expect("admin not found");
            if caller != admin {
                panic!("unauthorized");
            }
        }

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

        // Use SEP-41 allowance pattern for automated execution
        let token_client = token::Client::new(&env, &recurring.token);
        token_client.transfer_from(&env.current_contract_address(), &recurring.payer, &recurring.recipient, &recurring.amount);

        recurring.total_payments += 1;
        recurring.last_payment = now;

        // FIX: use checked_add to prevent overflow when advancing the schedule.
        recurring.next_payment = now
            .checked_add(recurring.interval_secs)
            .expect("next_payment timestamp overflow");

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

        // FIX #558: Only allow pausing Active payments
        if recurring.status != RecurringStatus::Active {
            panic!("can only pause an active payment");
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

        // FIX #557: Only allow resuming Paused payments
        if recurring.status != RecurringStatus::Paused {
            panic!("payment is not paused");
        }

        recurring.status = RecurringStatus::Active;

        // FIX: use checked_add to prevent overflow when resuming the schedule.
        recurring.next_payment = env
            .ledger()
            .timestamp()
            .checked_add(recurring.interval_secs)
            .expect("next_payment timestamp overflow");

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
