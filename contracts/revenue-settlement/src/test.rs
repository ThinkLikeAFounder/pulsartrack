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

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(env: &Env) -> (RevenueSettlementContractClient, Address, Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);
    let treasury = Address::generate(env);
    let platform = Address::generate(env);

    let contract_id = env.register_contract(None, RevenueSettlementContract);
    let client = RevenueSettlementContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &treasury, &platform);

    (client, admin, token_admin, token_addr, treasury, platform)
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevenueSettlementContract);
    let client = RevenueSettlementContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    let platform = Address::generate(&env);

    client.initialize(&admin, &token, &treasury, &platform);

    let pool = client.get_revenue_pool();
    assert_eq!(pool.total_revenue, 0);
    assert_eq!(pool.platform_pct, 250);
    assert_eq!(pool.publisher_pct, 9_000);
    assert_eq!(pool.treasury_pct, 500);
    assert_eq!(pool.burn_pct, 250);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevenueSettlementContract);
    let client = RevenueSettlementContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    let platform = Address::generate(&env);

    client.initialize(&admin, &token, &treasury, &platform);
    client.initialize(&admin, &token, &treasury, &platform);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, RevenueSettlementContract);
    let client = RevenueSettlementContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    let platform = Address::generate(&env);

    client.initialize(&admin, &token, &treasury, &platform);
}

// ─── record_revenue ──────────────────────────────────────────────────────────

#[test]
fn test_record_revenue_fee_split() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);

    // Record 100_000 revenue
    let settlement_id = client.record_revenue(&admin, &1u64, &100_000i128, &publisher);
    assert_eq!(settlement_id, 1);

    // Check pool splits:
    // platform: 100_000 * 250 / 10_000 = 2_500
    // treasury: 100_000 * 500 / 10_000 = 5_000
    // burn:     100_000 * 250 / 10_000 = 2_500
    // publisher: 100_000 - 2_500 - 5_000 - 2_500 = 90_000
    let pool = client.get_revenue_pool();
    assert_eq!(pool.total_revenue, 100_000);
    assert_eq!(pool.platform_share, 2_500);
    assert_eq!(pool.treasury_share, 5_000);
    assert_eq!(pool.burn_amount, 2_500);
    assert_eq!(pool.publisher_share, 90_000);

    // Check publisher balance
    assert_eq!(client.get_publisher_balance(&publisher), 90_000);

    // Check settlement record
    let record = client.get_settlement(&settlement_id).unwrap();
    assert_eq!(record.campaign_id, 1);
    assert_eq!(record.total_amount, 100_000);
    assert_eq!(record.platform_fee, 2_500);
    assert_eq!(record.publisher_amount, 90_000);
}

#[test]
fn test_record_revenue_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);

    client.record_revenue(&admin, &1u64, &100_000i128, &publisher);
    client.record_revenue(&admin, &2u64, &50_000i128, &publisher);

    let pool = client.get_revenue_pool();
    assert_eq!(pool.total_revenue, 150_000);

    // publisher balance = 90_000 + 45_000 = 135_000
    assert_eq!(client.get_publisher_balance(&publisher), 135_000);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_record_revenue_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _, _) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.record_revenue(&stranger, &1u64, &100_000i128, &publisher);
}

// ─── distribute_platform_revenue ─────────────────────────────────────────────

#[test]
fn test_distribute_platform_revenue() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let treasury = Address::generate(&env);
    let platform = Address::generate(&env);

    let contract_id = env.register_contract(None, RevenueSettlementContract);
    let client = RevenueSettlementContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &treasury, &platform);

    // Fund the contract
    mint(&env, &token_addr, &contract_id, 10_000_000);

    let publisher = Address::generate(&env);
    client.record_revenue(&admin, &1u64, &100_000i128, &publisher);

    env.ledger().with_mut(|li| { li.timestamp = 1000; });

    client.distribute_platform_revenue(&admin);

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&platform), 2_500);    // platform share
    assert_eq!(tc.balance(&treasury), 5_000);    // treasury share

    // Pool should be reset for platform and treasury
    let pool = client.get_revenue_pool();
    assert_eq!(pool.platform_share, 0);
    assert_eq!(pool.treasury_share, 0);
    assert_eq!(pool.last_settlement, 1000);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_distribute_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _, _) = setup(&env);
    let stranger = Address::generate(&env);

    client.distribute_platform_revenue(&stranger);
}

// ─── claim_publisher_balance ─────────────────────────────────────────────────

#[test]
fn test_claim_publisher_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let treasury = Address::generate(&env);
    let platform = Address::generate(&env);

    let contract_id = env.register_contract(None, RevenueSettlementContract);
    let client = RevenueSettlementContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &treasury, &platform);

    mint(&env, &token_addr, &contract_id, 10_000_000);

    let publisher = Address::generate(&env);
    client.record_revenue(&admin, &1u64, &100_000i128, &publisher);

    client.claim_publisher_balance(&publisher);

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&publisher), 90_000);
    assert_eq!(client.get_publisher_balance(&publisher), 0);
}

#[test]
#[should_panic(expected = "no balance to claim")]
fn test_claim_zero_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);

    client.claim_publisher_balance(&publisher);
}

// ─── read-only ───────────────────────────────────────────────────────────────

#[test]
fn test_get_publisher_balance_unknown() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _, _) = setup(&env);
    let unknown = Address::generate(&env);

    assert_eq!(client.get_publisher_balance(&unknown), 0);
}

#[test]
fn test_get_settlement_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _, _) = setup(&env);

    assert!(client.get_settlement(&999u64).is_none());
}
