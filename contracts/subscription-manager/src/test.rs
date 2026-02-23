#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (SubscriptionManagerContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let treasury = Address::generate(env);
    let id = env.register_contract(None, SubscriptionManagerContract);
    let c = SubscriptionManagerContractClient::new(env, &id);
    c.initialize(&admin, &token, &treasury);
    (c, admin, token_admin, token)
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    c.initialize(&admin, &token, &Address::generate(&env));
}

#[test]
fn test_subscribe() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, 500_000_000);
    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    assert!(c.is_active(&subscriber));
    let sub = c.get_subscription(&subscriber).unwrap();
    assert!(matches!(sub.tier, SubscriptionTier::Starter));
}

#[test]
fn test_cancel_subscription() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let subscriber = Address::generate(&env);
    mint(&env, &token, &subscriber, 500_000_000);
    c.subscribe(&subscriber, &SubscriptionTier::Starter, &false, &true);
    c.cancel_subscription(&subscriber);
    // Cancel only disables auto-renewal; subscription stays active until expiry
    let sub = c.get_subscription(&subscriber).unwrap();
    assert!(!sub.auto_renew);
}

#[test]
fn test_is_active_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(!c.is_active(&Address::generate(&env)));
}

#[test]
fn test_get_subscription_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_subscription(&Address::generate(&env)).is_none());
}
