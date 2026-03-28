# Cargo.toml Templates

## Workspace Root (no package)

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT"

[workspace.dependencies]
# Core
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"

# Async (runtime-agnostic where possible)
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

# Database
diesel = { version = "2", features = ["sqlite"] }
diesel_migrations = "2"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Testing
assert_matches = "1"

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

## Library Crate (core/domain)

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

[dev-dependencies]
assert_matches.workspace = true
```

## Adapter Crate (infra/client)

```toml
[package]
name = "my-client"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[lints]
workspace = true

[dependencies]
my-core = { path = "../my-core" }
serde.workspace = true
anyhow.workspace = true
tokio.workspace = true
tracing.workspace = true
diesel.workspace = true
diesel_migrations.workspace = true

[dev-dependencies]
my-testutils = { path = "../my-testutils" }
```

## Binary Crate (app)

```toml
[package]
name = "my-app"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[lints]
workspace = true

[dependencies]
my-core = { path = "../my-core" }
my-client = { path = "../my-client" }
anyhow.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
```

## Test Utilities Crate

```toml
[package]
name = "my-testutils"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish = false

[lints]
workspace = true

[dependencies]
my-core = { path = "../my-core" }
```

## Feature Flag Patterns

### Optional dependency gating

```toml
[features]
default = ["json"]
json = ["dep:serde_json"]
http-client = ["dep:reqwest"]

[dependencies]
serde_json = { workspace = true, optional = true }
reqwest = { version = "0.12", features = ["json"], optional = true }
```

### Feature-gated module

```rust
// In lib.rs
#[cfg(feature = "http-client")]
pub mod http_client;
```

### Propagating features across workspace crates

```toml
# In my-app/Cargo.toml
[features]
default = ["http-client"]
http-client = ["my-client/http-client"]
```

## Single-Crate Project (still a workspace)

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
edition = "2021"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
anyhow = "1"
thiserror = "2"

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

With a single member at `crates/my-project/Cargo.toml`:

```toml
[package]
name = "my-project"
version = "0.1.0"
edition.workspace = true

[lints]
workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true

[[bin]]
name = "my-project"
path = "src/bin/main.rs"
```
