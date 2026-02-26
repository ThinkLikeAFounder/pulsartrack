#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(
    env: &Env,
) -> (
    PayoutAutomationContractClient<'_>,
    Address,
    Address,
    Address,
) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr);

    (client, admin, token_admin, token_addr)
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
    client.initialize(&admin, &token);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
}

// ─── schedule_payout ──────────────────────────────────────────────────────────

#[test]
fn test_schedule_payout() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_addr) = setup(&env);
    let recipient = Address::generate(&env);

    let payout_id = client.schedule_payout(
        &admin,
        &recipient,
        &1_000_000i128,
        &3600u64, // execute_after = 1 hour from epoch 0
        &Some(99u64),
    );

    assert_eq!(payout_id, 1);

    let payout = client.get_payout(&payout_id).unwrap();
    assert_eq!(payout.recipient, recipient);
    assert_eq!(payout.amount, 1_000_000);
    assert_eq!(payout.execute_after, 3600);
    assert!(matches!(payout.status, PayoutStatus::Scheduled));
    assert_eq!(payout.campaign_id, Some(99));
    assert_eq!(payout.token, token_addr);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_schedule_payout_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let stranger = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.schedule_payout(&stranger, &recipient, &1_000_000i128, &3600u64, &None);
}

#[test]
fn test_schedule_multiple_payouts() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _) = setup(&env);

    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);

    let id1 = client.schedule_payout(&admin, &r1, &1_000_000i128, &100u64, &None);
    let id2 = client.schedule_payout(&admin, &r2, &2_000_000i128, &200u64, &None);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

// ─── execute_payout ──────────────────────────────────────────────────────────

#[test]
fn test_execute_payout() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    // mint tokens directly to the payout contract so it can pay out
    mint(&env, &token_addr, &contract_id, 10_000_000);

    let recipient = Address::generate(&env);
    let payout_id = client.schedule_payout(&admin, &recipient, &1_000_000i128, &100u64, &None);

    // advance past execute_after
    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

    client.execute_payout(&admin, &payout_id);

    let payout = client.get_payout(&payout_id).unwrap();
    assert!(matches!(payout.status, PayoutStatus::Completed));
    assert!(payout.executed_at.is_some());

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&recipient), 1_000_000);
    assert_eq!(tc.balance(&contract_id), 9_000_000);
}

#[test]
#[should_panic(expected = "too early to execute")]
fn test_execute_payout_too_early() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    mint(&env, &token_addr, &contract_id, 10_000_000);

    let recipient = Address::generate(&env);
    let payout_id = client.schedule_payout(
        &admin,
        &recipient,
        &1_000_000i128,
        &9999u64,
        &None, // far future
    );

    // ledger timestamp is still 0 → too early
    client.execute_payout(&admin, &payout_id);
}

#[test]
#[should_panic(expected = "payout not scheduled")]
fn test_execute_payout_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    mint(&env, &token_addr, &contract_id, 10_000_000);

    let recipient = Address::generate(&env);
    let payout_id = client.schedule_payout(&admin, &recipient, &1_000_000i128, &0u64, &None);

    client.execute_payout(&admin, &payout_id);
    client.execute_payout(&admin, &payout_id); // second attempt → "payout not scheduled"
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_execute_payout_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    mint(&env, &token_addr, &contract_id, 10_000_000);

    let recipient = Address::generate(&env);
    let payout_id = client.schedule_payout(&admin, &recipient, &1_000_000i128, &0u64, &None);

    let stranger = Address::generate(&env);
    client.execute_payout(&stranger, &payout_id);
}

// ─── publisher earnings ───────────────────────────────────────────────────────

#[test]
fn test_add_publisher_earnings() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _) = setup(&env);
    let publisher = Address::generate(&env);

    client.add_publisher_earnings(&admin, &publisher, &500_000i128);

    let earnings = client.get_publisher_earnings(&publisher).unwrap();
    assert_eq!(earnings.pending_amount, 500_000);
    assert_eq!(earnings.total_paid, 0);
}

#[test]
fn test_add_publisher_earnings_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _) = setup(&env);
    let publisher = Address::generate(&env);

    client.add_publisher_earnings(&admin, &publisher, &300_000i128);
    client.add_publisher_earnings(&admin, &publisher, &200_000i128);

    let earnings = client.get_publisher_earnings(&publisher).unwrap();
    assert_eq!(earnings.pending_amount, 500_000);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_publisher_earnings_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.add_publisher_earnings(&stranger, &publisher, &500_000i128);
}

#[test]
fn test_publisher_earnings_updated_after_payout() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PayoutAutomationContract);
    let client = PayoutAutomationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    mint(&env, &token_addr, &contract_id, 10_000_000);

    let publisher = Address::generate(&env);
    // register 1_500_000 pending earnings
    client.add_publisher_earnings(&admin, &publisher, &1_500_000i128);

    let payout_id = client.schedule_payout(&admin, &publisher, &1_000_000i128, &0u64, &None);

    // advance timestamp so last_payout is recorded as non-zero
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000;
    });

    client.execute_payout(&admin, &payout_id);

    let earnings = client.get_publisher_earnings(&publisher).unwrap();
    assert_eq!(earnings.total_paid, 1_000_000);
    // pending_amount uses saturating_sub: 1_500_000 - 1_000_000 = 500_000
    assert_eq!(earnings.pending_amount, 500_000);
    assert_eq!(earnings.last_payout, 1_000);
}

// ─── get_payout ────────────────────────────────────────────────────────────────

#[test]
fn test_get_payout_nonexistent_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);

    assert!(client.get_payout(&999u64).is_none());
}

#[test]
fn test_get_publisher_earnings_nonexistent_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let unknown = Address::generate(&env);

    assert!(client.get_publisher_earnings(&unknown).is_none());
}
