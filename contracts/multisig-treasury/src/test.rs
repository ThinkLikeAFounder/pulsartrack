#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env, String,
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
    MultisigTreasuryContractClient<'_>,
    Address,
    Vec<Address>,
    Address,
    Address,
) {
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

// ─── upgrade ─────────────────────────────────────────────────────────────────

#[test]
#[should_panic] // Panics because WASM hash 0x0...0 does not exist
fn test_upgrade_authorized_reaches_wasm_check() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _, _, _) = setup(&env);

    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
    client.upgrade(&new_wasm_hash);
}

#[test]
#[should_panic]
fn test_upgrade_unauthorized() {
    let env = Env::default();
    let (client, _admin, _, _, _) = setup(&env);

    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
    client.upgrade(&new_wasm_hash);
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
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
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

    client.propose_transaction(
        &stranger,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
    );
}

#[test]
#[should_panic(expected = "invalid amount")]
fn test_propose_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, token_addr) = setup(&env);

    let proposer = signers.get(0).unwrap();
    let recipient = Address::generate(&env);

    client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &0i128,
        &make_desc(&env),
        &86_400u64,
    );
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

    let tx_id = client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
    );

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

    let tx_id = client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
    );

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

    let tx_id = client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
    );

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

    let tx_id = client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &100u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

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
    let tx_id = client.propose_transaction(
        &signer1,
        &recipient,
        &token_addr,
        &50_000i128,
        &make_desc(&env),
        &86_400u64,
    );

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

    let tx_id = client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
    );

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

    let tx_id = client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
    );

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

    let tx_id = client.propose_transaction(
        &proposer,
        &recipient,
        &token_addr,
        &10_000i128,
        &make_desc(&env),
        &86_400u64,
    );

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

// signer management

#[test]
fn test_add_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = setup(&env);

    let new_signer = Address::generate(&env);
    client.add_signer(&admin, &new_signer);

    let signers = client.get_signers();
    assert_eq!(signers.len(), 4); // was 3, now 4
    assert!(signers.contains(&new_signer));
}

#[test]
#[should_panic(expected = "already a signer")]
fn test_add_signer_duplicate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, signers, _, _) = setup(&env);

    let existing = signers.get(0).unwrap();
    client.add_signer(&admin, &existing);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_signer_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _, _) = setup(&env);

    let stranger = Address::generate(&env);
    let new_signer = Address::generate(&env);
    client.add_signer(&stranger, &new_signer);
}

#[test]
fn test_remove_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, signers, _, _) = setup(&env);

    let to_remove = signers.get(2).unwrap(); // remove 3rd; 2 remain >= required(2)
    client.remove_signer(&admin, &to_remove);

    let updated = client.get_signers();
    assert_eq!(updated.len(), 2);
    assert!(!updated.contains(&to_remove));
}

#[test]
#[should_panic(expected = "cannot remove: would breach required signers threshold")]
fn test_remove_signer_below_required() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, signers, _, _) = setup(&env);

    // required=2, removing 2 signers would leave 1 < 2
    let s1 = signers.get(0).unwrap();
    let s2 = signers.get(1).unwrap();
    client.remove_signer(&admin, &s1);
    client.remove_signer(&admin, &s2); // this should panic
}

#[test]
#[should_panic(expected = "not a signer")]
fn test_remove_signer_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = setup(&env);

    let ghost = Address::generate(&env);
    client.remove_signer(&admin, &ghost);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_remove_signer_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, signers, _, _) = setup(&env);

    let stranger = Address::generate(&env);
    let target = signers.get(0).unwrap();
    client.remove_signer(&stranger, &target);
}
