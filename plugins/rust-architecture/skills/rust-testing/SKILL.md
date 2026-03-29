---
name: rust-testing
description: Use when the user asks about testing Rust code, writing test doubles, creating fakes, organizing test modules, integration vs unit tests, test utilities, test fixtures, test patterns for trait-based dependency injection, test size strategy, or testing HTTP APIs with axum-test.
version: 1.0.0
---

# Rust Testing — Fakes, Organization & Strategy

Opinionated testing approach built around hand-written fakes, trait-based dependency injection, and a strong preference for small, fast, in-process tests. No mocking frameworks — no `mockall`, no `#[automock]`, no `mock!` macros.

## Test Size Strategy — Small > Medium > Large

If you can test it with a fake, do that (small). If you need to verify real IO behavior, keep it in-process (medium). Only go multi-process (large) when there's no alternative.

- **Small:** No IO. Domain logic + service wiring with fakes. Fast, deterministic, bulk of the suite.
- **Medium:** In-process IO. SQLite in-memory for DB tests, `axum-test` for HTTP. No spawned servers.
- **Large:** Multi-process / external services. Avoid unless genuinely necessary.

## Hand-Written Fakes

Every trait gets a hand-written fake. Fakes implement *realistic behavior* — a `FakeUserRepo` actually stores and retrieves items. This catches bugs that mocks miss, and tests *behavior* rather than implementation details.

Since port traits require `Send + Sync` (needed for `Arc<dyn Trait>` in async/multi-threaded contexts), fakes use `Mutex` for interior mutability:

```rust
use std::sync::Mutex;
use my_core::{User, UserId, UserRepository, UserRepoError};

pub struct FakeUserRepository {
    users: Mutex<Vec<User>>,
    should_fail: Mutex<bool>,
}

impl FakeUserRepository {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
            should_fail: Mutex::new(false),
        }
    }

    pub fn with_users(users: Vec<User>) -> Self {
        Self {
            users: Mutex::new(users),
            should_fail: Mutex::new(false),
        }
    }

    pub fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }

    pub fn stored_users(&self) -> Vec<User> {
        self.users.lock().unwrap().clone()
    }
}

impl UserRepository for FakeUserRepository {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, UserRepoError> {
        if *self.should_fail.lock().unwrap() {
            return Err(UserRepoError::Storage("fake failure".into()));
        }
        Ok(self.users.lock().unwrap().iter().find(|u| u.id == *id).cloned())
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, UserRepoError> {
        if *self.should_fail.lock().unwrap() {
            return Err(UserRepoError::Storage("fake failure".into()));
        }
        Ok(self.users.lock().unwrap().iter().find(|u| u.email == email).cloned())
    }

    fn save(&self, user: &User) -> Result<(), UserRepoError> {
        if *self.should_fail.lock().unwrap() {
            return Err(UserRepoError::Storage("fake failure".into()));
        }
        let mut users = self.users.lock().unwrap();
        if let Some(pos) = users.iter().position(|u| u.id == user.id) {
            users[pos] = user.clone();
        } else {
            users.push(user.clone());
        }
        Ok(())
    }

    fn delete(&self, id: &UserId) -> Result<(), UserRepoError> {
        if *self.should_fail.lock().unwrap() {
            return Err(UserRepoError::Storage("fake failure".into()));
        }
        self.users.lock().unwrap().retain(|u| u.id != *id);
        Ok(())
    }
}
```

## Test Organization

### Unit tests: `#[cfg(test)] mod tests`

At the bottom of the file being tested. Test the module's public interface using fakes.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use my_testutils::{FakeUserRepository, FakeNotifier, a_user};

    #[test]
    fn register_saves_user_and_sends_welcome() {
        let users = FakeUserRepository::new();
        let notifier = FakeNotifier::new();
        let service = UserService::new(
            Arc::new(users) as Arc<dyn UserRepository>,
            Arc::new(notifier) as Arc<dyn Notifier>,
        );

        let result = service.register("alice@example.com".into(), "Alice".into());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "alice@example.com");
    }

    #[test]
    fn register_rejects_duplicate_email() {
        let users = FakeUserRepository::with_users(vec![
            a_user().with_email("taken@example.com").build()
        ]);
        let notifier = FakeNotifier::new();
        let service = UserService::new(
            Arc::new(users) as Arc<dyn UserRepository>,
            Arc::new(notifier) as Arc<dyn Notifier>,
        );

        let result = service.register("taken@example.com".into(), "Bob".into());
        assert!(result.is_err());
    }
}
```

### Integration tests: `tests/` directory

Each file in `tests/` at the crate root is compiled as a separate binary.

### Test utilities crate: `crates/my-testutils/`

For multi-crate workspaces, create a shared testutils crate that exports fakes, builders, and test setup helpers. Declare as `[dev-dependencies]` in consuming crates.

### Test naming

`{action}_when_{condition}` or `{expected_outcome}_when_{scenario}`:

```rust
fn returns_none_when_user_not_found() { }
fn saves_updated_email_when_user_exists() { }
fn returns_error_when_storage_is_unavailable() { }
```

## Test Fixtures with Builders

```rust
pub struct UserBuilder {
    id: String,
    email: String,
    name: String,
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self {
            id: "test-user-1".into(),
            email: "test@example.com".into(),
            name: "Test User".into(),
        }
    }
}

impl UserBuilder {
    pub fn with_id(mut self, id: impl Into<String>) -> Self { self.id = id.into(); self }
    pub fn with_email(mut self, email: impl Into<String>) -> Self { self.email = email.into(); self }
    pub fn with_name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }

    pub fn build(self) -> User {
        User { id: UserId(self.id), email: self.email, name: self.name }
    }
}

/// Reads naturally: `a_user().with_name("Alice").build()`
pub fn a_user() -> UserBuilder { UserBuilder::default() }
```

## Integration Test Pattern — `TestApp`

Use `Arc` to share fakes between the service and the test harness. The fake is wrapped in `Arc`, and the service receives its own `Arc` clone:

```rust
use std::sync::Arc;

pub struct TestApp {
    pub service: UserService,
    pub users: Arc<FakeUserRepository>,
    pub notifier: Arc<FakeNotifier>,
}

impl TestApp {
    pub fn new() -> Self {
        let users = Arc::new(FakeUserRepository::new());
        let notifier = Arc::new(FakeNotifier::new());

        let service = UserService::new(
            Arc::clone(&users) as Arc<dyn UserRepository>,
            Arc::clone(&notifier) as Arc<dyn Notifier>,
        );

        Self { service, users, notifier }
    }

    pub fn given_users_exist(&self, users: Vec<User>) {
        for user in users {
            self.users.save(&user).unwrap();
        }
    }
}
```

Usage:

```rust
#[test]
fn user_can_register_and_receive_welcome() {
    let app = TestApp::new();

    let user = app.service.register("alice@example.com".into(), "Alice".into()).unwrap();

    assert_eq!(user.email, "alice@example.com");
    assert_eq!(app.users.stored_users().len(), 1);
    assert!(app.notifier.was_notified(&user.id, "welcome"));
}
```

## Testing HTTP APIs with `axum-test`

Use `axum-test` to test the full HTTP stack in-process — no TCP listener, no spawned server. Build the `Router` the same way production does, inject fakes.

```rust
use axum_test::TestServer;
use std::sync::Arc;

async fn create_test_server() -> TestServer {
    let users: Arc<dyn UserRepository> = Arc::new(FakeUserRepository::new());
    let notifier: Arc<dyn Notifier> = Arc::new(FakeNotifier::new());

    let app_state = AppState { users, notifier };
    TestServer::new(create_router(app_state)).unwrap()
}

#[tokio::test]
async fn register_returns_created_user() {
    let server = create_test_server().await;

    let response = server
        .post("/api/users")
        .json(&json!({ "email": "alice@example.com", "name": "Alice" }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: User = response.json();
    assert_eq!(body.email, "alice@example.com");
}
```

This is a **medium test** — exercises real HTTP routing, middleware, extraction, and serialization, but stays in-process with faked dependencies.

For tests that also need a real database, wire in an in-memory SQLite instead of fakes:

```rust
use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

async fn create_test_server_with_db() -> TestServer {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();

    let users: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(conn));
    let notifier: Arc<dyn Notifier> = Arc::new(FakeNotifier::new());

    let app_state = AppState { users, notifier };
    TestServer::new(create_router(app_state)).unwrap()
}
```

## Async Test Considerations

- If the domain is async, use `#[tokio::test]` for domain unit tests too — the domain is runtime-agnostic, but tests can pick any runtime.
- Synchronous domain logic uses plain `#[test]` — don't wrap it in async unnecessarily.

## Related Skills

For trait-as-interface patterns that make fakes possible, see the **rust-architecture** skill.
For CI configuration and workspace-level tooling, see the **rust-ci-tooling** skill.
