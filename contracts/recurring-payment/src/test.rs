#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (RecurringPaymentContractClient, Address) {
    let admin = Address::generate(env);
    let id = env.register_contract(None, RecurringPaymentContract);
    let c = RecurringPaymentContractClient::new(env, &id);
    c.initialize(&admin);
    (c, admin)
}

#[test]
fn test_initialize() { let env = Env::default(); env.mock_all_auths(); setup(&env); }

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default(); env.mock_all_auths();
    let id = env.register_contract(None, RecurringPaymentContract);
    let c = RecurringPaymentContractClient::new(&env, &id);
    let a = Address::generate(&env); c.initialize(&a); c.initialize(&a);
}

#[test]
fn test_create_recurring() {
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let id = c.create_recurring(&payer, &payee, &token, &1000i128, &86_400u64, &None);
    assert_eq!(id, 1);
}

#[test]
fn test_pause_payment() {
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
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
    let env = Env::default(); env.mock_all_auths();
    let (c, _) = setup(&env);
    assert!(c.get_payment(&999u64).is_none());
}
