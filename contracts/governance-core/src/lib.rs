//! PulsarTrack - Governance Core (Soroban)
//! Core governance parameters, roles, and access control on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum Role {
    Admin,
    Moderator,
    Oracle,
    Operator,
}

#[contracttype]
#[derive(Clone)]
pub struct RoleGrant {
    pub role: Role,
    pub granted_by: Address,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub struct GovernanceParams {
    pub min_proposal_threshold: i128,
    pub voting_period_ledgers: u32,
    pub quorum_pct: u32,
    pub pass_threshold_pct: u32,
    pub timelock_ledgers: u32,
    pub max_active_proposals: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    GovernanceParams,
    RoleGrant(Address, Role),
    RoleCount(Role),
    ActiveProposalCount,
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 34_560;
const PERSISTENT_BUMP_AMOUNT: u32 = 259_200;

#[contract]
pub struct GovernanceCoreContract;

#[contractimpl]
impl GovernanceCoreContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        let params = GovernanceParams {
            min_proposal_threshold: 1_000_000,
            voting_period_ledgers: 10_000,
            quorum_pct: 10,
            pass_threshold_pct: 51,
            timelock_ledgers: 1_000,
            max_active_proposals: 20,
        };
        env.storage()
            .instance()
            .set(&DataKey::GovernanceParams, &params);
        env.storage()
            .instance()
            .set(&DataKey::ActiveProposalCount, &0u32);
    }

    pub fn grant_role(
        env: Env,
        admin: Address,
        account: Address,
        role: Role,
        expires_at: Option<u64>,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let grant = RoleGrant {
            role: role.clone(),
            granted_by: admin,
            granted_at: env.ledger().timestamp(),
            expires_at,
        };

        let _ttl_key = DataKey::RoleGrant(account.clone(), role.clone());
        env.storage().persistent().set(&_ttl_key, &grant);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::RoleCount(role.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::RoleCount(role), &(count + 1));

        env.events()
            .publish((symbol_short!("role"), symbol_short!("granted")), account);
    }

    pub fn revoke_role(env: Env, admin: Address, account: Address, role: Role) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        env.storage()
            .persistent()
            .remove(&DataKey::RoleGrant(account.clone(), role.clone()));

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::RoleCount(role.clone()))
            .unwrap_or(0);
        if count > 0 {
            env.storage()
                .instance()
                .set(&DataKey::RoleCount(role), &(count - 1));
        }
    }

    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(grant) = env
            .storage()
            .persistent()
            .get::<DataKey, RoleGrant>(&DataKey::RoleGrant(account, role))
        {
            if let Some(expires) = grant.expires_at {
                expires > env.ledger().timestamp()
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn update_params(env: Env, admin: Address, params: GovernanceParams) {
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
            .set(&DataKey::GovernanceParams, &params);
    }

    pub fn get_params(env: Env) -> GovernanceParams {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::GovernanceParams)
            .expect("not initialized")
    }

    pub fn get_role_grant(env: Env, account: Address, role: Role) -> Option<RoleGrant> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::RoleGrant(account, role))
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
