#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (AnomalyDetectorContractClient, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, AnomalyDetectorContract);
    let c = AnomalyDetectorContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin, oracle)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, AnomalyDetectorContract);
    let c = AnomalyDetectorContractClient::new(&env, &id);
    let a = Address::generate(&env); let o = Address::generate(&env);
    c.initialize(&a, &o); c.initialize(&a, &o);
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
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.set_baseline(&oracle, &1u64, &1000u64, &50u64, &5u32);
    let bl = c.get_baseline(&1u64).unwrap();
    assert_eq!(bl.avg_impressions_per_hour, 1000);
    assert_eq!(bl.avg_clicks_per_hour, 50);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_baseline_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    c.set_baseline(&Address::generate(&env), &1u64, &1000u64, &50u64, &5u32);
}

#[test]
fn test_report_anomaly() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let publisher = Address::generate(&env);
    c.report_anomaly(&oracle, &1u64, &Some(publisher.clone()), &AnomalyType::ClickFarming, &AnomalySeverity::Critical, &s(&env, "spike"), &s(&env, "{}"), &true);
    assert_eq!(c.get_report_count(), 1);
    assert!(c.is_publisher_flagged(&publisher));
}

#[test]
fn test_get_baseline_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_baseline(&999u64).is_none());
}

#[test]
fn test_is_publisher_flagged_false() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(!c.is_publisher_flagged(&Address::generate(&env)));
}

#[test]
fn test_get_report_count_initial() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert_eq!(c.get_report_count(), 0);
}
