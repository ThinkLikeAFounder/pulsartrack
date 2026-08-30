# Bug fixes: CEI violation in refund-processor, permissionless sync_balance in treasury-manager

## Summary

- **refund-processor**: fixed a Checks-Effects-Interactions violation in `process_refund` where the token transfer was executed before the refund status was persisted to storage, opening a re-entrancy window.
- **treasury-manager**: made `sync_balance` permissionless so any caller can reconcile the internal balance counter with the actual on-chain token balance (e.g. after a direct transfer to the contract address).
- **governance-dao**: the cross-contract `invoke_contract` call in `execute_proposal` was already present — no code change required.
- **whitelist-registry**: persistent TTL extension on individual whitelist entries was already in place for both `is_whitelisted` and `get_entry` — no code change required.

## Changes

### contracts/refund-processor/src/lib.rs
Reordered `process_refund` to follow CEI strictly:
1. Checks (status, deadline, balance)
2. Effects (set `Processed`, persist refund, remove `PendingRefund`, extend TTL)
3. Interaction (`token_client.transfer`)

### contracts/treasury-manager/src/lib.rs
Removed the `admin: Address` parameter and its auth / equality guards from `sync_balance`. The function now reads the live on-chain token balance and writes it to `state.balance` without any authorization requirement.

### contracts/treasury-manager/src/test.rs
- Updated `test_sync_balance` to call `sync_balance()` without an admin argument.
- Replaced the now-irrelevant `test_sync_balance_unauthorized` test with `test_sync_balance_permissionless`, which verifies that a direct token mint to the contract address is correctly reflected after calling `sync_balance`.

## Closes

closes #552
closes #549
closes #551
closes #550
