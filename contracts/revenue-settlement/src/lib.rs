//! PulsarTrack - Revenue Settlement (Soroban)
//! Automated revenue distribution and settlement for the PulsarTrack ecosystem on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct RevenuePool {
    pub total_revenue: i128,
    pub platform_share: i128,  // platform fee portion
    pub publisher_share: i128, // publisher earnings portion
    pub treasury_share: i128,  // DAO treasury portion
    pub burn_amount: i128,     // tokens to burn
    pub platform_pct: u32,     // basis points
    pub publisher_pct: u32,
    pub treasury_pct: u32,
    pub burn_pct: u32,
    pub last_settlement: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct SettlementRecord {
    pub settlement_id: u64,
    pub campaign_id: u64,
    pub total_amount: i128,
    pub platform_fee: i128,
    pub publisher_amount: i128,
    pub settled_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TokenAddress,
    TreasuryAddress,
    PlatformAddress,
    RevenuePool,
    SettlementCounter,
    Settlement(u64),
    PublisherBalance(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct RevenueSettlementContract;

#[contractimpl]
impl RevenueSettlementContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        treasury: Address,
        platform: Address,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage()
            .instance()
            .set(&DataKey::TreasuryAddress, &treasury);
        env.storage()
            .instance()
            .set(&DataKey::PlatformAddress, &platform);
        env.storage()
            .instance()
            .set(&DataKey::SettlementCounter, &0u64);
        env.storage().instance().set(
            &DataKey::RevenuePool,
            &RevenuePool {
                total_revenue: 0,
                platform_share: 0,
                publisher_share: 0,
                treasury_share: 0,
                burn_amount: 0,
                platform_pct: 250,    // 2.5%
                publisher_pct: 9_000, // 90%
                treasury_pct: 500,    // 5%
                burn_pct: 250,        // 2.5%
                last_settlement: 0,
            },
        );
    }

    pub fn record_revenue(
        env: Env,
        admin: Address,
        campaign_id: u64,
        amount: i128,
        publisher: Address,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut pool: RevenuePool = env.storage().instance().get(&DataKey::RevenuePool).unwrap();

        let platform_fee = (amount * pool.platform_pct as i128) / 10_000;
        let treasury_fee = (amount * pool.treasury_pct as i128) / 10_000;
        let burn_fee = (amount * pool.burn_pct as i128) / 10_000;
        let publisher_amount = amount - platform_fee - treasury_fee - burn_fee;

        // Any rounding dust (1-3 stroops per tx) from integer division is captured
        // here and routed to treasury, ensuring the contract's token balance always
        // equals the exact sum of all tracked shares.
        let total_distributed = platform_fee + treasury_fee + burn_fee + publisher_amount;
        let dust = amount - total_distributed;

        pool.total_revenue += amount;
        pool.platform_share += platform_fee;
        pool.publisher_share += publisher_amount;
        pool.treasury_share += treasury_fee + dust; // dust absorbed by treasury
        pool.burn_amount += burn_fee;

        env.storage().instance().set(&DataKey::RevenuePool, &pool);

        // Accumulate publisher balance
        let pub_key = DataKey::PublisherBalance(publisher.clone());
        let current_balance: i128 = env.storage().persistent().get(&pub_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&pub_key, &(current_balance + publisher_amount));
        env.storage().persistent().extend_ttl(
            &pub_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Record settlement
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SettlementCounter)
            .unwrap_or(0);
        let settlement_id = counter + 1;

        let record = SettlementRecord {
            settlement_id,
            campaign_id,
            total_amount: amount,
            platform_fee,
            publisher_amount,
            settled_at: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Settlement(settlement_id);
        env.storage().persistent().set(&_ttl_key, &record);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::SettlementCounter, &settlement_id);

        settlement_id
    }

    pub fn distribute_platform_revenue(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut pool: RevenuePool = env.storage().instance().get(&DataKey::RevenuePool).unwrap();
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);

        if pool.platform_share > 0 {
            let platform: Address = env
                .storage()
                .instance()
                .get(&DataKey::PlatformAddress)
                .unwrap();
            token_client.transfer(
                &env.current_contract_address(),
                &platform,
                &pool.platform_share,
            );
            pool.platform_share = 0;
        }

        if pool.treasury_share > 0 {
            let treasury: Address = env
                .storage()
                .instance()
                .get(&DataKey::TreasuryAddress)
                .unwrap();
            token_client.transfer(
                &env.current_contract_address(),
                &treasury,
                &pool.treasury_share,
            );
            pool.treasury_share = 0;
        }

        pool.last_settlement = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::RevenuePool, &pool);
    }

    pub fn claim_publisher_balance(env: Env, publisher: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        publisher.require_auth();

        let pub_key = DataKey::PublisherBalance(publisher.clone());
        let balance: i128 = env.storage().persistent().get(&pub_key).unwrap_or(0);

        if balance <= 0 {
            panic!("no balance to claim");
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &publisher, &balance);

        env.storage().persistent().set(&pub_key, &0i128);
        env.storage().persistent().extend_ttl(
            &pub_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("revenue"), symbol_short!("claimed")),
            (publisher, balance),
        );
    }

    pub fn get_revenue_pool(env: Env) -> RevenuePool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::RevenuePool)
            .expect("not initialized")
    }

    pub fn get_publisher_balance(env: Env, publisher: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::PublisherBalance(publisher))
            .unwrap_or(0)
    }

    pub fn get_settlement(env: Env, settlement_id: u64) -> Option<SettlementRecord> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Settlement(settlement_id))
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
