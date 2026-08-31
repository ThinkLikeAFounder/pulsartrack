//! PulsarTrack - Governance Token (Soroban / SEP-41 compatible)
//! PULSAR governance token with voting power and delegation on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

// ============================================================
// Data Types
// ============================================================

#[contracttype]
#[derive(Clone)]
pub struct Allowance {
    pub amount: i128,
    pub expiry: u32, // ledger sequence number after which the allowance is invalid
}

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
    DelegatedPower(Address),
    VotingSnapshot(Address, u32), // Address, ledger_sequence
    /// Number of checkpoints written for an account.
    CheckpointCount(Address),
    /// The nth checkpoint for an account: (ledger_sequence, voting_power).
    Checkpoint(Address, u32), // Address, checkpoint index
}

/// A point-in-time record of an account's voting power.
///
/// `ledger` is the ledger sequence at which `power` became effective. Reads for
/// a target ledger resolve to the newest checkpoint whose `ledger` is strictly
/// less than the target, so power created in a given ledger is never usable for
/// that same ledger. That is what closes the flash-loan window.
#[contracttype]
#[derive(Clone)]
pub struct Checkpoint {
    pub ledger: u32,
    pub power: i128,
}

// 1_000_000_000_000 base units. With the 7-decimal metadata below this is
// 100,000 PULSAR; adjust `decimals` to 6 if a 1,000,000-token cap is intended.
pub const MAX_SUPPLY: i128 = 1_000_000_000_000;

// ============================================================
// Contract
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

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

        if from == to {
            panic!("sender and recipient cannot be the same address");
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

        let from_delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(from.clone()));

        let mut from_delegate_cp: Option<Address> = None;
        if let Some(delegation) = from_delegation {
            let delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(delegation.delegate.clone()))
                .unwrap_or(0);
            from_delegate_cp = Some(delegation.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(delegation.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &delegate_power.saturating_sub(amount));
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

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

        let to_delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(to.clone()));

        let mut to_delegate_cp: Option<Address> = None;
        if let Some(delegation) = to_delegation {
            let delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(delegation.delegate.clone()))
                .unwrap_or(0);
            to_delegate_cp = Some(delegation.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(delegation.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &(delegate_power + amount));
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

        Self::write_checkpoint(&env, &from);
        Self::write_checkpoint(&env, &to);
        if let Some(d) = from_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }
        if let Some(d) = to_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }

        env.events()
            .publish((symbol_short!("transfer"),), (from, to, amount));
    }

    /// Transfer from (requires prior approval)
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        spender.require_auth();

        if from == to {
            panic!("from and to cannot be the same address");
        }

        let allowance: Allowance = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(Allowance {
                amount: 0,
                expiry: 0,
            });

        if env.ledger().sequence() > allowance.expiry {
            panic!("allowance expired");
        }

        if allowance.amount < amount {
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
        env.storage().persistent().set(
            &_ttl_key,
            &Allowance {
                amount: allowance.amount - amount,
                expiry: allowance.expiry,
            },
        );
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

        let from_delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(from.clone()));

        let mut from_delegate_cp: Option<Address> = None;
        if let Some(delegation) = from_delegation {
            let delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(delegation.delegate.clone()))
                .unwrap_or(0);
            from_delegate_cp = Some(delegation.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(delegation.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &delegate_power.saturating_sub(amount));
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

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

        let to_delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(to.clone()));

        let mut to_delegate_cp: Option<Address> = None;
        if let Some(delegation) = to_delegation {
            let delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(delegation.delegate.clone()))
                .unwrap_or(0);
            to_delegate_cp = Some(delegation.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(delegation.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &(delegate_power + amount));
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

        Self::write_checkpoint(&env, &from);
        Self::write_checkpoint(&env, &to);
        if let Some(d) = from_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }
        if let Some(d) = to_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }

        env.events()
            .publish((symbol_short!("transfer"),), (from, to, amount));
    }

    /// Approve token spending
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128, expiry: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        owner.require_auth();

        if amount < 0 {
            panic!("allowance amount must be non-negative");
        }

        if expiry <= env.ledger().sequence() {
            panic!("expiry must be a future ledger sequence");
        }

        let _ttl_key = DataKey::Allowance(owner.clone(), spender.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &Allowance { amount, expiry });
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("approve"),),
            (owner, spender, amount, expiry),
        );
    }

    /// Get allowance
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get::<DataKey, Allowance>(&DataKey::Allowance(owner, spender))
            .map(|a| a.amount)
            .unwrap_or(0)
    }

    /// Mint new tokens (admin only)
    pub fn mint(env: Env, admin: Address, recipient: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        if amount <= 0 {
            panic!("invalid amount");
        }
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let current_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        let new_supply = current_supply.checked_add(amount).expect("supply overflow");
        if new_supply > MAX_SUPPLY {
            panic!("exceeds max supply");
        }

        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(recipient.clone()))
            .unwrap_or(0);
        let _ttl_key = DataKey::Balance(recipient.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &(balance + amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let recipient_delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(recipient.clone()));

        let mut to_delegate_cp: Option<Address> = None;
        if let Some(delegation) = recipient_delegation {
            let delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(delegation.delegate.clone()))
                .unwrap_or(0);
            to_delegate_cp = Some(delegation.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(delegation.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &(delegate_power + amount));
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(current_supply + amount));

        Self::write_checkpoint(&env, &recipient);
        if let Some(d) = to_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }

        env.events()
            .publish((symbol_short!("mint"),), (recipient, amount));
    }

    /// Burn tokens
    pub fn burn(env: Env, from: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        from.require_auth();

        if amount <= 0 {
            panic!("burn amount must be positive");
        }

        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if balance < amount {
            panic!("insufficient balance");
        }

        let _ttl_key = DataKey::Balance(from.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &(balance - amount));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let from_delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(from.clone()));

        let mut from_delegate_cp: Option<Address> = None;
        if let Some(delegation) = from_delegation {
            let delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(delegation.delegate.clone()))
                .unwrap_or(0);
            from_delegate_cp = Some(delegation.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(delegation.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &delegate_power.saturating_sub(amount));
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

        let supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply - amount));

        Self::write_checkpoint(&env, &from);
        if let Some(d) = from_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }

        env.events()
            .publish((symbol_short!("burn"),), (from, amount));
    }

    /// Delegate voting power
    pub fn delegate(env: Env, delegator: Address, delegate_to: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        delegator.require_auth();

        if delegator == delegate_to {
            panic!("cannot delegate voting power to self");
        }

        let delegator_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(delegator.clone()))
            .unwrap_or(0);

        let existing_delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(delegator.clone()));

        let mut old_delegate_cp: Option<Address> = None;
        if let Some(old_delegation) = existing_delegation {
            let old_delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(old_delegation.delegate.clone()))
                .unwrap_or(0);
            let new_old_power = old_delegate_power.saturating_sub(delegator_balance);
            old_delegate_cp = Some(old_delegation.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(old_delegation.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &new_old_power);
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

        let new_delegate_power: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::DelegatedPower(delegate_to.clone()))
            .unwrap_or(0);
        let _ttl_key = DataKey::DelegatedPower(delegate_to.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &(new_delegate_power + delegator_balance));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

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

        Self::write_checkpoint(&env, &delegator);
        Self::write_checkpoint(&env, &delegate_to);
        if let Some(d) = old_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }

        env.events()
            .publish((symbol_short!("delegate"),), (delegator, delegate_to));
    }

    /// Revoke delegation
    pub fn revoke_delegation(env: Env, delegator: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        delegator.require_auth();

        let delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(delegator.clone()));

        let mut from_delegate_cp: Option<Address> = None;
        let mut revoked_delegate: Option<Address> = None;
        if let Some(delegation_info) = delegation {
            let delegator_balance: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Balance(delegator.clone()))
                .unwrap_or(0);

            let delegate_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(delegation_info.delegate.clone()))
                .unwrap_or(0);
            let new_power = delegate_power.saturating_sub(delegator_balance);
            from_delegate_cp = Some(delegation_info.delegate.clone());
            revoked_delegate = Some(delegation_info.delegate.clone());
            let _ttl_key = DataKey::DelegatedPower(delegation_info.delegate);
            env.storage()
                .persistent()
                .set(&_ttl_key, &new_power);
            env.storage().persistent().extend_ttl(
                &_ttl_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

        env.storage()
            .persistent()
            .remove(&DataKey::Delegation(delegator.clone()));

        Self::write_checkpoint(&env, &delegator);
        if let Some(d) = from_delegate_cp {
            Self::write_checkpoint(&env, &d);
        }

        if let Some(delegate) = revoked_delegate {
            env.events()
                .publish((symbol_short!("revoke"),), (delegator, delegate));
        }
    }

    /// Get voting power (0 if delegated, otherwise own balance plus received delegations)
    pub fn voting_power(env: Env, voter: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let delegation = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(voter.clone()));

        if delegation.is_some() {
            0
        } else {
            let own_balance: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Balance(voter.clone()))
                .unwrap_or(0);
            let delegated_power: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::DelegatedPower(voter))
                .unwrap_or(0);
            own_balance + delegated_power
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

    /// Record `voter`'s current voting power as a checkpoint at the current
    /// ledger. Called after every mutation that can change voting power
    /// (transfer, mint, burn, delegate, revoke).
    ///
    /// Writing a checkpoint for the current ledger does NOT make that power
    /// usable in the current ledger: `get_past_votes` only considers
    /// checkpoints strictly older than the ledger it is asked about.
    fn write_checkpoint(env: &Env, account: &Address) {
        let power = Self::voting_power(env.clone(), account.clone());
        let current_ledger = env.ledger().sequence();

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CheckpointCount(account.clone()))
            .unwrap_or(0);

        // Collapse repeated writes within the same ledger into one entry so a
        // single ledger cannot produce a contradictory history.
        if count > 0 {
            let last_key = DataKey::Checkpoint(account.clone(), count - 1);
            let last: Checkpoint = env.storage().persistent().get(&last_key).unwrap();
            if last.ledger == current_ledger {
                let updated = Checkpoint {
                    ledger: current_ledger,
                    power,
                };
                env.storage().persistent().set(&last_key, &updated);
                env.storage().persistent().extend_ttl(
                    &last_key,
                    PERSISTENT_LIFETIME_THRESHOLD,
                    PERSISTENT_BUMP_AMOUNT,
                );
                return;
            }
        }

        let key = DataKey::Checkpoint(account.clone(), count);
        env.storage().persistent().set(
            &key,
            &Checkpoint {
                ledger: current_ledger,
                power,
            },
        );
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let count_key = DataKey::CheckpointCount(account.clone());
        env.storage().persistent().set(&count_key, &(count + 1));
        env.storage().persistent().extend_ttl(
            &count_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Voting power `account` held as of a ledger strictly before
    /// `ledger_sequence`.
    ///
    /// This is the flash-loan-safe read that governance must use. Because the
    /// lookup requires `checkpoint.ledger < ledger_sequence`, tokens acquired
    /// in `ledger_sequence` itself — including inside the very transaction that
    /// borrows them — contribute nothing.
    pub fn get_past_votes(env: Env, account: Address, ledger_sequence: u32) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if ledger_sequence > env.ledger().sequence() {
            panic!("cannot read votes for a future ledger");
        }

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CheckpointCount(account.clone()))
            .unwrap_or(0);
        if count == 0 {
            return 0;
        }

        // Binary search for the newest checkpoint strictly older than
        // `ledger_sequence`.
        let mut low: u32 = 0;
        let mut high: u32 = count; // exclusive
        while low < high {
            let mid = low + (high - low) / 2;
            let cp: Checkpoint = env
                .storage()
                .persistent()
                .get(&DataKey::Checkpoint(account.clone(), mid))
                .unwrap();
            if cp.ledger < ledger_sequence {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if low == 0 {
            return 0;
        }

        let cp: Checkpoint = env
            .storage()
            .persistent()
            .get(&DataKey::Checkpoint(account, low - 1))
            .unwrap();
        cp.power
    }

    /// Number of checkpoints recorded for an account.
    pub fn get_checkpoint_count(env: Env, account: Address) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::CheckpointCount(account))
            .unwrap_or(0)
    }

    /// Take a voting snapshot for a voter at a given ledger sequence.
    /// Stores the voter's own balance plus any delegated power they hold
    /// at that point in time. Governance-dao should use this snapshot
    /// balance when a proposal is being voted on to prevent flash-loan attacks.
    pub fn take_snapshot(env: Env, voter: Address, ledger_sequence: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        voter.require_auth();

        // Only allow snapshotting at or before the current ledger
        if ledger_sequence > env.ledger().sequence() {
            panic!("cannot snapshot a future ledger");
        }

        // Resolve the power from the checkpoint history rather than from
        // current storage. Reading live balances here was the bug: it recorded
        // *present* power under a *past* ledger key, so a borrower could
        // snapshot their inflated balance and have it counted as historical.
        let snapshot_power = Self::get_past_votes(env.clone(), voter.clone(), ledger_sequence);

        let key = DataKey::VotingSnapshot(voter.clone(), ledger_sequence);
        env.storage().persistent().set(&key, &snapshot_power);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("snapshot"),),
            (voter, ledger_sequence, snapshot_power),
        );
    }

    /// Retrieve a previously taken voting snapshot.
    /// Returns None if no snapshot exists for this voter at the given ledger.
    pub fn get_voting_snapshot(
        env: Env,
        voter: Address,
        ledger_sequence: u32,
    ) -> Option<i128> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::VotingSnapshot(voter, ledger_sequence))
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

    pub fn cancel_admin_proposal(env: Env, current_admin: Address) {
        pulsar_common_admin::cancel_admin_proposal(&env, &DataKey::Admin, &DataKey::PendingAdmin, current_admin);
    }
}

mod test;
