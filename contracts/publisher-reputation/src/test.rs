#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup(env: &Env) -> (PublisherReputationContractClient, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register_contract(None, PublisherReputationContract);
    let c = PublisherReputationContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin, oracle)
}

#[test]
fn test_initialize() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PublisherReputationContract);
    let c = PublisherReputationContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, PublisherReputationContract);
    let c = PublisherReputationContractClient::new(&env, &id);
    let a = Address::generate(&env); let o = Address::generate(&env);
    c.initialize(&a, &o); c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register_contract(None, PublisherReputationContract);
    let c = PublisherReputationContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_init_publisher() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.score, 500);
    assert_eq!(rep.total_reviews, 0);
    assert_eq!(rep.uptime_score, 100);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_init_publisher_duplicate() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.init_publisher(&pub1);
}

#[test]
fn test_submit_positive_review() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let adv = Address::generate(&env);
    c.init_publisher(&pub1);
    c.submit_review(&adv, &pub1, &1u64, &true, &5u32);
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.total_reviews, 1);
    assert_eq!(rep.positive_reviews, 1);
    assert_eq!(rep.score, 510); // 500 + 5*2
    assert_eq!(c.get_review_count(&pub1), 1);
    let review = c.get_review(&pub1, &0u64).unwrap();
    assert!(review.positive);
    assert_eq!(review.rating, 5);
}

#[test]
fn test_submit_negative_review() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    let adv = Address::generate(&env);
    c.init_publisher(&pub1);
    c.submit_review(&adv, &pub1, &1u64, &false, &5u32);
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.negative_reviews, 1);
    assert_eq!(rep.score, 485); // 500 - 5*3
}

#[test]
#[should_panic(expected = "invalid rating")]
fn test_submit_review_invalid_rating_zero() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.submit_review(&Address::generate(&env), &pub1, &1u64, &true, &0u32);
}

#[test]
#[should_panic(expected = "invalid rating")]
fn test_submit_review_invalid_rating_high() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.submit_review(&Address::generate(&env), &pub1, &1u64, &true, &6u32);
}

#[test]
fn test_slash_publisher() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.slash_publisher(&oracle, &pub1, &100u32);
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.score, 400); // 500 - 100
    assert_eq!(rep.slashes, 1);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_slash_publisher_unauthorized() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.slash_publisher(&Address::generate(&env), &pub1, &100u32);
}

#[test]
fn test_slash_floor_at_zero() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.slash_publisher(&oracle, &pub1, &600u32); // 500 - 600 would be negative
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.score, 0);
}

#[test]
fn test_update_uptime() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.update_uptime(&oracle, &pub1, &90u32);
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.uptime_score, 90);
    // Score should increase by uptime/5 = 18 → 500 + 18 = 518
    assert_eq!(rep.score, 518);
}

#[test]
#[should_panic(expected = "invalid uptime")]
fn test_update_uptime_too_high() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.update_uptime(&oracle, &pub1, &101u32);
}

#[test]
fn test_get_reputation_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_reputation(&Address::generate(&env)).is_none());
}

#[test]
fn test_get_review_nonexistent() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_review(&Address::generate(&env), &0u64).is_none());
}

#[test]
fn test_get_review_count_initial() {
    let env = Env::default(); env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert_eq!(c.get_review_count(&Address::generate(&env)), 0);
}
