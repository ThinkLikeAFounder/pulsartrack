#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger, LedgerInfo},
    vec, Address, Env, IntoVal,
};

fn setup(env: &Env) -> (GovernanceTokenContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(GovernanceTokenContract, ());
    let c = GovernanceTokenContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.total_supply(), 0);
    assert_eq!(c.decimals(), 7);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(GovernanceTokenContract, ());
    let c = GovernanceTokenContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let id = env.register(GovernanceTokenContract, ());
    let c = GovernanceTokenContractClient::new(&env, &id);
    c.initialize(&Address::generate(&env));
}

#[test]
fn test_mint() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000_000i128);
    assert_eq!(c.balance(&user), 1_000_000);
    assert_eq!(c.total_supply(), 1_000_000);
}

#[test]
fn test_token_operations_emit_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let spender = Address::generate(&env);
    let delegate = Address::generate(&env);

    c.mint(&admin, &owner, &1_000);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                c.address.clone(),
                (symbol_short!("mint"),).into_val(&env),
                (owner.clone(), 1_000i128).into_val(&env),
            )
        ]
    );

    c.transfer(&owner, &recipient, &100);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                c.address.clone(),
                (symbol_short!("transfer"),).into_val(&env),
                (owner.clone(), recipient.clone(), 100i128).into_val(&env),
            )
        ]
    );

    c.approve(&owner, &spender, &50, &1_000);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                c.address.clone(),
                (symbol_short!("approve"),).into_val(&env),
                (owner.clone(), spender.clone(), 50i128, 1_000u32).into_val(&env),
            )
        ]
    );

    c.transfer_from(&spender, &owner, &recipient, &20);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                c.address.clone(),
                (symbol_short!("transfer"),).into_val(&env),
                (owner.clone(), recipient, 20i128).into_val(&env),
            )
        ]
    );

    c.burn(&owner, &25);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                c.address.clone(),
                (symbol_short!("burn"),).into_val(&env),
                (owner.clone(), 25i128).into_val(&env),
            )
        ]
    );

    c.delegate(&owner, &delegate);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                c.address.clone(),
                (symbol_short!("delegate"),).into_val(&env),
                (owner.clone(), delegate.clone()).into_val(&env),
            )
        ]
    );

    c.revoke_delegation(&owner);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                c.address.clone(),
                (symbol_short!("revoke"),).into_val(&env),
                (owner, delegate).into_val(&env),
            )
        ]
    );
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_mint_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    c.mint(
        &Address::generate(&env),
        &Address::generate(&env),
        &1_000i128,
    );
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    c.mint(&admin, &from, &1_000i128);
    c.transfer(&from, &to, &400i128);
    assert_eq!(c.balance(&from), 600);
    assert_eq!(c.balance(&to), 400);
}

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000i128);
    c.burn(&user, &300i128);
    assert_eq!(c.balance(&user), 700);
    assert_eq!(c.total_supply(), 700);
}

#[test]
fn test_delegate() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    c.mint(&admin, &delegator, &1_000i128);
    c.delegate(&delegator, &delegate);
    let d = c.get_delegation(&delegator).unwrap();
    assert_eq!(d.delegate, delegate);
    // Delegator loses voting power after delegating
    assert_eq!(c.voting_power(&delegator), 0);
}

#[test]
fn test_revoke_delegation() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    c.mint(&admin, &delegator, &1_000i128);
    c.delegate(&delegator, &delegate);
    c.revoke_delegation(&delegator);
    assert!(c.get_delegation(&delegator).is_none());
}

#[test]
fn test_voting_power_self() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &1_000i128);
    assert_eq!(c.voting_power(&user), 1_000);
}

#[test]
fn test_approve_and_allowance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    c.approve(&owner, &spender, &500i128, &1000u32);
    assert_eq!(c.allowance(&owner, &spender), 500);
}

#[test]
fn test_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let to = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    // expiry well above default ledger sequence (0)
    c.approve(&owner, &spender, &500i128, &1000u32);
    c.transfer_from(&spender, &owner, &to, &200i128);
    assert_eq!(c.balance(&owner), 800);
    assert_eq!(c.balance(&to), 200);
    assert_eq!(c.allowance(&owner, &spender), 300);
}

#[test]
#[should_panic(expected = "allowance expired")]
fn test_transfer_from_expired_allowance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let to = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    // approve with expiry = 5, then advance ledger past it
    c.approve(&owner, &spender, &500i128, &5u32);
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: 6,
        timestamp: env.ledger().timestamp(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3_110_400,
    });
    c.transfer_from(&spender, &owner, &to, &100i128);
}

#[test]
fn test_balance_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert_eq!(c.balance(&Address::generate(&env)), 0);
}

#[test]
#[should_panic(expected = "expiry must be a future ledger sequence")]
fn test_approve_expired_expiry_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    c.mint(&admin, &owner, &1_000i128);
    // ledger sequence starts at 0; expiry=0 must be rejected
    c.approve(&owner, &spender, &500i128, &0u32);
}

#[test]
#[should_panic(expected = "exceeds max supply")]
fn test_mint_exceeds_max_supply_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    // Any single mint above MAX_SUPPLY is rejected by the cap check.
    c.mint(&admin, &user, &(MAX_SUPPLY + 1));
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_mint_zero_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &0);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_mint_negative_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    c.mint(&admin, &user, &-100);
}

#[test]
#[should_panic(expected = "supply overflow")]
fn test_mint_supply_overflow_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);
    // Reach the cap, then mint a value large enough that
    // current_supply + amount overflows i128 — checked_add must catch this
    // before the cap comparison.
    c.mint(&admin, &user, &MAX_SUPPLY);
    c.mint(&admin, &user, &i128::MAX);
}

// ─── Checkpoint history / flash-loan resistance (#815) ───────────────────────

fn set_ledger(env: &Env, seq: u32) {
    env.ledger().with_mut(|li| li.sequence_number = seq);
}

#[test]
fn test_checkpoint_written_on_mint() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);

    set_ledger(&env, 100);
    c.mint(&admin, &user, &1_000);

    assert_eq!(c.get_checkpoint_count(&user), 1);
    // The mint landed in ledger 100, so it is not yet visible "before 100".
    assert_eq!(c.get_past_votes(&user, &100), 0);

    set_ledger(&env, 101);
    assert_eq!(c.get_past_votes(&user, &101), 1_000);
}

#[test]
fn test_get_past_votes_returns_zero_before_any_checkpoint() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);

    set_ledger(&env, 50);
    c.mint(&admin, &user, &500);

    set_ledger(&env, 100);
    // Ledger 10 predates the account's first checkpoint entirely.
    assert_eq!(c.get_past_votes(&user, &10), 0);
    assert_eq!(c.get_past_votes(&user, &100), 500);
}

#[test]
fn test_get_past_votes_resolves_correct_historical_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);

    set_ledger(&env, 10);
    c.mint(&admin, &user, &100);
    set_ledger(&env, 20);
    c.mint(&admin, &user, &200); // now 300
    set_ledger(&env, 30);
    c.mint(&admin, &user, &700); // now 1000

    set_ledger(&env, 100);
    assert_eq!(c.get_past_votes(&user, &15), 100);
    assert_eq!(c.get_past_votes(&user, &20), 100); // ledger-20 change not yet visible
    assert_eq!(c.get_past_votes(&user, &25), 300);
    assert_eq!(c.get_past_votes(&user, &30), 300);
    assert_eq!(c.get_past_votes(&user, &31), 1_000);
}

#[test]
#[should_panic(expected = "cannot read votes for a future ledger")]
fn test_get_past_votes_rejects_future_ledger() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);

    set_ledger(&env, 10);
    c.mint(&admin, &user, &100);
    c.get_past_votes(&user, &999);
}

#[test]
fn test_flash_loan_borrow_grants_no_usable_voting_power_same_ledger() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let whale = Address::generate(&env);
    let attacker = Address::generate(&env);

    // Whale holds a large balance well before the vote.
    set_ledger(&env, 100);
    c.mint(&admin, &whale, &1_000_000);

    // The proposal's snapshot ledger.
    let snapshot_ledger: u32 = 200;
    set_ledger(&env, snapshot_ledger);

    // Attacker borrows the whole whale balance IN the snapshot ledger — the
    // flash-loan step.
    c.transfer(&whale, &attacker, &1_000_000);

    // Live balance is inflated...
    assert_eq!(c.balance(&attacker), 1_000_000);
    // ...but the borrow is invisible to the historical read that governance
    // uses, so it buys no voting power.
    assert_eq!(c.get_past_votes(&attacker, &snapshot_ledger), 0);

    // Repaying in the same ledger leaves the attacker with nothing either way.
    c.transfer(&attacker, &whale, &1_000_000);
    assert_eq!(c.balance(&attacker), 0);
    assert_eq!(c.get_past_votes(&attacker, &snapshot_ledger), 0);
}

#[test]
fn test_legitimate_long_term_holder_keeps_voting_power() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let holder = Address::generate(&env);

    set_ledger(&env, 100);
    c.mint(&admin, &holder, &5_000);

    let snapshot_ledger: u32 = 200;
    set_ledger(&env, snapshot_ledger);

    // Held since well before the snapshot, so the power is fully usable.
    assert_eq!(c.get_past_votes(&holder, &snapshot_ledger), 5_000);
}

#[test]
fn test_checkpoints_track_delegated_power() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    set_ledger(&env, 10);
    c.mint(&admin, &delegator, &1_000);

    set_ledger(&env, 20);
    c.delegate(&delegator, &delegate);

    set_ledger(&env, 30);
    // Delegator gave their power away; the delegate received it.
    assert_eq!(c.get_past_votes(&delegator, &30), 0);
    assert_eq!(c.get_past_votes(&delegate, &30), 1_000);

    // Before the delegation the delegator still held it themselves.
    assert_eq!(c.get_past_votes(&delegator, &20), 1_000);
    assert_eq!(c.get_past_votes(&delegate, &20), 0);
}

#[test]
fn test_multiple_writes_in_one_ledger_collapse_to_one_checkpoint() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let user = Address::generate(&env);

    set_ledger(&env, 42);
    c.mint(&admin, &user, &100);
    c.mint(&admin, &user, &100);
    c.mint(&admin, &user, &100);

    assert_eq!(c.get_checkpoint_count(&user), 1);
    set_ledger(&env, 43);
    assert_eq!(c.get_past_votes(&user, &43), 300);
}

#[test]
fn test_take_snapshot_uses_checkpoint_history_not_live_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let whale = Address::generate(&env);
    let attacker = Address::generate(&env);

    set_ledger(&env, 100);
    c.mint(&admin, &whale, &1_000_000);

    set_ledger(&env, 200);
    c.transfer(&whale, &attacker, &1_000_000);

    // Snapshotting the current ledger must not capture the just-borrowed
    // balance — the old implementation read live storage and would record
    // 1_000_000 here.
    c.take_snapshot(&attacker, &200);
    assert_eq!(c.get_voting_snapshot(&attacker, &200), Some(0));
}
