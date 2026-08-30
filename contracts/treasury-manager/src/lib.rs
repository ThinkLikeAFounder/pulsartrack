//! PulsarTrack - Treasury Manager (Soroban)
//! Single-admin treasury for platform fund management on Stellar.
//!
//! Events:
//! - ("treasury", "deposit"): [token: Address, amount: i128]
//! - ("treasury", "withdraw"): [token: Address, recipient: Address, amount: i128]

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct TreasuryState {
    pub balance: i128,
    pub total_deposited: i128,
    pub total_withdrawn: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    State,
    TokenAddress,
    Paused,
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;

#[contract]
pub struct TreasuryManagerContract;

#[contractimpl]
impl TreasuryManagerContract {
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
        env.storage().instance().set(
            &DataKey::State,
            &TreasuryState {
                balance: 0,
                total_deposited: 0,
                total_withdrawn: 0,
            },
        );
    }

    pub fn deposit(env: Env, sender: Address, amount: i128) {
        Self::require_not_paused(&env);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        sender.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        let mut state: TreasuryState = env.storage().instance().get(&DataKey::State).unwrap();
        state.balance = state.balance.checked_add(amount).expect("balance overflow");
        state.total_deposited = state
            .total_deposited
            .checked_add(amount)
            .expect("total_deposited overflow");
        env.storage().instance().set(&DataKey::State, &state);

        env.events().publish(
            (symbol_short!("treasury"), symbol_short!("deposit")),
            (token, amount),
        );
    }

    pub fn withdraw(env: Env, admin: Address, recipient: Address, amount: i128) {
        Self::require_not_paused(&env);
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

        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token);

        // Use the actual on-chain balance as the authoritative source so that
        // any divergence between internal accounting and the real token balance
        // (e.g. from a direct transfer or a prior accounting bug) cannot cause
        // a panic inside token_client.transfer.
        let actual_balance = token_client.balance(&env.current_contract_address());
        if amount > actual_balance {
            panic!("insufficient on-chain token balance");
        }

        let mut state: TreasuryState = env.storage().instance().get(&DataKey::State).unwrap();
        state.balance = actual_balance
            .checked_sub(amount)
            .expect("balance underflow");
        state.total_withdrawn = state
            .total_withdrawn
            .checked_add(amount)
            .expect("total_withdrawn overflow");
        env.storage().instance().set(&DataKey::State, &state);

        token_client.transfer(&env.current_contract_address(), &recipient, &amount);

        env.events().publish(
            (symbol_short!("treasury"), symbol_short!("withdraw")),
            (token, recipient, amount),
        );
    }

    pub fn sync_balance(env: Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let actual = token::Client::new(&env, &token).balance(&env.current_contract_address());
        let mut state: TreasuryState = env.storage().instance().get(&DataKey::State).unwrap();
        state.balance = actual;
        env.storage().instance().set(&DataKey::State, &state);
    }

    pub fn get_state(env: Env) -> TreasuryState {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().instance().get(&DataKey::State).unwrap()
    }

    pub fn get_token(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap()
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
