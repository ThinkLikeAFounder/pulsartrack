#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (RefundProcessorContractClient<'_>, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let id = env.register_contract(None, RefundProcessorContract);
    let c = RefundProcessorContractClient::new(env, &id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
}
fn s(env: &Env, v: &str) -> String {
    String::from_str(env, v)
}

fn setup_campaign(env: &Env, contract_id: &Address, campaign_id: u64, budget: i128) {
    let key = DataKey::Campaign(campaign_id);
    let campaign = Campaign {
        total_budget: budget,
        end_time: 0,
        refund_deadline: u64::MAX,
    };
    env.as_contract(contract_id, || {
        env.storage().persistent().set(&key, &campaign);
    });
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
fn test_request_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "poor performance"));
    assert_eq!(id, 1);
    let refund = c.get_refund(&id).unwrap();
    assert!(matches!(refund.status, RefundStatus::Requested));
    assert_eq!(refund.amount_requested, 50_000);
}

#[test]
#[should_panic(expected = "refund already pending for this campaign")]
fn test_request_refund_duplicate_for_same_campaign_and_requester() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);

    c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.request_refund(&requester, &1u64, &25_000i128, &s(&env, "duplicate"));
}

#[test]
fn test_request_refund_allowed_after_rejection() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);

    let first_id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.reject_refund(&admin, &first_id);

    let second_id = c.request_refund(&requester, &1u64, &25_000i128, &s(&env, "new request"));
    assert_eq!(second_id, 2);
}

#[test]
#[should_panic(expected = "refund cannot be rejected in its current status")]
fn test_reject_already_processed_refund_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, token_admin, token) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);

    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.approve_refund(&admin, &id, &50_000i128);

    // Fund the contract so process_refund succeeds
    mint(&env, &token, &c.address, 50_000);
    let _ = token_admin; // suppress warning
    c.process_refund(&admin, &id);

    // Attempting to reject a Processed refund must panic
    c.reject_refund(&admin, &id);
}

#[test]
#[should_panic(expected = "refund not approved")]
fn test_process_refund_cannot_be_replayed() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);

    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.approve_refund(&admin, &id, &50_000i128);
    mint(&env, &token, &c.address, 100_000);

    c.process_refund(&admin, &id);

    let refund = c.get_refund(&id).unwrap();
    assert!(matches!(refund.status, RefundStatus::Processed));
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&requester), 50_000);

    c.process_refund(&admin, &id);
}

#[test]
fn test_approve_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.approve_refund(&admin, &id, &30_000i128);
    let refund = c.get_refund(&id).unwrap();
    assert!(matches!(refund.status, RefundStatus::Approved));
    assert_eq!(refund.amount_approved, 30_000);
}

#[test]
fn test_reject_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.reject_refund(&admin, &id);
    let refund = c.get_refund(&id).unwrap();
    assert!(matches!(refund.status, RefundStatus::Rejected));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_approve_refund_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let requester = Address::generate(&env);
    setup_campaign(&env, &c.address, 1, 100_000);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.approve_refund(&Address::generate(&env), &id, &30_000i128);
}

#[test]
fn test_get_refund_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_refund(&999u64).is_none());
}
