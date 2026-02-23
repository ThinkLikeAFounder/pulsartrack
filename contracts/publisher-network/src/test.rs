#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, vec};

fn setup(env: &Env) -> (PublisherNetworkContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, PublisherNetworkContract);
    let c = PublisherNetworkContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PublisherNetworkContract);
    let c = PublisherNetworkContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
    assert_eq!(c.get_node_count(), 0);
    let stats = c.get_network_stats();
    assert_eq!(stats.total_nodes, 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PublisherNetworkContract);
    let c = PublisherNetworkContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, PublisherNetworkContract);
    let c = PublisherNetworkContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_join_network() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let cats = vec![&env, s(&env, "tech"), s(&env, "news")];
    c.join_network(&pub1, &NodeType::Standard, &10_000u64, &100i128, &s(&env, "US-East"), &cats);
    assert_eq!(c.get_node_count(), 1);
    let node = c.get_node(&pub1).unwrap();
    assert!(node.is_active);
    assert_eq!(node.capacity, 10_000);
    let stats = c.get_network_stats();
    assert_eq!(stats.total_nodes, 1);
    assert_eq!(stats.active_nodes, 1);
    assert_eq!(stats.total_capacity, 10_000);
}

#[test]
#[should_panic(expected = "already in network")]
fn test_join_network_duplicate() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let cats = vec![&env, s(&env, "tech")];
    c.join_network(&pub1, &NodeType::Standard, &10_000u64, &100i128, &s(&env, "US-East"), &cats);
    c.join_network(&pub1, &NodeType::Premium, &20_000u64, &200i128, &s(&env, "EU-West"), &cats);
}

#[test]
fn test_heartbeat() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let cats = vec![&env, s(&env, "tech")];
    c.join_network(&pub1, &NodeType::Standard, &10_000u64, &100i128, &s(&env, "US"), &cats);
    c.heartbeat(&pub1);
}

#[test]
#[should_panic(expected = "not in network")]
fn test_heartbeat_not_in_network() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.heartbeat(&Address::generate(&env));
}

#[test]
fn test_deactivate() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let cats = vec![&env, s(&env, "tech")];
    c.join_network(&pub1, &NodeType::Standard, &10_000u64, &100i128, &s(&env, "US"), &cats);
    c.deactivate(&pub1);
    let node = c.get_node(&pub1).unwrap();
    assert!(!node.is_active);
    let stats = c.get_network_stats();
    assert_eq!(stats.active_nodes, 0);
}

#[test]
fn test_suspend_publisher() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let fraud = Address::generate(&env);
    c.set_fraud_contract(&admin, &fraud);
    let pub1 = Address::generate(&env);
    let cats = vec![&env, s(&env, "tech")];
    c.join_network(&pub1, &NodeType::Standard, &10_000u64, &100i128, &s(&env, "US"), &cats);
    c.suspend_publisher(&fraud, &pub1);
    let node = c.get_node(&pub1).unwrap();
    assert!(!node.is_active);
}

#[test]
#[should_panic(expected = "unauthorized fraud contract")]
fn test_suspend_publisher_wrong_fraud() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let fraud = Address::generate(&env);
    c.set_fraud_contract(&admin, &fraud);
    let pub1 = Address::generate(&env);
    let cats = vec![&env, s(&env, "tech")];
    c.join_network(&pub1, &NodeType::Standard, &10_000u64, &100i128, &s(&env, "US"), &cats);
    c.suspend_publisher(&Address::generate(&env), &pub1);
}

#[test]
fn test_record_impression() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let cats = vec![&env, s(&env, "tech")];
    c.join_network(&pub1, &NodeType::Standard, &10_000u64, &100i128, &s(&env, "US"), &cats);
    c.record_impression(&pub1);
    let stats = c.get_network_stats();
    assert_eq!(stats.total_impressions_served, 1);
}

#[test]
fn test_get_node_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_node(&Address::generate(&env)).is_none());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_fraud_contract_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    c.set_fraud_contract(&Address::generate(&env), &Address::generate(&env));
}
