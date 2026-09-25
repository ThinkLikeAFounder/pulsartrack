# Contract Upgrade Strategy

## Decision: Upgradeable via Admin-Gated Entrypoint

The PulsarTrack fund-custody contracts (escrow-vault, treasury-manager, liquidity-pool, payment-processor) are **upgradeable** via an admin-gated `upgrade(new_wasm_hash)` entrypoint, routed through the existing timelock-executor and multisig-treasury infrastructure.

### Rationale

- These contracts hold real user funds; an immutable design means any bug discovered after launch can only be answered by pausing and redeploying to a new address, requiring manual migration of every dependent contract, backend env var, and user balance.
- The repository already contains `timelock-executor` and `multisig-treasury`, which provide the governance controls needed to gate upgrades safely.
- Soroban's `env.deployer().update_current_contract_wasm(hash)` is the standard mechanism for contract upgrades.

## How It Works

1. **Admin-only entrypoint**: Each upgradeable contract exposes an `upgrade(new_wasm_hash: BytesN<32>)` function.
2. **Authorization**: Only the contract's admin address can call it. The admin is expected to be the multisig-treasury contract.
3. **Timelock**: The multisig-treasury enforces a timelock delay before execution, giving stakeholders time to review the proposed upgrade.
4. **Event emission**: Every `upgrade` call emits an `Upgraded { new_wasm_hash }` event for indexer visibility and audit trails.
5. **State preservation**: Soroban preserves all persistent storage across WASM upgrades — no data migration is needed.

## Implementation Pattern

```rust
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
    let admin = storage::get_admin(&env)?;
    admin.require_auth();

    env.deployer().update_current_contract_wasm(new_wasm_hash);

    events::Upgraded {
        new_wasm_hash: new_wasm_hash.clone(),
    }
    .publish(&env);

    Ok(())
}
```

## Rollback Plan

If an upgrade introduces a regression:
1. The multisig-treasury can immediately propose a new upgrade reverting to the previous WASM hash.
2. The timelock delay applies again, but operators can expedite via the multisig quorum.
3. No on-chain state is lost — storage is preserved regardless of WASM version.

## What Does NOT Upgrade

- On-chain contract addresses (each deployment has a fixed address).
- Backend/frontend environment variables pointing to contract IDs.
- User wallets or Stellar accounts.

## Audit Trail

- WASM hashes for every deployment are tracked in `deployments/` and in the `Upgraded` event.
- The timelock-executor logs all proposed and executed upgrades.
- Indexers can reconstruct the full upgrade history from `Upgraded` events across all contracts.
