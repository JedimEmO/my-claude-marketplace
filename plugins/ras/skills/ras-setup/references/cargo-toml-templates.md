# Cargo.toml Templates for RAS Projects

## Workspace Root

```toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
# RAS — not on crates.io, use git dependency. Include only the crates you use.
ras-rest-macro = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-rest-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-jsonrpc-macro = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-jsonrpc-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-file-macro = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-jsonrpc-bidirectional-macro = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-jsonrpc-bidirectional-server = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-jsonrpc-bidirectional-client = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-auth-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-identity-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-identity-session = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-identity-local = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-identity-oauth2 = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-observability-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-observability-otel = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }

# Standard
serde = { version = "1", features = ["derive"] }
schemars = "1.0.0-alpha.20"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
axum = "0.8"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

# Testing
axum-test = "18"

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

## API Crate (REST)

```toml
[package]
name = "my-api"
version = "0.1.0"
edition.workspace = true

[lints]
workspace = true

[dependencies]
ras-rest-macro.workspace = true
ras-rest-core.workspace = true
ras-auth-core.workspace = true
serde.workspace = true
schemars.workspace = true
async-trait.workspace = true

# Domain types
my-core = { path = "../my-core" }

[features]
default = ["server", "client"]
server = []
client = []
```

## API Crate (JSON-RPC)

```toml
[package]
name = "my-rpc-api"
version = "0.1.0"
edition.workspace = true

[lints]
workspace = true

[dependencies]
ras-jsonrpc-macro.workspace = true
ras-jsonrpc-core.workspace = true
ras-auth-core.workspace = true
serde.workspace = true
schemars.workspace = true
async-trait.workspace = true

my-core = { path = "../my-core" }
```

## API Crate (File Service)

```toml
[package]
name = "my-file-api"
version = "0.1.0"
edition.workspace = true

[lints]
workspace = true

[dependencies]
ras-file-macro.workspace = true
ras-rest-core.workspace = true
ras-auth-core.workspace = true
serde.workspace = true
schemars.workspace = true
async-trait.workspace = true
```

## Service / Binary Crate

```toml
[package]
name = "my-service"
version = "0.1.0"
edition.workspace = true

[lints]
workspace = true

[dependencies]
my-core = { path = "../my-core" }
my-api = { path = "../my-api" }

ras-auth-core.workspace = true
ras-rest-core.workspace = true
ras-identity-session.workspace = true
ras-identity-local.workspace = true
ras-observability-otel.workspace = true

axum.workspace = true
tokio.workspace = true
anyhow.workspace = true
tracing.workspace = true
async-trait.workspace = true

[dev-dependencies]
my-testutils = { path = "../my-testutils" }
axum-test.workspace = true
```

## Core / Domain Crate

```toml
[package]
name = "my-core"
version = "0.1.0"
edition.workspace = true

[lints]
workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true
chrono.workspace = true
uuid.workspace = true
async-trait.workspace = true
```

No RAS dependencies here — the domain crate stays pure.

## Test Utilities Crate

```toml
[package]
name = "my-testutils"
version = "0.1.0"
edition.workspace = true

[lints]
workspace = true

[dependencies]
my-core = { path = "../my-core" }
ras-auth-core.workspace = true
serde.workspace = true
async-trait.workspace = true
```

This crate exports hand-written fakes (`FakeAuthProvider`, `FakeUserRepository`, etc.) and test fixture builders. Only ever referenced as `[dev-dependencies]`.
