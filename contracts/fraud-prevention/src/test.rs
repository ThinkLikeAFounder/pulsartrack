#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (FraudPreventionContractClient, Address) {
    let admin = Address::generate(env);

    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(env, &contract_id);
    client.initialize(&admin);

    (client, admin)
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // This should panic because admin didn't authorize it and we haven't mocked it
    client.initialize(&admin);
}

// ─── set_dependent_contracts ─────────────────────────────────────────────────

#[test]
fn test_set_dependent_contracts() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);

    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_dependent_contracts_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);

    client.set_dependent_contracts(&stranger, &lifecycle, &network, &vault);
}

// ─── set_threshold ───────────────────────────────────────────────────────────

#[test]
fn test_set_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_threshold(&admin, &90u32);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_threshold_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);

    client.set_threshold(&stranger, &90u32);
}

#[test]
#[should_panic(expected = "invalid threshold")]
fn test_set_threshold_too_low() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_threshold(&admin, &40u32); // < 50
}

#[test]
#[should_panic(expected = "invalid threshold")]
fn test_set_threshold_too_high() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_threshold(&admin, &101u32); // > 100
}

// ─── oracle management ───────────────────────────────────────────────────────

#[test]
fn test_add_oracle_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    client.add_oracle(&admin, &oracle);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_oracle_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let oracle = Address::generate(&env);

    client.add_oracle(&stranger, &oracle);
}

#[test]
fn test_remove_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    client.add_oracle(&admin, &oracle);
    client.remove_oracle(&admin, &oracle);
}

// ─── flag_suspicious ─────────────────────────────────────────────────────────

#[test]
fn test_flag_suspicious_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);

    client.flag_suspicious(&admin, &publisher);

    let status = client.get_suspicious_status(&publisher).unwrap();
    assert_eq!(status.suspicious_views, 1);
    assert_eq!(status.total_flags, 1);
    assert!(!status.suspended);
}

#[test]
fn test_flag_suspicious_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.add_oracle(&admin, &oracle);
    client.flag_suspicious(&oracle, &publisher);

    let status = client.get_suspicious_status(&publisher).unwrap();
    assert_eq!(status.suspicious_views, 1);
    assert!(!status.suspended);
}

#[test]
#[should_panic(expected = "unauthorized - only admin or oracle can flag publishers")]
fn test_flag_suspicious_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.flag_suspicious(&stranger, &publisher);
}

#[test]
fn test_flag_suspicious_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);

    client.flag_suspicious(&admin, &publisher);
    client.flag_suspicious(&admin, &publisher);
    client.flag_suspicious(&admin, &publisher);

    let status = client.get_suspicious_status(&publisher).unwrap();
    assert_eq!(status.suspicious_views, 3);
    assert_eq!(status.total_flags, 3);
}

// ─── suspend_publisher ───────────────────────────────────────────────────────

#[test]
fn test_suspend_publisher_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    client.suspend_publisher(&admin, &publisher);

    assert!(client.is_publisher_suspended(&publisher));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_suspend_publisher_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.suspend_publisher(&stranger, &publisher);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_suspend_publisher_oracle_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.add_oracle(&admin, &oracle);
    // Oracle can flag, but cannot suspend
    client.suspend_publisher(&oracle, &publisher);
}

// ─── clear_flag ──────────────────────────────────────────────────────────────

#[test]
fn test_clear_flag() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    client.flag_suspicious(&admin, &publisher);
    assert!(client.get_suspicious_status(&publisher).is_some());

    client.clear_flag(&admin, &publisher);
    assert!(client.get_suspicious_status(&publisher).is_none());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_clear_flag_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.flag_suspicious(&admin, &publisher);
    client.clear_flag(&stranger, &publisher);
}

// ─── verify_view ─────────────────────────────────────────────────────────────

#[test]
fn test_verify_view_with_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    // Threshold is 80, base_score=80 + proof_bonus=10 = 90 >= 80 → verified
    let proof = BytesN::from_array(&env, &[1u8; 32]);
    let result = client.verify_view(&1u64, &publisher, &viewer, &Some(proof));
    assert!(result);

    assert_eq!(client.get_total_verifications(), 1);
}

#[test]
fn test_verify_view_without_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    // base_score = 80, no proof bonus → 80 >= 80 → verified
    let result = client.verify_view(&1u64, &publisher, &viewer, &None);
    assert!(result);
}

#[test]
fn test_verification_stats_updated() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    client.verify_view(&1u64, &publisher, &viewer, &None);

    let stats = client.get_verification_stats(&1u64);
    assert_eq!(stats.total_views, 1);
    assert_eq!(stats.verified_views, 1);
    assert_eq!(stats.rejected_views, 0);
}

// ─── read-only ───────────────────────────────────────────────────────────────

#[test]
fn test_get_suspicious_status_unknown() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let unknown = Address::generate(&env);

    assert!(client.get_suspicious_status(&unknown).is_none());
}

#[test]
fn test_get_total_verifications_initial() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    assert_eq!(client.get_total_verifications(), 0);
}

#[test]
fn test_fraud_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, FraudPreventionContract);
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.initialize(&admin);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    client.flag_suspicious(&admin, &publisher);
}
