#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env};

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
    CampaignOrchestratorContractClient<'_>,
    Address,
    Address,
    Address,
) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let id = env.register(CampaignOrchestratorContract, ());
    let c = CampaignOrchestratorContractClient::new(env, &id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
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
fn test_create_campaign() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let advertiser = Address::generate(&env);
    // min_budget=1_000_000, duration 100-10_000, default platform_fee=2%
    // budget=1_000_000 + fee=20_000 = 1_020_000 needed
    mint(&env, &token, &advertiser, 5_000_000);
    let id = c.create_campaign(&CampaignCreateArgs {
        advertiser: advertiser.clone(),
        campaign_type: 1u32,
        budget: 1_000_000i128,
        cost_per_view: 100i128,
        duration: 1000u32,
        target_views: 10_000u64,
        daily_view_limit: 5_000u64,
        refundable: true,
    });
    assert_eq!(id, 1);
    assert_eq!(c.get_campaign_count(), 1);
    let campaign = c.get_campaign(&id).unwrap();
    assert_eq!(campaign.budget, 1_000_000);
}

#[test]
fn test_verify_publisher() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    c.verify_publisher(&admin, &publisher, &80u32);
    let pm = c.get_publisher_metrics(&publisher).unwrap();
    assert_eq!(pm.reputation_score, 80);
}

#[test]
fn test_pause_resume_campaign() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let advertiser = Address::generate(&env);
    mint(&env, &token, &advertiser, 5_000_000);
    let id = c.create_campaign(&CampaignCreateArgs {
        advertiser: advertiser.clone(),
        campaign_type: 1u32,
        budget: 1_000_000i128,
        cost_per_view: 100i128,
        duration: 1000u32,
        target_views: 10_000u64,
        daily_view_limit: 5_000u64,
        refundable: true,
    });
    c.pause_campaign(&advertiser, &id);
    let campaign = c.get_campaign(&id).unwrap();
    assert!(matches!(campaign.status, CampaignStatus::Paused));
    c.resume_campaign(&advertiser, &id);
    let campaign = c.get_campaign(&id).unwrap();
    assert!(matches!(campaign.status, CampaignStatus::Active));
}

#[test]
fn test_cancel_campaign_decrements_active_campaigns() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let advertiser = Address::generate(&env);
    mint(&env, &token, &advertiser, 5_000_000);

    let id = c.create_campaign(&CampaignCreateArgs {
        advertiser: advertiser.clone(),
        campaign_type: 1u32,
        budget: 1_000_000i128,
        cost_per_view: 100i128,
        duration: 1000u32,
        target_views: 10_000u64,
        daily_view_limit: 5_000u64,
        refundable: true,
    });

    let stats_before = c.get_advertiser_stats(&advertiser).unwrap();
    assert_eq!(stats_before.total_campaigns, 1);
    assert_eq!(stats_before.active_campaigns, 1);

    c.cancel_campaign(&advertiser, &id);

    let stats_after = c.get_advertiser_stats(&advertiser).unwrap();
    assert_eq!(stats_after.total_campaigns, 1);
    assert_eq!(stats_after.active_campaigns, 0);
}

#[test]
fn test_set_platform_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    c.set_platform_fee(&admin, &5u32); // max is 10
}

#[test]
#[should_panic(expected = "fee must be between 1 and 10")]
fn test_set_platform_fee_zero_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    c.set_platform_fee(&admin, &0u32);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_platform_fee_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    c.set_platform_fee(&Address::generate(&env), &5u32);
}

#[test]
fn test_get_campaign_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_campaign(&999u64).is_none());
}

// ── record_view tests (#783) ────────────────────────────────────────────────

fn setup_campaign_with_publisher(
    env: &Env,
) -> (
    CampaignOrchestratorContractClient<'_>,
    Address,
    Address,
    u64,
) {
    let (c, admin, token_admin, token) = setup(env);
    let advertiser = Address::generate(env);
    let publisher = Address::generate(env);

    mint(env, &token, &advertiser, 5_000_000);
    c.verify_publisher(&admin, &publisher, &80u32);

    let id = c.create_campaign(&CampaignCreateArgs {
        advertiser: advertiser.clone(),
        campaign_type: 1u32,
        budget: 1_000_000i128,
        cost_per_view: 100i128,
        duration: 1000u32,
        target_views: 10_000u64,
        daily_view_limit: 100u64,
        refundable: true,
    });
    (c, advertiser, publisher, id)
}

#[test]
fn test_record_view_valid() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, publisher, id) = setup_campaign_with_publisher(&env);

    c.record_view(&id, &publisher);

    let campaign = c.get_campaign(&id).unwrap();
    assert_eq!(campaign.current_views, 1);
    assert_eq!(campaign.remaining_budget, 999_900); // 1_000_000 - 100
}

#[test]
#[should_panic(expected = "publisher not verified")]
fn test_record_view_unverified_publisher() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, id) = setup_campaign_with_publisher(&env);
    let random = Address::generate(&env);

    c.record_view(&id, &random);
}

#[test]
#[should_panic(expected = "campaign not active")]
fn test_record_view_paused_campaign() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, advertiser, publisher, id) = setup_campaign_with_publisher(&env);

    c.pause_campaign(&advertiser, &id);
    c.record_view(&id, &publisher);
}

#[test]
#[should_panic(expected = "insufficient budget")]
fn test_record_view_insufficient_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, token_admin, token) = setup(&env);
    let advertiser = Address::generate(&env);
    let publisher = Address::generate(&env);
    let _ = token_admin;

    mint(&env, &token, &advertiser, 5_000_000);
    c.verify_publisher(&admin, &publisher, &80u32);

    // target_views and daily_view_limit are set above the budget-implied view
    // count so that budget exhaustion is the first guard to trip.
    let id = c.create_campaign(&CampaignCreateArgs {
        advertiser: advertiser.clone(),
        campaign_type: 1u32,
        budget: 1_000_000i128,
        cost_per_view: 100i128,
        duration: 1000u32,
        target_views: 20_000u64,
        daily_view_limit: 20_000u64,
        refundable: true,
    });

    // Exhaust budget: cost_per_view=100, remaining=1_000_000 → 10000 views needed
    for _ in 0..10_000 {
        c.record_view(&id, &publisher);
    }
    // Next view should fail
    c.record_view(&id, &publisher);
}

#[test]
#[should_panic(expected = "daily view limit reached")]
fn test_record_view_daily_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, publisher, id) = setup_campaign_with_publisher(&env);

    // daily_view_limit=100
    for _ in 0..100 {
        c.record_view(&id, &publisher);
    }
    c.record_view(&id, &publisher);
}

// ── Admin config setter tests (#784) ────────────────────────────────────────

#[test]
fn test_set_lifecycle_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let contract = Address::generate(&env);
    c.set_lifecycle_contract(&admin, &contract);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_lifecycle_contract_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    c.set_lifecycle_contract(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_set_escrow_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let contract = Address::generate(&env);
    c.set_escrow_contract(&admin, &contract);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_escrow_contract_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    c.set_escrow_contract(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_set_targeting_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let contract = Address::generate(&env);
    c.set_targeting_contract(&admin, &contract);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_targeting_contract_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    c.set_targeting_contract(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_set_auction_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, _) = setup(&env);
    let contract = Address::generate(&env);
    c.set_auction_contract(&admin, &contract);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_auction_contract_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    c.set_auction_contract(&Address::generate(&env), &Address::generate(&env));
}
