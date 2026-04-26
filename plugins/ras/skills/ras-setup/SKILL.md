---
name: ras-setup
description: Use when the user asks to create a new RAS project, set up a Rust Agent Stack workspace, configure Cargo.toml for RAS crates, add RAS dependencies, scaffold a service crate, or asks about RAS workspace structure and crate organization.
version: 1.0.0
---

# RAS Project Setup — Workspace Scaffolding & Dependencies

RAS projects follow the same workspace-first, crate-split conventions from the **rust-project-setup** skill. The key addition: an **API crate** where macro invocations define the service contract, sitting between the domain core and the binary that wires everything together.

> **Starter template:** For a ready-to-compile RAS project with tests, see the **scaffold-fullstack** skill.

## Project Structure

A typical RAS project adds an API crate to the standard layout:

```
my-project/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── my-core/                # domain types, traits (ports), pure logic
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── my-api/                 # RAS macro invocations + request/response types
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── my-service/             # binary: wires implementations into generated builders
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── my-testutils/           # shared fakes and fixtures (dev-dependency only)
│       ├── Cargo.toml
│       └── src/lib.rs
```

- **`my-core`** — Domain types and port traits. No IO dependencies. No RAS dependency.
- **`my-api`** — Invokes `rest_service!`, `jsonrpc_service!`, etc. Defines request/response types. Depends on RAS macro crates + `my-core`.
- **`my-service`** — Implements the generated service traits, constructs adapters, wires auth via `Arc<dyn AuthProvider>`, and runs the Axum server.
- **`my-testutils`** — Hand-written fakes including `FakeAuthProvider`. Only a `[dev-dependencies]` entry.

The API crate exists because macro invocations generate both server traits and a native Rust client — keeping them in a separate crate lets other services depend on just the client and shared types (via `features = ["client"]`) without pulling in the full service implementation.

## Where RAS Macros Live

Macros belong in the API crate, not the domain crate. The API crate defines the contract:

```rust
// crates/my-api/src/lib.rs
use ras_rest_macro::rest_service;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

// Request/response types — must derive all three
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
    pub total: usize,
}

rest_service!({
    service_name: TaskService,
    base_path: "/api/v1",
    openapi: true,
    serve_docs: true,
    endpoints: [
        GET UNAUTHORIZED tasks() -> TasksResponse,
        POST WITH_PERMISSIONS(["user"]) tasks(CreateTaskRequest) -> Task,
        GET UNAUTHORIZED tasks/{id: String}() -> Task,
        DELETE WITH_PERMISSIONS(["admin"]) tasks/{id: String}() -> (),
    ]
});
```

The domain crate stays clean — no macro dependencies, no HTTP/RPC concerns.

With `serve_docs: true`, the router hosts the built-in RAS API explorer at `/api/v1/docs` and the OpenAPI JSON at `/api/v1/docs/openapi.json`. The explorer supports bearer-token testing and keeps tokens in browser `sessionStorage`, not persistent `localStorage`.

## Workspace Cargo.toml

Follow `rust-project-setup` conventions with RAS crates in `[workspace.dependencies]`:

```toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
# RAS crates (not on crates.io — use git dependency)
ras-rest-macro = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-rest-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-jsonrpc-macro = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-jsonrpc-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-file-macro = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-auth-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-identity-session = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-identity-local = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-identity-oauth2 = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-observability-core = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }
ras-observability-otel = { git = "https://github.com/JedimEmO/rust-agent-stack.git" }

# Standard deps
serde = { version = "1", features = ["derive"] }
schemars = "1.0.0-alpha.20"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
axum = "0.8"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
async-trait = "0.1"

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

Read `references/cargo-toml-templates.md` for complete, copy-pasteable member crate templates.

## API Crate Cargo.toml

The API crate depends on RAS macro crates and serialization:

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

[features]
default = ["server", "client"]
server = []
client = []
```

All request/response types must derive `Serialize`, `Deserialize`, and `JsonSchema`. Missing `JsonSchema` causes a compile error when `openapi: true` is set.

## Binary Crate — Wiring

The service crate implements the generated trait and wires everything together using `Arc<dyn AuthProvider>`:

```rust
// crates/my-service/src/main.rs
use my_api::{TaskServiceBuilder, TaskServiceTrait};
use ras_auth_core::AuthenticatedUser;
use ras_rest_core::{RestResult, RestResponse, RestError};
use ras_identity_session::{SessionService, SessionConfig, JwtAuthProvider};
use std::sync::Arc;

struct TaskServiceImpl { /* domain deps via Arc<dyn Trait> */ }

#[async_trait::async_trait]
impl TaskServiceTrait for TaskServiceImpl {
    async fn get_tasks(&self) -> RestResult<TasksResponse> {
        Ok(RestResponse::ok(TasksResponse { tasks: vec![], total: 0 }))
    }

    async fn post_tasks(
        &self,
        user: &AuthenticatedUser,
        request: CreateTaskRequest,
    ) -> RestResult<Task> {
        // Authenticated endpoints receive &AuthenticatedUser automatically
        Ok(RestResponse::created(Task { /* ... */ }))
    }
    // ...
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let jwt_secret = std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?;
    let session_service = Arc::new(SessionService::new(SessionConfig::new(jwt_secret)?)?);
    let auth: Arc<dyn ras_auth_core::AuthProvider> =
        Arc::new(JwtAuthProvider::new(session_service));

    let service = TaskServiceImpl { /* inject domain deps */ };
    let router = TaskServiceBuilder::new(service)
        .auth_provider(auth)
        .build();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await.context("server failed")?;
    Ok(())
}
```

## Adding a New Service to an Existing Workspace

1. Create the API crate: `cargo new crates/my-new-api --lib`
2. Add RAS macro deps to its `Cargo.toml` (inherit from workspace)
3. Define types and invoke the service macro
4. Implement the generated trait in the service crate (or a new service crate)
5. Wire into the existing Axum router via `.merge()` or `.nest()`

Multiple RAS services compose naturally — each macro generates an independent `Router`:

```rust
let app = Router::new()
    .merge(task_router)
    .merge(user_router)
    .merge(otel.metrics_router());
```

## Related Skills

For general workspace conventions, crate-split decisions, and feature flag strategy, see the **rust-project-setup** skill.
For the trait-as-interface DI pattern used for `AuthProvider` wiring, see the **rust-architecture** skill.
For macro syntax and endpoint definition, see the **ras-api-design** skill.
