//! PulsarTrack - KYC Registry (Soroban)
//! Know Your Customer verification registry for compliance on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum KycLevel {
    None,
    Basic,    // Email + Phone verified
    Standard, // ID document verified
    Enhanced, // Full KYC with AML checks
}

#[contracttype]
#[derive(Clone)]
pub struct KycRecord {
    pub account: Address,
    pub level: KycLevel,
    pub provider: String,
    pub document_hash: String, // Hash of KYC documents
    pub jurisdiction: String,
    pub verified: bool,
    pub submitted_at: u64,
    pub verified_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub verifier: Option<Address>,
}

#[contracttype]
#[derive(Clone)]
pub struct KycProvider {
    pub provider_address: Address,
    pub name: String,
    pub is_active: bool,
    pub total_verifications: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    KycRecord(Address),
    Provider(Address),
    RequiredLevel(String), // operation -> required KYC level
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct KycRegistryContract;

#[contractimpl]
impl KycRegistryContract {
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

    pub fn register_provider(env: Env, admin: Address, provider: Address, name: String) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let kyc_provider = KycProvider {
            provider_address: provider.clone(),
            name,
            is_active: true,
            total_verifications: 0,
        };

        let _ttl_key = DataKey::Provider(provider);
        env.storage().persistent().set(&_ttl_key, &kyc_provider);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn submit_kyc(
        env: Env,
        account: Address,
        provider: Address,
        level: KycLevel,
        document_hash: String,
        jurisdiction: String,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        account.require_auth();

        let kyc_provider: KycProvider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider.clone()))
            .expect("provider not registered");

        if !kyc_provider.is_active {
            panic!("provider not active");
        }

        let record = KycRecord {
            account: account.clone(),
            level: level.clone(),
            provider: kyc_provider.name,
            document_hash,
            jurisdiction,
            verified: false,
            submitted_at: env.ledger().timestamp(),
            verified_at: None,
            expires_at: None,
            verifier: None,
        };

        let _ttl_key = DataKey::KycRecord(account.clone());
        env.storage().persistent().set(&_ttl_key, &record);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("kyc"), symbol_short!("submitted")),
            (account, level),
        );
    }

    pub fn verify_kyc(env: Env, provider: Address, account: Address, expires_in_secs: Option<u64>) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        provider.require_auth();

        let mut provider_data: KycProvider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider.clone()))
            .expect("provider not found");

        if !provider_data.is_active {
            panic!("provider not active");
        }

        let mut record: KycRecord = env
            .storage()
            .persistent()
            .get(&DataKey::KycRecord(account.clone()))
            .expect("kyc not submitted");

        let now = env.ledger().timestamp();
        record.verified = true;
        record.verified_at = Some(now);
        record.expires_at = expires_in_secs.map(|d| now + d);
        record.verifier = Some(provider.clone());

        let _ttl_key = DataKey::KycRecord(account.clone());
        env.storage().persistent().set(&_ttl_key, &record);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        provider_data.total_verifications += 1;
        let _ttl_key = DataKey::Provider(provider);
        env.storage().persistent().set(&_ttl_key, &provider_data);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events()
            .publish((symbol_short!("kyc"), symbol_short!("verified")), account);
    }

    pub fn revoke_kyc(env: Env, admin: Address, account: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut record: KycRecord = env
            .storage()
            .persistent()
            .get(&DataKey::KycRecord(account.clone()))
            .expect("kyc not found");

        record.verified = false;
        let _ttl_key = DataKey::KycRecord(account);
        env.storage().persistent().set(&_ttl_key, &record);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn is_kyc_valid(env: Env, account: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(record) = env
            .storage()
            .persistent()
            .get::<DataKey, KycRecord>(&DataKey::KycRecord(account))
        {
            if !record.verified {
                return false;
            }
            if let Some(expires) = record.expires_at {
                expires > env.ledger().timestamp()
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn get_kyc_record(env: Env, account: Address) -> Option<KycRecord> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::KycRecord(account))
    }

    pub fn get_kyc_level(env: Env, account: Address) -> KycLevel {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(record) = env
            .storage()
            .persistent()
            .get::<DataKey, KycRecord>(&DataKey::KycRecord(account))
        {
            if record.verified {
                record.level
            } else {
                KycLevel::None
            }
        } else {
            KycLevel::None
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
