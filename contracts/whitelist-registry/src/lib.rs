//! PulsarTrack - Whitelist Registry (Soroban)
//! Address whitelist management with grace-period removal on Stellar.
//!
//! Removing an address is a two-phase operation:
//!   1. `remove_from_whitelist` marks the entry with `removal_scheduled_at`.
//!   2. `is_whitelisted` returns false only after GRACE_PERIOD_SECS have elapsed,
//!      allowing in-flight operations against the address to complete safely.
//!
//! Events:
//! - ("whitelist", "added"):   [list_type: ListType, address: Address]
//! - ("whitelist", "removal"): [list_type: ListType, address: Address, effective_at: u64]
//! - ("whitelist", "purged"):  [list_type: ListType, address: Address]

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

/// 24-hour grace window between scheduling removal and the entry becoming invalid.
const GRACE_PERIOD_SECS: u64 = 86_400;

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ListType {
    Publisher,
    Advertiser,
    Oracle,
}

#[contracttype]
#[derive(Clone)]
pub struct WhitelistEntry {
    pub address: Address,
    pub list_type: ListType,
    pub added_at: u64,
    /// When set, the address will be treated as de-whitelisted after this timestamp.
    pub removal_scheduled_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    Whitelist(ListType, Address),
}

#[contract]
pub struct WhitelistRegistryContract;

#[contractimpl]
impl WhitelistRegistryContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn add_to_whitelist(env: Env, admin: Address, address: Address, list_type: ListType) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let entry = WhitelistEntry {
            address: address.clone(),
            list_type: list_type.clone(),
            added_at: env.ledger().timestamp(),
            removal_scheduled_at: None,
        };

        let _ttl_key = DataKey::Whitelist(list_type.clone(), address.clone());
        env.storage().persistent().set(&_ttl_key, &entry);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("whitelist"), symbol_short!("added")),
            (list_type, address),
        );
    }

    /// Schedule removal with a grace period so that in-flight operations relying
    /// on this address's whitelisted status are not abruptly broken mid-execution.
    pub fn remove_from_whitelist(env: Env, admin: Address, address: Address, list_type: ListType) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let key = DataKey::Whitelist(list_type.clone(), address.clone());
        let mut entry: WhitelistEntry = env
            .storage()
            .persistent()
            .get(&key)
            .expect("address not whitelisted");

        // Idempotent: if removal is already scheduled, leave the existing timestamp.
        if entry.removal_scheduled_at.is_none() {
            entry.removal_scheduled_at =
                Some(env.ledger().timestamp().saturating_add(GRACE_PERIOD_SECS));
        }

        let effective_at = entry.removal_scheduled_at.unwrap();
        env.storage().persistent().set(&key, &entry);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("whitelist"), symbol_short!("removal")),
            (list_type, address, effective_at),
        );
    }

    /// Permanently delete an entry whose grace period has already expired.
    pub fn purge_entry(env: Env, address: Address, list_type: ListType) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let key = DataKey::Whitelist(list_type.clone(), address.clone());
        let entry: WhitelistEntry = env
            .storage()
            .persistent()
            .get(&key)
            .expect("address not whitelisted");

        let effective_at = entry
            .removal_scheduled_at
            .expect("removal not scheduled for this address");

        if env.ledger().timestamp() < effective_at {
            panic!("grace period has not elapsed yet");
        }

        env.storage().persistent().remove(&key);

        env.events().publish(
            (symbol_short!("whitelist"), symbol_short!("purged")),
            (list_type, address),
        );
    }

    /// Returns true only when the address is whitelisted AND its grace period
    /// has not yet expired (or removal has not been scheduled at all).
    pub fn is_whitelisted(env: Env, address: Address, list_type: ListType) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let key = DataKey::Whitelist(list_type, address);
        if let Some(entry) = env
            .storage()
            .persistent()
            .get::<DataKey, WhitelistEntry>(&key)
        {
            env.storage().persistent().extend_ttl(
                &key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
            match entry.removal_scheduled_at {
                None => true,
                Some(effective_at) => env.ledger().timestamp() < effective_at,
            }
        } else {
            false
        }
    }

    pub fn get_entry(
        env: Env,
        address: Address,
        list_type: ListType,
    ) -> Option<WhitelistEntry> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let key = DataKey::Whitelist(list_type, address);
        let entry = env.storage().persistent().get(&key);
        if entry.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }
        entry
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
