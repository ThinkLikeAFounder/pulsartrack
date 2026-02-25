//! PulsarTrack - Token Bridge (Soroban)
//! Cross-chain token bridge for multi-network ad campaign funding on Stellar.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env, String,
};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum BridgeStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
}

#[contracttype]
#[derive(Clone)]
pub struct BridgeDeposit {
    pub deposit_id: u64,
    pub sender: Address,
    pub recipient_chain: String,
    pub recipient_address: String, // Address on target chain
    pub token: Address,
    pub amount: i128,
    pub bridge_fee: i128,
    pub status: BridgeStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub tx_hash: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    DepositCounter,
    BridgeFeesBps,
    SupportedChain(String),
    DailyVolume(String, u64), // (chain, day_number) -> tracks daily volume per chain
    Deposit(u64),
    RelayerAddress,
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct TokenBridgeContract;

#[contractimpl]
impl TokenBridgeContract {
    pub fn initialize(env: Env, admin: Address, relayer: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::RelayerAddress, &relayer);
        env.storage()
            .instance()
            .set(&DataKey::DepositCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::BridgeFeesBps, &50u32); // 0.5%
    }

    pub fn add_supported_chain(env: Env, admin: Address, chain: String, max_daily_limit: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        let _ttl_key = DataKey::SupportedChain(chain);
        env.storage().persistent().set(&_ttl_key, &max_daily_limit);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn deposit_for_bridge(
        env: Env,
        sender: Address,
        token: Address,
        amount: i128,
        recipient_chain: String,
        recipient_address: String,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        sender.require_auth();

        if amount <= 0 {
            panic!("invalid amount");
        }

        // Verify chain is supported and read max daily limit
        let max_daily_limit: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::SupportedChain(recipient_chain.clone()))
            .expect("chain not supported");

        // Enforce daily transfer limit per chain
        let current_day = env.ledger().timestamp() / 86_400;
        let daily_volume_key = DataKey::DailyVolume(recipient_chain.clone(), current_day);
        let current_daily_volume: i128 = env
            .storage()
            .persistent()
            .get(&daily_volume_key)
            .unwrap_or(0);

        if current_daily_volume + amount > max_daily_limit {
            panic!("daily transfer limit exceeded for chain");
        }

        let fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::BridgeFeesBps)
            .unwrap_or(50);
        let bridge_fee = (amount * fee_bps as i128) / 10_000;
        let net_amount = amount - bridge_fee;

        // Lock tokens in bridge contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        // Update daily volume for this chain
        let new_daily_volume = current_daily_volume + amount;
        env.storage()
            .persistent()
            .set(&daily_volume_key, &new_daily_volume);
        env.storage().persistent().extend_ttl(
            &daily_volume_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DepositCounter)
            .unwrap_or(0);
        let deposit_id = counter + 1;

        let deposit = BridgeDeposit {
            deposit_id,
            sender: sender.clone(),
            recipient_chain,
            recipient_address,
            token,
            amount: net_amount,
            bridge_fee,
            status: BridgeStatus::Pending,
            created_at: env.ledger().timestamp(),
            completed_at: None,
            tx_hash: None,
        };

        let _ttl_key = DataKey::Deposit(deposit_id);
        env.storage().persistent().set(&_ttl_key, &deposit);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::DepositCounter, &deposit_id);

        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("deposit")),
            (deposit_id, sender, net_amount),
        );

        deposit_id
    }

    pub fn confirm_bridge(env: Env, relayer: Address, deposit_id: u64, tx_hash: BytesN<32>) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        relayer.require_auth();
        let stored_relayer: Address = env
            .storage()
            .instance()
            .get(&DataKey::RelayerAddress)
            .unwrap();
        if relayer != stored_relayer {
            panic!("unauthorized relayer");
        }

        let mut deposit: BridgeDeposit = env
            .storage()
            .persistent()
            .get(&DataKey::Deposit(deposit_id))
            .expect("deposit not found");

        if deposit.status != BridgeStatus::Pending {
            panic!("not pending");
        }

        deposit.status = BridgeStatus::Completed;
        deposit.completed_at = Some(env.ledger().timestamp());
        deposit.tx_hash = Some(tx_hash);

        let _ttl_key = DataKey::Deposit(deposit_id);
        env.storage().persistent().set(&_ttl_key, &deposit);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("confirmed")),
            deposit_id,
        );
    }

    pub fn refund_deposit(env: Env, admin: Address, deposit_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut deposit: BridgeDeposit = env
            .storage()
            .persistent()
            .get(&DataKey::Deposit(deposit_id))
            .expect("deposit not found");

        if deposit.status != BridgeStatus::Pending && deposit.status != BridgeStatus::Failed {
            panic!("cannot refund");
        }

        let total_refund = deposit.amount + deposit.bridge_fee;
        let token_client = token::Client::new(&env, &deposit.token);
        token_client.transfer(
            &env.current_contract_address(),
            &deposit.sender,
            &total_refund,
        );

        deposit.status = BridgeStatus::Refunded;
        let _ttl_key = DataKey::Deposit(deposit_id);
        env.storage().persistent().set(&_ttl_key, &deposit);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_deposit(env: Env, deposit_id: u64) -> Option<BridgeDeposit> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Deposit(deposit_id))
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        pulsar_common_admin::propose_admin(
            &env,
            &DataKey::Admin,
            &DataKey::PendingAdmin,
            current_admin,
            new_admin,
        );
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        pulsar_common_admin::accept_admin(&env, &DataKey::Admin, &DataKey::PendingAdmin, new_admin);
    }
}

mod test;
