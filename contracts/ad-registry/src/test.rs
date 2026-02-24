#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (AdRegistryContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, AdRegistryContract);
    let c = AdRegistryContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

fn s(env: &Env, v: &str) -> String {
    String::from_str(env, v)
}

fn register(c: &AdRegistryContractClient, env: &Env) -> u64 {
    c.register_content(
        &1u64,
        &s(env, "QmHash"),
        &ContentFormat::Image,
        &500u64,
        &s(env, "Title"),
        &s(env, "Desc"),
        &s(env, "CTA"),
        &s(env, "https://example.com"),
    )
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, AdRegistryContract);
    let c = AdRegistryContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, AdRegistryContract);
    let c = AdRegistryContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, AdRegistryContract);
    let c = AdRegistryContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_register_content() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let cid = register(&c, &env);
    assert_eq!(cid, 1);
    assert_eq!(c.get_nonce(), 1);
    let content = c.get_content(&cid).unwrap();
    assert_eq!(content.campaign_id, 1);
    assert!(matches!(content.status, ContentStatus::Pending));
    assert_eq!(content.flags_count, 0);
}

#[test]
fn test_register_multiple() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let id1 = register(&c, &env);
    let id2 = register(&c, &env);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(c.get_nonce(), 2);
}

#[test]
#[should_panic(expected = "invalid content size")]
fn test_register_content_too_small() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    c.register_content(
        &1u64,
        &s(&env, "QmHash"),
        &ContentFormat::Image,
        &10u64,
        &s(&env, "T"),
        &s(&env, "D"),
        &s(&env, "C"),
        &s(&env, "U"),
    );
}

#[test]
fn test_update_status() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let cid = register(&c, &env);
    c.update_status(&admin, &cid, &ContentStatus::Approved);
    assert!(c.is_approved(&cid));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_update_status_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let cid = register(&c, &env);
    c.update_status(&Address::generate(&env), &cid, &ContentStatus::Approved);
}

#[test]
fn test_flag_content() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let cid = register(&c, &env);
    let reporter = Address::generate(&env);
    c.flag_content(&reporter, &cid, &s(&env, "spam"));
    let content = c.get_content(&cid).unwrap();
    assert_eq!(content.flags_count, 1);
}

#[test]
fn test_flag_threshold_suspends() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.set_flag_threshold(&admin, &2u32);
    let cid = register(&c, &env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    c.flag_content(&r1, &cid, &s(&env, "spam"));
    c.flag_content(&r2, &cid, &s(&env, "fraud"));
    let content = c.get_content(&cid).unwrap();
    assert!(matches!(content.status, ContentStatus::Suspended));
}

#[test]
fn test_track_view() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let cid = register(&c, &env);
    c.update_status(&admin, &cid, &ContentStatus::Approved);
    c.track_view(&cid);
    let perf = c.get_performance(&cid).unwrap();
    assert_eq!(perf.total_views, 1);
}

#[test]
#[should_panic(expected = "content not approved")]
fn test_track_view_unapproved() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let cid = register(&c, &env);
    c.track_view(&cid);
}

#[test]
fn test_track_click() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let cid = register(&c, &env);
    c.update_status(&admin, &cid, &ContentStatus::Approved);
    c.track_view(&cid);
    c.track_click(&cid);
    let perf = c.get_performance(&cid).unwrap();
    assert_eq!(perf.total_clicks, 1);
    assert_eq!(perf.click_through_rate, 10_000); // 1/1 * 10000
}

#[test]
fn test_get_content_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_content(&999u64).is_none());
}

#[test]
fn test_get_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let cid = register(&c, &env);
    let meta = c.get_metadata(&cid).unwrap();
    assert_eq!(meta.title, s(&env, "Title"));
}

#[test]
fn test_set_flag_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    c.set_flag_threshold(&admin, &10u32);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_flag_threshold_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    c.set_flag_threshold(&Address::generate(&env), &10u32);
}
#[test]
fn test_admin_transfer_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    c.propose_admin(&admin, &new_admin);
    c.accept_admin(&new_admin);

    // Verify new admin can perform admin actions
    c.set_flag_threshold(&new_admin, &99u32);
}

#[test]
#[should_panic]
fn test_propose_admin_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let stranger = Address::generate(&env);
    let new_admin = Address::generate(&env);

    c.propose_admin(&stranger, &new_admin);
}

#[test]
#[should_panic]
fn test_accept_admin_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let new_admin = Address::generate(&env);
    let stranger = Address::generate(&env);

    c.propose_admin(&admin, &new_admin);
    c.accept_admin(&stranger);
}
