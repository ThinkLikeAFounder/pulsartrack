//! PulsarTrack - Governance Token (Soroban / SEP-41 compatible)
//! PULSAR governance token with voting power and delegation on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

// ============================================================
// Data Types
// ============================================================

#[contracttype]
#[derive(Clone)]
pub struct Delegation {
    pub delegate: Address,
    pub delegated_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

// ============================================================
// Storage Keys
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TotalSupply,
    MaxSupply,
    Metadata,
    Balance(Address),
    Allowance(Address, Address),
    Delegation(Address),
    VotingSnapshot(Address, u32), // Address, ledger_sequence
}

pub const MAX_SUPPLY: i128 = 1_000_000_000_000; // 1M tokens with 6 decimals

// ============================================================
// Contract
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 34_560;
const PERSISTENT_BUMP_AMOUNT: u32 = 259_200;

#[contract]
pub struct GovernanceTokenContract;

#[contractimpl]
impl GovernanceTokenContract {
    /// Initialize the PULSAR governance token
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::MaxSupply, &MAX_SUPPLY);

        let metadata = TokenMetadata {
            name: String::from_str(&env, "PulsarTrack Governance"),
            symbol: String::from_str(&env, "PULSAR"),
            decimals: 7,
        };
        env.storage().instance().set(&DataKey::Metadata, &metadata);
    }

    /// Get token name
    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let meta: TokenMetadata = env.storage().instance().get(&DataKey::Metadata).unwrap();
        meta.name
    }

    /// Get token symbol
    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let meta: TokenMetadata = env.storage().instance().get(&DataKey::Metadata).unwrap();
        meta.symbol
    }

    /// Get token decimals
    pub fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let meta: TokenMetadata = env.storage().instance().get(&DataKey::Metadata).unwrap();
        meta.decimals
    }

    /// Get balance of an address
    pub fn balance(env: Env, account: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Balance(account))
            .unwrap_or(0)
    }

    /// Get total supply
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    /// Transfer tokens
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        from.require_auth();

        if amount <= 0 {
            panic!("invalid amount");
        }

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if from_balance < amount {
            panic!("insufficient balance");
        }

        let _ttl_key = DataKey::Balance(from.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &(from_balance - amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        let _ttl_key = DataKey::Balance(to.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &(to_balance + amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events()
            .publish((symbol_short!("transfer"),), (from, to, amount));
    }

    /// Transfer from (requires prior approval)
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        spender.require_auth();

        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);

        if allowance < amount {
            panic!("insufficient allowance");
        }

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if from_balance < amount {
            panic!("insufficient balance");
        }

        let _ttl_key = DataKey::Allowance(from.clone(), spender);
        env.storage()
            .persistent()
            .set(&_ttl_key, &(allowance - amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::Balance(from.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &(from_balance - amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        let _ttl_key = DataKey::Balance(to.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &(to_balance + amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Approve token spending
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128, _expiry: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        owner.require_auth();
        let _ttl_key = DataKey::Allowance(owner, spender);
        env.storage().persistent().set(&_ttl_key, &amount);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Get allowance
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Allowance(owner, spender))
            .unwrap_or(0)
    }

    /// Mint new tokens (admin only)
    pub fn mint(env: Env, admin: Address, recipient: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let current_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        if current_supply + amount > MAX_SUPPLY {
            panic!("exceeds max supply");
        }

        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(recipient.clone()))
            .unwrap_or(0);
        let _ttl_key = DataKey::Balance(recipient);
        env.storage()
            .persistent()
            .set(&_ttl_key, &(balance + amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(current_supply + amount));
    }

    /// Burn tokens
    pub fn burn(env: Env, from: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        from.require_auth();

        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if balance < amount {
            panic!("insufficient balance");
        }

        let _ttl_key = DataKey::Balance(from);
        env.storage()
            .persistent()
            .set(&_ttl_key, &(balance - amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply - amount));
    }

    /// Delegate voting power
    pub fn delegate(env: Env, delegator: Address, delegate_to: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        delegator.require_auth();

        let delegation = Delegation {
            delegate: delegate_to.clone(),
            delegated_at: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Delegation(delegator.clone());
        env.storage().persistent().set(&_ttl_key, &delegation);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events()
            .publish((symbol_short!("delegate"),), (delegator, delegate_to));
    }

    /// Revoke delegation
    pub fn revoke_delegation(env: Env, delegator: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        delegator.require_auth();
        env.storage()
            .persistent()
            .remove(&DataKey::Delegation(delegator));
    }

    /// Get voting power (0 if delegated)
    pub fn voting_power(env: Env, voter: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(voter.clone()));

        if delegation.is_some() {
            // Delegated - no direct voting power
            0
        } else {
            env.storage()
                .persistent()
                .get(&DataKey::Balance(voter))
                .unwrap_or(0)
        }
    }

    /// Get delegation info
    pub fn get_delegation(env: Env, delegator: Address) -> Option<Delegation> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Delegation(delegator))
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
