#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}

fn mint(env: &Env, token_addr: &Address, admin: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(
    env: &Env,
) -> (
    PaymentProcessorContractClient,
    Address, // admin
    Address, // treasury
    Address, // token_admin
    Address, // token_addr
) {
    let admin = Address::generate(env);
    let treasury = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(env, &contract_id);
    client.initialize(&admin, &treasury);

    (client, admin, treasury, token_admin, token_addr)
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
    client.initialize(&admin, &treasury);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
}

// ─── add_token / remove_token ─────────────────────────────────────────────────

#[test]
fn test_add_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_admin, token_addr) = setup(&env);

    client.add_token(&admin, &token_addr, &1_000i128, &10_000_000i128);

    let cfg = client.get_token_config(&token_addr).unwrap();
    assert!(cfg.enabled);
    assert_eq!(cfg.min_amount, 1_000);
    assert_eq!(cfg.daily_limit, 10_000_000);
    let _ = token_admin; // used in deploy only
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_token_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, token_admin, token_addr) = setup(&env);
    let stranger = Address::generate(&env);
    let _ = token_admin;

    client.add_token(&stranger, &token_addr, &1_000i128, &10_000_000i128);
}

#[test]
fn test_remove_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, token_addr) = setup(&env);

    client.add_token(&admin, &token_addr, &1_000i128, &10_000_000i128);
    assert!(client.get_token_config(&token_addr).is_some());

    client.remove_token(&admin, &token_addr);
    assert!(client.get_token_config(&token_addr).is_none());
}

// ─── process_payment ─────────────────────────────────────────────────────────

#[test]
fn test_process_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
    client.add_token(&admin, &token_addr, &1_000i128, &100_000_000i128);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    let payment_id = client.process_payment(&payer, &recipient, &token_addr, &10_000i128);
    assert_eq!(payment_id, 1);

    let tc = TokenClient::new(&env, &token_addr);
    // fee = 10_000 * 250 / 10_000 = 250 → net = 9_750
    assert_eq!(tc.balance(&recipient), 9_750);
    assert_eq!(tc.balance(&treasury), 250);
    assert_eq!(tc.balance(&payer), 990_000);
}

#[test]
fn test_process_payment_fee_split() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
    client.add_token(&admin, &token_addr, &1_000i128, &100_000_000i128);

    // Set fee to 500 bps (5%)
    client.set_platform_fee(&admin, &500u32);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    client.process_payment(&payer, &recipient, &token_addr, &20_000i128);

    let tc = TokenClient::new(&env, &token_addr);
    // fee = 20_000 * 500 / 10_000 = 1_000 → net = 19_000
    assert_eq!(tc.balance(&recipient), 19_000);
    assert_eq!(tc.balance(&treasury), 1_000);
}

#[test]
fn test_payment_records_stored() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
    client.add_token(&admin, &token_addr, &1_000i128, &100_000_000i128);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    let payment_id = client.process_payment(&payer, &recipient, &token_addr, &10_000i128);

    let payment = client.get_payment(&payment_id).unwrap();
    assert_eq!(payment.amount, 10_000);
    assert_eq!(payment.payer, payer);
    assert_eq!(payment.recipient, recipient);
}

#[test]
fn test_user_stats_updated() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
    client.add_token(&admin, &token_addr, &1_000i128, &100_000_000i128);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    client.process_payment(&payer, &recipient, &token_addr, &10_000i128);
    client.process_payment(&payer, &recipient, &token_addr, &5_000i128);

    let stats = client.get_user_stats(&payer).unwrap();
    assert_eq!(stats.total_payments, 2);
    assert_eq!(stats.total_spent, 15_000);
}

#[test]
fn test_revenue_stats_updated() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, PaymentProcessorContract);
    let client = PaymentProcessorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury);
    client.add_token(&admin, &token_addr, &1_000i128, &100_000_000i128);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    client.process_payment(&payer, &recipient, &token_addr, &10_000i128);

    let stats = client.get_revenue_stats(&token_addr).unwrap();
    // fee = 10_000 * 250 / 10_000 = 250
    assert_eq!(stats.total_fees_collected, 250);
    assert_eq!(stats.total_volume, 10_000);
    assert_eq!(stats.payment_count, 1);
}

// ─── payment error paths ──────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "cannot pay yourself")]
fn test_cannot_pay_yourself() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_admin, token_addr) = setup(&env);
    client.add_token(&admin, &token_addr, &1_000i128, &100_000_000i128);

    let payer = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    client.process_payment(&payer, &payer, &token_addr, &10_000i128);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_payment_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_admin, token_addr) = setup(&env);
    client.add_token(&admin, &token_addr, &1_000i128, &100_000_000i128);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    client.process_payment(&payer, &recipient, &token_addr, &0i128);
}

#[test]
#[should_panic(expected = "token not whitelisted")]
fn test_payment_token_not_whitelisted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _) = setup(&env);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let random_token = Address::generate(&env);

    client.process_payment(&payer, &recipient, &random_token, &10_000i128);
}

#[test]
#[should_panic(expected = "amount below minimum")]
fn test_payment_below_minimum() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_admin, token_addr) = setup(&env);
    client.add_token(&admin, &token_addr, &5_000i128, &100_000_000i128); // min = 5_000

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    client.process_payment(&payer, &recipient, &token_addr, &1_000i128); // below min
}

#[test]
#[should_panic(expected = "daily limit exceeded")]
fn test_payment_daily_limit_exceeded() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, token_admin, token_addr) = setup(&env);
    // daily limit of 15_000
    client.add_token(&admin, &token_addr, &1_000i128, &15_000i128);

    let payer = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token_addr, &token_admin, &payer, 1_000_000);

    client.process_payment(&payer, &recipient, &token_addr, &10_000i128);
    client.process_payment(&payer, &recipient, &token_addr, &10_000i128); // 20_000 > 15_000
}

// ─── set_platform_fee ────────────────────────────────────────────────────────

#[test]
fn test_set_platform_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = setup(&env);
    client.set_platform_fee(&admin, &100u32); // 1%
}

#[test]
#[should_panic(expected = "fee too high")]
fn test_set_platform_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = setup(&env);
    client.set_platform_fee(&admin, &1001u32); // > 10%
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_platform_fee_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _) = setup(&env);
    let stranger = Address::generate(&env);
    client.set_platform_fee(&stranger, &100u32);
}
