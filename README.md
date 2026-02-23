# PulsarTrack

**Privacy-preserving, blockchain-powered ad tracking on the Stellar network.**

PulsarTrack connects advertisers and publishers through 39 Soroban smart contracts on Stellar. It provides zero-knowledge privacy, real-time bidding auctions, on-chain reputation scoring, and instant XLM settlements.

---

## Architecture

```
PulsarTrack/
├── contracts/          # 39 Soroban smart contracts (Rust/Wasm)
├── frontend/           # Next.js app with @stellar/stellar-sdk
├── backend/            # Express API + Horizon event indexer
├── scripts/            # Deployment & initialization scripts
└── deployments/        # Deployed contract ID records
```

---

## Smart Contracts (Soroban)

| Category | Contracts |
|---|---|
| **Core Ad** | ad-registry, campaign-orchestrator, escrow-vault, fraud-prevention, payment-processor |
| **Governance** | governance-token (PULSAR), governance-dao, governance-core, timelock-executor |
| **Publishers** | publisher-verification, publisher-network, publisher-reputation |
| **Analytics** | analytics-aggregator, campaign-analytics, campaign-lifecycle |
| **Privacy** | privacy-layer (ZKP consent), targeting-engine, audience-segments |
| **Identity** | identity-registry, kyc-registry |
| **Marketplace** | auction-engine (RTB), creative-marketplace |
| **Subscriptions** | subscription-manager, subscription-benefits |
| **Finance** | liquidity-pool, milestone-tracker, multisig-treasury, oracle-integration, payout-automation, performance-oracle, recurring-payment, refund-processor, revenue-settlement, rewards-distributor |
| **Bridge** | token-bridge, wrapped-token |
| **Utility** | dispute-resolution, budget-optimizer, anomaly-detector |

---

## Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/smart-contracts/getting-started/setup) (`stellar`)
- [Node.js](https://nodejs.org/) 20+
- [PostgreSQL](https://www.postgresql.org/) 14+

### Install Rust WASM target

```bash
rustup target add wasm32-unknown-unknown
```

### Install Stellar CLI

```bash
cargo install --locked stellar-cli --features opt
```

---

## Quick Start

### 1. Build all contracts

```bash
cargo build --release --target wasm32-unknown-unknown
```

### 2. Deploy to testnet

```bash
# Setup identity and fund from Friendbot
./scripts/setup-identity.sh

# Deploy all 39 contracts
./scripts/deploy.sh

# Initialize contracts (sets admin, treasury, etc.)
./scripts/initialize.sh
```

### 3. Start the frontend

```bash
cd frontend
npm install

# 1. Copy the example environment file
cp .env.local.example .env.local

# 2. Add your deployed contract IDs to .env.local
# You can find these in deployments/deployed-testnet.json after running deploy.sh

# 3. Start the dev server
npm run dev
```

Frontend runs on http://localhost:3000

### 4. Start the backend

```bash
cd backend
npm install
cp .env.example .env              # Configure DB and contract IDs
npm run dev
```

Backend runs on http://localhost:4000 with WebSocket on ws://localhost:4000/ws

---

## Wallet Integration

PulsarTrack uses [Freighter](https://www.freighter.app/) for Stellar wallet connection.

Install the Freighter browser extension, then connect from the app header.

---

## Key Features

### Real-Time Bidding (RTB)
Publishers create impression slots with floor/reserve prices. Advertisers bid in real-time via the `auction-engine` contract. Winning bids settle via XLM token transfer.

### Privacy Layer (ZKP)
GDPR-compliant consent management with zero-knowledge proof submission for anonymous audience segmentation. Users control exactly what data is used.

### Reputation System
Publisher reputation scoring (0-1000) with:
- Advertiser reviews (weighted by rating)
- Oracle-reported uptime scores
- Slashing for fraudulent activity
- Tiered access: Bronze → Silver → Gold → Platinum

### PULSAR Governance
On-chain DAO using PULSAR token (SEP-41 compatible) for:
- Platform parameter changes
- Fee structure updates
- New feature approvals
- Timelock-protected execution

### XLM Settlements
All payments use Soroban token interface:
- Campaign funding → escrow
- Per-impression payouts → publishers
- Platform fees → treasury
- Revenue settlement: 90% publisher, 5% treasury, 2.5% platform, 2.5% burn

---

## Networks

| Network | Horizon URL | Soroban RPC |
|---|---|---|
| Testnet | https://horizon-testnet.stellar.org | https://soroban-testnet.stellar.org |
| Mainnet | https://horizon.stellar.org | https://mainnet.sorobanrpc.com |

Set `NEXT_PUBLIC_NETWORK=testnet` in `frontend/.env.local` and `STELLAR_NETWORK=testnet` in `backend/.env`.

---

## Environment Variables

### Frontend (`frontend/.env.local`)

```env
NEXT_PUBLIC_NETWORK=testnet
NEXT_PUBLIC_WS_URL=ws://localhost:4000/ws
NEXT_PUBLIC_API_URL=http://localhost:4000
NEXT_PUBLIC_CONTRACT_CAMPAIGN_ORCHESTRATOR=<contract-id>
# ... (see frontend/.env.local for full list)
```

### Backend (`backend/.env`)

```bash
cd backend
cp .env.example .env   # then fill in contract IDs and DB credentials
```

See [`backend/.env.example`](backend/.env.example) for the full list of required variables including database, Stellar network, all 17 contract IDs, Redis, and auth configuration.

---

## Token: PULSAR

- **Name**: PulsarTrack Governance
- **Symbol**: PULSAR
- **Decimals**: 7
- **Max Supply**: 1,000,000,000,000 (1M tokens at 7 decimals)
- **Standard**: SEP-41 (Stellar token standard)

---

## License

MIT
