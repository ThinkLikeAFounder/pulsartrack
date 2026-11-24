# Monorepo Versioning Policy — PulsarTrack

This document describes the versioning strategy and release policies for PulsarTrack.

---

## 1. Versioning Model: Synchronized Semantic Versioning

PulsarTrack uses **Synchronized Semantic Versioning (SemVer 2.0.0)** across all components of the monorepo:

- **Frontend**: `frontend/package.json`
- **Backend**: `backend/package.json`
- **Smart Contracts**: `contracts/*/Cargo.toml` (all 45 Soroban smart contracts)

### Version Alignment
All sub-packages and contracts share a single release version sequence (e.g. `0.1.0` -> `0.2.0` -> `1.0.0`). When a major release or milestone is tagged (e.g. initial production launch `1.0.0`), all workspace components move together to the target version.

---

## 2. Release Lifecycle

- **Pre-launch / Initial Development (`0.x.y`)**: Rapid feature development and contract iterations. Breaking changes may occur between minor versions.
- **Production Launch (`1.0.0`)**: Formal production launch. Post `1.0.0`, versioning strictly follows SemVer:
  - **MAJOR (`x.0.0`)**: Breaking API or smart contract state schema/interface changes.
  - **MINOR (`0.x.0`)**: Backwards-compatible feature additions and new contract endpoints.
  - **PATCH (`0.0.x`)**: Backwards-compatible bug fixes and internal refactoring.

---

## 3. Maintenance & Documentation Requirements

1. **CHANGELOG Updates**: Every pull request introducing significant features, contract modifications, security fixes, or breaking changes must add an entry under `[Unreleased]` in [`CHANGELOG.md`](CHANGELOG.md).
2. **Version Bumps**: Tagged releases bump `version` in `frontend/package.json`, `backend/package.json`, and all `contracts/*/Cargo.toml` files simultaneously via workspace scripts.
