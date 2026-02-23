#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, vec,
};

// ─── helpers ────────────────────────────────────────────────────────────────

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}

fn setup(env: &Env) -> (EscrowVaultContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let oracle = Address::generate(env);

    let token_addr = deploy_token(env, &token_admin);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    (client, admin, token_addr, oracle)
}

fn mint(env: &Env, token_admin: &Address, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    let oracle = Address::generate(&env);

    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &oracle);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    let oracle = Address::generate(&env);

    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &oracle);
    client.initialize(&admin, &token, &oracle);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    // deliberately no mock_all_auths so require_auth panics
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);
    let oracle = Address::generate(&env);

    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &oracle);
}

// ─── create_escrow ───────────────────────────────────────────────────────────

#[test]
fn test_create_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor,
        &1u64,
        &beneficiary,
        &100_000i128,
        &0u64,  // no time lock
        &0u32,  // 0% performance threshold
        &86_400u64,
        &vec![&env, approver.clone()],
    );

    assert_eq!(escrow_id, 1);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.amount, 100_000);
    assert_eq!(escrow.locked_amount, 100_000);
    assert_eq!(escrow.released_amount, 0);
    assert_eq!(escrow.depositor, depositor);
    assert_eq!(escrow.beneficiary, beneficiary);

    // tokens moved from depositor to vault
    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&depositor), 900_000);
    assert_eq!(tc.balance(&contract_id), 100_000);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_create_escrow_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    client.create_escrow(
        &depositor,
        &1u64,
        &beneficiary,
        &0i128,
        &0u64,
        &0u32,
        &86_400u64,
        &vec![&env],
    );
}

#[test]
#[should_panic(expected = "invalid performance threshold")]
fn test_create_escrow_invalid_performance_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    client.create_escrow(
        &depositor,
        &1u64,
        &beneficiary,
        &100_000i128,
        &0u64,
        &101u32, // > 100 → invalid
        &86_400u64,
        &vec![&env],
    );
}

// ─── approve_release ─────────────────────────────────────────────────────────

#[test]
fn test_approve_release() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &86_400u64,
        &vec![&env, approver.clone()],
    );

    assert_eq!(client.get_approval_count(&escrow_id), 0);
    client.approve_release(&approver, &escrow_id);
    assert_eq!(client.get_approval_count(&escrow_id), 1);
}

#[test]
#[should_panic(expected = "not a required approver")]
fn test_approve_release_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let stranger = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &86_400u64,
        &vec![&env],  // no required approvers
    );

    client.approve_release(&stranger, &escrow_id); // should panic
}

// ─── release_escrow ──────────────────────────────────────────────────────────

#[test]
fn test_release_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,  // far-future expiry
        &vec![&env, approver.clone()],
    );

    client.approve_release(&approver, &escrow_id);
    client.release_escrow(&depositor, &escrow_id);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.released_amount, 100_000);
    assert_eq!(escrow.locked_amount, 0);

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&beneficiary), 100_000);
    assert_eq!(tc.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "time lock active")]
fn test_release_escrow_time_lock_active() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &3600u64, // 1 hour time lock — still active
        &0u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    client.approve_release(&approver, &escrow_id);
    client.release_escrow(&depositor, &escrow_id); // panics: time lock active
}

#[test]
#[should_panic(expected = "approval required")]
fn test_release_escrow_no_approval() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env],  // no approvers registered → count stays 0
    );

    // min_threshold = 1, approvals = 0 → panic
    client.release_escrow(&depositor, &escrow_id);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_release_escrow_unauthorized_caller() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    let stranger = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    client.approve_release(&approver, &escrow_id);
    client.release_escrow(&stranger, &escrow_id); // not depositor or admin
}

// ─── release_partial ─────────────────────────────────────────────────────────

#[test]
fn test_release_partial() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    client.approve_release(&approver, &escrow_id);
    client.release_partial(&depositor, &escrow_id, &40_000i128);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.released_amount, 40_000);
    assert_eq!(escrow.locked_amount, 60_000);

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&beneficiary), 40_000);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_release_partial_exceeds_locked() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    client.approve_release(&approver, &escrow_id);
    client.release_partial(&depositor, &escrow_id, &200_000i128); // more than locked
}

// ─── refund_escrow ───────────────────────────────────────────────────────────

#[test]
fn test_refund_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    // expires_in = 100 seconds from now (ledger timestamp = 0 by default)
    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &100u64,
        &vec![&env],
    );

    // advance ledger past expiry
    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

    client.refund_escrow(&depositor, &escrow_id);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.refunded_amount, 100_000);
    assert_eq!(escrow.locked_amount, 0);

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&depositor), 1_000_000); // got funds back
}

#[test]
#[should_panic(expected = "escrow not yet expired")]
fn test_refund_escrow_not_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64, // far future expiry
        &vec![&env],
    );

    client.refund_escrow(&depositor, &escrow_id); // too early
}

// ─── update_performance ──────────────────────────────────────────────────────

#[test]
fn test_update_performance() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &80u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    client.update_performance(&oracle, &escrow_id, &90u32, &1000u64, &50u64);

    let perf = client.get_performance(&escrow_id).unwrap();
    assert_eq!(perf.current_performance, 90);
    assert_eq!(perf.views_delivered, 1000);
    assert_eq!(perf.clicks_delivered, 50);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_update_performance_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env],
    );

    let fake_oracle = Address::generate(&env);
    client.update_performance(&fake_oracle, &escrow_id, &50u32, &100u64, &5u64);
}

#[test]
#[should_panic(expected = "performance threshold not met")]
fn test_release_blocked_by_performance_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    // performance_threshold = 80, but we'll record only 50
    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &80u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    client.update_performance(&oracle, &escrow_id, &50u32, &500u64, &10u64); // below threshold
    client.approve_release(&approver, &escrow_id);
    client.release_escrow(&depositor, &escrow_id); // should panic
}

// ─── hold_for_fraud ──────────────────────────────────────────────────────────

#[test]
fn test_hold_for_fraud() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let fraud_contract = Address::generate(&env);
    client.set_fraud_contract(&admin, &fraud_contract);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env],
    );

    client.hold_for_fraud(&fraud_contract, &escrow_id);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert!(matches!(escrow.state, EscrowState::Disputed));
}

#[test]
#[should_panic(expected = "escrow is disputed due to fraud")]
fn test_release_disputed_escrow_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let fraud_contract = Address::generate(&env);
    client.set_fraud_contract(&admin, &fraud_contract);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    client.hold_for_fraud(&fraud_contract, &escrow_id);
    client.approve_release(&approver, &escrow_id);
    client.release_escrow(&depositor, &escrow_id); // should panic
}

// ─── can_release ─────────────────────────────────────────────────────────────

#[test]
fn test_can_release_returns_true_when_conditions_met() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &oracle);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let approver = Address::generate(&env);
    mint(&env, &token_admin, &token_addr, &depositor, 1_000_000);

    let escrow_id = client.create_escrow(
        &depositor, &1u64, &beneficiary, &100_000i128,
        &0u64, &0u32, &999_999u64,
        &vec![&env, approver.clone()],
    );

    assert!(!client.can_release(&escrow_id)); // no approval yet
    client.approve_release(&approver, &escrow_id);
    assert!(client.can_release(&escrow_id)); // now it can
}
