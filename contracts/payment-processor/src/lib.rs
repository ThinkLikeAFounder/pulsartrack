//! PulsarTrack - Payment Processor (Soroban)
//! Multi-token payment support with fee distribution on Stellar.

#![no_std]
use pulsar_common_fees::{calculate_fee_bps, checked_add};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

// ============================================================
// Data Types
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[contracttype]
#[derive(Clone)]
pub struct Payment {
    pub payer: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub fee_charged: i128,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub processed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub struct TokenConfig {
    pub enabled: bool,
    pub min_amount: i128,
    pub daily_limit: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct UserPaymentStats {
    pub total_payments: u64,
    pub total_spent: i128,
    pub last_payment: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct RevenueStats {
    pub total_fees_collected: i128,
    pub total_volume: i128,
    pub payment_count: u64,
}

// ============================================================
// Storage Keys
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TreasuryAddress,
    NextPaymentId,
    PlatformFeeBps, // basis points (250 = 2.5%)
    TokenConfig(Address),
    Payment(u64),
    UserStats(Address),
    RevenueStats(Address),
    DailyVolume(Address, u64),
    Paused,
}

// ============================================================
// Contract
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;
const SECONDS_PER_DAY: u64 = 86_400;
const APPROX_SECONDS_PER_LEDGER: u64 = 5;

#[contract]
pub struct PaymentProcessorContract;

#[contractimpl]
impl PaymentProcessorContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address, treasury: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::TreasuryAddress, &treasury);
        env.storage().instance().set(&DataKey::NextPaymentId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &250u32); // 2.5%
    }

    /// Whitelist a token for payments
    pub fn add_token(
        env: Env,
        admin: Address,
        token: Address,
        min_amount: i128,
        daily_limit: i128,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let config = TokenConfig {
            enabled: true,
            min_amount,
            daily_limit,
        };

        let _ttl_key = DataKey::TokenConfig(token);
        env.storage().persistent().set(&_ttl_key, &config);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Disable a payment token
    pub fn remove_token(env: Env, admin: Address, token: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        env.storage()
            .persistent()
            .remove(&DataKey::TokenConfig(token));
    }

    /// Process a payment
    pub fn process_payment(
        env: Env,
        payer: Address,
        recipient: Address,
        token: Address,
        amount: i128,
    ) -> u64 {
        Self::require_not_paused(&env);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        payer.require_auth();

        if payer == recipient {
            panic!("cannot pay yourself");
        }

        if amount <= 0 {
            panic!("invalid amount");
        }

        let config: TokenConfig = env
            .storage()
            .persistent()
            .get(&DataKey::TokenConfig(token.clone()))
            .expect("token not whitelisted");

        if !config.enabled {
            panic!("token disabled");
        }

        if amount < config.min_amount {
            panic!("amount below minimum");
        }

        // Check daily limit
        let current_day = env.ledger().timestamp() / SECONDS_PER_DAY;
        let daily_key = DataKey::DailyVolume(token.clone(), current_day);
        let daily_vol: i128 = env.storage().temporary().get(&daily_key).unwrap_or(0);

        let new_daily_vol = checked_add(daily_vol, amount);
        if new_daily_vol > config.daily_limit {
            panic!("daily limit exceeded");
        }

        // Calculate fee
        let fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .unwrap_or(250);
        let fee = calculate_fee_bps(amount, fee_bps);
        let net_amount = amount - fee;

        let payment_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextPaymentId)
            .unwrap_or(1);

        // Execute token transfers
        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();

        if token_client.balance(&payer) < amount {
            panic!("insufficient balance");
        }

        // FIX #559: Record payment BEFORE executing transfers (CEI pattern)
        // This ensures the payment exists in storage before any external calls
        let payment = Payment {
            payer: payer.clone(),
            recipient: recipient.clone(),
            token: token.clone(),
            amount,
            fee_charged: fee,
            status: PaymentStatus::Processing,
            created_at: env.ledger().timestamp(),
            processed_at: None,
        };

        let _ttl_key = DataKey::Payment(payment_id);
        env.storage().persistent().set(&_ttl_key, &payment);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::NextPaymentId, &(payment_id + 1));

        // Now execute token transfers
        // Transfer the full amount to the contract first so that both outgoing
        // distributions share a single payer authorization.  If either
        // distribution fails the entire transaction rolls back, including this
        // inbound transfer — making the two-leg payout atomic.
        token_client.transfer(&payer, &contract_address, &amount);
        token_client.transfer(&contract_address, &recipient, &net_amount);

        if fee > 0 {
            let treasury: Address = env
                .storage()
                .instance()
                .get(&DataKey::TreasuryAddress)
                .unwrap();
            token_client.transfer(&contract_address, &treasury, &fee);
        }

        // Update payment status to Completed after successful transfers
        let mut completed_payment = payment;
        completed_payment.status = PaymentStatus::Completed;
        completed_payment.processed_at = Some(env.ledger().timestamp());
        env.storage()
            .persistent()
            .set(&_ttl_key, &completed_payment);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Update daily volume
        env.storage().temporary().set(&daily_key, &new_daily_vol);
        let remaining_ledgers = Self::daily_volume_ttl_ledgers(&env);
        env.storage()
            .temporary()
            .extend_ttl(&daily_key, remaining_ledgers, remaining_ledgers);

        // Update user stats
        Self::_update_user_stats(&env, &payer, amount);

        // Update revenue stats
        Self::_update_revenue_stats(&env, &token, fee, amount);

        env.events().publish(
            (symbol_short!("payment"), symbol_short!("processed")),
            (payment_id, payer, recipient, amount),
        );

        payment_id
    }

    /// Update platform fee (admin only)
    pub fn set_platform_fee(env: Env, admin: Address, fee_bps: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        if fee_bps > 1000 {
            panic!("fee too high"); // max 10%
        }
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &fee_bps);
    }

    // ============================================================
    // Read-Only Functions
    // ============================================================

    pub fn get_payment(env: Env, payment_id: u64) -> Option<Payment> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
    }

    pub fn get_user_stats(env: Env, user: Address) -> Option<UserPaymentStats> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::UserStats(user))
    }

    pub fn get_revenue_stats(env: Env, token: Address) -> Option<RevenueStats> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::RevenueStats(token))
    }

    pub fn get_token_config(env: Env, token: Address) -> Option<TokenConfig> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::TokenConfig(token))
    }

    pub fn get_daily_volume(env: Env, token: Address) -> i128 {
        let day = env.ledger().timestamp() / SECONDS_PER_DAY;
        env.storage()
            .temporary()
            .get(&DataKey::DailyVolume(token, day))
            .unwrap_or(0)
    }

    // ============================================================
    // Internal Helpers
    // ============================================================

    fn _update_user_stats(env: &Env, user: &Address, amount: i128) {
        let key = DataKey::UserStats(user.clone());
        let mut stats: UserPaymentStats =
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(UserPaymentStats {
                    total_payments: 0,
                    total_spent: 0,
                    last_payment: 0,
                });

        stats.total_payments += 1;
        stats.total_spent += amount;
        stats.last_payment = env.ledger().timestamp();
        env.storage().persistent().set(&key, &stats);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    fn _update_revenue_stats(env: &Env, token: &Address, fee: i128, volume: i128) {
        let key = DataKey::RevenueStats(token.clone());
        let mut stats: RevenueStats =
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(RevenueStats {
                    total_fees_collected: 0,
                    total_volume: 0,
                    payment_count: 0,
                });

        stats.total_fees_collected += fee;
        stats.total_volume += volume;
        stats.payment_count += 1;
        env.storage().persistent().set(&key, &stats);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    fn daily_volume_ttl_ledgers(env: &Env) -> u32 {
        let seconds_elapsed_today = env.ledger().timestamp() % SECONDS_PER_DAY;
        let seconds_left_today = SECONDS_PER_DAY - seconds_elapsed_today;

        // Keep the temporary daily-volume key alive through the end of the
        // current day so the limit cannot reset early.
        ((seconds_left_today / APPROX_SECONDS_PER_LEDGER) + 1) as u32
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
