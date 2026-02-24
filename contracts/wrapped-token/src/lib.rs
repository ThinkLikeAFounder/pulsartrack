//! PulsarTrack - Wrapped Token Manager (Soroban)
//! Manages wrapped tokens from other chains for use in PulsarTrack campaigns on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct WrappedToken {
    pub symbol: String,
    pub name: String,
    pub decimals: u32,
    pub underlying_chain: String,
    pub underlying_address: String,
    pub stellar_token: Address,
    pub total_wrapped: i128,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct WrapRecord {
    pub record_id: u64,
    pub user: Address,
    pub token: String,
    pub amount: i128,
    pub source_tx: String, // Transaction ID on source chain
    pub wrapped_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    RelayerAddress,
    WrapRecordCounter,
    WrappedToken(String), // symbol
    WrapRecord(u64),
    UserBalance(String, Address), // symbol, user
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct WrappedTokenContract;

#[contractimpl]
impl WrappedTokenContract {
    pub fn initialize(env: Env, admin: Address, relayer: Address) {
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
            .set(&DataKey::RelayerAddress, &relayer);
        env.storage()
            .instance()
            .set(&DataKey::WrapRecordCounter, &0u64);
    }

    pub fn register_wrapped_token(
        env: Env,
        admin: Address,
        symbol: String,
        name: String,
        decimals: u32,
        underlying_chain: String,
        underlying_address: String,
        stellar_token: Address,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let wrapped = WrappedToken {
            symbol: symbol.clone(),
            name,
            decimals,
            underlying_chain,
            underlying_address,
            stellar_token,
            total_wrapped: 0,
            is_active: true,
        };

        let _ttl_key = DataKey::WrappedToken(symbol);
        env.storage().persistent().set(&_ttl_key, &wrapped);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn mint_wrapped(
        env: Env,
        relayer: Address,
        symbol: String,
        recipient: Address,
        amount: i128,
        source_tx: String,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        relayer.require_auth();
        let stored_relayer: Address = env
            .storage()
            .instance()
            .get(&DataKey::RelayerAddress)
            .unwrap();
        if relayer != stored_relayer {
            panic!("unauthorized relayer");
        }

        let mut wrapped: WrappedToken = env
            .storage()
            .persistent()
            .get(&DataKey::WrappedToken(symbol.clone()))
            .expect("token not registered");

        if !wrapped.is_active {
            panic!("token not active");
        }

        // Mint stellar-side tokens
        // NOTE: This would use the token contract's mint function in production
        // For now, we track the internal balance
        let key = DataKey::UserBalance(symbol.clone(), recipient.clone());
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(current + amount));
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        wrapped.total_wrapped += amount;
        let _ttl_key = DataKey::WrappedToken(symbol.clone());
        env.storage().persistent().set(&_ttl_key, &wrapped);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::WrapRecordCounter)
            .unwrap_or(0);
        let record_id = counter + 1;

        let record = WrapRecord {
            record_id,
            user: recipient.clone(),
            token: symbol,
            amount,
            source_tx,
            wrapped_at: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::WrapRecord(record_id);
        env.storage().persistent().set(&_ttl_key, &record);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::WrapRecordCounter, &record_id);

        env.events().publish(
            (symbol_short!("wrapped"), symbol_short!("minted")),
            (record_id, recipient, amount),
        );

        record_id
    }

    pub fn burn_wrapped(
        env: Env,
        user: Address,
        symbol: String,
        amount: i128,
        target_address: String,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        user.require_auth();

        let key = DataKey::UserBalance(symbol.clone(), user.clone());
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);

        if current < amount {
            panic!("insufficient balance");
        }

        env.storage().persistent().set(&key, &(current - amount));
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let mut wrapped: WrappedToken = env
            .storage()
            .persistent()
            .get(&DataKey::WrappedToken(symbol.clone()))
            .expect("token not registered");

        wrapped.total_wrapped -= amount;
        let _ttl_key = DataKey::WrappedToken(symbol);
        env.storage().persistent().set(&_ttl_key, &wrapped);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("wrapped"), symbol_short!("burned")),
            (user, amount, target_address),
        );
    }

    pub fn get_wrapped_token(env: Env, symbol: String) -> Option<WrappedToken> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::WrappedToken(symbol))
    }

    pub fn get_user_balance(env: Env, symbol: String, user: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::UserBalance(symbol, user))
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
