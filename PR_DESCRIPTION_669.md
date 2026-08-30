## 📌 Summary
Fixes a CEI (Checks-Effects-Interactions) violation in `file_dispute` by persisting the dispute record before any token transfers.

## 🎯 Purpose / Motivation
Previously, `file_dispute` transferred both the filing fee and claim amount to the contract before creating and storing the `Dispute` record. If any operation between the second transfer and storage panicked, funds would be locked in the contract with no dispute record to reference. Re-entrant callbacks during transfers also could not inspect dispute state.

## 🛠️ Changes Made
- Reordered `file_dispute` in `contracts/dispute-resolution/src/lib.rs`:
  1. Build the `Dispute` struct
  2. Persist to persistent storage and bump the dispute counter
  3. Transfer filing fee (if any)
  4. Transfer claim amount to lock funds
  5. Emit the `dispute/filed` event
- No API or storage schema changes; behavior is unchanged on the happy path.

## 🧪 How to Test
1. From the repo root, run dispute-resolution contract tests:
   ```bash
   cargo test -p dispute-resolution
   ```
2. Expected: all existing tests pass, including `test_file_dispute`, `test_file_multiple_disputes`, and resolve/assign flows that depend on filed disputes.
3. Manual verification (optional): file a dispute on testnet and confirm the dispute record exists in storage before token balances change.

## 📸 Screenshots (if applicable)
N/A — contract-only change.

## ⚠️ Breaking Changes
- None.

## 🔗 Related Issues
Closes ThinkLikeAFounder/pulsartrack#669

## ✅ Checklist
- [x] Code builds successfully
- [x] Tests added/updated
- [x] No console errors
- [ ] Documentation updated (if needed)
