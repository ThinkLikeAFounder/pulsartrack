#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String,
};

fn setup(env: &Env) -> (MilestoneTrackerContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register(MilestoneTrackerContract, ());
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
    let duration_secs = 86_400u64; // 1 day
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &duration_secs,
    );
    assert_eq!(id, 1);
    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Pending));
    assert_eq!(m.target_value, 1000);
    assert_eq!(m.deadline, env.ledger().timestamp() + duration_secs);
    assert_eq!(c.get_campaign_milestone_count(&1u64), 1);
}

#[test]
fn test_update_progress() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64, // 1 day
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
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64, // 1 day
    );
    c.update_progress(&oracle, &id, &1000u64);
    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Achieved));
    assert!(m.achieved_at.is_some());
}

#[test]
fn test_dispute_pending_milestone() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64, // 1 day
    );
    c.dispute_milestone(&advertiser, &id);
    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Disputed));
}

#[test]
fn test_dispute_in_progress_milestone() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64,
    );

    c.update_progress(&oracle, &id, &500u64);
    c.dispute_milestone(&advertiser, &id);

    let m = c.get_milestone(&id).unwrap();
    assert!(matches!(m.status, MilestoneStatus::Disputed));
}

#[test]
#[should_panic(expected = "milestone cannot be disputed in its current status")]
fn test_dispute_achieved_milestone_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64,
    );

    c.update_progress(&oracle, &id, &1000u64);
    c.dispute_milestone(&advertiser, &id);
}

#[test]
#[should_panic(expected = "milestone cannot be disputed in its current status")]
fn test_dispute_missed_milestone_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &MIN_DURATION_SECS,
    );

    env.ledger().with_mut(|li| {
        li.timestamp += MIN_DURATION_SECS + 1;
    });
    c.update_progress(&oracle, &id, &500u64);
    c.dispute_milestone(&advertiser, &id);
}

#[test]
#[should_panic(expected = "milestone cannot be disputed in its current status")]
fn test_dispute_disputed_milestone_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64,
    );

    c.dispute_milestone(&advertiser, &id);
    c.dispute_milestone(&advertiser, &id);
}

#[test]
#[should_panic(expected = "unauthorized: only advertiser or admin can dispute")]
fn test_dispute_milestone_rejects_unauthorized_caller() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let attacker = Address::generate(&env);

    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64,
    );

    c.dispute_milestone(&attacker, &id);
}

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _) = setup(&env);
    let advertiser = Address::generate(&env);
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &86_400u64, // 1 day
    );
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

    // Create with minimum valid duration (1 hour)
    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &MIN_DURATION_SECS,
    );

    // Advance time past deadline
    env.ledger().with_mut(|li| {
        li.timestamp += MIN_DURATION_SECS + 1;
    });

    // Update progress but don't reach target
    c.update_progress(&oracle, &id, &500u64);
    let m = c.get_milestone(&id).unwrap();

    assert!(matches!(m.status, MilestoneStatus::Missed));
    assert_eq!(m.current_value, 500);
}

#[test]
fn test_deadline_is_computed_from_now_plus_duration() {
    let env = Env::default();
    env.mock_all_auths();

    env.ledger().with_mut(|li| {
        li.timestamp = 1_000_000;
    });

    let (c, _, oracle) = setup(&env);
    let advertiser = Address::generate(&env);
    let duration_secs = 86_400u64; // 1 day
    let now = env.ledger().timestamp();

    let id = c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &duration_secs,
    );

    let m = c.get_milestone(&id).unwrap();

    // deadline must equal now + duration_secs, not a raw ledger sequence
    assert_eq!(m.deadline, now + duration_secs);
    assert_eq!(m.created_at, now);

    // Achieve the milestone and verify all time fields are in the same domain
    c.update_progress(&oracle, &id, &1000u64);
    let m = c.get_milestone(&id).unwrap();
    assert!(m.achieved_at.unwrap() >= m.created_at);
    assert!(m.achieved_at.unwrap() <= m.deadline);
}

#[test]
#[should_panic(expected = "duration too short")]
fn test_create_milestone_rejects_zero_duration() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &0u64, // zero duration — must panic
    );
}

#[test]
#[should_panic(expected = "duration too short")]
fn test_create_milestone_rejects_sub_minimum_duration() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);
    c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &50_000i128,
        &(MIN_DURATION_SECS - 1), // one second below minimum
    );
}

#[test]
#[should_panic(expected = "target_value must be greater than zero")]
fn test_create_milestone_rejects_zero_target_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);

    c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &0u64,
        &50_000i128,
        &MIN_DURATION_SECS,
    );
}

#[test]
#[should_panic(expected = "reward_amount must be positive")]
fn test_create_milestone_rejects_zero_reward_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);

    c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &0i128,
        &MIN_DURATION_SECS,
    );
}

#[test]
#[should_panic(expected = "reward_amount must be positive")]
fn test_create_milestone_rejects_negative_reward_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let advertiser = Address::generate(&env);

    c.create_milestone(
        &advertiser,
        &1u64,
        &s(&env, "1000 views"),
        &s(&env, "views"),
        &1000u64,
        &-1i128,
        &MIN_DURATION_SECS,
    );
}
