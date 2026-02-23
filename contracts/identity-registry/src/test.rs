#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (IdentityRegistryContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, IdentityRegistryContract);
    let c = IdentityRegistryContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

fn s(env: &Env, v: &str) -> String { String::from_str(env, v) }

#[test]
fn test_initialize() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, IdentityRegistryContract);
    let c = IdentityRegistryContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
    assert_eq!(c.get_identity_count(), 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, IdentityRegistryContract);
    let c = IdentityRegistryContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a); c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, IdentityRegistryContract);
    let c = IdentityRegistryContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_register() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let account = Address::generate(&env);
    c.register(&account, &IdentityType::Advertiser, &s(&env, "Alice"), &s(&env, "QmMeta"));
    assert_eq!(c.get_identity_count(), 1);
    let id = c.get_identity(&account).unwrap();
    assert_eq!(id.display_name, s(&env, "Alice"));
    assert!(matches!(id.status, IdentityStatus::Pending));
    assert!(id.verified_at.is_none());
}

#[test]
#[should_panic(expected = "already registered")]
fn test_register_duplicate() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let account = Address::generate(&env);
    c.register(&account, &IdentityType::Publisher, &s(&env, "Bob"), &s(&env, "QmMeta"));
    c.register(&account, &IdentityType::Publisher, &s(&env, "Bob2"), &s(&env, "QmMeta"));
}

#[test]
#[should_panic(expected = "name taken")]
fn test_register_duplicate_name() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    c.register(&a1, &IdentityType::Advertiser, &s(&env, "Alice"), &s(&env, "QmMeta"));
    c.register(&a2, &IdentityType::Publisher, &s(&env, "Alice"), &s(&env, "QmMeta2"));
}

#[test]
fn test_verify_identity() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let account = Address::generate(&env);
    c.register(&account, &IdentityType::Advertiser, &s(&env, "Alice"), &s(&env, "QmMeta"));
    c.verify_identity(&admin, &account, &s(&env, "CredHash"));
    let id = c.get_identity(&account).unwrap();
    assert!(matches!(id.status, IdentityStatus::Verified));
    assert!(id.verified_at.is_some());
    assert!(c.is_verified(&account));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_verify_identity_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let account = Address::generate(&env);
    c.register(&account, &IdentityType::Advertiser, &s(&env, "Alice"), &s(&env, "QmMeta"));
    c.verify_identity(&Address::generate(&env), &account, &s(&env, "CredHash"));
}

#[test]
fn test_update_metadata() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let account = Address::generate(&env);
    c.register(&account, &IdentityType::Advertiser, &s(&env, "Alice"), &s(&env, "QmOld"));
    c.update_metadata(&account, &s(&env, "QmNew"));
    let id = c.get_identity(&account).unwrap();
    assert_eq!(id.metadata_hash, s(&env, "QmNew"));
}

#[test]
fn test_suspend_identity() {
    let env = Env::default(); env.mock_all_auths();
    let (c, admin) = setup(&env);
    let account = Address::generate(&env);
    c.register(&account, &IdentityType::Advertiser, &s(&env, "Alice"), &s(&env, "QmMeta"));
    c.suspend_identity(&admin, &account);
    let id = c.get_identity(&account).unwrap();
    assert!(matches!(id.status, IdentityStatus::Suspended));
    assert!(!c.is_verified(&account));
}

#[test]
fn test_get_by_name() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let account = Address::generate(&env);
    c.register(&account, &IdentityType::Advertiser, &s(&env, "Alice"), &s(&env, "QmMeta"));
    let owner = c.get_by_name(&s(&env, "Alice")).unwrap();
    assert_eq!(owner, account);
}

#[test]
fn test_get_identity_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_identity(&Address::generate(&env)).is_none());
}

#[test]
fn test_is_verified_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(!c.is_verified(&Address::generate(&env)));
}
