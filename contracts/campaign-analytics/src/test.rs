#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup(env: &Env) -> (CampaignAnalyticsContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, CampaignAnalyticsContract);
    let c = CampaignAnalyticsContractClient::new(env, &id);
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
    let id = env.register_contract(None, CampaignAnalyticsContract);
    let c = CampaignAnalyticsContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let o = Address::generate(&env);
    c.initialize(&a, &o);
    c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, CampaignAnalyticsContract);
    let c = CampaignAnalyticsContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_record_snapshot() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.record_snapshot(&oracle, &1u64, &1000u64, &50u64, &10u64, &5000i128, &800u64);
    let snap = c.get_snapshot(&1u64, &0u32).unwrap();
    assert_eq!(snap.impressions, 1000);
    assert_eq!(snap.clicks, 50);
}

#[test]
fn test_update_retention() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.update_retention(&oracle, &1u64, &80u32, &60u32, &40u32, &120u64, &25u32);
    let ret = c.get_retention(&1u64).unwrap();
    assert_eq!(ret.day_1_retention, 80);
    assert_eq!(ret.day_7_retention, 60);
    assert_eq!(ret.day_30_retention, 40);
}

#[test]
fn test_get_snapshot_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_snapshot(&999u64, &0u32).is_none());
}

#[test]
fn test_get_retention_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_retention(&999u64).is_none());
}

#[test]
fn test_snapshot_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    c.record_snapshot(&oracle, &1u64, &1000u64, &50u64, &10u64, &5000i128, &800u64);
    assert_eq!(c.get_snapshot_count(&1u64), 1);
}

#[test]
fn test_ring_buffer_overwrites_oldest() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);

    let max = c.get_max_snapshots();

    // Fill the ring buffer completely
    for i in 0..max {
        c.record_snapshot(
            &oracle,
            &1u64,
            &(i as u64), // impressions = index for identification
            &0u64,
            &0u64,
            &0i128,
            &0u64,
        );
    }

    // Verify the first snapshot is at index 0 with impressions = 0
    let first = c.get_snapshot(&1u64, &0u32).unwrap();
    assert_eq!(first.impressions, 0);

    // Record one more — should overwrite index 0
    c.record_snapshot(&oracle, &1u64, &9999u64, &0u64, &0u64, &0i128, &0u64);

    // Index 0 now holds the newest snapshot
    let overwritten = c.get_snapshot(&1u64, &0u32).unwrap();
    assert_eq!(overwritten.impressions, 9999);

    // Total count keeps incrementing beyond MAX_SNAPSHOTS
    assert_eq!(c.get_snapshot_count(&1u64), max + 1);

    // Stored count is capped at MAX_SNAPSHOTS
    assert_eq!(c.get_stored_snapshot_count(&1u64), max);
}

#[test]
fn test_ring_buffer_multiple_wraps() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);

    let max = c.get_max_snapshots();

    // Write 2.5x the max to wrap multiple times
    let total = max * 2 + max / 2;
    for i in 0..total {
        c.record_snapshot(&oracle, &1u64, &(i as u64), &0u64, &0u64, &0i128, &0u64);
    }

    assert_eq!(c.get_snapshot_count(&1u64), total);
    assert_eq!(c.get_stored_snapshot_count(&1u64), max);

    // The most recently written index is (total - 1) % max
    let last_index = (total - 1) % max;
    let last = c.get_snapshot(&1u64, &last_index).unwrap();
    assert_eq!(last.impressions, (total - 1) as u64);
}

#[test]
fn test_stored_snapshot_count_below_max() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);

    // Record fewer than MAX_SNAPSHOTS
    for i in 0..5u32 {
        c.record_snapshot(&oracle, &1u64, &(i as u64), &0u64, &0u64, &0i128, &0u64);
    }

    assert_eq!(c.get_snapshot_count(&1u64), 5);
    assert_eq!(c.get_stored_snapshot_count(&1u64), 5);
}

#[test]
fn test_ring_buffer_independent_campaigns() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);

    // Record snapshots for two different campaigns
    for i in 0..3u32 {
        c.record_snapshot(
            &oracle,
            &1u64,
            &(i as u64 * 100),
            &0u64,
            &0u64,
            &0i128,
            &0u64,
        );
        c.record_snapshot(
            &oracle,
            &2u64,
            &(i as u64 * 200),
            &0u64,
            &0u64,
            &0i128,
            &0u64,
        );
    }

    // Each campaign has its own independent count
    assert_eq!(c.get_snapshot_count(&1u64), 3);
    assert_eq!(c.get_snapshot_count(&2u64), 3);

    // Verify data is independent
    let snap1 = c.get_snapshot(&1u64, &0u32).unwrap();
    let snap2 = c.get_snapshot(&2u64, &0u32).unwrap();
    assert_eq!(snap1.impressions, 0);
    assert_eq!(snap2.impressions, 0);

    let snap1 = c.get_snapshot(&1u64, &2u32).unwrap();
    let snap2 = c.get_snapshot(&2u64, &2u32).unwrap();
    assert_eq!(snap1.impressions, 200);
    assert_eq!(snap2.impressions, 400);
}

#[test]
fn test_get_max_snapshots() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert_eq!(c.get_max_snapshots(), MAX_SNAPSHOTS);
}
