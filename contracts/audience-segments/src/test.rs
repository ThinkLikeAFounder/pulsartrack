#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (AudienceSegmentsContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, AudienceSegmentsContract);
    let c = AudienceSegmentsContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}
fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, AudienceSegmentsContract);
    let c = AudienceSegmentsContractClient::new(&env, &id);
    let a = Address::generate(&env); c.initialize(&a); c.initialize(&a);
}

#[test]
fn test_create_segment() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let creator = Address::generate(&env);
    let sid = c.create_segment(&creator, &s(&env, "Tech Enthusiasts"), &s(&env, "Users interested in tech"), &s(&env, "QmCriteria"), &true);
    assert_eq!(sid, 1);
    assert_eq!(c.get_segment_count(), 1);
    let seg = c.get_segment(&sid).unwrap();
    assert!(seg.is_public);
}

#[test]
fn test_add_member() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    let sid = c.create_segment(&creator, &s(&env, "Segment"), &s(&env, "Desc"), &s(&env, "QmC"), &true);
    let member = Address::generate(&env);
    c.add_member(&admin, &sid, &member, &75u32);
    assert!(c.is_member(&sid, &member));
    assert_eq!(c.get_member_count(&sid), 1);
    let m = c.get_membership(&sid, &member).unwrap();
    assert_eq!(m.score, 75);
}

#[test]
fn test_remove_member() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let creator = Address::generate(&env);
    let sid = c.create_segment(&creator, &s(&env, "Segment"), &s(&env, "Desc"), &s(&env, "QmC"), &true);
    let member = Address::generate(&env);
    c.add_member(&admin, &sid, &member, &75u32);
    c.remove_member(&admin, &sid, &member);
    assert!(!c.is_member(&sid, &member));
    assert_eq!(c.get_member_count(&sid), 0);
}

#[test]
fn test_is_member_false() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.is_member(&1u64, &Address::generate(&env)));
}

#[test]
fn test_get_segment_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_segment(&999u64).is_none());
}

#[test]
fn test_get_segment_count_initial() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.get_segment_count(), 0);
}
