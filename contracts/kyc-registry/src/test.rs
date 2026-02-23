#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

fn setup(env: &Env) -> (KycRegistryContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, KycRegistryContract);
    let c = KycRegistryContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, KycRegistryContract);
    let c = KycRegistryContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, KycRegistryContract);
    let c = KycRegistryContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, KycRegistryContract);
    let c = KycRegistryContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_register_provider() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let provider = Address::generate(&env);
    c.register_provider(&admin, &provider, &s(&env, "VerifyInc"));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_register_provider_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.register_provider(&Address::generate(&env), &Address::generate(&env), &s(&env, "X"));
}

#[test]
fn test_submit_kyc() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let provider = Address::generate(&env);
    let account = Address::generate(&env);
    c.register_provider(&admin, &provider, &s(&env, "VerifyInc"));
    c.submit_kyc(&account, &provider, &KycLevel::Standard, &s(&env, "DocHash"), &s(&env, "US"));
    let record = c.get_kyc_record(&account).unwrap();
    assert!(!record.verified);
    assert_eq!(record.level, KycLevel::Standard);
}

#[test]
#[should_panic(expected = "provider not registered")]
fn test_submit_kyc_invalid_provider() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let account = Address::generate(&env);
    c.submit_kyc(&account, &Address::generate(&env), &KycLevel::Basic, &s(&env, "DocHash"), &s(&env, "US"));
}

#[test]
fn test_verify_kyc() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let provider = Address::generate(&env);
    let account = Address::generate(&env);
    c.register_provider(&admin, &provider, &s(&env, "VerifyInc"));
    c.submit_kyc(&account, &provider, &KycLevel::Enhanced, &s(&env, "DocHash"), &s(&env, "US"));
    c.verify_kyc(&provider, &account, &Some(86_400u64));
    let record = c.get_kyc_record(&account).unwrap();
    assert!(record.verified);
    assert!(record.verified_at.is_some());
    assert!(record.expires_at.is_some());
    assert!(c.is_kyc_valid(&account));
    assert_eq!(c.get_kyc_level(&account), KycLevel::Enhanced);
}

#[test]
fn test_verify_kyc_no_expiry() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let provider = Address::generate(&env);
    let account = Address::generate(&env);
    c.register_provider(&admin, &provider, &s(&env, "VerifyInc"));
    c.submit_kyc(&account, &provider, &KycLevel::Basic, &s(&env, "DocHash"), &s(&env, "US"));
    c.verify_kyc(&provider, &account, &None);
    let record = c.get_kyc_record(&account).unwrap();
    assert!(record.expires_at.is_none());
    assert!(c.is_kyc_valid(&account));
}

#[test]
fn test_kyc_expired() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let provider = Address::generate(&env);
    let account = Address::generate(&env);
    c.register_provider(&admin, &provider, &s(&env, "VerifyInc"));
    c.submit_kyc(&account, &provider, &KycLevel::Standard, &s(&env, "DocHash"), &s(&env, "US"));
    c.verify_kyc(&provider, &account, &Some(100u64));
    env.ledger().with_mut(|li| { li.timestamp = 200; });
    assert!(!c.is_kyc_valid(&account));
}

#[test]
fn test_revoke_kyc() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let provider = Address::generate(&env);
    let account = Address::generate(&env);
    c.register_provider(&admin, &provider, &s(&env, "VerifyInc"));
    c.submit_kyc(&account, &provider, &KycLevel::Standard, &s(&env, "DocHash"), &s(&env, "US"));
    c.verify_kyc(&provider, &account, &None);
    assert!(c.is_kyc_valid(&account));
    c.revoke_kyc(&admin, &account);
    assert!(!c.is_kyc_valid(&account));
    assert_eq!(c.get_kyc_level(&account), KycLevel::None);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_revoke_kyc_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let provider = Address::generate(&env);
    let account = Address::generate(&env);
    c.register_provider(&admin, &provider, &s(&env, "VerifyInc"));
    c.submit_kyc(&account, &provider, &KycLevel::Standard, &s(&env, "DocHash"), &s(&env, "US"));
    c.revoke_kyc(&Address::generate(&env), &account);
}

#[test]
fn test_get_kyc_level_none() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.get_kyc_level(&Address::generate(&env)), KycLevel::None);
}

#[test]
fn test_is_kyc_valid_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.is_kyc_valid(&Address::generate(&env)));
}
