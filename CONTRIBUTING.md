# Contributing to PulsarTrack

Thank you for your interest in contributing to PulsarTrack! This guide will help you get started with the contribution process.

## Getting Started

### Finding an Issue to Work On

1. Browse the [Issues](https://github.com/greyforreal/pulsartrack/issues) page
2. Look for issues labeled `good first issue` or `help wanted`
3. Read the issue description carefully to understand the requirements
4. Comment on the issue to let maintainers know you'd like to work on it
5. Wait for confirmation from a maintainer before starting work

### Setting Up Your Development Environment

#### Prerequisites

- Rust (latest stable) with wasm32-unknown-unknown target
- Node.js 20.x or higher
- npm or yarn package manager
- Git

#### Clone the Repository

```bash
git clone https://github.com/greyforreal/pulsartrack.git
cd pulsartrack
```

#### Install Dependencies

**Contracts:**
```bash
cargo build --workspace
```

**Backend:**
```bash
cd backend
npm install
```

**Frontend:**
```bash
cd frontend
npm install
```

## Development Workflow

### Branch Naming Conventions

Create a new branch from `main` using this naming pattern:

- `feat/short-description` - For new features
- `fix/short-description` - For bug fixes
- `docs/short-description` - For documentation changes
- `refactor/short-description` - For code refactoring
- `test/short-description` - For test additions or fixes

Example: `fix/wallet-connection-timeout`

### Making Changes

1. Create a new branch:
   ```bash
   git checkout -b fix/your-issue-description
   ```

2. Make your changes following our code style guidelines

3. Run tests locally before committing:
   ```bash
   # Test contracts
   cargo test --workspace
   
   # Test backend
   cd backend
   npm test
   
   # Test frontend
   cd frontend
   npm test
   npm run test:e2e
   ```

4. Commit your changes with a clear commit message:
   ```bash
   git add .
   git commit -m "fix: description of what was fixed"
   ```

### Commit Message Guidelines

Follow the conventional commits format:

- `feat:` - A new feature
- `fix:` - A bug fix
- `docs:` - Documentation changes
- `test:` - Adding or updating tests
- `refactor:` - Code refactoring without changing functionality
- `chore:` - Maintenance tasks

Example: `fix: resolve wallet connection timeout issue`

## Running Tests Locally

### Contracts

```bash
# Build all contracts
cargo build --workspace

# Run all contract tests
cargo test --workspace

# Run tests for a specific contract
cargo test -p campaign-contract
```

### Backend

```bash
cd backend

# Type check
npm run typecheck

# Run tests
npm test

# Build
npm run build
```

### Frontend

```bash
cd frontend

# Type check
npm run typecheck

# Lint code
npm run lint

# Run unit tests
npm test

# Run E2E tests
npm run test:e2e

# Build
npm run build
```

## Pull Request Process

### Before Submitting

1. Ensure all tests pass locally
2. Update documentation if you've changed functionality
3. Make sure your branch is up to date with `main`:
   ```bash
   git fetch origin
   git rebase origin/main
   ```

### Submitting Your PR

1. Push your branch to your fork:
   ```bash
   git push origin your-branch-name
   ```

2. Go to the repository on GitHub and click "New Pull Request"

3. Fill out the PR template with:
   - Clear description of the changes
   - Reference to the issue number (e.g., "Closes #123")
   - Any relevant testing notes

4. Submit the pull request

### CI-Gated Auto-Merge

This repository uses CI-gated auto-merge. Your PR must meet these requirements:

**Required Status Checks:**
- Contracts CI - All contract tests must pass
- Backend CI - Backend type check, build, and tests must pass
- Frontend CI - Frontend type check, lint, build, and tests must pass
- E2E Tests - End-to-end tests must pass

**Additional Requirements:**
- PR must be up to date with the `main` branch
- No merge conflicts
- At least one approval from a maintainer

Once all checks pass and you have approval, the PR will be automatically merged.

### After Your PR is Merged

1. Delete your local branch:
   ```bash
   git branch -d your-branch-name
   ```

2. Update your local main branch:
   ```bash
   git checkout main
   git pull origin main
   ```

## Code Style Guidelines

### Rust (Contracts)

- Follow standard Rust formatting with `rustfmt`
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Run `cargo clippy` before committing

### TypeScript (Backend/Frontend)

- Follow the existing ESLint configuration
- Use TypeScript strict mode
- Prefer functional components in React
- Add JSDoc comments for complex functions

## Getting Help

If you have questions or need help:

1. Check existing issues and discussions
2. Ask in the issue you're working on
3. Create a new discussion in the GitHub Discussions section

## Code of Conduct

Please be respectful and professional in all interactions. We are building an inclusive community where everyone feels welcome to contribute.

Thank you for contributing to PulsarTrack!
