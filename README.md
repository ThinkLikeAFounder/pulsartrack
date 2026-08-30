<div align="center">

# PulsarTrack

**Privacy-preserving, blockchain-powered ad tracking on the Stellar network.**

PulsarTrack connects advertisers and publishers through **43 Soroban smart contracts** on Stellar — delivering zero-knowledge privacy, real-time bidding auctions, on-chain reputation scoring, and instant XLM settlements.

</div>

---

## Table of Contents

- [Architecture](#architecture)
- [Smart Contracts](#smart-contracts-soroban)
- [Quick Start](#quick-start)
- [Building & Deploying Contracts](#building--deploying-contracts)
- [Running the Apps](#running-the-apps)
- [Wallet Integration](#wallet-integration)
- [Key Features](#key-features)
- [Networks](#networks)
- [Environment Variables](#environment-variables)
- [PULSAR Token](#pulsar-token)
- [Contributing](#contributing)
- [License](#license)

---

## Architecture

```
PulsarTrack/
├── contracts/          # 43 Soroban smart contracts (Rust/Wasm) + common-admin lib
├── frontend/           # Next.js 16 app with @stellar/stellar-sdk + Freighter
├── backend/            # Express API + Horizon event indexer (WebSocket streaming)
├── scripts/            # Deployment & initialization scripts
└── deployments/        # Deployed contract ID records
```

A shared `common-admin` Rust library provides the two-step admin-transfer
helpers (`propose_admin` / `accept_admin`) reused across every contract. It is a
library crate, not a deployable contract, so it is not counted in the 43.

---

## Smart Contracts (Soroban)

All contracts are written in Rust, compiled to Wasm, and independently deployable
and verifiable on the Stellar ledger.

| Category              | Contracts                                                                                                                                                                                                 |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Core Ad Engine**    | `ad-registry`, `campaign-orchestrator`, `escrow-vault`, `fraud-prevention`, `payment-processor`                                                                                                            |
| **Governance**        | `governance-token` (PULSAR), `governance-dao`, `governance-core`, `timelock-executor`                                                                                                                      |
| **Publishers**        | `publisher-verification`, `publisher-network`, `publisher-reputation`                                                                                                                                      |
| **Analytics**         | `analytics-aggregator`, `campaign-analytics`, `campaign-lifecycle`                                                                                                                                         |
| **Privacy & Targeting** | `privacy-layer` (ZKP consent), `targeting-engine`, `audience-segments`                                                                                                                                   |
| **Identity & Access** | `identity-registry`, `kyc-registry`, `access-control`                                                                                                                                                      |
| **Marketplace**       | `auction-engine` (RTB), `creative-marketplace`                                                                                                                                                             |
| **Subscriptions**     | `subscription-manager`, `subscription-benefits`                                                                                                                                                            |
| **Finance**           | `liquidity-pool`, `milestone-tracker`, `multisig-treasury`, `oracle-integration`, `payout-automation`, `performance-oracle`, `recurring-payment`, `refund-processor`, `revenue-settlement`, `rewards-distributor`, `treasury-manager`, `vesting-schedule` |
| **Bridge**            | `token-bridge`, `wrapped-token`                                                                                                                                                                            |
| **Utility**           | `dispute-resolution`, `budget-optimizer`, `anomaly-detector`, `whitelist-registry`                                                                                                                         |

> **Total: 43 deployable contracts** across 11 subsystems.

---

## Quick Start

### Option 1 — Docker (recommended)

The fastest way to get the full stack running locally:

```bash
# 1. Clone the repository
git clone https://github.com/ThinkLikeAFounder/pulsartrack.git
cd pulsartrack

# 2. Copy the environment file
cp .env.example .env

# 3. Start all services (frontend, backend, PostgreSQL, Redis)
docker-compose up
```

Once the containers are healthy:

| Service      | URL                       |
| ------------ | ------------------------- |
| Frontend     | http://localhost:3000     |
| Backend API  | http://localhost:3001     |
| WebSocket    | ws://localhost:3001/ws    |
| PostgreSQL   | localhost:5432            |
| Redis        | localhost:6379            |

Docker automatically builds the frontend and backend, provisions PostgreSQL,
configures Redis for caching, and wires the services together.

```bash
docker-compose down            # stop services
docker-compose up --build      # rebuild after code changes
```

### Option 2 — Manual setup

Prefer to run services individually or develop the contracts directly?

**Prerequisites**

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/smart-contracts/getting-started/setup) (`stellar`)
- [Node.js](https://nodejs.org/) 20+
- [PostgreSQL](https://www.postgresql.org/) 14+
- [Redis](https://redis.io/) 7+

```bash
# Install the Rust Wasm target
rustup target add wasm32-unknown-unknown

# Install the Stellar CLI
cargo install --locked stellar-cli --features opt
```

---

## Building & Deploying Contracts

```bash
# 1. Build every contract to Wasm
cargo build --release --target wasm32-unknown-unknown

# 2. Set up a deployer identity and fund it from Friendbot
./scripts/setup-identity.sh

# 3. Deploy all 43 contracts to testnet
./scripts/deploy.sh

# 4. Initialize contracts (sets admin, treasury, oracle wiring, etc.)
./scripts/initialize.sh
```

Deployed contract IDs are written to `deployments/deployed-testnet.json`.

### Running the test suite

```bash
cargo test                     # all contract unit + integration tests
```

---

## Running the Apps

### Frontend

```bash
cd frontend
npm install

# Configure frontend/.env.local with your deployed contract IDs
# (copy the IDs from deployments/deployed-testnet.json after deploy.sh)

npm run dev
```

Frontend runs on **http://localhost:3000**.

### Backend

```bash
cd backend
npm install
cp .env.example .env           # configure DB, Redis, Stellar network & contract IDs
npm run dev
```

Backend runs on **http://localhost:3001** with WebSocket streaming on
**ws://localhost:3001/ws**.

---

## Wallet Integration

PulsarTrack uses [Freighter](https://www.freighter.app/) for Stellar wallet
connection. Install the Freighter browser extension, then connect from the app
header. The app detects network mismatches and prompts you to switch to the
configured network (testnet by default).

---

## Key Features

### Real-Time Bidding (RTB)

Publishers create impression slots with floor and reserve prices. Advertisers bid
in real time through the `auction-engine` contract. Winning bids settle instantly
via XLM token transfer, and losing bids are refunded on settlement.

### Privacy Layer (ZKP)

GDPR-compliant consent management with zero-knowledge proof submission for
anonymous audience segmentation. Users control exactly which data may be used, and
consent can be revoked on-chain at any time.

### Reputation System

Publisher reputation scoring (0–1000) driven by:

- Advertiser reviews, weighted by rating
- Oracle-reported uptime scores
- Slashing for fraudulent activity (rate-limited by a cooldown)
- Tiered access: Bronze → Silver → Gold → Platinum

### PULSAR Governance

On-chain DAO using the PULSAR token (SEP-41 compatible) for:

- Platform parameter changes
- Fee-structure updates
- New-feature approvals
- Timelock-protected execution of passed proposals

### XLM Settlements

Every payment uses the Soroban token interface:

- Campaign funding → escrow
- Per-impression payouts → publishers
- Platform fees → treasury

Revenue settlement splits incoming revenue **90% publisher · 5% treasury ·
2.5% platform · 2.5% burn**, with any rounding dust routed to the treasury so the
contract balance always reconciles exactly.

---

## Networks

| Network | Horizon URL                         | Soroban RPC                         |
| ------- | ----------------------------------- | ----------------------------------- |
| Testnet | https://horizon-testnet.stellar.org | https://soroban-testnet.stellar.org |
| Mainnet | https://horizon.stellar.org         | https://mainnet.sorobanrpc.com      |

Set `NEXT_PUBLIC_STELLAR_NETWORK=testnet` in `frontend/.env.local` and
`STELLAR_NETWORK=testnet` in `backend/.env`.

---

## Environment Variables

### Frontend (`frontend/.env.local`)

```env
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws
NEXT_PUBLIC_CONTRACT_CAMPAIGN_ORCHESTRATOR=<contract-id>
# ... one NEXT_PUBLIC_CONTRACT_* entry per deployed contract
```

### Backend (`backend/.env`)

```bash
cd backend
cp .env.example .env            # then fill in contract IDs and DB credentials
```

See [`backend/.env.example`](backend/.env.example) for the full list of required
variables, including the database connection, Stellar network, deployed contract
IDs, Redis, and auth configuration.

---

## PULSAR Token

| Property      | Value                                            |
| ------------- | ------------------------------------------------ |
| **Name**      | PulsarTrack Governance                           |
| **Symbol**    | PULSAR                                            |
| **Decimals**  | 7                                                |
| **Max supply**| 1,000,000,000,000 base units (7 decimals)        |
| **Standard**  | SEP-41 (Stellar token standard)                  |

The token contract supports balances, allowances, minting (capped at max supply),
burning, and on-chain vote delegation with aggregated delegated voting power.

---

## Contributing

Contributions are welcome. Please:

1. Fork the repo and create a feature branch.
2. Keep contract changes covered by unit tests (`cargo test`) and frontend/backend
   changes covered by their respective suites (`npm test`).
3. Follow the Checks-Effects-Interactions pattern for any contract that moves
   tokens, and validate all external inputs.
4. Open a pull request referencing the issue it closes.

See [`SECURITY.md`](SECURITY.md) for responsible-disclosure guidelines.

---

## License

[MIT](LICENSE)


---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines on:

- How to find and claim issues
- Setting up your development environment
- Branch naming and commit message conventions
- Running tests locally
- The CI-gated auto-merge process
- Code style guidelines

---

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.
