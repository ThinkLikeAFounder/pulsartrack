#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (RefundProcessorContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let id = env.register_contract(None, RefundProcessorContract);
    let c = RefundProcessorContractClient::new(env, &id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    c.initialize(&admin, &token);
}

#[test]
fn test_request_refund() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let requester = Address::generate(&env);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "poor performance"));
    assert_eq!(id, 1);
    let refund = c.get_refund(&id).unwrap();
    assert!(matches!(refund.status, RefundStatus::Requested));
    assert_eq!(refund.amount_requested, 50_000);
}

#[test]
fn test_approve_refund() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let requester = Address::generate(&env);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.approve_refund(&admin, &id, &30_000i128);
    let refund = c.get_refund(&id).unwrap();
    assert!(matches!(refund.status, RefundStatus::Approved));
    assert_eq!(refund.amount_approved, 30_000);
}

#[test]
fn test_reject_refund() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let requester = Address::generate(&env);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.reject_refund(&admin, &id);
    let refund = c.get_refund(&id).unwrap();
    assert!(matches!(refund.status, RefundStatus::Rejected));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_approve_refund_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let requester = Address::generate(&env);
    let id = c.request_refund(&requester, &1u64, &50_000i128, &s(&env, "reason"));
    c.approve_refund(&Address::generate(&env), &id, &30_000i128);
}

#[test]
fn test_get_refund_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_refund(&999u64).is_none());
}
