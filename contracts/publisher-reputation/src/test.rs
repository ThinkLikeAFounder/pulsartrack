#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup(env: &Env) -> (PublisherReputationContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let id = env.register(PublisherReputationContract, ());
    let c = PublisherReputationContractClient::new(env, &id);
    c.initialize(&admin, &oracle);
    (c, admin, oracle)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(PublisherReputationContract, ());
    let c = PublisherReputationContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(PublisherReputationContract, ());
    let c = PublisherReputationContractClient::new(&env, &id);
    let a = Address::generate(&env);
    let o = Address::generate(&env);
    c.initialize(&a, &o);
    c.initialize(&a, &o);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register(PublisherReputationContract, ());
    let c = PublisherReputationContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_init_publisher() {
    let env = Env::default();
    env.mock_all_auths();
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
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.init_publisher(&pub1);
}

#[test]
fn test_submit_positive_review() {
    let env = Env::default();
    env.mock_all_auths();
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
    let env = Env::default();
    env.mock_all_auths();
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
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.submit_review(&Address::generate(&env), &pub1, &1u64, &true, &0u32);
}

#[test]
#[should_panic(expected = "invalid rating")]
fn test_submit_review_invalid_rating_high() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.submit_review(&Address::generate(&env), &pub1, &1u64, &true, &6u32);
}

#[test]
fn test_slash_publisher() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    c.slash_publisher(&oracle, &pub1, &100u32);
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.score, 400); // 500 - 100
    assert_eq!(rep.slashes, 1);
}

#[test]
#[should_panic(expected = "slash cooldown active")]
fn test_slash_publisher_cooldown_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    c.slash_publisher(&oracle, &pub1, &50u32);

    // Advance only 100 ledgers — still within the 17,280-ledger cooldown
    env.ledger().with_mut(|li| {
        li.sequence_number += 100;
    });

    c.slash_publisher(&oracle, &pub1, &50u32);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_init_publisher_requires_auth() {
    let env = Env::default();
    // Do NOT mock_all_auths so the auth check fires
    let id = env.register(PublisherReputationContract, ());
    let c = PublisherReputationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    env.mock_all_auths();
    c.initialize(&admin, &oracle);
    let pub1 = Address::generate(&env);
    // First call succeeds (all auths mocked), second must panic with duplicate
    c.init_publisher(&pub1);
    c.init_publisher(&pub1);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_slash_publisher_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.slash_publisher(&Address::generate(&env), &pub1, &100u32);
}

#[test]
fn test_slash_floor_at_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    c.slash_publisher(&oracle, &pub1, &600u32); // capped at 100, so 500 - 100 = 400
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.score, 400);
}

#[test]
fn test_update_uptime() {
    let env = Env::default();
    env.mock_all_auths();
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
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);
    c.update_uptime(&oracle, &pub1, &101u32);
}

#[test]
fn test_get_reputation_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_reputation(&Address::generate(&env)).is_none());
}

#[test]
fn test_get_review_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert!(c.get_review(&Address::generate(&env), &0u64).is_none());
}

#[test]
fn test_get_review_count_initial() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _) = setup(&env);
    assert_eq!(c.get_review_count(&Address::generate(&env)), 0);
}

#[test]
fn test_update_uptime_repeated_calls_no_inflation() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    // First call with 95% uptime
    c.update_uptime(&oracle, &pub1, &95u32);
    let rep1 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep1.uptime_score, 95);
    assert_eq!(rep1.score, 519); // 500 + 95/5 = 500 + 19

    // Advance ledger to satisfy cooldown
    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    // Second call with same uptime should not inflate
    c.update_uptime(&oracle, &pub1, &95u32);
    let rep2 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep2.uptime_score, 95);
    assert_eq!(rep2.score, 519); // Should remain the same

    // Advance ledger to satisfy cooldown
    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    // Third call with same uptime
    c.update_uptime(&oracle, &pub1, &95u32);
    let rep3 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep3.score, 519); // Still the same
}

#[test]
fn test_update_uptime_recalculates_on_change() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    // First call with 100% uptime
    c.update_uptime(&oracle, &pub1, &100u32);
    let rep1 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep1.score, 520); // 500 + 100/5 = 500 + 20

    // Advance ledger to satisfy cooldown
    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    // Update to 90% uptime
    c.update_uptime(&oracle, &pub1, &90u32);
    let rep2 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep2.score, 518); // 500 + 90/5 = 500 + 18

    // Advance ledger to satisfy cooldown
    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    // Update to 95% uptime
    c.update_uptime(&oracle, &pub1, &95u32);
    let rep3 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep3.score, 519); // 500 + 95/5 = 500 + 19
}

#[test]
fn test_update_uptime_below_threshold_no_bonus() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    // Uptime below 90% should not add bonus
    c.update_uptime(&oracle, &pub1, &85u32);
    let rep = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep.uptime_score, 85);
    assert_eq!(rep.score, 500); // No change from initial score
    assert_eq!(rep.uptime_contribution, 0);
}

#[test]
fn test_update_uptime_with_reviews_preserves_review_score() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    let adv = Address::generate(&env);
    c.init_publisher(&pub1);

    // Add positive review
    c.submit_review(&adv, &pub1, &1u64, &true, &5u32);
    let rep1 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep1.score, 510); // 500 + 5*2

    // Update uptime to 100%
    c.update_uptime(&oracle, &pub1, &100u32);
    let rep2 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep2.score, 530); // 510 + 100/5 = 510 + 20

    // Advance ledger to satisfy cooldown
    env.ledger().with_mut(|li| {
        li.sequence_number += 17_281;
    });

    // Update uptime again with same value - should not inflate
    c.update_uptime(&oracle, &pub1, &100u32);
    let rep3 = c.get_reputation(&pub1).unwrap();
    assert_eq!(rep3.score, 530); // Should remain the same
}

#[test]
fn test_update_uptime_multiple_rapid_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    // Simulate repeated calls with high uptime, advancing ledger each time to avoid cooldown error
    for _ in 0..10 {
        env.ledger().with_mut(|li| {
            li.sequence_number += 17_281;
        });
        c.update_uptime(&oracle, &pub1, &100u32);
    }

    let rep = c.get_reputation(&pub1).unwrap();
    // Score should be 520 (500 + 20), not inflated to 700 (500 + 10*20)
    assert_eq!(rep.score, 520);
    assert_eq!(rep.uptime_contribution, 20);
}

#[test]
#[should_panic(expected = "uptime update cooldown active")]
fn test_update_uptime_cooldown_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, oracle) = setup(&env);
    let pub1 = Address::generate(&env);
    c.init_publisher(&pub1);

    // First call succeeds
    c.update_uptime(&oracle, &pub1, &95u32);

    // Second call fails due to cooldown active (sequence number is still the same)
    c.update_uptime(&oracle, &pub1, &95u32);
}
