#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (PublisherVerificationContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, PublisherVerificationContract);
    let c = PublisherVerificationContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PublisherVerificationContract);
    let c = PublisherVerificationContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
    assert_eq!(c.get_publisher_count(), 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PublisherVerificationContract);
    let c = PublisherVerificationContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, PublisherVerificationContract);
    let c = PublisherVerificationContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_register_publisher() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    assert_eq!(c.get_publisher_count(), 1);
    let p = c.get_publisher(&pub1).unwrap();
    assert!(matches!(p.status, VerificationStatus::Pending));
    assert_eq!(p.reputation_score, 0);
}

#[test]
#[should_panic(expected = "already registered")]
fn test_register_publisher_duplicate() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.register_publisher(&pub1, &s(&env, "other.com"));
}

#[test]
#[should_panic(expected = "domain already registered")]
fn test_register_publisher_duplicate_domain() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    c.register_publisher(&p1, &s(&env, "example.com"));
    c.register_publisher(&p2, &s(&env, "example.com"));
}

#[test]
fn test_submit_kyc() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.submit_kyc(&pub1, &s(&env, "KycHash"), &s(&env, "KycProvider"));
    let kyc = c.get_kyc(&pub1).unwrap();
    assert!(!kyc.verified);
}

#[test]
#[should_panic(expected = "not registered")]
fn test_submit_kyc_unregistered() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.submit_kyc(&Address::generate(&env), &s(&env, "KycHash"), &s(&env, "KycProvider"));
}

#[test]
fn test_verify_publisher() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.submit_kyc(&pub1, &s(&env, "KycHash"), &s(&env, "KycProvider"));
    c.verify_publisher(&admin, &pub1, &PublisherTier::Gold);
    let p = c.get_publisher(&pub1).unwrap();
    assert!(matches!(p.status, VerificationStatus::Verified));
    assert!(p.verified_at.is_some());
    assert_eq!(p.reputation_score, 100);
    assert!(c.is_verified(&pub1));
    let kyc = c.get_kyc(&pub1).unwrap();
    assert!(kyc.verified);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_verify_publisher_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.verify_publisher(&Address::generate(&env), &pub1, &PublisherTier::Bronze);
}

#[test]
fn test_suspend_publisher() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.verify_publisher(&admin, &pub1, &PublisherTier::Silver);
    c.suspend_publisher(&admin, &pub1);
    assert!(!c.is_verified(&pub1));
}

#[test]
fn test_update_reputation() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.update_reputation(&admin, &pub1, &850u32);
    let p = c.get_publisher(&pub1).unwrap();
    assert_eq!(p.reputation_score, 850);
}

#[test]
#[should_panic(expected = "invalid score")]
fn test_update_reputation_too_high() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.update_reputation(&admin, &pub1, &1001u32);
}

#[test]
fn test_record_impression() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let pub1 = Address::generate(&env);
    let caller = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.verify_publisher(&admin, &pub1, &PublisherTier::Gold);
    c.record_impression(&caller, &pub1, &1000i128);
    let p = c.get_publisher(&pub1).unwrap();
    assert_eq!(p.total_impressions, 1);
    assert_eq!(p.total_earnings, 1000);
}

#[test]
#[should_panic(expected = "publisher not verified")]
fn test_record_impression_unverified() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let caller = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    c.record_impression(&caller, &pub1, &1000i128);
}

#[test]
fn test_get_domain_owner() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.register_publisher(&pub1, &s(&env, "example.com"));
    let owner = c.get_domain_owner(&s(&env, "example.com")).unwrap();
    assert_eq!(owner, pub1);
}

#[test]
fn test_is_verified_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.is_verified(&Address::generate(&env)));
}
