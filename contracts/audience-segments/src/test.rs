#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (AudienceSegmentsContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, AudienceSegmentsContract);
    let c = AudienceSegmentsContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
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
    let id = env.register_contract(None, AudienceSegmentsContract);
    let c = AudienceSegmentsContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
fn test_create_segment() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let creator = Address::generate(&env);
    let sid = c.create_segment(
        &creator,
        &s(&env, "Tech Enthusiasts"),
        &s(&env, "Users interested in tech"),
        &s(&env, "QmCriteria"),
        &true,
    );
    assert_eq!(sid, 1);
    assert_eq!(c.get_segment_count(), 1);
    let seg = c.get_segment(&sid).unwrap();
    assert!(seg.is_public);
}

#[test]
fn test_add_member() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    let sid = c.create_segment(
        &creator,
        &s(&env, "Segment"),
        &s(&env, "Desc"),
        &s(&env, "QmC"),
        &true,
    );
    let member = Address::generate(&env);
    c.add_member(&admin, &sid, &member, &75u32);
    assert!(c.is_member(&sid, &member));
    assert_eq!(c.get_member_count(&sid), 1);
    let m = c.get_membership(&sid, &member).unwrap();
    assert_eq!(m.score, 75);
}

#[test]
fn test_remove_member() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    let sid = c.create_segment(
        &creator,
        &s(&env, "Segment"),
        &s(&env, "Desc"),
        &s(&env, "QmC"),
        &true,
    );
    let member = Address::generate(&env);
    c.add_member(&admin, &sid, &member, &75u32);
    c.remove_member(&admin, &sid, &member);
    assert!(!c.is_member(&sid, &member));
    assert_eq!(c.get_member_count(&sid), 0);
}

#[test]
fn test_is_member_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.is_member(&1u64, &Address::generate(&env)));
}

#[test]
fn test_get_segment_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_segment(&999u64).is_none());
}

#[test]
fn test_get_segment_count_initial() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.get_segment_count(), 0);
}

#[test]
fn test_segment_member_count_synced_on_add() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    
    let sid = c.create_segment(
        &creator,
        &s(&env, "Test Segment"),
        &s(&env, "Description"),
        &s(&env, "QmCriteria"),
        &true,
    );
    
    // Initially member_count should be 0
    let seg = c.get_segment(&sid).unwrap();
    assert_eq!(seg.member_count, 0);
    
    // Add first member
    let member1 = Address::generate(&env);
    c.add_member(&admin, &sid, &member1, &80u32);
    
    // Check that segment.member_count is updated
    let seg = c.get_segment(&sid).unwrap();
    assert_eq!(seg.member_count, 1);
    assert_eq!(c.get_member_count(&sid), 1);
    
    // Add second member
    let member2 = Address::generate(&env);
    c.add_member(&admin, &sid, &member2, &90u32);
    
    // Check that segment.member_count is updated again
    let seg = c.get_segment(&sid).unwrap();
    assert_eq!(seg.member_count, 2);
    assert_eq!(c.get_member_count(&sid), 2);
}

#[test]
fn test_segment_member_count_synced_on_remove() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    
    let sid = c.create_segment(
        &creator,
        &s(&env, "Test Segment"),
        &s(&env, "Description"),
        &s(&env, "QmCriteria"),
        &true,
    );
    
    // Add three members
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);
    
    c.add_member(&admin, &sid, &member1, &80u32);
    c.add_member(&admin, &sid, &member2, &85u32);
    c.add_member(&admin, &sid, &member3, &90u32);
    
    // Verify count is 3
    let seg = c.get_segment(&sid).unwrap();
    assert_eq!(seg.member_count, 3);
    
    // Remove one member
    c.remove_member(&admin, &sid, &member2);
    
    // Check that segment.member_count is updated
    let seg = c.get_segment(&sid).unwrap();
    assert_eq!(seg.member_count, 2);
    assert_eq!(c.get_member_count(&sid), 2);
    
    // Remove another member
    c.remove_member(&admin, &sid, &member1);
    
    // Check that segment.member_count is updated again
    let seg = c.get_segment(&sid).unwrap();
    assert_eq!(seg.member_count, 1);
    assert_eq!(c.get_member_count(&sid), 1);
    
    // Remove last member
    c.remove_member(&admin, &sid, &member3);
    
    // Check that segment.member_count is back to 0
    let seg = c.get_segment(&sid).unwrap();
    assert_eq!(seg.member_count, 0);
    assert_eq!(c.get_member_count(&sid), 0);
}

#[test]
fn test_segment_last_updated_on_member_changes() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    
    let sid = c.create_segment(
        &creator,
        &s(&env, "Test Segment"),
        &s(&env, "Description"),
        &s(&env, "QmCriteria"),
        &true,
    );
    
    let seg_initial = c.get_segment(&sid).unwrap();
    let initial_timestamp = seg_initial.last_updated;
    
    // Add a member
    let member = Address::generate(&env);
    c.add_member(&admin, &sid, &member, &80u32);
    
    // Check that last_updated changed
    let seg_after_add = c.get_segment(&sid).unwrap();
    assert!(seg_after_add.last_updated >= initial_timestamp);
    
    let timestamp_after_add = seg_after_add.last_updated;
    
    // Remove the member
    c.remove_member(&admin, &sid, &member);
    
    // Check that last_updated changed again
    let seg_after_remove = c.get_segment(&sid).unwrap();
    assert!(seg_after_remove.last_updated >= timestamp_after_add);
}

#[test]
fn test_member_count_consistency() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    
    let sid = c.create_segment(
        &creator,
        &s(&env, "Test Segment"),
        &s(&env, "Description"),
        &s(&env, "QmCriteria"),
        &true,
    );
    
    // Add multiple members and verify consistency at each step
    for i in 0..5 {
        let member = Address::generate(&env);
        c.add_member(&admin, &sid, &member, &(70 + i * 5));
        
        let seg = c.get_segment(&sid).unwrap();
        let expected_count = (i + 1) as u64;
        
        // Both methods should return the same count
        assert_eq!(seg.member_count, expected_count);
        assert_eq!(c.get_member_count(&sid), expected_count);
    }
}
