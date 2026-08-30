#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup(env: &Env) -> (AnalyticsAggregatorContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register(AnalyticsAggregatorContract, ());
    let c = AnalyticsAggregatorContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin, oracle)
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
    let id = env.register(AnalyticsAggregatorContract, ());
    let c = AnalyticsAggregatorContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let o = Address::generate(&env);
    c.initialize(&a, &o);
    c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register(AnalyticsAggregatorContract, ());
    let c = AnalyticsAggregatorContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_record_impression() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer = Address::generate(&env);
    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.total_impressions, 1);
    assert_eq!(a.unique_viewers, 1);
    let stats = c.get_global_stats();
    assert_eq!(stats.total_campaigns, 1);
    assert_eq!(stats.total_impressions, 1);
}

#[test]
fn test_record_click() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer = Address::generate(&env);
    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_click(&oracle, &1u64);
    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.total_clicks, 1);
}

#[test]
#[should_panic(expected = "only oracle can record analytics")]
fn test_record_impression_rejects_non_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let caller = Address::generate(&env);
    let viewer = Address::generate(&env);

    c.record_impression(&caller, &1u64, &viewer, &100i128);
}

#[test]
#[should_panic(expected = "spend must be non-negative")]
fn test_record_impression_rejects_negative_spend() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer = Address::generate(&env);

    c.record_impression(&oracle, &1u64, &viewer, &-1i128);
}

#[test]
fn test_record_impression_tracks_unique_viewers() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer1 = Address::generate(&env);
    let viewer2 = Address::generate(&env);

    c.record_impression(&oracle, &1u64, &viewer1, &100i128);
    c.record_impression(&oracle, &1u64, &viewer1, &100i128);
    c.record_impression(&oracle, &1u64, &viewer2, &100i128);

    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.total_impressions, 3);
    assert_eq!(a.unique_viewers, 2);
    let stats = c.get_global_stats();
    assert_eq!(stats.total_campaigns, 1);
}

#[test]
fn test_total_campaigns_not_double_counted() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer = Address::generate(&env);

    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_impression(&oracle, &1u64, &viewer, &100i128);

    let stats = c.get_global_stats();
    assert_eq!(stats.total_campaigns, 1);
    assert_eq!(stats.total_impressions, 3);
}

#[test]
fn test_total_campaigns_counts_distinct_campaigns() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer = Address::generate(&env);

    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_impression(&oracle, &2u64, &viewer, &200i128);
    c.record_impression(&oracle, &3u64, &viewer, &300i128);

    let stats = c.get_global_stats();
    assert_eq!(stats.total_campaigns, 3);
    assert_eq!(stats.total_impressions, 3);
    assert_eq!(stats.total_spend, 600);
}

#[test]
fn test_hourly_stats_ttl_covers_current_hour() {
    assert_eq!(AnalyticsAggregatorContract::hourly_stats_ttl_ledgers(), 720);
}

#[test]
#[should_panic(expected = "only oracle can record analytics")]
fn test_record_click_rejects_non_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let caller = Address::generate(&env);
    let viewer = Address::generate(&env);

    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_click(&caller, &1u64);
}

#[test]
fn test_get_campaign_analytics_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_campaign_analytics(&999u64).is_none());
}

#[test]
fn test_get_global_stats() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let stats = c.get_global_stats();
    assert_eq!(stats.total_campaigns, 0);
    assert_eq!(stats.total_impressions, 0);
    assert_eq!(stats.total_clicks, 0);
    assert_eq!(stats.total_spend, 0);
}

#[test]
fn test_get_global_stats_after_multiple_campaigns() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer = Address::generate(&env);

    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_impression(&oracle, &2u64, &viewer, &200i128);
    c.record_click(&oracle, &1u64);

    let stats = c.get_global_stats();
    assert_eq!(stats.total_campaigns, 2);
    assert_eq!(stats.total_impressions, 2);
    assert_eq!(stats.total_clicks, 1);
    assert_eq!(stats.total_spend, 300);
}

#[test]
fn test_record_conversion_increments_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let viewer = Address::generate(&env);
    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_click(&oracle, &1u64);
    c.record_conversion(&oracle, &1u64);

    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.total_conversions, 1);
}

#[test]
fn test_record_conversion_calculates_cvr() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    // 4 impressions, 2 clicks, 1 conversion → cvr = 1/2 * 10000 = 5000
    for _ in 0..4 {
        let viewer = Address::generate(&env);
        c.record_impression(&oracle, &1u64, &viewer, &100i128);
    }
    c.record_click(&oracle, &1u64);
    c.record_click(&oracle, &1u64);
    c.record_conversion(&oracle, &1u64);

    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.cvr, 5000);
}

#[test]
fn test_cvr_stays_zero_without_clicks() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);

    // Impressions but no clicks — cvr guard prevents divide-by-zero
    let viewer = Address::generate(&env);
    c.record_impression(&oracle, &1u64, &viewer, &100i128);

    // Manually set total_conversions via record_conversion requires a click first;
    // here we just confirm cvr is 0 without any clicks
    let a = c.get_campaign_analytics(&1u64).unwrap();
    assert_eq!(a.cvr, 0);
}

#[test]
#[should_panic(expected = "analytics not found")]
fn test_record_conversion_without_analytics_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);

    c.record_conversion(&oracle, &999u64);
}

#[test]
#[should_panic(expected = "only oracle can record analytics")]
fn test_record_conversion_rejects_non_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let caller = Address::generate(&env);
    let viewer = Address::generate(&env);

    c.record_impression(&oracle, &1u64, &viewer, &100i128);
    c.record_click(&oracle, &1u64);
    c.record_conversion(&caller, &1u64);
}
