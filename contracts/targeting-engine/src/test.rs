#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[contract]
pub struct MockCampaignLifecycle;

#[contractimpl]
impl MockCampaignLifecycle {
    pub fn set_advertiser(env: Env, advertiser: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("ADVERTSR"), &advertiser);
    }

    pub fn get_lifecycle(env: Env, campaign_id: u64) -> Option<CampaignLifecycle> {
        let advertiser: Address = env.storage().instance().get(&symbol_short!("ADVERTSR"))?;
        Some(CampaignLifecycle {
            campaign_id,
            advertiser,
            state: LifecycleState::Active,
            created_at: 0,
            activated_at: Some(0),
            paused_at: None,
            completed_at: None,
            cancelled_at: None,
            pause_count: 0,
            extension_count: 0,
            original_end_ledger: 0,
            current_end_ledger: u32::MAX,
        })
    }
}

fn setup<'a>(env: &'a Env, advertiser: &Address) -> (TargetingEngineContractClient<'a>, Address) {
    let admin = Address::generate(env);
    let lifecycle_id = env.register(MockCampaignLifecycle, ());
    MockCampaignLifecycleClient::new(env, &lifecycle_id).set_advertiser(advertiser);
    let id = env.register(TargetingEngineContract, ());
    let c = TargetingEngineContractClient::new(env, &id);
    c.initialize(&admin, &lifecycle_id);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String {
    String::from_str(env, v)
}

fn default_params(env: &Env) -> TargetingParams {
    TargetingParams {
        geographic_targets: s(env, "US,EU"),
        interest_segments: s(env, "tech,news"),
        excluded_segments: s(env, ""),
        min_age: 18,
        max_age: 65,
        device_types: s(env, "mobile,desktop"),
        languages: s(env, "en"),
        min_reputation: 50,
        exclude_fraud: true,
        require_kyc: false,
        max_cpm: 10_000,
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser = Address::generate(&env);
    let (c, admin) = setup(&env, &advertiser);
    let stored_admin =
        env.as_contract(&c.address, || env.storage().instance().get(&DataKey::Admin));
    assert_eq!(stored_admin, Some(admin));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(TargetingEngineContract, ());
    let c = TargetingEngineContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let lifecycle_contract = Address::generate(&env);
    c.initialize(&a, &lifecycle_contract);
    c.initialize(&a, &lifecycle_contract);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register(TargetingEngineContract, ());
    let c = TargetingEngineContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_add_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser = Address::generate(&env);
    let (c, admin) = setup(&env, &advertiser);
    c.add_oracle(&admin, &Address::generate(&env));
}

#[test]
fn test_set_targeting() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser = Address::generate(&env);
    let (c, _) = setup(&env, &advertiser);
    let params = default_params(&env);
    c.set_targeting(&advertiser, &1u64, &params);
    let config = c.get_targeting(&1u64).unwrap();
    assert_eq!(config.geographic_targets, s(&env, "US,EU"));
}

#[test]
fn test_compute_score() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser = Address::generate(&env);
    let (c, admin) = setup(&env, &advertiser);
    let oracle = Address::generate(&env);
    let pub1 = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.set_targeting(&advertiser, &1u64, &default_params(&env));
    c.compute_score(
        &oracle,
        &1u64,
        &pub1,
        &750u32,
        &s(&env, "geo_match,interest_match"),
    );
    let score = c.get_targeting_score(&1u64, &pub1).unwrap();
    assert_eq!(score.score, 750);
}

#[test]
fn test_is_publisher_targeted() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser = Address::generate(&env);
    let (c, admin) = setup(&env, &advertiser);
    let oracle = Address::generate(&env);
    let pub1 = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.set_targeting(&advertiser, &1u64, &default_params(&env));
    c.compute_score(&oracle, &1u64, &pub1, &750u32, &s(&env, "match"));
    assert!(c.is_publisher_targeted(&1u64, &pub1, &600u32));
    assert!(!c.is_publisher_targeted(&1u64, &pub1, &800u32));
}

#[test]
fn test_get_targeting_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser = Address::generate(&env);
    let (c, _) = setup(&env, &advertiser);
    assert!(c.get_targeting(&999u64).is_none());
}

#[test]
#[should_panic(expected = "score must be 0-1000")]
fn test_compute_score_rejects_above_max() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser = Address::generate(&env);
    let (c, admin) = setup(&env, &advertiser);
    let oracle = Address::generate(&env);
    let pub1 = Address::generate(&env);
    c.add_oracle(&admin, &oracle);
    c.compute_score(&oracle, &1u64, &pub1, &5001u32, &s(&env, "match"));
}

#[test]
#[should_panic(expected = "unauthorized: campaign belongs to a different advertiser")]
fn test_set_targeting_rejects_wrong_advertiser() {
    let env = Env::default();
    env.mock_all_auths();
    let advertiser_a = Address::generate(&env);
    let (c, _) = setup(&env, &advertiser_a);
    let advertiser_b = Address::generate(&env);
    c.set_targeting(&advertiser_a, &42u64, &default_params(&env));
    c.set_targeting(&advertiser_b, &42u64, &default_params(&env));
}
