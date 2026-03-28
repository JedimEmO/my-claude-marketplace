# Trait Patterns — Complete Examples

Concrete, copy-pasteable examples that complement the patterns in the SKILL.md.

## Port Trait + Domain Types (core crate)

```rust
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct UserId(pub String);

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Error)]
pub enum UserRepoError {
    #[error("user not found: {0:?}")]
    NotFound(UserId),
    #[error("duplicate email: {0}")]
    DuplicateEmail(String),
    #[error("storage error: {0}")]
    Storage(String),
}

pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, UserRepoError>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>, UserRepoError>;
    fn save(&self, user: &User) -> Result<(), UserRepoError>;
    fn delete(&self, id: &UserId) -> Result<(), UserRepoError>;
}
```

## SQLite Adapter (adapter crate)

```rust
use my_core::{User, UserId, UserRepository, UserRepoError};
use rusqlite::{Connection, OptionalExtension};

pub struct SqliteUserRepository {
    conn: Connection,
}

impl SqliteUserRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl UserRepository for SqliteUserRepository {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, UserRepoError> {
        self.conn
            .query_row(
                "SELECT id, email, name FROM users WHERE id = ?1",
                [&id.0],
                |row| Ok(User {
                    id: UserId(row.get(0)?),
                    email: row.get(1)?,
                    name: row.get(2)?,
                }),
            )
            .optional()
            .map_err(|e| UserRepoError::Storage(e.to_string()))
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, UserRepoError> {
        self.conn
            .query_row(
                "SELECT id, email, name FROM users WHERE email = ?1",
                [email],
                |row| Ok(User {
                    id: UserId(row.get(0)?),
                    email: row.get(1)?,
                    name: row.get(2)?,
                }),
            )
            .optional()
            .map_err(|e| UserRepoError::Storage(e.to_string()))
    }

    fn save(&self, user: &User) -> Result<(), UserRepoError> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO users (id, email, name) VALUES (?1, ?2, ?3)",
                (&user.id.0, &user.email, &user.name),
            )
            .map_err(|e| UserRepoError::Storage(e.to_string()))?;
        Ok(())
    }

    fn delete(&self, id: &UserId) -> Result<(), UserRepoError> {
        self.conn
            .execute("DELETE FROM users WHERE id = ?1", [&id.0])
            .map_err(|e| UserRepoError::Storage(e.to_string()))?;
        Ok(())
    }
}
```

## Service with `Arc<dyn Trait>` Dependencies

```rust
use std::sync::Arc;
use anyhow::Context;
use my_core::{User, UserId, UserRepository, Notifier};

pub struct UserService {
    users: Arc<dyn UserRepository>,
    notifier: Arc<dyn Notifier>,
}

impl UserService {
    pub fn new(users: Arc<dyn UserRepository>, notifier: Arc<dyn Notifier>) -> Self {
        Self { users, notifier }
    }

    pub fn register(&self, email: String, name: String) -> anyhow::Result<User> {
        if self.users.find_by_email(&email)?.is_some() {
            anyhow::bail!("user with email {} already exists", email);
        }

        let user = User {
            id: UserId(generate_id()),
            email,
            name,
        };

        self.users.save(&user)
            .context("failed to save new user")?;
        self.notifier.send_welcome(&user)
            .context("failed to send welcome notification")?;

        Ok(user)
    }
}
```

## `thiserror` Enum with `#[from]` Conversions

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("order not found: {0}")]
    NotFound(String),

    #[error("insufficient stock for item {item_id}: requested {requested}, available {available}")]
    InsufficientStock {
        item_id: String,
        requested: u32,
        available: u32,
    },

    #[error("user error")]
    User(#[from] UserRepoError),

    #[error("payment failed")]
    Payment(#[from] PaymentError),
}
```

## App Wiring with `anyhow`

```rust
use std::sync::Arc;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()
        .context("failed to load configuration")?;

    let conn = rusqlite::Connection::open(&config.database_path)
        .context("failed to open database")?;

    let users: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(conn));
    let notifier: Arc<dyn Notifier> = Arc::new(EmailNotifier::new(&config.smtp));
    let service = UserService::new(users, notifier);

    let server = build_server(service, &config)
        .context("failed to build HTTP server")?;

    server.run().await
        .context("server exited with error")
}
```

## Notifier Trait (secondary port)

```rust
pub trait Notifier: Send + Sync {
    fn send_welcome(&self, user: &User) -> Result<(), NotifyError>;
    fn send_password_reset(&self, user: &User, token: &str) -> Result<(), NotifyError>;
}

pub struct EmailNotifier { /* smtp config */ }
impl Notifier for EmailNotifier { /* sends real emails */ }
```

For the corresponding `FakeNotifier` with configurable failure, see the **rust-testing** skill's reference file.

## Async Trait with `async-trait` (object-safe)

When you need `dyn` dispatch with async methods, use the `async-trait` crate:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, stream: &str, events: &[Event]) -> Result<(), EventStoreError>;
    async fn read_stream(&self, stream: &str) -> Result<Vec<Event>, EventStoreError>;
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append(&self, stream: &str, events: &[Event]) -> Result<(), EventStoreError> {
        // Use tokio::task::spawn_blocking for rusqlite calls in async context
        todo!()
    }

    async fn read_stream(&self, stream: &str) -> Result<Vec<Event>, EventStoreError> {
        todo!()
    }
}
```

Note: `rusqlite::Connection` is blocking and not `Send`. When using it inside async code, wrap calls in `tokio::task::spawn_blocking` or use `tokio-rusqlite` for an async-friendly wrapper.
