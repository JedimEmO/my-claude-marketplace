---
name: rust-ci-tooling
description: Use when the user asks about CI/CD for Rust projects, clippy lints, rustfmt configuration, feature flag CI matrix, workspace-level tooling, justfile or Makefile setup, GitHub Actions for Rust, or pre-commit hooks for Rust projects.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Rust CI & Tooling — Clippy, Formatting & Automation

Workspace-level tooling configuration and CI patterns. Keep it simple — use `cargo test` and standard tooling. The goal: a single `just ci` command that catches everything, and a GitHub Actions workflow that mirrors it.

## Clippy Configuration

Configure clippy at the workspace level in the root `Cargo.toml`:

```toml
[workspace.lints.clippy]
# Start with pedantic, then selectively allow the noisy ones
pedantic = { level = "warn", priority = -1 }

# These fire too often to be useful
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"

# Enforce in library crates (but allow in tests/binaries via per-crate overrides)
unwrap_used = "warn"

[workspace.lints.rust]
unsafe_code = "deny"
```

Member crates inherit with:

```toml
[lints]
workspace = true
```

Binary/test crates can relax specific lints:

```toml
[lints]
workspace = true

[lints.clippy]
unwrap_used = "allow"  # fine in binaries and tests
```

### CI command

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

The `-- -D warnings` promotes all warnings to errors in CI, ensuring clippy issues block the build.

## rustfmt Configuration

Create `.rustfmt.toml` at the workspace root:

```toml
edition = "2021"
max_width = 100
```

### CI command

```bash
cargo fmt --all -- --check
```

## Feature Flag CI Strategy

Test multiple feature configurations to catch conditional compilation issues:

```bash
# Default features (what users get)
cargo test --workspace

# No default features (catches missing feature guards)
cargo test --workspace --no-default-features

# All features (catches feature conflicts)
cargo test --workspace --all-features
```

For crates with meaningful feature combinations, add a CI matrix. Document tested combinations in the workflow.

## justfile — Workspace Task Runner

Use `just` (https://just.systems) for common workspace commands. Create a `justfile` at the workspace root:

```just
# Default: run all checks
default: check test

# Format code
fmt:
    cargo fmt --all

# Check formatting (CI)
fmt-check:
    cargo fmt --all -- --check

# Run clippy
check:
    cargo clippy --workspace --all-targets -- -D warnings

# Run tests
test:
    cargo test --workspace

# Run tests with all feature combinations
test-features:
    cargo test --workspace
    cargo test --workspace --no-default-features
    cargo test --workspace --all-features

# Full CI pipeline locally
ci: fmt-check check test-features

# Build in release mode
build:
    cargo build --workspace --release

# Clean build artifacts
clean:
    cargo clean

# Watch and run tests on change (requires cargo-watch)
watch:
    cargo watch -x 'test --workspace'
```

## GitHub Actions Workflow

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --workspace

      - name: Run tests (no default features)
        run: cargo test --workspace --no-default-features

      - name: Run tests (all features)
        run: cargo test --workspace --all-features
```

## Pre-Commit Hooks

Keep pre-commit hooks lightweight. Only run fast checks:

```bash
#!/bin/sh
# .git/hooks/pre-commit (or via pre-commit framework)
cargo fmt --all -- --check
```

**Do NOT run clippy or tests in pre-commit** — they're too slow and will frustrate the workflow. Those belong in CI.

Consider `typos-cli` for catching spelling mistakes:

```bash
cargo install typos-cli
typos  # runs on all files
```

## Useful Cargo Extensions

| Tool | Purpose | Install |
|------|---------|---------|
| `cargo-watch` | Re-run on file changes | `cargo install cargo-watch` |
| `cargo-deny` | Audit dependencies (licenses, advisories) | `cargo install cargo-deny` |
| `cargo-machete` | Find unused dependencies | `cargo install cargo-machete` |
| `typos-cli` | Spell checker for code | `cargo install typos-cli` |

## Related Skills

For workspace-level `Cargo.toml` setup and `[workspace.lints]`, see the **rust-project-setup** skill.
For test organization, see the **rust-testing** skill.
