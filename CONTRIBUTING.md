# Contributing to PulsarTrack

Welcome! Thank you for your interest in contributing to PulsarTrack. This guide will help you set up your development environment and understand our workflow.

## 🚀 Getting Started

### Prerequisites

To build and run PulsarTrack, you'll need the following installed:

- **Rust**: Version 1.75+ with the `wasm32-unknown-unknown` target.
- **Soroban CLI**: For contract interaction and deployment.
- **Node.js**: Version 20+ (LTS recommended).
- **Docker & Docker Compose**: Recommended for local infrastructure (PostgreSQL, Redis).
- **PostgreSQL**: Version 14+ (if running manually).
- **Redis**: Version 7+ (if running manually).

### Local Development Setup

#### Option 1: Docker (Recommended)

The fastest way to get started is using Docker:

```bash
# 1. Clone your fork
git clone https://github.com/YOUR_USERNAME/pulsartrack.git
cd pulsartrack

# 2. Setup environment
cp .env.example .env

# 3. Start services
docker-compose up
```

#### Option 2: Manual Setup

If you need to develop specific components:

1. **Contracts**:
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```
2. **Backend**:
   ```bash
   cd backend
   npm install
   cp .env.example .env
   npm run dev
   ```
3. **Frontend**:
   ```bash
   cd frontend
   npm install
   cp .env.local.example .env.local
   npm run dev
   ```

---

## 📂 Project Structure

- `contracts/`: 39 Soroban smart contracts written in Rust.
- `backend/`: Express.js API and event indexer.
- `frontend/`: Next.js web application.
- `scripts/`: Deployment, initialization, and utility scripts.
- `deployments/`: Historical contract ID records.

---

## 🛠 Coding Standards

### Rust (Contracts)

- Follow the official [Rust Style Guide](https://github.com/rust-lang/rust/tree/master/src/doc/style-guide).
- Use `cargo fmt` before committing.
- Run `cargo clippy --all-targets --all-features` to check for common mistakes.

### TypeScript / JavaScript (Frontend & Backend)

- Use functional components in React.
- Ensure 100% type safety in TypeScript files.
- Run `npm run lint` to check for style violations.

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat(scope): ...` for new features.
- `fix(scope): ...` for bug fixes.
- `docs(scope): ...` for documentation changes.
- `test(scope): ...` for adding/updating tests.

---

## 🧪 Testing

### Smart Contracts

Run tests for a individual contract or the whole workspace:

```bash
cargo test
# Or for a specific package:
cargo test -p pulsar-auction-engine
```

### Backend & Frontend

Both use Vitest:

```bash
npm test
```

---

## 📤 Pull Request Process

1. **Fork the repository** to your own account.
2. **Create a branch** for your feature/fix (`feat/my-feature` or `fix/issue-id`).
3. **Keep it surgical**: Avoid unnecessary deletions or sweeping refactors.
4. **Link Issues**: Include `Closes #ID` in your PR description to link the issue.
5. **CI Compliance**: Ensure all GitHub Actions pass before requesting a review.

---

## 🏷 Issue Labels

- `bug`: Something isn't working as expected.
- `feature`: New functionality.
- `enhancement`: Improvement to existing code.
- `priority`: High-impact tasks.

---

## ⚖ License

By contributing, you agree that your contributions will be licensed under the **MIT License**.
