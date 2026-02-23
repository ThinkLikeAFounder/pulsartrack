//! PulsarTrack - Subscription Manager (Soroban)
//! Manages platform subscription plans and billing on Stellar.
//!
//! # Subscription Lifecycle
//!
//! - `subscribe()`    — New subscriptions only (panics if one is active).
//! - `change_tier()`  — Upgrade to a higher tier with prorated credit (blocks downgrades).
//! - `renew()`        — Extend the current tier; stacks on top of remaining time.
//! - `cancel()`       — Disables auto-renewal; subscription remains active until expiry.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, String,
};

// ============================================================
// Data Types
// ============================================================

/// Subscription tier ordered by rank. Use `tier_rank()` for comparison.
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum SubscriptionTier {
    Starter,
    Growth,
    Business,
    Enterprise,
}

#[contracttype]
#[derive(Clone)]
pub struct SubscriptionPlan {
    pub tier: SubscriptionTier,
    pub name: String,
    pub price_monthly: i128,
    pub price_annual: i128,
    pub max_campaigns: u32,
    pub max_impressions_per_month: u64,
    pub max_publishers: u32,
    pub analytics_enabled: bool,
    pub api_access: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Subscription {
    pub subscriber: Address,
    pub tier: SubscriptionTier,
    pub is_annual: bool,
    pub amount_paid: i128,      // last payment amount; used for proration
    pub started_at: u64,        // original subscription start; preserved across upgrades
    pub expires_at: u64,
    pub auto_renew: bool,
    pub campaigns_used: u32,    // preserved across upgrades and renewals
    pub impressions_used: u64,  // preserved across upgrades and renewals
}

// ============================================================
// Storage Keys
// ============================================================

#[contracttype]
pub enum DataKey {
    Admin,
    TokenAddress,
    TreasuryAddress,
    Plan(SubscriptionTier),
    Subscription(Address),
}

// ============================================================
// Error Codes
// ============================================================

/// Subscriber already has an active subscription. Use `change_tier()` or `renew()`.
pub const ERR_ALREADY_ACTIVE: u32 = 1;
/// No active subscription found for this address.
pub const ERR_NO_ACTIVE_SUB: u32 = 2;
/// Cannot downgrade while a subscription is active.
pub const ERR_DOWNGRADE_BLOCKED: u32 = 3;
/// New tier is the same as the current tier. Use `renew()` instead.
pub const ERR_SAME_TIER: u32 = 4;

// ============================================================
// TTL Constants
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

// ============================================================
// Pure Helpers  (zero allocations, O(1))
// ============================================================

/// Returns a numeric rank for tier comparison.
/// Higher rank = higher tier.
#[inline]
fn tier_rank(tier: &SubscriptionTier) -> u32 {
    match tier {
        SubscriptionTier::Starter    => 0,
        SubscriptionTier::Growth     => 1,
        SubscriptionTier::Business   => 2,
        SubscriptionTier::Enterprise => 3,
    }
}

/// Returns the billing period in seconds.
#[inline]
fn plan_period_secs(is_annual: bool) -> u64 {
    if is_annual {
        365 * 24 * 3600u64
    } else {
        30 * 24 * 3600u64
    }
}

/// Computes the prorated credit for an existing subscription.
///
/// Formula (integer arithmetic, no floating point):
///   `credit = (amount_paid * remaining_secs) / total_period_secs`
///
/// Division truncates toward zero (rounds down the refund — conservative,
/// safe for the platform). Maximum rounding error is 1 stroop.
///
/// Returns 0 if `now >= expires_at` (expired; no credit due).
#[inline]
fn prorated_credit(amount_paid: i128, started_at: u64, expires_at: u64, now: u64) -> i128 {
    if now >= expires_at {
        return 0;
    }
    let remaining_secs = (expires_at - now) as i128;
    // Use the actual subscription period rather than a fixed constant,
    // so annual and monthly subs are both handled correctly.
    let total_period_secs = (expires_at - started_at) as i128;
    if total_period_secs == 0 {
        return 0;
    }
    (amount_paid * remaining_secs) / total_period_secs
}

// ============================================================
// Storage Helpers  (deduplicate persistent write + TTL bump)
// ============================================================

#[inline]
fn save_subscription(env: &Env, sub: &Subscription) {
    let key = DataKey::Subscription(sub.subscriber.clone());
    env.storage().persistent().set(&key, sub);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

#[inline]
fn load_subscription(env: &Env, subscriber: &Address) -> Option<Subscription> {
    env.storage()
        .persistent()
        .get(&DataKey::Subscription(subscriber.clone()))
}

#[inline]
fn load_plan(env: &Env, tier: SubscriptionTier) -> SubscriptionPlan {
    env.storage()
        .persistent()
        .get(&DataKey::Plan(tier))
        .expect("plan not found")
}

#[inline]
fn load_token_and_treasury(env: &Env) -> (Address, Address) {
    let token: Address = env
        .storage()
        .instance()
        .get(&DataKey::TokenAddress)
        .unwrap();
    let treasury: Address = env
        .storage()
        .instance()
        .get(&DataKey::TreasuryAddress)
        .unwrap();
    (token, treasury)
}

/// Transfers `amount` from `from` to treasury. If `amount` is 0, skips the transfer
/// (avoids a wasted host call and unnecessary fee).
#[inline]
fn charge(env: &Env, from: &Address, amount: i128) {
    if amount > 0 {
        let (token_addr, treasury) = load_token_and_treasury(env);
        token::Client::new(env, &token_addr).transfer(from, &treasury, &amount);
    }
}

#[inline]
fn plan_price(plan: &SubscriptionPlan, is_annual: bool) -> i128 {
    if is_annual {
        plan.price_annual
    } else {
        plan.price_monthly
    }
}

// ============================================================
// Contract
// ============================================================

#[contract]
pub struct SubscriptionManagerContract;

#[contractimpl]
impl SubscriptionManagerContract {
    // ----------------------------------------------------------
    // Admin
    // ----------------------------------------------------------

    pub fn initialize(env: Env, admin: Address, token: Address, treasury: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage().instance().set(&DataKey::TreasuryAddress, &treasury);
        Self::_init_plans(&env);
    }

    // ----------------------------------------------------------
    // Lifecycle: New Subscriptions
    // ----------------------------------------------------------

    /// Create a **new** subscription.
    ///
    /// Panics with `ERR_ALREADY_ACTIVE` if the subscriber already has an active
    /// subscription. Use `change_tier()` to upgrade, or `renew()` to extend.
    pub fn subscribe(
        env: Env,
        subscriber: Address,
        tier: SubscriptionTier,
        is_annual: bool,
        auto_renew: bool,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        subscriber.require_auth();

        let now = env.ledger().timestamp();

        // Guard: block re-subscription while active.
        if let Some(existing) = load_subscription(&env, &subscriber) {
            if existing.expires_at > now {
                panic!("already active: use change_tier or renew");
            }
        }

        let plan = load_plan(&env, tier.clone());
        let amount = plan_price(&plan, is_annual);
        let period = plan_period_secs(is_annual);

        charge(&env, &subscriber, amount);

        let sub = Subscription {
            subscriber: subscriber.clone(),
            tier,
            is_annual,
            amount_paid: amount,
            started_at: now,
            expires_at: now + period,
            auto_renew,
            campaigns_used: 0,
            impressions_used: 0,
        };
        save_subscription(&env, &sub);

        env.events().publish(
            (symbol_short!("sub"), symbol_short!("new")),
            (subscriber, amount),
        );
    }

    // ----------------------------------------------------------
    // Lifecycle: Upgrades
    // ----------------------------------------------------------

    /// Upgrade an active subscription to a **higher** tier.
    ///
    /// - Computes a prorated credit for remaining time on the current plan.
    /// - Charges only the delta (`new_price - credit`), clamped to 0.
    /// - Preserves `campaigns_used` and `impressions_used`.
    /// - Resets the billing period from now for the new tier.
    ///
    /// Panics:
    /// - `ERR_NO_ACTIVE_SUB`     — no active subscription exists.
    /// - `ERR_DOWNGRADE_BLOCKED` — `new_tier` has a lower rank than the current tier.
    /// - `ERR_SAME_TIER`         — `new_tier` is identical; use `renew()`.
    pub fn change_tier(
        env: Env,
        subscriber: Address,
        new_tier: SubscriptionTier,
        is_annual: bool,
        auto_renew: bool,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        subscriber.require_auth();

        let now = env.ledger().timestamp();

        let existing = load_subscription(&env, &subscriber)
            .filter(|s| s.expires_at > now)
            .unwrap_or_else(|| panic!("no active subscription"));

        let current_rank = tier_rank(&existing.tier);
        let new_rank = tier_rank(&new_tier);

        if new_rank < current_rank {
            panic!("downgrade not allowed while active");
        }
        if new_rank == current_rank {
            panic!("same tier: use renew instead");
        }

        // Prorate credit from the existing subscription.
        let credit = prorated_credit(
            existing.amount_paid,
            existing.started_at,
            existing.expires_at,
            now,
        );

        let new_plan = load_plan(&env, new_tier.clone());
        let new_price = plan_price(&new_plan, is_annual);

        // Net charge clamped to 0 (credit can never exceed what was paid).
        let net_charge = (new_price - credit).max(0);
        charge(&env, &subscriber, net_charge);

        let period = plan_period_secs(is_annual);
        let sub = Subscription {
            subscriber: subscriber.clone(),
            tier: new_tier,
            is_annual,
            amount_paid: new_price,  // store full price for future proration
            started_at: now,
            expires_at: now + period,
            auto_renew,
            // Preserve accumulated usage data.
            campaigns_used: existing.campaigns_used,
            impressions_used: existing.impressions_used,
        };
        save_subscription(&env, &sub);

        env.events().publish(
            (symbol_short!("sub"), symbol_short!("upgrade")),
            (subscriber, net_charge, credit),
        );
    }

    // ----------------------------------------------------------
    // Lifecycle: Renewal
    // ----------------------------------------------------------

    /// Renew the **current** tier, extending expiry by one billing period.
    ///
    /// - If the subscription is still active, the new expiry stacks on top of
    ///   the existing `expires_at` (no time is lost).
    /// - If the subscription has lapsed, renewal starts from now.
    /// - Preserves `campaigns_used` and `impressions_used`.
    /// - `started_at` is preserved (reflects the original join date).
    ///
    /// Panics if no subscription record exists for the address.
    pub fn renew(
        env: Env,
        subscriber: Address,
        is_annual: bool,
        auto_renew: bool,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        subscriber.require_auth();

        let now = env.ledger().timestamp();

        let mut existing = load_subscription(&env, &subscriber)
            .unwrap_or_else(|| panic!("no subscription found"));

        let plan = load_plan(&env, existing.tier.clone());
        let amount = plan_price(&plan, is_annual);
        let period = plan_period_secs(is_annual);

        charge(&env, &subscriber, amount);

        // Stack on top of remaining time; never go backwards.
        let base = existing.expires_at.max(now);
        existing.expires_at = base + period;
        existing.is_annual = is_annual;
        existing.auto_renew = auto_renew;
        existing.amount_paid = amount;

        save_subscription(&env, &existing);

        env.events().publish(
            (symbol_short!("sub"), symbol_short!("renew")),
            (subscriber, amount),
        );
    }

    // ----------------------------------------------------------
    // Lifecycle: Cancellation
    // ----------------------------------------------------------

    /// Disable auto-renewal. The subscription remains active until `expires_at`.
    ///
    /// Panics if no subscription record exists.
    pub fn cancel(env: Env, subscriber: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        subscriber.require_auth();

        let mut sub = load_subscription(&env, &subscriber)
            .unwrap_or_else(|| panic!("subscription not found"));

        sub.auto_renew = false;
        save_subscription(&env, &sub);
    }

    // ----------------------------------------------------------
    // Usage Tracking  (called by campaign-orchestrator)
    // ----------------------------------------------------------

    /// Increment campaign usage counter for the subscriber.
    /// Panics if no subscription exists.
    pub fn record_campaign_used(env: Env, subscriber: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let mut sub = load_subscription(&env, &subscriber)
            .unwrap_or_else(|| panic!("subscription not found"));

        sub.campaigns_used += 1;
        save_subscription(&env, &sub);
    }

    /// Increment impression usage counter for the subscriber.
    /// Panics if no subscription exists.
    pub fn record_impression(env: Env, subscriber: Address, count: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let mut sub = load_subscription(&env, &subscriber)
            .unwrap_or_else(|| panic!("subscription not found"));

        sub.impressions_used += count;
        save_subscription(&env, &sub);
    }

    // ----------------------------------------------------------
    // Read-Only Views
    // ----------------------------------------------------------

    pub fn is_active(env: Env, subscriber: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        load_subscription(&env, &subscriber)
            .map(|s| s.expires_at > env.ledger().timestamp())
            .unwrap_or(false)
    }

    pub fn get_subscription(env: Env, subscriber: Address) -> Option<Subscription> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        load_subscription(&env, &subscriber)
    }

    pub fn get_plan(env: Env, tier: SubscriptionTier) -> Option<SubscriptionPlan> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Plan(tier))
    }

    // ----------------------------------------------------------
    // Internal
    // ----------------------------------------------------------

    fn _init_plans(env: &Env) {
        // (tier, name, monthly_stroops, annual_stroops, max_campaigns,
        //  max_impressions/month, max_publishers, analytics, api_access)
        let plans: [(SubscriptionTier, &str, i128, i128, u32, u64, u32, bool, bool); 4] = [
            (SubscriptionTier::Starter,    "Starter",    99_000_000,         990_000_000,       5,    100_000,    10,  false, false),
            (SubscriptionTier::Growth,     "Growth",     299_000_000,       2_990_000_000,      25,   500_000,    50,  true,  false),
            (SubscriptionTier::Business,   "Business",   799_000_000,       7_990_000_000,     100, 2_000_000,   200,  true,  true),
            (SubscriptionTier::Enterprise, "Enterprise", 1_999_000_000,    19_990_000_000,    1000, 10_000_000, 1000,  true,  true),
        ];

        for (tier, name, monthly, annual, max_campaigns, max_impressions, max_pubs, analytics, api) in plans {
            let plan = SubscriptionPlan {
                tier: tier.clone(),
                name: String::from_str(env, name),
                price_monthly: monthly,
                price_annual: annual,
                max_campaigns,
                max_impressions_per_month: max_impressions,
                max_publishers: max_pubs,
                analytics_enabled: analytics,
                api_access: api,
            };
            let key = DataKey::Plan(tier);
            env.storage().persistent().set(&key, &plan);
            env.storage()
                .persistent()
                .extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        }
    }
}

mod test;
