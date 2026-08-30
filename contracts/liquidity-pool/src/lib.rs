//! PulsarTrack - Liquidity Pool (Soroban)
//! Ad budget liquidity pool for campaign funding on Stellar.

#![no_std]
use pulsar_common_fees::calculate_fee_bps;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct PoolState {
    pub total_liquidity: i128,
    pub total_borrowed: i128,
    pub reserve_factor: u32,   // percentage kept as reserve
    pub utilization_rate: u32, // percentage borrowed
    pub borrow_rate_bps: u32,  // annual rate in basis points
    pub last_updated: u64,
    pub interest_reserve: i128, // Accumulated interest payments
}

#[contracttype]
#[derive(Clone)]
pub struct ProviderPosition {
    pub provider: Address,
    pub deposited: i128,
    pub shares: i128,
    pub deposited_at: u64,
    pub last_claim: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct BorrowPosition {
    pub borrower: Address,
    pub campaign_id: u64,
    pub borrowed: i128,
    pub interest_accrued: i128,
    pub borrowed_at: u64,
    pub due_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TokenAddress,
    PoolState,
    TotalShares,
    Provider(Address),
    Borrow(u64), // campaign_id
    BorrowCount,
    Paused,
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;
// Maximum fraction of total_liquidity a single borrow transaction may draw (25%).
const MAX_BORROW_FRACTION_BPS: u32 = 2_500;

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolContract {
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
        env.storage().instance().set(&DataKey::TotalShares, &0i128);
        env.storage().instance().set(
            &DataKey::PoolState,
            &PoolState {
                total_liquidity: 0,
                total_borrowed: 0,
                reserve_factor: 10,
                utilization_rate: 0,
                borrow_rate_bps: 500, // 5% annual
                last_updated: env.ledger().timestamp(),
                interest_reserve: 0,
            },
        );
    }

    pub fn deposit(env: Env, provider: Address, amount: i128) -> i128 {
        Self::require_not_paused(&env);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        provider.require_auth();

        if amount <= 0 {
            panic!("invalid amount");
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);

        // --- Effects: compute shares and commit all state before the external call ---
        let mut pool: PoolState = env.storage().instance().get(&DataKey::PoolState).unwrap();
        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);

        // Calculate shares (1:1 for first deposit)
        let shares = if pool.total_liquidity == 0 || total_shares == 0 {
            amount
        } else {
            (amount * total_shares) / pool.total_liquidity
        };

        pool.total_liquidity += amount;
        pool.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::PoolState, &pool);
        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares + shares));

        let provider_key = DataKey::Provider(provider.clone());
        let mut position: ProviderPosition = env
            .storage()
            .persistent()
            .get(&provider_key)
            .unwrap_or(ProviderPosition {
                provider: provider.clone(),
                deposited: 0,
                shares: 0,
                deposited_at: env.ledger().timestamp(),
                last_claim: env.ledger().timestamp(),
            });

        position.deposited += amount;
        position.shares += shares;
        env.storage().persistent().set(&provider_key, &position);
        env.storage().persistent().extend_ttl(
            &provider_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // --- Interaction: pull tokens from provider after all state is committed ---
        token_client.transfer(&provider, &env.current_contract_address(), &amount);

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("deposit")),
            (provider, amount, shares),
        );

        shares
    }

    pub fn withdraw(env: Env, provider: Address, shares: i128) -> i128 {
        Self::require_not_paused(&env);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        provider.require_auth();

        if shares <= 0 {
            panic!("shares must be positive");
        }

        let mut position: ProviderPosition = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider.clone()))
            .expect("no position");

        if position.shares < shares {
            panic!("insufficient shares");
        }

        let mut pool: PoolState = env.storage().instance().get(&DataKey::PoolState).unwrap();
        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);

        if total_shares == 0 {
            panic!("no shares in pool");
        }
        let amount = shares
            .checked_mul(pool.total_liquidity)
            .expect("share redemption calculation overflows i128")
            / total_shares;
        let available = pool.total_liquidity - pool.total_borrowed;

        if amount > available {
            panic!("insufficient liquidity");
        }

        pool.total_liquidity -= amount;
        pool.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::PoolState, &pool);
        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares - shares));

        position.shares -= shares;
        position.deposited = position.deposited.saturating_sub(amount);
        let _ttl_key = DataKey::Provider(provider.clone());
        env.storage().persistent().set(&_ttl_key, &position);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &provider, &amount);

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("withdraw")),
            (provider, shares, amount),
        );

        amount
    }

    pub fn borrow(env: Env, borrower: Address, campaign_id: u64, amount: i128, duration_secs: u64) {
        Self::require_not_paused(&env);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        borrower.require_auth();

        // --- Checks: input validation before any storage read (fail-fast) ---
        if amount <= 0 {
            panic!("borrow amount must be positive");
        }

        if duration_secs == 0 {
            panic!("duration_secs must be greater than zero");
        }

        // Reject unreasonably long loan durations (max 1 year = 31,557,600 seconds)
        const MAX_DURATION_SECS: u64 = 31_557_600;
        if duration_secs > MAX_DURATION_SECS {
            panic!("duration_secs exceeds maximum allowed loan duration");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Borrow(campaign_id))
        {
            panic!("already has borrow");
        }

        // --- Pool state checks (single storage read shared by both guards) ---
        let mut pool: PoolState = env.storage().instance().get(&DataKey::PoolState).unwrap();
        let available = pool.total_liquidity - pool.total_borrowed;

        if amount > available {
            panic!("insufficient liquidity");
        }

        let max_borrow = calculate_fee_bps(pool.total_liquidity, MAX_BORROW_FRACTION_BPS);
        if amount > max_borrow {
            panic!("borrow exceeds per-transaction limit");
        }

        // --- Effects ---
        pool.total_borrowed += amount;
        if pool.total_liquidity > 0 {
            pool.utilization_rate = ((pool.total_borrowed * 100) / pool.total_liquidity) as u32;
        }
        pool.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::PoolState, &pool);

        let now = env.ledger().timestamp();
        let borrow = BorrowPosition {
            borrower: borrower.clone(),
            campaign_id,
            borrowed: amount,
            interest_accrued: 0,
            borrowed_at: now,
            due_at: now
                .checked_add(duration_secs)
                .expect("due_at timestamp overflow"),
        };

        let borrow_key = DataKey::Borrow(campaign_id);
        env.storage().persistent().set(&borrow_key, &borrow);
        env.storage().persistent().extend_ttl(
            &borrow_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // --- Interaction ---
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &borrower, &amount);

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("borrow")),
            (borrower, campaign_id, amount),
        );
    }

    /// Calculate interest accrued on a borrow position
    /// Formula: interest = principal * rate * time / (10000 * seconds_per_year)
    /// rate is in basis points (bps), so 500 bps = 5%
    fn calculate_interest(env: &Env, borrowed: i128, borrowed_at: u64, rate_bps: u32) -> i128 {
        let now = env.ledger().timestamp();
        let time_elapsed = now.saturating_sub(borrowed_at);

        // Approximate seconds per year (365.25 days)
        const SECONDS_PER_YEAR: u64 = 31_557_600;

        // interest = principal * rate_bps * time_elapsed / (10000 * SECONDS_PER_YEAR)
        // Using i128 to avoid overflow

        borrowed
            .saturating_mul(rate_bps as i128)
            .saturating_mul(time_elapsed as i128)
            / (10000i128 * SECONDS_PER_YEAR as i128)
    }

    /// Accrue interest on a borrow position
    pub fn accrue_interest(env: Env, caller: Address, campaign_id: u64) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let mut borrow: BorrowPosition = env
            .storage()
            .persistent()
            .get(&DataKey::Borrow(campaign_id))
            .expect("borrow not found");

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != admin && caller != borrow.borrower {
            panic!("unauthorized");
        }

        let pool: PoolState = env.storage().instance().get(&DataKey::PoolState).unwrap();

        // Calculate new interest since last accrual
        let new_interest = Self::calculate_interest(
            &env,
            borrow.borrowed,
            borrow.borrowed_at,
            pool.borrow_rate_bps,
        );

        borrow.interest_accrued = new_interest;

        let _ttl_key = DataKey::Borrow(campaign_id);
        env.storage().persistent().set(&_ttl_key, &borrow);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("accrue")),
            (campaign_id, new_interest),
        );

        new_interest
    }

    pub fn repay(env: Env, borrower: Address, campaign_id: u64, amount: i128) {
        Self::require_not_paused(&env);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        borrower.require_auth();

        let mut borrow: BorrowPosition = env
            .storage()
            .persistent()
            .get(&DataKey::Borrow(campaign_id))
            .expect("borrow not found");

        if borrow.borrower != borrower {
            panic!("unauthorized");
        }

        // Accrue interest up to current time
        let pool: PoolState = env.storage().instance().get(&DataKey::PoolState).unwrap();
        let accrued_interest = Self::calculate_interest(
            &env,
            borrow.borrowed,
            borrow.borrowed_at,
            pool.borrow_rate_bps,
        );
        borrow.interest_accrued = accrued_interest;

        let total_owed = borrow.borrowed + borrow.interest_accrued;

        if amount < total_owed {
            panic!("insufficient payment");
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&borrower, &env.current_contract_address(), &amount);

        let mut pool: PoolState = env.storage().instance().get(&DataKey::PoolState).unwrap();

        // Separate principal repayment from interest
        let principal_repaid = borrow.borrowed;
        let interest_paid = borrow.interest_accrued;
        let overpayment = amount.saturating_sub(total_owed);

        // Reduce total_borrowed by principal repaid
        pool.total_borrowed -= principal_repaid;

        // Split interest: reserve_factor% → protocol reserve, rest → lenders (via total_liquidity)
        let protocol_share = (interest_paid * pool.reserve_factor as i128) / 100;
        let lender_share = interest_paid - protocol_share;
        pool.interest_reserve += protocol_share;
        pool.total_liquidity += lender_share;

        if pool.total_liquidity > 0 {
            pool.utilization_rate = ((pool.total_borrowed * 100) / pool.total_liquidity) as u32;
        }
        pool.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::PoolState, &pool);

        env.storage()
            .persistent()
            .remove(&DataKey::Borrow(campaign_id));

        // Return overpayment if any
        if overpayment > 0 {
            token_client.transfer(&env.current_contract_address(), &borrower, &overpayment);
        }

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("repay")),
            (borrower, campaign_id, principal_repaid, interest_paid),
        );
    }

    pub fn get_pool_state(env: Env) -> PoolState {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::PoolState)
            .expect("not initialized")
    }

    pub fn get_provider_position(env: Env, provider: Address) -> Option<ProviderPosition> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Provider(provider))
    }

    pub fn get_borrow(env: Env, campaign_id: u64) -> Option<BorrowPosition> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Borrow(campaign_id))
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
