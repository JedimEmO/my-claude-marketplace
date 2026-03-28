---
name: rust-project-setup
description: Use when the user asks to scaffold a new Rust project, set up a Cargo workspace, configure Cargo.toml, manage workspace dependencies, set up feature flags, decide on crate boundaries, or asks about when to split a single crate into multiple crates.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Rust Project Setup — Workspace Scaffolding & Crate Layout

Opinionated guide for structuring Rust projects. Start simple, split when there's a reason to. Every project is a workspace from day one (even single-crate projects benefit from workspace-level configuration).

> **Starter template:** For a ready-to-compile project that demonstrates these patterns, see the **scaffold-fullstack** skill.

## Start with a Single Crate

New projects begin as a single crate with internal module boundaries that anticipate future splits. Don't create multiple crates speculatively.

```
my-project/
├── Cargo.toml          # workspace root + single member
├── crates/
│   └── my-project/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── domain/       # pure logic, no IO
│           │   └── mod.rs
│           ├── infra/        # trait implementations, IO
│           │   └── mod.rs
│           └── bin/
│               └── main.rs   # wiring, entry point
```

Even in a single crate, maintain module boundaries: `domain/` has no imports from `infra/`, and `bin/main.rs` wires adapters into domain logic. This makes future crate splits trivial — just move the module to its own crate.

## When to Split into Multiple Crates

Split when one of these conditions is met — not before:

1. **DI boundary solidifies** — You have a trait defined in domain code with multiple real implementations (e.g., a `Storage` trait with both SQLite and in-memory adapters). Move the trait to a core crate, implementations to adapter crates.

2. **Compile times suffer** — A module has grown large enough that incremental compilation is noticeably slow. Splitting it into its own crate gives better parallelism.

3. **Reuse across binaries** — You need shared logic between multiple binaries (CLI tool + web server, library + integration test harness).

4. **Independent versioning** — A piece of the project is useful as a standalone library with its own semver.

## Standard Multi-Crate Layout

When you do split, use this structure:

```
my-project/
├── Cargo.toml              # workspace root (no [package])
├── crates/
│   ├── my-core/            # domain logic, trait definitions, no IO deps
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── my-client/          # adapters: HTTP, DB, file IO implementations
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── my-app/             # binary: wires core + adapters together
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── my-testutils/       # shared test fakes and fixtures (dev-dependency only)
│       ├── Cargo.toml
│       └── src/lib.rs
```

- **`my-core`** depends only on `std` and domain-specific crates (e.g., `chrono`, `uuid`). Never on IO crates. Defines traits (ports) for external dependencies.
- **`my-client`** depends on `my-core` + IO crates (`reqwest`, `diesel`, etc.). Implements the port traits.
- **`my-app`** depends on `my-core` + `my-client`. Constructs concrete adapters and injects them. Contains `main()`.
- **`my-testutils`** exports shared fakes, builders, and fixtures. Only ever a `[dev-dependencies]` entry.

## Workspace Setup

Always use a workspace, even for single-crate projects. The root `Cargo.toml`:

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.75"

[workspace.dependencies]
# Pin shared dependencies here — members inherit with `.workspace = true`
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
diesel = { version = "2", features = ["sqlite"] }
diesel_migrations = "2"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
unwrap_used = "warn"

[workspace.lints.rust]
unsafe_code = "deny"
```

Member crates inherit from the workspace:

```toml
[package]
name = "my-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[lints]
workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true
```

## Workspace Dependencies

**All shared dependencies go in `[workspace.dependencies]`.** Member crates reference them with `.workspace = true`. This ensures version consistency and makes upgrades a single-line change.

Rules:
- If two or more crates use the same dependency, it goes in `[workspace.dependencies]`
- If only one crate uses a dependency and it's unlikely to be shared, it can be declared locally
- Feature flags on workspace deps are the *superset* — individual crates can use `default-features = false` and select specific features

## Feature Flags

Conventions:
- Name features in `lowercase-kebab-case`
- The `default` feature set should be the common case — users opt out, not in
- Use feature flags for optional functionality (e.g., `json-logging`, `http-client`), not for build-time config that should be env vars
- Document features in the crate's `Cargo.toml` using `[package.metadata.docs.rs]` or inline comments
- Don't use feature flags to alter core behavior in surprising ways — a feature should add capability, not change existing semantics
- Propagate features through workspace crates explicitly:
  ```toml
  [features]
  http-client = ["my-client/http-client"]
  ```

## Related Skills

For dependency inversion patterns and trait-as-interface design, see the **rust-architecture** skill.
For CI configuration and workspace-level tooling, see the **rust-ci-tooling** skill.
