---
name: rust-architecture
description: Use when the user asks about dependency injection in Rust, trait-as-interface patterns, module boundaries, hexagonal architecture, ports and adapters, error handling strategy, when to use generics vs dyn Trait, how to structure application layers, or how to wire dependencies together.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Rust Architecture — DI, Trait Boundaries & Error Handling

Opinionated patterns for structuring Rust applications around testability and clean dependency boundaries. The core principle: **domain logic never knows about IO.**

## Trait-as-Interface

Every significant dependency boundary is a trait. The trait is the *port*; the struct implementing it is the *adapter*.

- Define traits in the core/domain crate (or module). They describe *what* the system needs, not *how* it's done.
- Implement traits in adapter crates/modules. These are the concrete HTTP clients, database repos, file handlers.
- The application layer wires concrete adapters into domain logic.
- All port traits require `Send + Sync` — this enables `Arc<dyn Trait>` for async/multi-threaded use and test sharing.

```rust
// In core — defines what we need
pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, UserRepoError>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>, UserRepoError>;
    fn save(&self, user: &User) -> Result<(), UserRepoError>;
    fn delete(&self, id: &UserId) -> Result<(), UserRepoError>;
}

// In adapter — implements it
pub struct SqliteUserRepository { /* connection */ }
impl UserRepository for SqliteUserRepository { /* real IO */ }
```

Traits should be object-safe when practical — this preserves the option of using `dyn` dispatch for app-level wiring and testing.

## Generics vs `dyn Trait` — Context-Dependent

This is not an either/or choice. Use both, in different contexts:

### Generics: for hot paths and library code

Use generics with trait bounds when the concrete type is known at compile time within a crate, or when you're writing library code where monomorphization matters.

```rust
pub fn find_active_users<R: UserRepository>(
    repo: &R,
    ids: &[UserId],
) -> Result<Vec<User>, UserRepoError> {
    ids.iter()
        .filter_map(|id| match repo.find_by_id(id) {
            Ok(Some(user)) => Some(Ok(user)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .collect()
}
```

**Use generics when:**
- The function is called in a tight loop
- It's part of a library's public API
- You want the compiler to inline and optimize
- The concrete type is known at the call site

### `dyn Trait`: for app-level wiring and test doubles

Use `Arc<dyn Trait>` when composing the application, holding dependencies in long-lived structs, or swapping implementations in tests. `Arc` is preferred over `Box` because it allows sharing the same instance between the service and the test harness.

```rust
pub struct UserService {
    users: Arc<dyn UserRepository>,
    notifier: Arc<dyn Notifier>,
}

impl UserService {
    pub fn new(users: Arc<dyn UserRepository>, notifier: Arc<dyn Notifier>) -> Self {
        Self { users, notifier }
    }
}
```

**Use `dyn` when:**
- Constructing or holding application-level service objects
- The indirection cost is negligible (one vtable lookup per call, not in a tight loop)
- You need to swap implementations at runtime or in tests
- The struct is long-lived and owns its dependencies

### Decision rule

If you're unsure: **if the function constructs or holds things, use `dyn`. If the function processes things, use generics.** Both can coexist in the same codebase.

## Hexagonal-ish Architecture

Three layers, loosely held. You don't need a framework — the principle is enough.

### Core / Domain

Pure logic — no IO, no runtime dependency. Can use `async` when the domain is inherently async, but must not depend on a specific runtime. Depends only on `std` and domain-specific crates (`chrono`, `uuid`, `serde` for derive). Defines traits (ports) for every external dependency it needs.

This layer is trivially testable — no fakes needed for pure functions, and trait-based fakes for anything with dependencies.

### Adapters

Implement the port traits. HTTP clients, database access, file IO, message queues. Each adapter depends on the core crate (for the trait definition) plus its IO crates (`reqwest`, `diesel`, etc.).

Adapters live in separate modules or separate crates, depending on project size (see **rust-project-setup** for when to split).

### App / Wiring

`main.rs` or an app module that constructs concrete adapters and injects them into domain services. This is where `dyn` dispatch happens. This layer reads config, sets up logging/tracing, and builds the dependency graph.

```rust
fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let mut db_conn = SqliteConnection::establish(&config.database_url)?;
    run_pending_migrations(&mut db_conn)?;

    let users: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(db_conn));
    let notifier: Arc<dyn Notifier> = Arc::new(EmailNotifier::new(&config.smtp));

    let service = UserService::new(users, notifier);
    run_server(service, &config.bind_addr)
}
```

### Module boundaries in a single crate

Even before splitting into multiple crates, enforce the layering at the module level:

```
src/
├── domain/          # traits, types, pure logic — no `use crate::infra`
├── infra/           # trait implementations, IO
└── bin/main.rs      # wiring
```

The rule: `domain/` never imports from `infra/`. Enforce this by convention (and by code review). When this boundary justifies a crate split, it's a clean move.

## Error Handling Strategy

### Libraries: `thiserror`

Reusable crates define typed error enums. Errors are part of the public API.

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserRepoError {
    #[error("user not found: {0:?}")]
    NotFound(UserId),
    #[error("duplicate email: {0}")]
    DuplicateEmail(String),
    #[error("storage error: {0}")]
    Storage(String),
}
```

- Each crate defines its own error type
- Use `#[from]` for automatic conversion from downstream errors where appropriate
- Keep variants meaningful — don't wrap every error in a generic `Internal(String)`

### Applications: `anyhow`

Binary crates and top-level application code use `anyhow` for ergonomic error propagation.

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let raw = std::fs::read_to_string("config.toml")
        .context("failed to read config file")?;
    let config: Config = toml::from_str(&raw)
        .context("failed to parse config")?;
    Ok(config)
}
```

- `Result<T>` means `anyhow::Result<T>` in app code
- Use `.context("what we were doing")` liberally — it creates a chain of context that makes debugging straightforward
- Library errors convert automatically via the `Error` trait

### The boundary

At the point where library errors enter application code, add context:

```rust
let user = repo.find_by_id(&id)
    .context("failed to look up user during checkout")?;
```

Adapter crates that serve only one application can use either style. If the adapter might be reused, use `thiserror`.

## Async Strategy — Runtime-Agnostic Core

The core/domain crate can absolutely use `async` — the key constraint is **no runtime dependency**. Domain logic can define and use async traits, return futures, and await other domain operations. What it must not do is pull in `tokio`, `async-std`, or any specific runtime as a dependency.

- Domain traits freely use `async fn` when the domain is inherently async
- The core crate should not depend on a specific runtime — no `tokio::spawn`, no `tokio::time::sleep`
- The binary crate selects the runtime (`tokio`, `async-std`, etc.)
- Pure domain functions that don't need async should remain synchronous — don't make everything async just because some things are

If a trait needs async methods, use `async fn` in trait (stabilized in Rust 1.75+). Note that native async trait methods are not object-safe — use the `async-trait` crate if you need `dyn` dispatch with async:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, stream: &str, events: &[Event]) -> Result<(), EventStoreError>;
    async fn read_stream(&self, stream: &str) -> Result<Vec<Event>, EventStoreError>;
}
```

## Related Skills

For workspace layout and crate splitting decisions, see the **rust-project-setup** skill.
For hand-written fakes and testing patterns that exercise these trait boundaries, see the **rust-testing** skill.
