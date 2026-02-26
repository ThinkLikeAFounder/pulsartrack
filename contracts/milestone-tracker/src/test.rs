#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

fn setup(env: &Env) -> (MilestoneTrackerContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, MilestoneTrackerContract);
    let c = MilestoneTrackerContractClient::new(env, &id);
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
    let (c, admin, oracle) = setup(&env);
    c.initialize(&admin, &oracle);
}

#[test]
fn test_create_milestone() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 86_400; // 1 day from now
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &deadline,
    );
    assert_eq!(id, 1);
    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Pending));
    assert_eq!(m.target_value, 1000);
    assert_eq!(m.deadline, deadline);
    assert_eq!(c.get_campaign_milestone_count(&1u64), 1);
}

#[test]
fn test_update_progress() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 86_400; // 1 day from now
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &deadline,
    );
    c.update_progress(&oracle, &id, &500u64);
    let m = c.get_milestone(&id).unwrap();
    assert_eq!(m.current_value, 500);
    assert!(matches!(m.status, MilestoneStatus::InProgress));
}

#[test]
fn test_update_progress_achieves() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 86_400; // 1 day from now
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &deadline,
    );
    c.update_progress(&oracle, &id, &1000u64);
    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Achieved));
    assert!(m.achieved_at.is_some());
}

#[test]
fn test_dispute_milestone() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 86_400; // 1 day from now
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &deadline,
    );
    c.update_progress(&oracle, &id, &1000u64);
    c.dispute_milestone(&advertiser, &id);
    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Disputed));
}

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 86_400; // 1 day from now
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &deadline,
    );
    c.update_progress(&oracle, &id, &1000u64);
    c.dispute_milestone(&advertiser, &id);
    c.resolve_dispute(&admin, &id, &true);
    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Achieved));
}

#[test]
fn test_get_milestone_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_milestone(&999u64).is_none());
}

#[test]
fn test_milestone_missed_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    
    // Set deadline to current timestamp (already expired)
    let deadline = env.ledger().timestamp();
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &deadline,
    );
    
    // Advance time by 1 second
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 1;
    });
    
    // Update progress but don't reach target
    c.update_progress(&oracle, &id, &500u64);
    let m = c.get_milestone(&id).unwrap();
    
    // Should be marked as Missed because deadline passed
    assert!(matches!(m.status, MilestoneStatus::Missed));
    assert_eq!(m.current_value, 500);
}

#[test]
fn test_time_domain_consistency() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Set a non-zero timestamp
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000_000;
    });
    
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    
    let current_time = env.ledger().timestamp();
    let deadline = current_time + 86_400; // 1 day from now
    
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &deadline,
    );
    
    // Achieve the milestone
    c.update_progress(&oracle, &id, &1000u64);
    let m = c.get_milestone(&id).unwrap();
    
    // All time fields should be in the same domain (Unix timestamps)
    assert!(m.created_at > 0);
    assert_eq!(m.deadline, deadline);
    assert!(m.achieved_at.is_some());
    
    let achieved_time = m.achieved_at.unwrap();
    
    // achieved_at should be >= created_at
    assert!(achieved_time >= m.created_at);
    
    // achieved_at should be <= deadline (achieved before deadline)
    assert!(achieved_time <= m.deadline);
}
