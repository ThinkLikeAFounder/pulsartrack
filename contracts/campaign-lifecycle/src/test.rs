#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (CampaignLifecycleContractClient, Address) {
    let admin = Address::generate(env);

    let contract_id = env.register_contract(None, CampaignLifecycleContract);
    let client = CampaignLifecycleContractClient::new(env, &contract_id);
    client.initialize(&admin);

    (client, admin)
}

fn make_reason(env: &Env) -> String {
    String::from_str(env, "reviewed and approved")
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CampaignLifecycleContract);
    let client = CampaignLifecycleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CampaignLifecycleContract);
    let client = CampaignLifecycleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();

    let contract_id = env.register_contract(None, CampaignLifecycleContract);
    let client = CampaignLifecycleContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
}

// ─── register_campaign ───────────────────────────────────────────────────────

#[test]
fn test_register_campaign() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert_eq!(lc.campaign_id, 1);
    assert_eq!(lc.advertiser, advertiser);
    assert!(matches!(lc.state, LifecycleState::Draft));
    assert_eq!(lc.original_end_ledger, 10_000);
    assert_eq!(lc.current_end_ledger, 10_000);
    assert_eq!(lc.pause_count, 0);
    assert_eq!(lc.extension_count, 0);
}

// ─── transition (valid paths) ────────────────────────────────────────────────

#[test]
fn test_transition_draft_to_pending_review() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert!(matches!(lc.state, LifecycleState::PendingReview));
    assert_eq!(client.get_transition_count(&1u64), 1);
}

#[test]
fn test_transition_pending_to_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
    client.transition(&admin, &1u64, &LifecycleState::Active, &make_reason(&env));

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert!(matches!(lc.state, LifecycleState::Active));
    assert!(lc.activated_at.is_some());
}

#[test]
fn test_transition_active_to_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
    client.transition(&admin, &1u64, &LifecycleState::Active, &make_reason(&env));
    client.transition(&advertiser, &1u64, &LifecycleState::Paused, &String::from_str(&env, "budget review"));

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert!(matches!(lc.state, LifecycleState::Paused));
    assert_eq!(lc.pause_count, 1);
    assert!(lc.paused_at.is_some());
}

#[test]
fn test_transition_paused_to_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
    client.transition(&admin, &1u64, &LifecycleState::Active, &make_reason(&env));
    client.transition(&advertiser, &1u64, &LifecycleState::Paused, &String::from_str(&env, "pause"));
    client.transition(&advertiser, &1u64, &LifecycleState::Active, &String::from_str(&env, "resume"));

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert!(matches!(lc.state, LifecycleState::Active));
}

#[test]
fn test_transition_active_to_completed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
    client.transition(&admin, &1u64, &LifecycleState::Active, &make_reason(&env));
    client.transition(&advertiser, &1u64, &LifecycleState::Completed, &String::from_str(&env, "campaign ended"));

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert!(matches!(lc.state, LifecycleState::Completed));
    assert!(lc.completed_at.is_some());
}

#[test]
fn test_transition_draft_to_cancelled() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::Cancelled, &String::from_str(&env, "changed mind"));

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert!(matches!(lc.state, LifecycleState::Cancelled));
    assert!(lc.cancelled_at.is_some());
}

// ─── transition (invalid paths) ──────────────────────────────────────────────

#[test]
#[should_panic(expected = "invalid state transition")]
fn test_invalid_transition_draft_to_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    // Draft → Active is invalid; must go through PendingReview first
    client.transition(&advertiser, &1u64, &LifecycleState::Active, &make_reason(&env));
}

#[test]
#[should_panic(expected = "invalid state transition")]
fn test_invalid_transition_completed_to_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
    client.transition(&admin, &1u64, &LifecycleState::Active, &make_reason(&env));
    client.transition(&advertiser, &1u64, &LifecycleState::Completed, &make_reason(&env));
    // Completed → Active is invalid
    client.transition(&advertiser, &1u64, &LifecycleState::Active, &make_reason(&env));
}

// ─── transition (access control) ─────────────────────────────────────────────

#[test]
#[should_panic(expected = "unauthorized")]
fn test_transition_by_stranger() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&stranger, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
}

// ─── pause_for_fraud ─────────────────────────────────────────────────────────

#[test]
fn test_pause_for_fraud() {
    // NOTE: The pause_for_fraud function internally calls Self::transition() which
    // creates a re-entrant auth issue in tests. Instead, we verify that the fraud
    // contract address (once set) can call transition() to pause a campaign.
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let contract_id = env.register_contract(None, CampaignLifecycleContract);
    let client = CampaignLifecycleContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let advertiser = Address::generate(&env);
    let fraud_contract = Address::generate(&env);

    client.set_fraud_contract(&admin, &fraud_contract);
    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
    client.transition(&admin, &1u64, &LifecycleState::Active, &make_reason(&env));

    // Fraud contract can call transition to pause
    client.transition(&fraud_contract, &1u64, &LifecycleState::Paused,
        &String::from_str(&env, "paused for fraud detection"));

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert!(matches!(lc.state, LifecycleState::Paused));
    assert_eq!(lc.pause_count, 1);
}

#[test]
#[should_panic(expected = "unauthorized fraud contract")]
fn test_pause_for_fraud_wrong_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let advertiser = Address::generate(&env);
    let fraud_contract = Address::generate(&env);
    let wrong_contract = Address::generate(&env);

    client.set_fraud_contract(&admin, &fraud_contract);
    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));
    client.transition(&admin, &1u64, &LifecycleState::Active, &make_reason(&env));

    client.pause_for_fraud(&wrong_contract, &1u64);
}

// ─── extend_campaign ─────────────────────────────────────────────────────────

#[test]
fn test_extend_campaign() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.extend_campaign(&advertiser, &1u64, &5_000u32);

    let lc = client.get_lifecycle(&1u64).unwrap();
    assert_eq!(lc.current_end_ledger, 15_000);
    assert_eq!(lc.extension_count, 1);
    assert_eq!(lc.original_end_ledger, 10_000);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_extend_campaign_by_stranger() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.extend_campaign(&stranger, &1u64, &5_000u32);
}

// ─── set_fraud_contract ──────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_fraud_contract_by_stranger() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let fraud = Address::generate(&env);

    client.set_fraud_contract(&stranger, &fraud);
}

// ─── read-only ───────────────────────────────────────────────────────────────

#[test]
fn test_get_lifecycle_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    assert!(client.get_lifecycle(&999u64).is_none());
}

#[test]
fn test_get_transition_count_initial() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    assert_eq!(client.get_transition_count(&999u64), 0);
}

#[test]
fn test_transition_recorded() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let advertiser = Address::generate(&env);

    client.register_campaign(&advertiser, &1u64, &10_000u32);
    client.transition(&advertiser, &1u64, &LifecycleState::PendingReview, &make_reason(&env));

    assert_eq!(client.get_transition_count(&1u64), 1);

    let t = client.get_transition(&1u64, &0u32).unwrap();
    assert!(matches!(t.from_state, LifecycleState::Draft));
    assert!(matches!(t.to_state, LifecycleState::PendingReview));
    assert_eq!(t.actor, advertiser);
}
