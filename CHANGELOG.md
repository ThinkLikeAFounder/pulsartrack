# Changelog

All notable changes to the PulsarTrack project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- **Real-time Event Streaming**: wired WebSocket events into the frontend app for live publisher, campaign, and audience activity updates (#1344).
- **Publisher Payout Event Indexing**: added event-driven indexing and tracking for publisher payout workflows (#1396).

### Changed
- **Governance UI Components**: refactored governance-related components for reuse across the application, reducing duplication.

### Fixed
- **Deployment Initialization**: `scripts/initialize.sh` now initializes all 43 contracts deployed by `scripts/deploy.sh`, fails on real invocation errors, treats already-initialized contracts as idempotent, and accepts configurable admin/config arguments.
- **Targeting Engine Tests**: updated `initialize` calls for the lifecycle contract argument and replaced the stale WASM cross-contract import with a local contract client interface so workspace builds and tests do not require prebuilt artifacts.
- **Panic Tests**: added specific expected panic messages to all previously bare `#[should_panic]` attributes across the workspace.
- **Initialize Test Coverage**: added stored-admin state assertions to every `test_initialize` that lacked one.
- **CI/Test Stability**: fixed pre-existing clippy lint, expired-campaign ledger setup, fraud-prevention lifecycle mocks, and workspace formatting drift so CI passes.
- **Campaign Orchestrator**: require active campaign status before allowing state transitions (e.g. pausing) (#1370).
- **Frontend Analytics**: include `/api` prefix in timeseries analytics backend proxy calls (#1373).
- **Access Control**: prevent lower-ranked callers from overwriting higher-ranked role assignments (#1377).
- **Budget Optimizer**: enforce budget caps when recording campaign spend and validate non-negative daily budgets.
- **Privacy Layer**: enable timelocked admin rotation for enhanced key management security (#1349).
- **Revenue Settlement**: reorder operations to follow checks-effects-interactions (CEI) pattern, ensuring accounting updates before external transfers.
- **Access Control Events**: emit stored role in `revoke_role` events for accurate tracking in indexers.
- **Oracle Integration**: keep OracleCount in sync with authorization state changes.
- **Fraud Prevention**: require publisher authentication in `verify_view` calls for stricter access control.
- **AD Registry**: use persistent storage with TTL extensions for viewer deduplication.
- **Wrapped Token Manager**: reorder mint operations to follow CEI pattern (#1362).
- **KYC Registry**: prevent duplicate `verify_kyc` calls on already-verified records.
- **Identity & Publisher Contracts**: add `reinstate` and `revoke` operations for identity and publisher lifecycle management.
- **Security**: cleared failing scheduled dependency audits and strengthened wallet/toast UI foundations.
- **Rate Limiting & Payout**: guarded write limiter, bounded reward program duration, and removed dead repository entries (#1314, #1315, #1316).
- **Prisma & Docker**: fixed prisma generation in Docker workflows and synchronized contract environment variables across compose services.

---

## [0.1.0] - 2026-09-24

### Added
- **Error Tracking / APM Integration**:
  - Integrated `@sentry/nextjs` into `frontend/` and `@sentry/node` into `backend/`.
  - Added recursive data scrubbing (`scrubEvent`) to strip tokens, passwords, authorization headers, and PII from error reports.
  - Added Sentry DSN configuration instructions to `.env.example` and `DOCKER_SETUP.md`.
- **Governance Token Contract Events**:
  - Validated and tested event emissions (`transfer`, `approve`, `mint`, `burn`, `delegate`, `revoke`) for standard indexer and explorer tracking in `contracts/governance-token`.
- **Recurring Payment Contract Events**:
  - Added event publishing (`recurring` topic with `created`, `paused`, `resumed`, `cancelled` subtopics) to all lifecycle functions in `contracts/recurring-payment`.
  - Added unit test suite `test_recurring_lifecycle_events` asserting event publication for payment creation, pausing, resuming, and cancellation.
- **Monorepo Version Alignment & Documentation**:
  - Created `VERSIONING.md` defining synchronized SemVer across frontend, backend, and all 45 Soroban contracts.
  - Aligned backend versioning to `0.1.0`.
  - Updated `CONTRIBUTING.md` with guidelines for updating versioning policy and `CHANGELOG.md`.
