#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup(env: &Env) -> (PrivacyLayerContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, PrivacyLayerContract);
    let c = PrivacyLayerContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PrivacyLayerContract);
    let c = PrivacyLayerContractClient::new(&env, &id);
    let a = Address::generate(&env); c.initialize(&a); c.initialize(&a);
}

#[test]
fn test_set_consent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    c.set_consent(&user, &true, &true, &false, &false, &None);
    let consent = c.get_consent(&user).unwrap();
    assert!(consent.data_processing);
    assert!(consent.targeted_ads);
    assert!(!consent.analytics);
}

#[test]
fn test_set_consent_with_expiry() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    c.set_consent(&user, &true, &true, &true, &true, &Some(86_400u64));
    let consent = c.get_consent(&user).unwrap();
    assert!(consent.expires_at.is_some());
}

#[test]
fn test_revoke_consent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    c.set_consent(&user, &true, &true, &true, &true, &None);
    c.revoke_consent(&user);
    assert!(!c.has_consent(&user, &s(&env, "analytics")));
}

#[test]
fn test_submit_zkp() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    let zkp_hash = BytesN::from_array(&env, &[2u8; 32]);
    let proof_id = c.submit_zkp(&user, &s(&env, "1,2,3"), &zkp_hash);
    let proof = c.get_proof(&proof_id).unwrap();
    assert!(!proof.verified);
}

#[test]
fn test_verify_zkp() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    let zkp_hash = BytesN::from_array(&env, &[2u8; 32]);
    let proof_id = c.submit_zkp(&user, &s(&env, "1,2,3"), &zkp_hash);
    c.verify_zkp(&admin, &proof_id);
    let proof = c.get_proof(&proof_id).unwrap();
    assert!(proof.verified);
}

#[test]
fn test_has_consent_false() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.has_consent(&Address::generate(&env), &s(&env, "analytics")));
}

#[test]
fn test_get_proof_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pid = BytesN::from_array(&env, &[99u8; 32]);
    assert!(c.get_proof(&pid).is_none());
}
