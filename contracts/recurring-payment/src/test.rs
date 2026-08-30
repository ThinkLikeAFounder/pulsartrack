#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

fn approve_allowance(env: &Env, token: &Address, from: &Address, spender: &Address, amount: i128) {
    TokenClient::new(env, token).approve(
        from,
        spender,
        &amount,
        &(env.ledger().sequence() + 100_000u32),
    );
}

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (RecurringPaymentContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let id = env.register(RecurringPaymentContract, ());
    let c = RecurringPaymentContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
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
    let id = env.register(RecurringPaymentContract, ());
    let c = RecurringPaymentContractClient::new(&env, &id);
    let a = Address::generate(&env);
    c.initialize(&a);
    c.initialize(&a);
}

#[test]
fn test_create_recurring() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &86_400u64, &Some(12u32));
    assert_eq!(id, 1);
    let payment = c.get_payment(&id).unwrap();
    assert!(matches!(payment.status, RecurringStatus::Active));
    assert_eq!(payment.amount, 1000);
}

#[test]
fn test_create_recurring_no_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &86_400u64, &None);
    assert_eq!(id, 1);
}

#[test]
fn test_pause_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &86_400u64, &None);
    c.pause_payment(&payer, &id);
    let payment = c.get_payment(&id).unwrap();
    assert!(matches!(payment.status, RecurringStatus::Paused));
}

#[test]
fn test_resume_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &86_400u64, &None);
    c.pause_payment(&payer, &id);
    c.resume_payment(&payer, &id);
    let payment = c.get_payment(&id).unwrap();
    assert!(matches!(payment.status, RecurringStatus::Active));
}

#[test]
fn test_cancel_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &86_400u64, &None);
    c.cancel_payment(&payer, &id);
    let payment = c.get_payment(&id).unwrap();
    assert!(matches!(payment.status, RecurringStatus::Cancelled));
}

#[test]
fn test_get_payment_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_payment(&999u64).is_none());
}

#[test]
fn test_execute_payment_by_payer() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    mint(&env, &token, &payer, 10_000);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &1u64, &None);
    approve_allowance(&env, &token, &payer, &c.address, 10_000);

    env.ledger().with_mut(|li| li.timestamp = 2);

    c.execute_payment(&payer, &id);
    let payment = c.get_payment(&id).unwrap();
    assert_eq!(payment.total_payments, 1);
}

#[test]
fn test_execute_payment_by_recipient() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    mint(&env, &token, &payer, 10_000);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &1u64, &None);
    approve_allowance(&env, &token, &payer, &c.address, 10_000);

    env.ledger().with_mut(|li| li.timestamp = 2);

    c.execute_payment(&payee, &id);
    let payment = c.get_payment(&id).unwrap();
    assert_eq!(payment.total_payments, 1);
}

#[test]
fn test_execute_payment_by_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    mint(&env, &token, &payer, 10_000);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &1u64, &None);
    approve_allowance(&env, &token, &payer, &c.address, 10_000);

    env.ledger().with_mut(|li| li.timestamp = 2);

    c.execute_payment(&admin, &id);
    let payment = c.get_payment(&id).unwrap();
    assert_eq!(payment.total_payments, 1);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_execute_payment_by_stranger_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let stranger = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &1u64, &None);

    // Fast forward time to allow execution
    env.ledger().with_mut(|li| li.timestamp = 2);

    // Stranger cannot execute
    c.execute_payment(&stranger, &id);
}

#[test]
#[should_panic(expected = "too early")]
fn test_execute_payment_too_early() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &86_400u64, &None);

    // Try to execute immediately (too early)
    c.execute_payment(&payer, &id);
}

#[test]
#[should_panic(expected = "max payments reached")]
fn test_execute_payment_max_reached() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    mint(&env, &token, &payer, 10_000);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &1u64, &Some(1u32));
    approve_allowance(&env, &token, &payer, &c.address, 10_000);

    env.ledger().with_mut(|li| li.timestamp = 2);
    c.execute_payment(&payer, &id);

    // Try to execute second payment (should fail - max reached)
    env.ledger().with_mut(|li| li.timestamp = 4);
    c.execute_payment(&payer, &id);
}

/// Verify the SEP-41 allowance pattern: payer pre-approves the contract once,
/// then an admin keeper executes without any payer co-signature, and real token
/// balances move correctly (payer -1000, payee +1000).
#[test]
fn test_execute_payment_transfers_balance_via_allowance() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    mint(&env, &token, &payer, 10_000);

    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &1u64, &Some(3u32));

    // Payer approves the recurring-payment contract for the total allowance (3 payments × 1000)
    soroban_sdk::token::Client::new(&env, &token).approve(
        &payer,
        &c.address,
        &3_000i128,
        &(env.ledger().sequence() + 100_000u32),
    );

    env.ledger().with_mut(|li| li.timestamp = 2);

    // Admin (keeper) executes — no payer co-sign needed beyond the one-time approve
    c.execute_payment(&admin, &id);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&payee), 1_000, "payee should receive 1000");
    assert_eq!(token_client.balance(&payer), 9_000, "payer should be debited 1000");

    let payment = c.get_payment(&id).unwrap();
    assert_eq!(payment.total_payments, 1);
}
