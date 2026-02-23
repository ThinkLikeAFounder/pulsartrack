#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(env: &Env) -> (DisputeResolutionContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);

    let contract_id = env.register_contract(None, DisputeResolutionContract);
    let client = DisputeResolutionContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &1000i128);

    (client, admin, token_admin, token_addr)
}

fn make_desc(env: &Env) -> String {
    String::from_str(env, "fraudulent clicks")
}

fn make_evidence(env: &Env) -> String {
    String::from_str(env, "QmHash123")
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, DisputeResolutionContract);
    let client = DisputeResolutionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let filing_fee = 1000i128;

    client.initialize(&admin, &token, &filing_fee);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, DisputeResolutionContract);
    let client = DisputeResolutionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let filing_fee = 1000i128;

    client.initialize(&admin, &token, &filing_fee);
    client.initialize(&admin, &token, &filing_fee);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, DisputeResolutionContract);
    let client = DisputeResolutionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let filing_fee = 1000i128;

    // This should panic because admin didn't authorize it and we haven't mocked it
    client.initialize(&admin, &token, &filing_fee);
}

// ─── authorize_arbitrator ────────────────────────────────────────────────────

#[test]
fn test_authorize_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _) = setup(&env);
    let arbitrator = Address::generate(&env);

    client.authorize_arbitrator(&admin, &arbitrator);
    // No panic = success
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_authorize_arbitrator_by_stranger() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let stranger = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    client.authorize_arbitrator(&stranger, &arbitrator);
}

// ─── file_dispute ────────────────────────────────────────────────────────────

#[test]
fn test_file_dispute() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, token_admin, token_addr) = setup(&env);

    let claimant = Address::generate(&env);
    let respondent = Address::generate(&env);
    mint(&env, &token_addr, &claimant, 1_000_000);

    let dispute_id = client.file_dispute(
        &claimant,
        &respondent,
        &1u64,
        &50_000i128,
        &make_desc(&env),
        &make_evidence(&env),
    );

    assert_eq!(dispute_id, 1);
    assert_eq!(client.get_dispute_count(), 1);

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.claimant, claimant);
    assert_eq!(dispute.respondent, respondent);
    assert_eq!(dispute.claim_amount, 50_000);
    assert!(matches!(dispute.status, DisputeStatus::Filed));
    assert!(matches!(dispute.outcome, DisputeOutcome::Pending));
    assert!(dispute.arbitrator.is_none());
    let _ = token_admin;
}

#[test]
fn test_file_multiple_disputes() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_addr) = setup(&env);

    let claimant = Address::generate(&env);
    let respondent = Address::generate(&env);
    mint(&env, &token_addr, &claimant, 1_000_000);

    let id1 = client.file_dispute(&claimant, &respondent, &1u64, &10_000i128, &make_desc(&env), &make_evidence(&env));
    let id2 = client.file_dispute(&claimant, &respondent, &2u64, &20_000i128, &make_desc(&env), &make_evidence(&env));

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(client.get_dispute_count(), 2);
}

// ─── assign_arbitrator ───────────────────────────────────────────────────────

#[test]
fn test_assign_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_addr) = setup(&env);

    let claimant = Address::generate(&env);
    let respondent = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    mint(&env, &token_addr, &claimant, 1_000_000);

    let dispute_id = client.file_dispute(&claimant, &respondent, &1u64, &50_000i128, &make_desc(&env), &make_evidence(&env));

    client.authorize_arbitrator(&admin, &arbitrator);
    client.assign_arbitrator(&admin, &dispute_id, &arbitrator);

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.arbitrator, Some(arbitrator));
    assert!(matches!(dispute.status, DisputeStatus::UnderReview));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_assign_arbitrator_by_stranger() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_addr) = setup(&env);

    let claimant = Address::generate(&env);
    let respondent = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let stranger = Address::generate(&env);
    mint(&env, &token_addr, &claimant, 1_000_000);

    let dispute_id = client.file_dispute(&claimant, &respondent, &1u64, &50_000i128, &make_desc(&env), &make_evidence(&env));
    client.authorize_arbitrator(&admin, &arbitrator);
    client.assign_arbitrator(&stranger, &dispute_id, &arbitrator);
}

#[test]
#[should_panic(expected = "arbitrator not authorized")]
fn test_assign_unauthorized_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_addr) = setup(&env);

    let claimant = Address::generate(&env);
    let respondent = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    mint(&env, &token_addr, &claimant, 1_000_000);

    let dispute_id = client.file_dispute(&claimant, &respondent, &1u64, &50_000i128, &make_desc(&env), &make_evidence(&env));
    // arbitrator not authorized first
    client.assign_arbitrator(&admin, &dispute_id, &arbitrator);
}

// ─── resolve_dispute ─────────────────────────────────────────────────────────

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_addr) = setup(&env);

    let claimant = Address::generate(&env);
    let respondent = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    mint(&env, &token_addr, &claimant, 1_000_000);

    let dispute_id = client.file_dispute(&claimant, &respondent, &1u64, &50_000i128, &make_desc(&env), &make_evidence(&env));
    client.authorize_arbitrator(&admin, &arbitrator);
    client.assign_arbitrator(&admin, &dispute_id, &arbitrator);

    env.ledger().with_mut(|li| { li.timestamp = 1000; });

    client.resolve_dispute(&arbitrator, &dispute_id, &DisputeOutcome::Claimant, &String::from_str(&env, "claimant wins"));

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert!(matches!(dispute.status, DisputeStatus::Resolved));
    assert!(dispute.resolved_at.is_some());
    assert_eq!(dispute.resolved_at, Some(1000));
}

#[test]
#[should_panic(expected = "not assigned arbitrator")]
fn test_resolve_by_wrong_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_addr) = setup(&env);

    let claimant = Address::generate(&env);
    let respondent = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let wrong_arb = Address::generate(&env);
    mint(&env, &token_addr, &claimant, 1_000_000);

    let dispute_id = client.file_dispute(&claimant, &respondent, &1u64, &50_000i128, &make_desc(&env), &make_evidence(&env));
    client.authorize_arbitrator(&admin, &arbitrator);
    client.assign_arbitrator(&admin, &dispute_id, &arbitrator);

    client.resolve_dispute(&wrong_arb, &dispute_id, &DisputeOutcome::Respondent, &String::from_str(&env, "wrong"));
}

// ─── read-only ───────────────────────────────────────────────────────────────

#[test]
fn test_get_dispute_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);

    assert!(client.get_dispute(&999u64).is_none());
}

#[test]
fn test_get_dispute_count_initial() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);

    assert_eq!(client.get_dispute_count(), 0);
}
