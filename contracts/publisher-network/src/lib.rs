//! PulsarTrack - Publisher Network (Soroban)
//! Manages the decentralized publisher network on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct NetworkNode {
    pub publisher: Address,
    pub node_type: NodeType,
    pub capacity: u64, // Max impressions per day
    pub min_cpm: i128, // Minimum cost per mille in stroops
    pub geographic_zone: String,
    pub content_categories: Vec<String>,
    pub is_active: bool,
    pub joined_at: u64,
    pub last_heartbeat: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum NodeType {
    Standard,
    Premium,
    Enterprise,
}

#[contracttype]
#[derive(Clone)]
pub struct NetworkStats {
    pub total_nodes: u64,
    pub active_nodes: u64,
    pub total_capacity: u64,
    pub total_impressions_served: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    FraudContract,
    NodeCount,
    NetworkStats,
    Node(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

#[contract]
pub struct PublisherNetworkContract;

#[contractimpl]
impl PublisherNetworkContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NodeCount, &0u64);
        env.storage().instance().set(
            &DataKey::NetworkStats,
            &NetworkStats {
                total_nodes: 0,
                active_nodes: 0,
                total_capacity: 0,
                total_impressions_served: 0,
                last_updated: env.ledger().timestamp(),
            },
        );
    }

    pub fn join_network(
        env: Env,
        publisher: Address,
        node_type: NodeType,
        capacity: u64,
        min_cpm: i128,
        geographic_zone: String,
        content_categories: Vec<String>,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        publisher.require_auth();

        if env
            .storage()
            .persistent()
            .has(&DataKey::Node(publisher.clone()))
        {
            panic!("already in network");
        }

        let node = NetworkNode {
            publisher: publisher.clone(),
            node_type,
            capacity,
            min_cpm,
            geographic_zone,
            content_categories,
            is_active: true,
            joined_at: env.ledger().timestamp(),
            last_heartbeat: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Node(publisher.clone());
        env.storage().persistent().set(&_ttl_key, &node);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NodeCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::NodeCount, &(count + 1));

        let mut stats: NetworkStats = env
            .storage()
            .instance()
            .get(&DataKey::NetworkStats)
            .unwrap();
        stats.total_nodes += 1;
        stats.active_nodes += 1;
        stats.total_capacity += capacity;
        env.storage().instance().set(&DataKey::NetworkStats, &stats);

        env.events().publish(
            (symbol_short!("network"), symbol_short!("joined")),
            publisher,
        );
    }

    pub fn heartbeat(env: Env, publisher: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        publisher.require_auth();

        let mut node: NetworkNode = env
            .storage()
            .persistent()
            .get(&DataKey::Node(publisher.clone()))
            .expect("not in network");

        node.last_heartbeat = env.ledger().timestamp();
        let _ttl_key = DataKey::Node(publisher);
        env.storage().persistent().set(&_ttl_key, &node);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn deactivate(env: Env, publisher: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        publisher.require_auth();

        Self::_deactivate_node(&env, publisher);
    }

    pub fn set_fraud_contract(env: Env, admin: Address, fraud_contract: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage()
            .instance()
            .set(&DataKey::FraudContract, &fraud_contract);
    }

    pub fn suspend_publisher(env: Env, fraud_contract: Address, publisher: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        fraud_contract.require_auth();

        let stored_fraud: Address = env
            .storage()
            .instance()
            .get(&DataKey::FraudContract)
            .expect("fraud contract not set");
        if fraud_contract != stored_fraud {
            panic!("unauthorized fraud contract");
        }

        Self::_deactivate_node(&env, publisher);
    }

    fn _deactivate_node(env: &Env, publisher: Address) {
        let mut node: NetworkNode = env
            .storage()
            .persistent()
            .get(&DataKey::Node(publisher.clone()))
            .expect("not in network");

        if !node.is_active {
            return;
        }

        node.is_active = false;
        let _ttl_key = DataKey::Node(publisher);
        env.storage().persistent().set(&_ttl_key, &node);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let mut stats: NetworkStats = env
            .storage()
            .instance()
            .get(&DataKey::NetworkStats)
            .unwrap();
        if stats.active_nodes > 0 {
            stats.active_nodes -= 1;
        }
        env.storage().instance().set(&DataKey::NetworkStats, &stats);
    }

    pub fn record_impression(env: Env, _publisher: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let mut stats: NetworkStats = env
            .storage()
            .instance()
            .get(&DataKey::NetworkStats)
            .unwrap();
        stats.total_impressions_served += 1;
        stats.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::NetworkStats, &stats);
    }

    pub fn get_node(env: Env, publisher: Address) -> Option<NetworkNode> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Node(publisher))
    }

    pub fn get_network_stats(env: Env) -> NetworkStats {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::NetworkStats)
            .unwrap_or(NetworkStats {
                total_nodes: 0,
                active_nodes: 0,
                total_capacity: 0,
                total_impressions_served: 0,
                last_updated: 0,
            })
    }

    pub fn get_node_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::NodeCount)
            .unwrap_or(0)
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
