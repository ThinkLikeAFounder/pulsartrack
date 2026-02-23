#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String, vec,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(env: &Env) -> (MultisigTreasuryContractClient, Address, Vec<Address>, Address, Address) {
    let admin = Address::generate(env);
    let signer1 = Address::generate(env);
    let signer2 = Address::generate(env);
    let signer3 = Address::generate(env);
    let signers = vec![env, signer1.clone(), signer2.clone(), signer3.clone()];

    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);

    let contract_id = env.register_contract(None, MultisigTreasuryContract);
    let client = MultisigTreasuryContractClient::new(env, &contract_id);
    client.initialize(&admin, &signers, &2u32); // require 2 of 3

    (client, admin, signers, token_admin, token_addr)
}

fn make_desc(env: &Env) -> String {
    String::from_str(env, "Pay team salary")
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let signers = vec![&env, signer.clone()];

    let contract_id = env.register_contract(None, MultisigTreasuryContract);
    let client = MultisigTreasuryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &signers, &1u32);

    let stored_signers = client.get_signers();
    assert_eq!(stored_signers.len(), 1);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let signers = vec![&env, signer.clone()];

    let contract_id = env.register_contract(None, MultisigTreasuryContract);
    let client = MultisigTreasuryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &signers, &1u32);
    client.initialize(&admin, &signers, &1u32);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let signers = vec![&env, signer.clone()];

    let contract_id = env.register_contract(None, MultisigTreasuryContract);
    let client = MultisigTreasuryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &signers, &1u32);
}

#[test]
#[should_panic(expected = "invalid required signers")]
fn test_initialize_zero_required() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let signers = vec![&env, signer.clone()];

    let contract_id = env.register_contract(None, MultisigTreasuryContract);
    let client = MultisigTreasuryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &signers, &0u32);
}

#[test]
#[should_panic(expected = "invalid required signers")]
fn test_initialize_required_exceeds_signers() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let signers = vec![&env, signer.clone()];

    let contract_id = env.register_contract(None, MultisigTreasuryContract);
    let client = MultisigTreasuryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &signers, &5u32);
}

// ─── propose_transaction ────────────────────────────────────────────────────

#[test]
fn test_propose_transaction() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let recipient = Address::generate(&env);
    let proposer = signers.get(0).unwrap();

    let tx_id = client.propose_transaction(
        &proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64,
    );

    assert_eq!(tx_id, 1);

    let tx = client.get_transaction(&tx_id).unwrap();
    assert_eq!(tx.proposer, proposer);
    assert_eq!(tx.amount, 10_000);
    assert!(matches!(tx.status, TxStatus::Pending));
    assert_eq!(tx.approvals, 0);
}

#[test]
#[should_panic(expected = "not a signer")]
fn test_propose_by_non_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, token_addr) = setup(&env);

    let stranger = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.propose_transaction(&stranger, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64);
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_propose_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let recipient = Address::generate(&env);

    client.propose_transaction(&proposer, &recipient, &token_addr, &0i128, &make_desc(&env), &86_400u64);
}

// ─── approve_transaction ─────────────────────────────────────────────────────

#[test]
fn test_approve_transaction() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let signer2 = signers.get(1).unwrap();
    let recipient = Address::generate(&env);

    let tx_id = client.propose_transaction(&proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64);

    client.approve_transaction(&proposer, &tx_id);
    let tx = client.get_transaction(&tx_id).unwrap();
    assert_eq!(tx.approvals, 1);
    assert!(matches!(tx.status, TxStatus::Pending));

    client.approve_transaction(&signer2, &tx_id);
    let tx = client.get_transaction(&tx_id).unwrap();
    assert_eq!(tx.approvals, 2);
    assert!(matches!(tx.status, TxStatus::Approved));
}

#[test]
#[should_panic(expected = "already voted")]
fn test_approve_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let recipient = Address::generate(&env);

    let tx_id = client.propose_transaction(&proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64);

    client.approve_transaction(&proposer, &tx_id);
    client.approve_transaction(&proposer, &tx_id);
}

#[test]
#[should_panic(expected = "not a signer")]
fn test_approve_by_non_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let stranger = Address::generate(&env);
    let recipient = Address::generate(&env);

    let tx_id = client.propose_transaction(&proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64);

    client.approve_transaction(&stranger, &tx_id);
}

#[test]
#[should_panic(expected = "tx expired")]
fn test_approve_expired_transaction() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let recipient = Address::generate(&env);

    let tx_id = client.propose_transaction(&proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &100u64);

    env.ledger().with_mut(|li| { li.timestamp = 200; });

    client.approve_transaction(&proposer, &tx_id);
}

// ─── execute_transaction ─────────────────────────────────────────────────────

#[test]
fn test_execute_transaction() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone()];
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, MultisigTreasuryContract);
    let client = MultisigTreasuryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &signers, &2u32);

    // Fund the treasury contract
    mint(&env, &token_addr, &contract_id, 1_000_000);

    let recipient = Address::generate(&env);
    let tx_id = client.propose_transaction(&signer1, &recipient, &token_addr, &50_000i128, &make_desc(&env), &86_400u64);

    client.approve_transaction(&signer1, &tx_id);
    client.approve_transaction(&signer2, &tx_id);
    client.execute_transaction(&signer1, &tx_id);

    let tx = client.get_transaction(&tx_id).unwrap();
    assert!(matches!(tx.status, TxStatus::Executed));
    assert!(tx.executed_at.is_some());

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&recipient), 50_000);
    assert_eq!(tc.balance(&contract_id), 950_000);
}

#[test]
#[should_panic(expected = "tx not approved")]
fn test_execute_pending_transaction() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let recipient = Address::generate(&env);

    let tx_id = client.propose_transaction(&proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64);

    // Not yet approved → should panic
    client.execute_transaction(&proposer, &tx_id);
}

// ─── reject_transaction ─────────────────────────────────────────────────────

#[test]
fn test_reject_transaction() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let signer2 = signers.get(1).unwrap();
    let recipient = Address::generate(&env);

    let tx_id = client.propose_transaction(&proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64);

    // 3 signers, required=2 → need 2+ rejections to get Rejected status
    client.reject_transaction(&proposer, &tx_id);
    let tx = client.get_transaction(&tx_id).unwrap();
    assert_eq!(tx.rejections, 1);
    assert!(matches!(tx.status, TxStatus::Pending)); // still pending: max_possible_approvals = 2 >= required

    client.reject_transaction(&signer2, &tx_id);
    let tx = client.get_transaction(&tx_id).unwrap();
    assert_eq!(tx.rejections, 2);
    assert!(matches!(tx.status, TxStatus::Rejected)); // max_possible_approvals = 1 < required=2
}

#[test]
#[should_panic(expected = "not a signer")]
fn test_reject_by_non_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let stranger = Address::generate(&env);
    let recipient = Address::generate(&env);

    let tx_id = client.propose_transaction(&proposer, &recipient, &token_addr, &10_000i128, &make_desc(&env), &86_400u64);

    client.reject_transaction(&stranger, &tx_id);
}

// ─── read-only ───────────────────────────────────────────────────────────────

#[test]
fn test_get_transaction_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _) = setup(&env);

    assert!(client.get_transaction(&999u64).is_none());
}

#[test]
fn test_get_signers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, _) = setup(&env);

    let stored = client.get_signers();
    assert_eq!(stored.len(), signers.len());
}
