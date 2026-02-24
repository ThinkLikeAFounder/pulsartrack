//! PulsarTrack - Creative Marketplace (Soroban)
//! A marketplace for buying, selling and licensing ad creatives on Stellar.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String,
};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ListingStatus {
    Active,
    Sold,
    Unlicensed,
    Removed,
}

#[contracttype]
#[derive(Clone)]
pub enum LicenseType {
    OneTime,
    Recurring,
    Exclusive,
    OpenSource,
}

#[contracttype]
#[derive(Clone)]
pub struct CreativeListing {
    pub listing_id: u64,
    pub creator: Address,
    pub content_hash: String, // IPFS hash
    pub title: String,
    pub description: String,
    pub price: i128,
    pub license_type: LicenseType,
    pub status: ListingStatus,
    pub sale_count: u64,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct License {
    pub listing_id: u64,
    pub licensee: Address,
    pub license_type: LicenseType,
    pub paid_amount: i128,
    pub purchased_at: u64,
    pub expires_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TokenAddress,
    ListingCounter,
    PlatformFeeBps,
    Listing(u64),
    License(u64, Address), // listing_id, licensee
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct CreativeMarketplaceContract;

#[contractimpl]
impl CreativeMarketplaceContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
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
            .set(&DataKey::ListingCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &250u32); // 2.5%
    }

    pub fn create_listing(
        env: Env,
        creator: Address,
        content_hash: String,
        title: String,
        description: String,
        price: i128,
        license_type: LicenseType,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        creator.require_auth();

        if price <= 0 {
            panic!("invalid price");
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ListingCounter)
            .unwrap_or(0);
        let listing_id = counter + 1;

        let listing = CreativeListing {
            listing_id,
            creator: creator.clone(),
            content_hash,
            title,
            description,
            price,
            license_type,
            status: ListingStatus::Active,
            sale_count: 0,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Listing(listing_id);
        env.storage().persistent().set(&_ttl_key, &listing);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::ListingCounter, &listing_id);

        env.events().publish(
            (symbol_short!("listing"), symbol_short!("created")),
            (listing_id, creator),
        );

        listing_id
    }

    pub fn purchase_license(
        env: Env,
        buyer: Address,
        listing_id: u64,
        license_duration_secs: Option<u64>,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        buyer.require_auth();

        let mut listing: CreativeListing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .expect("listing not found");

        if listing.status != ListingStatus::Active {
            panic!("listing not active");
        }

        // Check not already licensed
        if env
            .storage()
            .persistent()
            .has(&DataKey::License(listing_id, buyer.clone()))
        {
            panic!("already licensed");
        }

        // Calculate fee
        let fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .unwrap_or(250);
        let fee = (listing.price * fee_bps as i128) / 10_000;
        let creator_amount = listing.price - fee;

        // Process payment
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&buyer, &listing.creator, &creator_amount);

        // Fee to admin
        if fee > 0 {
            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            token_client.transfer(&buyer, &admin, &fee);
        }

        let now = env.ledger().timestamp();
        let expires_at = license_duration_secs.map(|d| now + d);

        let license = License {
            listing_id,
            licensee: buyer.clone(),
            license_type: listing.license_type.clone(),
            paid_amount: listing.price,
            purchased_at: now,
            expires_at,
        };

        let _ttl_key = DataKey::License(listing_id, buyer);
        env.storage().persistent().set(&_ttl_key, &license);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        listing.sale_count += 1;
        listing.last_updated = now;

        // Exclusive licenses close the listing
        if matches!(listing.license_type, LicenseType::Exclusive) {
            listing.status = ListingStatus::Sold;
        }

        let _ttl_key = DataKey::Listing(listing_id);
        env.storage().persistent().set(&_ttl_key, &listing);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("license"), symbol_short!("purchased")),
            (listing_id, listing.price),
        );
    }

    pub fn remove_listing(env: Env, creator: Address, listing_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        creator.require_auth();

        let mut listing: CreativeListing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .expect("listing not found");

        if listing.creator != creator {
            panic!("unauthorized");
        }

        listing.status = ListingStatus::Removed;
        listing.last_updated = env.ledger().timestamp();
        let _ttl_key = DataKey::Listing(listing_id);
        env.storage().persistent().set(&_ttl_key, &listing);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_listing(env: Env, listing_id: u64) -> Option<CreativeListing> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
    }

    pub fn get_license(env: Env, listing_id: u64, licensee: Address) -> Option<License> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::License(listing_id, licensee))
    }

    pub fn has_license(env: Env, listing_id: u64, licensee: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(license) = env
            .storage()
            .persistent()
            .get::<DataKey, License>(&DataKey::License(listing_id, licensee))
        {
            if let Some(expires) = license.expires_at {
                expires > env.ledger().timestamp()
            } else {
                true
            }
        } else {
            false
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
