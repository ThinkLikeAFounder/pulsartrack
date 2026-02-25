# Contributing to PulsarTrack

Thanks for contributing. This guide covers the required tooling versions and a minimal workflow.

## Required Tooling

- Rust: 1.78.0 (pinned in `rust-toolchain.toml`)
- Soroban CLI: `stellar-cli` v22.0.0
- WASM target: `wasm32-unknown-unknown`

Install the CLI and target:

```bash
cargo install --locked stellar-cli --version 22.0.0 --features opt
rustup target add wasm32-unknown-unknown
```

Verify:

```bash
stellar --version
rustc --version
```

## Workflow

1. Create a feature branch.
2. Make focused changes with tests.
3. Run `cargo fmt`, `cargo clippy`, and `cargo test`.
4. Open a PR with a clear description and test results.
