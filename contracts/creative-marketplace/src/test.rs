#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(
    env: &Env,
) -> (
    CreativeMarketplaceContractClient<'_>,
    Address,
    Address,
    Address,
) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let id = env.register_contract(None, CreativeMarketplaceContract);
    let c = CreativeMarketplaceContractClient::new(env, &id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
}
fn s(env: &Env, v: &str) -> String {
    String::from_str(env, v)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    setup(&env);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    c.initialize(&admin, &token);
}

#[test]
fn test_create_listing() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let creator = Address::generate(&env);
    let listing_id = c.create_listing(
        &creator,
        &s(&env, "QmHash"),
        &s(&env, "Banner Ad"),
        &s(&env, "A beautiful banner"),
        &10_000i128,
        &LicenseType::OneTime,
    );
    assert_eq!(listing_id, 1);
    let listing = c.get_listing(&listing_id).unwrap();
    assert_eq!(listing.price, 10_000);
    assert!(matches!(listing.status, ListingStatus::Active));
}

#[test]
fn test_purchase_license() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    mint(&env, &token, &buyer, 1_000_000);
    let listing_id = c.create_listing(
        &creator,
        &s(&env, "QmHash"),
        &s(&env, "Banner"),
        &s(&env, "Desc"),
        &10_000i128,
        &LicenseType::OneTime,
    );
    c.purchase_license(&buyer, &listing_id, &Some(86_400u64));
    assert!(c.has_license(&listing_id, &buyer));
    let license = c.get_license(&listing_id, &buyer).unwrap();
    assert_eq!(license.listing_id, listing_id);
}

#[test]
fn test_remove_listing() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let creator = Address::generate(&env);
    let listing_id = c.create_listing(
        &creator,
        &s(&env, "QmHash"),
        &s(&env, "Banner"),
        &s(&env, "Desc"),
        &10_000i128,
        &LicenseType::OneTime,
    );
    c.remove_listing(&creator, &listing_id);
    let listing = c.get_listing(&listing_id).unwrap();
    assert!(matches!(listing.status, ListingStatus::Removed));
}

#[test]
fn test_get_listing_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_listing(&999u64).is_none());
}

#[test]
fn test_has_license_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(!c.has_license(&1u64, &Address::generate(&env)));
}

#[test]
#[should_panic(expected = "content already listed - check for exclusive licenses")]
fn test_duplicate_exclusive_content_blocked() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let creator = Address::generate(&env);
    let content_hash = s(&env, "QmExclusiveContent123");

    // Create first exclusive listing
    c.create_listing(
        &creator,
        &content_hash,
        &s(&env, "Exclusive Banner"),
        &s(&env, "First listing"),
        &50_000i128,
        &LicenseType::Exclusive,
    );

    // Attempt to create second listing with same content hash - should panic
    c.create_listing(
        &creator,
        &content_hash,
        &s(&env, "Duplicate Banner"),
        &s(&env, "Second listing"),
        &30_000i128,
        &LicenseType::Exclusive,
    );
}

#[test]
fn test_non_exclusive_allows_duplicate_content() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let creator = Address::generate(&env);
    let content_hash = s(&env, "QmNonExclusiveContent456");

    // Create first non-exclusive listing
    let listing_id_1 = c.create_listing(
        &creator,
        &content_hash,
        &s(&env, "Banner 1"),
        &s(&env, "First listing"),
        &10_000i128,
        &LicenseType::OneTime,
    );

    // Create second listing with same content hash - should succeed for non-exclusive
    let listing_id_2 = c.create_listing(
        &creator,
        &content_hash,
        &s(&env, "Banner 2"),
        &s(&env, "Second listing"),
        &15_000i128,
        &LicenseType::Recurring,
    );

    assert_eq!(listing_id_1, 1);
    assert_eq!(listing_id_2, 2);
}

#[test]
fn test_remove_exclusive_listing_allows_recreation() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let creator = Address::generate(&env);
    let content_hash = s(&env, "QmExclusiveRemovable789");

    // Create exclusive listing
    let listing_id_1 = c.create_listing(
        &creator,
        &content_hash,
        &s(&env, "Exclusive Banner"),
        &s(&env, "First listing"),
        &50_000i128,
        &LicenseType::Exclusive,
    );

    // Remove the listing
    c.remove_listing(&creator, &listing_id_1);

    // Now should be able to create new listing with same content hash
    let listing_id_2 = c.create_listing(
        &creator,
        &content_hash,
        &s(&env, "New Exclusive Banner"),
        &s(&env, "Second listing after removal"),
        &60_000i128,
        &LicenseType::Exclusive,
    );

    assert_eq!(listing_id_2, 2);
    let listing = c.get_listing(&listing_id_2).unwrap();
    assert_eq!(listing.price, 60_000);
}

#[test]
fn test_exclusive_license_marks_sold() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    mint(&env, &token, &buyer, 1_000_000);

    let listing_id = c.create_listing(
        &creator,
        &s(&env, "QmExclusiveSold"),
        &s(&env, "Exclusive Banner"),
        &s(&env, "Exclusive content"),
        &100_000i128,
        &LicenseType::Exclusive,
    );

    // Purchase exclusive license
    c.purchase_license(&buyer, &listing_id, &None);

    // Verify listing is marked as Sold
    let listing = c.get_listing(&listing_id).unwrap();
    assert!(matches!(listing.status, ListingStatus::Sold));
}
