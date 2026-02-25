#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (AnomalyDetectorContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, AnomalyDetectorContract);
    let c = AnomalyDetectorContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin, oracle)
}
fn s(env: &Env, v: &str) -> String {
    String::from_str(env, v)
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
    let id = env.register_contract(None, AnomalyDetectorContract);
    let c = AnomalyDetectorContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let o = Address::generate(&env);
    c.initialize(&a, &o);
    c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, AnomalyDetectorContract);
    let c = AnomalyDetectorContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_set_baseline() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.set_baseline(&oracle, &1u64, &1000u64, &50u64, &5u32);
    let bl = c.get_baseline(&1u64).unwrap();
    assert_eq!(bl.avg_impressions_per_hour, 1000);
    assert_eq!(bl.avg_clicks_per_hour, 50);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_baseline_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    c.set_baseline(&Address::generate(&env), &1u64, &1000u64, &50u64, &5u32);
}

#[test]
fn test_report_anomaly() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let publisher = Address::generate(&env);
    
    // Set baseline first
    c.set_baseline(&oracle, &1u64, &1000u64, &50u64, &300u32);
    
    // Report anomaly with metrics exceeding threshold (300% = 3x)
    // 4000 impressions > 3000 threshold (1000 * 300%)
    c.report_anomaly(
        &oracle,
        &1u64,
        &Some(publisher.clone()),
        &AnomalyType::ClickFarming,
        &AnomalySeverity::Critical,
        &s(&env, "spike"),
        &s(&env, "{}"),
        &true,
        &4000u64, // current_impressions_per_hour
        &200u64,  // current_clicks_per_hour
    );
    assert_eq!(c.get_report_count(), 1);
    assert!(c.is_publisher_flagged(&publisher));
}

#[test]
fn test_get_baseline_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_baseline(&999u64).is_none());
}

#[test]
fn test_is_publisher_flagged_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(!c.is_publisher_flagged(&Address::generate(&env)));
}

#[test]
fn test_get_report_count_initial() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert_eq!(c.get_report_count(), 0);
}

#[test]
#[should_panic(expected = "metrics do not exceed baseline thresholds")]
fn test_report_anomaly_below_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let publisher = Address::generate(&env);
    
    // Set baseline
    c.set_baseline(&oracle, &1u64, &1000u64, &50u64, &300u32);
    
    // Try to report anomaly with metrics NOT exceeding threshold
    // 2000 impressions < 3000 threshold (1000 * 300%)
    // 100 clicks < 150 threshold (50 * 300%)
    c.report_anomaly(
        &oracle,
        &1u64,
        &Some(publisher.clone()),
        &AnomalyType::ClickFarming,
        &AnomalySeverity::Critical,
        &s(&env, "spike"),
        &s(&env, "{}"),
        &true,
        &2000u64, // current_impressions_per_hour (below threshold)
        &100u64,  // current_clicks_per_hour (below threshold)
    );
}

#[test]
fn test_report_anomaly_no_baseline() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let publisher = Address::generate(&env);
    
    // Report anomaly without setting baseline (should succeed)
    c.report_anomaly(
        &oracle,
        &1u64,
        &Some(publisher.clone()),
        &AnomalyType::ClickFarming,
        &AnomalySeverity::Critical,
        &s(&env, "spike"),
        &s(&env, "{}"),
        &true,
        &4000u64,
        &200u64,
    );
    assert_eq!(c.get_report_count(), 1);
}

#[test]
fn test_report_anomaly_clicks_exceed_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let publisher = Address::generate(&env);
    
    // Set baseline
    c.set_baseline(&oracle, &1u64, &1000u64, &50u64, &300u32);
    
    // Report anomaly where only clicks exceed threshold
    // 2000 impressions < 3000 threshold
    // 200 clicks > 150 threshold (50 * 300%)
    c.report_anomaly(
        &oracle,
        &1u64,
        &Some(publisher.clone()),
        &AnomalyType::ClickFarming,
        &AnomalySeverity::High,
        &s(&env, "click spike"),
        &s(&env, "{}"),
        &true,
        &2000u64, // below threshold
        &200u64,  // exceeds threshold
    );
    assert_eq!(c.get_report_count(), 1);
}
