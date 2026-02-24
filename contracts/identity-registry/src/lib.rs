//! PulsarTrack - Identity Registry (Soroban)
//! Decentralized identity and credential management for the PulsarTrack ecosystem on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum IdentityStatus {
    Unverified,
    Pending,
    Verified,
    Suspended,
    Revoked,
}

#[contracttype]
#[derive(Clone)]
pub enum IdentityType {
    Advertiser,
    Publisher,
    DataProvider,
    Operator,
}

#[contracttype]
#[derive(Clone)]
pub struct Identity {
    pub account: Address,
    pub identity_type: IdentityType,
    pub status: IdentityStatus,
    pub display_name: String,
    pub metadata_hash: String,    // IPFS hash for extended metadata
    pub credentials_hash: String, // hash of verified credentials
    pub registered_at: u64,
    pub verified_at: Option<u64>,
    pub last_activity: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    IdentityCount,
    Identity(Address),
    NameOwner(String),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct IdentityRegistryContract;

#[contractimpl]
impl IdentityRegistryContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::IdentityCount, &0u64);
    }

    pub fn register(
        env: Env,
        account: Address,
        identity_type: IdentityType,
        display_name: String,
        metadata_hash: String,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        account.require_auth();

        if env
            .storage()
            .persistent()
            .has(&DataKey::Identity(account.clone()))
        {
            panic!("already registered");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::NameOwner(display_name.clone()))
        {
            panic!("name taken");
        }

        let identity = Identity {
            account: account.clone(),
            identity_type,
            status: IdentityStatus::Pending,
            display_name: display_name.clone(),
            metadata_hash,
            credentials_hash: String::from_str(&env, ""),
            registered_at: env.ledger().timestamp(),
            verified_at: None,
            last_activity: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Identity(account.clone());
        env.storage().persistent().set(&_ttl_key, &identity);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::NameOwner(display_name);
        env.storage().persistent().set(&_ttl_key, &account);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::IdentityCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::IdentityCount, &(count + 1));

        env.events().publish(
            (symbol_short!("identity"), symbol_short!("register")),
            account,
        );
    }

    pub fn verify_identity(env: Env, admin: Address, account: Address, credentials_hash: String) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut identity: Identity = env
            .storage()
            .persistent()
            .get(&DataKey::Identity(account.clone()))
            .expect("identity not found");

        identity.status = IdentityStatus::Verified;
        identity.credentials_hash = credentials_hash;
        identity.verified_at = Some(env.ledger().timestamp());

        let _ttl_key = DataKey::Identity(account.clone());
        env.storage().persistent().set(&_ttl_key, &identity);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("identity"), symbol_short!("verified")),
            account,
        );
    }

    pub fn update_metadata(env: Env, account: Address, metadata_hash: String) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        account.require_auth();

        let mut identity: Identity = env
            .storage()
            .persistent()
            .get(&DataKey::Identity(account.clone()))
            .expect("identity not found");

        identity.metadata_hash = metadata_hash;
        identity.last_activity = env.ledger().timestamp();

        let _ttl_key = DataKey::Identity(account);
        env.storage().persistent().set(&_ttl_key, &identity);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn suspend_identity(env: Env, admin: Address, account: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut identity: Identity = env
            .storage()
            .persistent()
            .get(&DataKey::Identity(account.clone()))
            .expect("identity not found");

        identity.status = IdentityStatus::Suspended;
        let _ttl_key = DataKey::Identity(account);
        env.storage().persistent().set(&_ttl_key, &identity);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_identity(env: Env, account: Address) -> Option<Identity> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Identity(account))
    }

    pub fn is_verified(env: Env, account: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(identity) = env
            .storage()
            .persistent()
            .get::<DataKey, Identity>(&DataKey::Identity(account))
        {
            matches!(identity.status, IdentityStatus::Verified)
        } else {
            false
        }
    }

    pub fn get_by_name(env: Env, display_name: String) -> Option<Address> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::NameOwner(display_name))
    }

    pub fn get_identity_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::IdentityCount)
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
