#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn setup(env: &Env) -> (PerformanceOracleContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, PerformanceOracleContract);
    let c = PerformanceOracleContractClient::new(env, &id);
    c.initialize(&admin, &2u32);
    (c, admin)
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PerformanceOracleContract);
    let c = PerformanceOracleContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a, &2u32); c.initialize(&a, &2u32);
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
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let attester = Address::generate(&env);
    c.authorize_attester(&admin, &attester);
}

#[test]
fn test_submit_attestation() {
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let att = Address::generate(&env);
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    c.authorize_attester(&admin, &att);
    c.submit_attestation(&att, &1u64, &1000u64, &50u64, &5u32, &90u32, &data_hash);
    assert_eq!(c.get_attestation_count(&1u64), 1);
}

#[test]
fn test_get_consensus_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_consensus(&999u64).is_none());
}
