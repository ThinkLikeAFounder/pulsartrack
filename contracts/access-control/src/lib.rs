//! PulsarTrack - Access Control (Soroban)
//! Role-based access control with hierarchical role revocation on Stellar.
//!
//! Events:
//! - ("access", "granted"): [account: Address, role: Role]
//! - ("access", "revoked"): [account: Address, role: Role]

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum Role {
    Operator,
    Moderator,
    Admin,
    SuperAdmin,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    Role(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 34_560;
const PERSISTENT_BUMP_AMOUNT: u32 = 259_200;

/// Returns a numeric rank for the role: higher = more privileged.
fn role_rank(role: &Role) -> u32 {
    match role {
        Role::Operator => 1,
        Role::Moderator => 2,
        Role::Admin => 3,
        Role::SuperAdmin => 4,
    }
}

#[contract]
pub struct AccessControlContract;

#[contractimpl]
impl AccessControlContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        let _ttl_key = DataKey::Role(admin.clone());
        env.storage()
            .persistent()
            .set(&_ttl_key, &Role::SuperAdmin);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn grant_role(env: Env, caller: Address, account: Address, role: Role) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let caller_role = Self::_get_role(&env, &caller);
        if role_rank(&caller_role) <= role_rank(&role) {
            panic!("cannot grant a role equal to or higher than your own");
        }

        let _ttl_key = DataKey::Role(account.clone());
        env.storage().persistent().set(&_ttl_key, &role);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("access"), symbol_short!("granted")),
            (account, role),
        );
    }

    pub fn revoke_role(env: Env, caller: Address, target: Address, role: Role) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let caller_role = Self::_get_role(&env, &caller);
        let target_role = Self::_get_role(&env, &target);

        // Enforce hierarchy: callers can only revoke roles strictly below their own.
        // This prevents admins from revoking super-admins, or accidental self-lockout.
        if role_rank(&caller_role) <= role_rank(&target_role) {
            panic!("cannot revoke a role equal to or higher than your own");
        }

        env.storage()
            .persistent()
            .remove(&DataKey::Role(target.clone()));

        env.events().publish(
            (symbol_short!("access"), symbol_short!("revoked")),
            (target, role),
        );
    }

    pub fn get_role(env: Env, account: Address) -> Role {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        Self::_get_role(&env, &account)
    }

    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if let Some(stored) = env
            .storage()
            .persistent()
            .get::<DataKey, Role>(&DataKey::Role(account))
        {
            stored == role
        } else {
            false
        }
    }

    fn _get_role(env: &Env, account: &Address) -> Role {
        env.storage()
            .persistent()
            .get::<DataKey, Role>(&DataKey::Role(account.clone()))
            .unwrap_or_else(|| panic!("account has no role"))
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
