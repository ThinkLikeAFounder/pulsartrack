#![no_std]
use soroban_sdk::{Address, Env, IntoVal, TryFromVal, Val};

pub fn cancel_admin_proposal<K>(env: &Env, admin_key: &K, pending_key: &K, current_admin: Address)
where
    K: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
{
    current_admin.require_auth();
    let stored: Address = env
        .storage()
        .instance()
        .get(admin_key)
        .expect("not initialized");
    if current_admin != stored {
        panic!("unauthorized");
    }
    env.storage().instance().remove(pending_key);
}

pub fn propose_admin<K>(
    env: &Env,
    admin_key: &K,
    pending_key: &K,
    current_admin: Address,
    new_admin: Address,
) where
    K: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
{
    current_admin.require_auth();
    let stored: Address = env
        .storage()
        .instance()
        .get(admin_key)
        .expect("not initialized");
    if current_admin != stored {
        panic!("unauthorized");
    }
    if env.storage().instance().has(pending_key) {
        panic!("pending admin proposal already exists");
    }
    env.storage().instance().set(pending_key, &(new_admin, env.ledger().sequence()));
}

pub fn accept_admin<K>(env: &Env, admin_key: &K, pending_key: &K, new_admin: Address)
where
    K: IntoVal<Env, Val> + TryFromVal<Env, Val> + Clone,
{
    new_admin.require_auth();
    let (pending, proposed_at): (Address, u32) = env
        .storage()
        .instance()
        .get(pending_key)
        .expect("no pending admin");
    if new_admin != pending {
        panic!("not pending admin");
    }

    const MIN_ADMIN_DELAY_LEDGERS: u32 = 17280;
    if env.ledger().sequence() < proposed_at + MIN_ADMIN_DELAY_LEDGERS {
        panic!("admin transfer time lock not elapsed");
    }

    env.storage().instance().set(admin_key, &new_admin);
    env.storage().instance().remove(pending_key);
}
