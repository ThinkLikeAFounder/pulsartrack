#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (CampaignOrchestratorContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let id = env.register_contract(None, CampaignOrchestratorContract);
    let c = CampaignOrchestratorContractClient::new(env, &id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
}

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
fn test_create_campaign() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let advertiser = Address::generate(&env);
    // min_budget=1_000_000, duration 100-10_000, default platform_fee=2%
    // budget=1_000_000 + fee=20_000 = 1_020_000 needed
    mint(&env, &token, &advertiser, 5_000_000);
    let id = c.create_campaign(&advertiser, &1u32, &1_000_000i128, &100i128, &1000u32, &10_000u64, &5_000u64, &true);
    assert_eq!(id, 1);
    assert_eq!(c.get_campaign_count(), 1);
    let campaign = c.get_campaign(&id).unwrap();
    assert_eq!(campaign.budget, 1_000_000);
}

#[test]
fn test_verify_publisher() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    c.verify_publisher(&admin, &publisher, &80u32);
    let pm = c.get_publisher_metrics(&publisher).unwrap();
    assert_eq!(pm.reputation_score, 80);
}

#[test]
fn test_pause_resume_campaign() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let advertiser = Address::generate(&env);
    mint(&env, &token, &advertiser, 5_000_000);
    let id = c.create_campaign(&advertiser, &1u32, &1_000_000i128, &100i128, &1000u32, &10_000u64, &5_000u64, &true);
    c.pause_campaign(&advertiser, &id);
    let campaign = c.get_campaign(&id).unwrap();
    assert!(matches!(campaign.status, CampaignStatus::Paused));
    c.resume_campaign(&advertiser, &id);
    let campaign = c.get_campaign(&id).unwrap();
    assert!(matches!(campaign.status, CampaignStatus::Active));
}

#[test]
fn test_set_platform_fee() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    c.set_platform_fee(&admin, &5u32); // max is 10
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_platform_fee_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    c.set_platform_fee(&Address::generate(&env), &5u32);
}

#[test]
fn test_get_campaign_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_campaign(&999u64).is_none());
}
