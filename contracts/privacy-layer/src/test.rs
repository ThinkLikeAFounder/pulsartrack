#![cfg(test)]
use super::*;
use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String,
};

const MIN_ADMIN_DELAY_LEDGERS: u32 = 17_280;

fn setup(env: &Env) -> (PrivacyLayerContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(PrivacyLayerContract, ());
    let c = PrivacyLayerContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

fn sign(env: &Env, key: &SigningKey, proof_id: &BytesN<32>) -> BytesN<64> {
    BytesN::from_array(env, &key.sign(&proof_id.to_array()).to_bytes())
}

fn setup_verifier_key(
    env: &Env,
    c: &PrivacyLayerContractClient<'_>,
    admin: &Address,
) -> SigningKey {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    c.set_verifier_key(
        admin,
        &BytesN::from_array(env, &key.verifying_key().to_bytes()),
    );
    key
}

#[test]
fn test_admin_rotation_after_timelock() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    c.propose_admin(&admin, &new_admin);
    let pending = env.as_contract(&c.address, || {
        env.storage()
            .instance()
            .get::<DataKey, (Address, u32)>(&DataKey::PendingAdmin)
    });
    let (pending_admin, proposed_at) = pending.unwrap();
    assert_eq!(pending_admin, new_admin);

    env.ledger()
        .with_mut(|ledger| ledger.sequence_number = proposed_at + MIN_ADMIN_DELAY_LEDGERS);
    c.accept_admin(&new_admin);

    let stored_admin = env.as_contract(&c.address, || {
        env.storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
    });
    assert_eq!(stored_admin, Some(new_admin));
    let pending = env.as_contract(&c.address, || {
        env.storage()
            .instance()
            .get::<DataKey, (Address, u32)>(&DataKey::PendingAdmin)
    });
    assert!(pending.is_none());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_non_admin_cannot_propose_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    c.propose_admin(&Address::generate(&env), &Address::generate(&env));
}

#[test]
#[should_panic(expected = "admin transfer time lock not elapsed")]
fn test_accept_admin_before_timelock_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let new_admin = Address::generate(&env);
    c.propose_admin(&admin, &new_admin);

    let proposed_at = env.as_contract(&c.address, || {
        env.storage()
            .instance()
            .get::<DataKey, (Address, u32)>(&DataKey::PendingAdmin)
            .unwrap()
            .1
    });
    env.ledger().with_mut(|ledger| {
        ledger.sequence_number = proposed_at + MIN_ADMIN_DELAY_LEDGERS - 1;
    });
    c.accept_admin(&new_admin);
}

#[test]
#[should_panic(expected = "not pending admin")]
fn test_non_pending_admin_cannot_accept_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let new_admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    c.propose_admin(&admin, &new_admin);

    let proposed_at = env.as_contract(&c.address, || {
        env.storage()
            .instance()
            .get::<DataKey, (Address, u32)>(&DataKey::PendingAdmin)
            .unwrap()
            .1
    });
    env.ledger()
        .with_mut(|ledger| ledger.sequence_number = proposed_at + MIN_ADMIN_DELAY_LEDGERS);
    c.accept_admin(&stranger);
}

fn s(env: &Env, v: &str) -> String {
    String::from_str(env, v)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, ..) = setup(&env);
    let stored_admin =
        env.as_contract(&c.address, || env.storage().instance().get(&DataKey::Admin));
    assert_eq!(stored_admin, Some(admin));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(PrivacyLayerContract, ());
    let c = PrivacyLayerContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
fn test_set_consent() {
    let env = Env::default();
    env.mock_all_auths();
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
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    c.set_consent(&user, &true, &true, &true, &true, &Some(86_400u64));
    let consent = c.get_consent(&user).unwrap();
    assert!(consent.expires_at.is_some());
}

#[test]
fn test_revoke_consent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    c.set_consent(&user, &true, &true, &true, &true, &None);
    c.revoke_consent(&user);
    assert!(!c.has_consent(&user, &s(&env, "analytics")));
}

#[test]
fn test_submit_zkp() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    let zkp_hash = BytesN::from_array(&env, &[2u8; 32]);
    let proof_id = c.submit_zkp(&user, &s(&env, "1,2,3"), &zkp_hash);
    let proof = c.get_proof(&proof_id).unwrap();
    assert!(!proof.verified);
}

#[test]
fn test_verify_zkp() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let key = setup_verifier_key(&env, &c, &admin);
    let user = Address::generate(&env);
    let zkp_hash = BytesN::from_array(&env, &[2u8; 32]);
    let proof_id = c.submit_zkp(&user, &s(&env, "1,2,3"), &zkp_hash);
    let sig = sign(&env, &key, &proof_id);
    c.verify_zkp(&admin, &proof_id, &sig);
    let proof = c.get_proof(&proof_id).unwrap();
    assert!(proof.verified);
    assert_eq!(proof.signature, Some(sig));
}

#[test]
fn test_verify_zkp_rejects_invalid_signature() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    setup_verifier_key(&env, &c, &admin);
    let user = Address::generate(&env);
    let zkp_hash = BytesN::from_array(&env, &[2u8; 32]);
    let proof_id = c.submit_zkp(&user, &s(&env, "1,2,3"), &zkp_hash);

    // Valid signature from a key other than the configured verifier key.
    let rogue = SigningKey::from_bytes(&[9u8; 32]);
    let forged = sign(&env, &rogue, &proof_id);
    assert!(c.try_verify_zkp(&admin, &proof_id, &forged).is_err());

    // Arbitrary bytes are rejected too.
    let garbage = BytesN::from_array(&env, &[1u8; 64]);
    assert!(c.try_verify_zkp(&admin, &proof_id, &garbage).is_err());

    let proof = c.get_proof(&proof_id).unwrap();
    assert!(!proof.verified);
    assert_eq!(proof.signature, None);
}

#[test]
#[should_panic(expected = "verifier key not set")]
fn test_verify_zkp_requires_verifier_key() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    let zkp_hash = BytesN::from_array(&env, &[2u8; 32]);
    let proof_id = c.submit_zkp(&user, &s(&env, "1,2,3"), &zkp_hash);
    c.verify_zkp(&admin, &proof_id, &BytesN::from_array(&env, &[1u8; 64]));
}

#[test]
fn test_has_consent_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.has_consent(&Address::generate(&env), &s(&env, "analytics")));
}

#[test]
fn test_get_proof_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let pid = BytesN::from_array(&env, &[99u8; 32]);
    assert!(c.get_proof(&pid).is_none());
}

#[test]
fn test_consent_hash_unique_per_user() {
    // Same flags, different users must produce different hashes.
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    c.set_consent(&user_a, &true, &true, &false, &false, &None);
    c.set_consent(&user_b, &true, &true, &false, &false, &None);
    let hash_a = c.get_consent(&user_a).unwrap().consent_hash;
    let hash_b = c.get_consent(&user_b).unwrap().consent_hash;
    assert_ne!(hash_a, hash_b);
}

#[test]
fn test_consent_hash_unique_per_flags() {
    // Same user, different flag combinations must produce different hashes.
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    c.set_consent(&user, &true, &true, &false, &false, &None);
    let hash_all_on = c.get_consent(&user).unwrap().consent_hash;
    c.set_consent(&user, &false, &false, &false, &false, &None);
    let hash_all_off = c.get_consent(&user).unwrap().consent_hash;
    assert_ne!(hash_all_on, hash_all_off);
}

#[test]
fn test_has_consent_third_party_sharing() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let user = Address::generate(&env);
    c.set_consent(&user, &false, &false, &false, &true, &None);
    assert!(c.has_consent(&user, &s(&env, "third_party_sharing")));
    assert!(!c.has_consent(&user, &s(&env, "analytics")));
}

#[test]
fn test_data_request_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let requester = Address::generate(&env);

    assert_eq!(c.get_request_counter(), 0);

    let req_id = c.request_data_access(
        &requester,
        &s(&env, "demographics"),
        &s(&env, "ad-attribution"),
    );
    assert_eq!(req_id, 1);
    assert_eq!(c.get_request_counter(), 1);

    let req = c.get_data_request(&req_id).unwrap();
    assert_eq!(req.request_id, 1);
    assert_eq!(req.requester, requester);
    assert_eq!(req.data_type, s(&env, "demographics"));
    assert_eq!(req.purpose, s(&env, "ad-attribution"));
    assert!(!req.approved);
    assert!(!req.rejected);

    // Approve
    c.approve_data_request(&admin, &req_id);
    let req_approved = c.get_data_request(&req_id).unwrap();
    assert!(req_approved.approved);
    assert!(!req_approved.rejected);

    // Reject
    c.reject_data_request(&admin, &req_id);
    let req_rejected = c.get_data_request(&req_id).unwrap();
    assert!(!req_rejected.approved);
    assert!(req_rejected.rejected);
}

#[test]
fn test_data_request_counter_increments() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);

    let id1 = c.request_data_access(&r1, &s(&env, "type1"), &s(&env, "p1"));
    let id2 = c.request_data_access(&r2, &s(&env, "type2"), &s(&env, "p2"));

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(c.get_request_counter(), 2);
}

#[test]
fn test_data_request_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_data_request(&999u64).is_none());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_approve_data_request_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let requester = Address::generate(&env);
    let rogue = Address::generate(&env);

    let req_id = c.request_data_access(&requester, &s(&env, "type"), &s(&env, "purpose"));
    c.approve_data_request(&rogue, &req_id);
}

#[test]
#[should_panic(expected = "request not found")]
fn test_approve_data_request_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.approve_data_request(&admin, &42u64);
}
