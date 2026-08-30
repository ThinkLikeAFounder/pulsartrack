//! PulsarTrack - Wrapped Token Manager (Soroban)
//! Manages wrapped tokens from other chains for use in PulsarTrack campaigns on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env, String};

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
pub struct RegisterTokenParams {
    pub symbol: String,
    pub name: String,
    pub decimals: u32,
    pub underlying_chain: String,
    pub underlying_address: String,
    pub stellar_token: Address,
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
    MintingPaused,
    WrapRecordCounter,
    WrappedToken(String), // symbol
    WrapRecord(u64),
    UserBalance(String, Address), // symbol, user
    ProcessedTx(String),          // source transaction ID
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
            .set(&DataKey::MintingPaused, &false);
        env.storage()
            .instance()
            .set(&DataKey::WrapRecordCounter, &0u64);
    }

    pub fn register_wrapped_token(env: Env, admin: Address, params: RegisterTokenParams) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let wrapped = WrappedToken {
            symbol: params.symbol.clone(),
            name: params.name,
            decimals: params.decimals,
            underlying_chain: params.underlying_chain,
            underlying_address: params.underlying_address,
            stellar_token: params.stellar_token,
            total_wrapped: 0,
            is_active: true,
        };

        let _ttl_key = DataKey::WrappedToken(params.symbol);
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
        let minting_paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::MintingPaused)
            .unwrap_or(false);
        if minting_paused {
            panic!("minting paused");
        }

        // Check for replay attack - ensure source_tx hasn't been processed
        let tx_key = DataKey::ProcessedTx(source_tx.clone());
        if env.storage().persistent().has(&tx_key) {
            panic!("source transaction already processed");
        }

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut wrapped: WrappedToken = env
            .storage()
            .persistent()
            .get(&DataKey::WrappedToken(symbol.clone()))
            .expect("token not registered");

        if !wrapped.is_active {
            panic!("token not active");
        }

        // Mint stellar-side tokens; the wrapped-token contract must be the stellar token's admin
        let stellar_asset_client = token::StellarAssetClient::new(&env, &wrapped.stellar_token);
        stellar_asset_client.mint(&recipient, &amount);

        wrapped.total_wrapped = wrapped
            .total_wrapped
            .checked_add(amount)
            .expect("total_wrapped overflow");
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
            token: symbol.clone(),
            amount,
            source_tx: source_tx.clone(),
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

        // Mark source transaction as processed to prevent replay attacks
        env.storage().persistent().set(&tx_key, &true);
        env.storage().persistent().extend_ttl(
            &tx_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

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

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut wrapped: WrappedToken = env
            .storage()
            .persistent()
            .get(&DataKey::WrappedToken(symbol.clone()))
            .expect("token not registered");

        // Verify user's on-chain balance before burning
        let token_client = token::Client::new(&env, &wrapped.stellar_token);
        let current_balance = token_client.balance(&user);

        if current_balance < amount {
            panic!("insufficient balance");
        }

        if amount > wrapped.total_wrapped {
            panic!("burn amount exceeds total wrapped supply");
        }

        // FIX #560: Update total_wrapped BEFORE burning (CEI pattern)
        // This ensures accounting is updated before the external call
        wrapped.total_wrapped = wrapped
            .total_wrapped
            .checked_sub(amount)
            .expect("total_wrapped underflow");
        let _ttl_key = DataKey::WrappedToken(symbol);
        env.storage().persistent().set(&_ttl_key, &wrapped);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Now burn the user's stellar-side tokens (SEP-41 burn requires user auth in the call tree)
        token_client.burn(&user, &amount);

        env.events().publish(
            (symbol_short!("wrapped"), symbol_short!("burned")),
            (user, amount, target_address),
        );
    }

    pub fn set_relayer(env: Env, admin: Address, new_relayer: Address) {
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
            .set(&DataKey::RelayerAddress, &new_relayer);
    }

    pub fn set_minting_paused(env: Env, admin: Address, paused: bool) {
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
            .set(&DataKey::MintingPaused, &paused);
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
        let wrapped: WrappedToken = match env
            .storage()
            .persistent()
            .get(&DataKey::WrappedToken(symbol))
        {
            Some(w) => w,
            None => return 0,
        };
        token::Client::new(&env, &wrapped.stellar_token).balance(&user)
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
