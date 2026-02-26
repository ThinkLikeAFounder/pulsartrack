#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn setup(env: &Env) -> (PerformanceOracleContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, PerformanceOracleContract);
    let c = PerformanceOracleContractClient::new(env, &id);
    c.initialize(&admin, &2u32);
    (c, admin)
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
    let id = env.register_contract(None, PerformanceOracleContract);
    let c = PerformanceOracleContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a, &2u32);
    c.initialize(&a, &2u32);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, PerformanceOracleContract);
    let c = PerformanceOracleContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &2u32);
}

#[test]
fn test_authorize_attester() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let attester = Address::generate(&env);
    c.authorize_attester(&admin, &attester);
}

#[test]
fn test_submit_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let att = Address::generate(&env);
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    c.authorize_attester(&admin, &att);
    c.submit_attestation(&att, &1u64, &1000u64, &50u64, &5u32, &90u32, &data_hash);
    let a = c.get_attestation(&1u64, &att).unwrap();
    assert_eq!(a.impressions_verified, 1000);
}

#[test]
fn test_get_attestation_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let att = Address::generate(&env);
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    c.authorize_attester(&admin, &att);
    c.submit_attestation(&att, &1u64, &1000u64, &50u64, &5u32, &90u32, &data_hash);
    assert_eq!(c.get_attestation_count(&1u64), 1);
}

#[test]
fn test_get_consensus_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_consensus(&999u64).is_none());
}

#[test]
fn test_consensus_with_actual_averaging() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    
    // Create 3 attesters
    let att1 = Address::generate(&env);
    let att2 = Address::generate(&env);
    let att3 = Address::generate(&env);
    
    c.authorize_attester(&admin, &att1);
    c.authorize_attester(&admin, &att2);
    c.authorize_attester(&admin, &att3);
    
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // Attester 1: 1000 impressions, 100 clicks, 10% fraud, 80 quality
    c.submit_attestation(&att1, &1u64, &1000u64, &100u64, &1000u32, &80u32, &data_hash);
    
    // Should not have consensus yet (need min 2 attesters)
    assert!(c.get_consensus(&1u64).is_none());
    
    // Attester 2: 2000 impressions, 200 clicks, 20% fraud, 90 quality
    c.submit_attestation(&att2, &1u64, &2000u64, &200u64, &2000u32, &90u32, &data_hash);
    
    // Now we have consensus with 2 attesters
    let consensus_opt = c.get_consensus(&1u64);
    assert!(consensus_opt.is_some());
    
    let consensus = consensus_opt.unwrap();
    
    // Average of 1000 and 2000 = 1500
    assert_eq!(consensus.avg_impressions, 1500);
    
    // Average of 100 and 200 = 150
    assert_eq!(consensus.avg_clicks, 150);
    
    // Average of 1000 and 2000 = 1500
    assert_eq!(consensus.avg_fraud_rate, 1500);
    
    // Average of 80 and 90 = 85
    assert_eq!(consensus.avg_quality_score, 85);
    
    assert_eq!(consensus.total_attesters, 2);
    assert!(consensus.consensus_reached);
}

#[test]
fn test_consensus_with_three_attesters() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    
    let att1 = Address::generate(&env);
    let att2 = Address::generate(&env);
    let att3 = Address::generate(&env);
    
    c.authorize_attester(&admin, &att1);
    c.authorize_attester(&admin, &att2);
    c.authorize_attester(&admin, &att3);
    
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // Attester 1: 900 impressions, 90 clicks, 5% fraud, 70 quality
    c.submit_attestation(&att1, &1u64, &900u64, &90u64, &500u32, &70u32, &data_hash);
    
    // Attester 2: 1200 impressions, 120 clicks, 10% fraud, 80 quality
    c.submit_attestation(&att2, &1u64, &1200u64, &120u64, &1000u32, &80u32, &data_hash);
    
    // Attester 3: 1500 impressions, 150 clicks, 15% fraud, 90 quality
    c.submit_attestation(&att3, &1u64, &1500u64, &150u64, &1500u32, &90u32, &data_hash);
    
    let consensus = c.get_consensus(&1u64).unwrap();
    
    // Average of 900, 1200, 1500 = 1200
    assert_eq!(consensus.avg_impressions, 1200);
    
    // Average of 90, 120, 150 = 120
    assert_eq!(consensus.avg_clicks, 120);
    
    // Average of 500, 1000, 1500 = 1000
    assert_eq!(consensus.avg_fraud_rate, 1000);
    
    // Average of 70, 80, 90 = 80
    assert_eq!(consensus.avg_quality_score, 80);
    
    assert_eq!(consensus.total_attesters, 3);
}

#[test]
fn test_no_consensus_with_insufficient_attesters() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    
    let att1 = Address::generate(&env);
    c.authorize_attester(&admin, &att1);
    
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // Only 1 attester (need min 2)
    c.submit_attestation(&att1, &1u64, &1000u64, &100u64, &500u32, &80u32, &data_hash);
    
    // Should not have consensus yet
    assert!(c.get_consensus(&1u64).is_none());
}

#[test]
fn test_last_attester_does_not_override_consensus() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    
    let att1 = Address::generate(&env);
    let att2 = Address::generate(&env);
    let att3 = Address::generate(&env);
    
    c.authorize_attester(&admin, &att1);
    c.authorize_attester(&admin, &att2);
    c.authorize_attester(&admin, &att3);
    
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // First two attesters report similar values
    c.submit_attestation(&att1, &1u64, &1000u64, &100u64, &500u32, &80u32, &data_hash);
    c.submit_attestation(&att2, &1u64, &1100u64, &110u64, &600u32, &85u32, &data_hash);
    
    // Third attester reports wildly different values
    c.submit_attestation(&att3, &1u64, &9000u64, &900u64, &5000u32, &50u32, &data_hash);
    
    let consensus = c.get_consensus(&1u64).unwrap();
    
    // Average should include all three, not just the last one
    // (1000 + 1100 + 9000) / 3 = 3700
    assert_eq!(consensus.avg_impressions, 3700);
    
    // (100 + 110 + 900) / 3 = 370
    assert_eq!(consensus.avg_clicks, 370);
    
    // The consensus is influenced by all attesters, not just the last one
    assert_eq!(consensus.total_attesters, 3);
}
