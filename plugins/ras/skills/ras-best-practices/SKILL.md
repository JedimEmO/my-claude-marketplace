---
name: ras-best-practices
description: Use when the user asks about RAS observability, error handling in RAS services, usage tracking, method duration tracking, Prometheus metrics, OpenTelemetry integration, using the generated Rust client, service-to-service communication, testing RAS services, or general best practices for production RAS deployments.
version: 1.0.0
---

# RAS Best Practices — Observability, Errors, Clients & Testing

Production RAS services need structured errors, observability hooks, generated clients, and testable handler implementations. This skill covers the patterns that bridge the gap between a working macro invocation and a production deployment.

## Error Handling

RAS error handling follows the **rust-architecture** convention: `thiserror` for library/domain errors, `anyhow` only in the binary crate.

### Domain Errors → REST Errors

Define domain errors with `thiserror`, then convert to `RestError` at the handler boundary:

```rust
use thiserror::Error;
use ras_rest_core::{RestResult, RestResponse, RestError};

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("task not found: {0}")]
    NotFound(String),
    #[error("duplicate title: {0}")]
    DuplicateTitle(String),
    #[error("storage error")]
    Storage(#[source] anyhow::Error),
}

impl From<TaskError> for RestError {
    fn from(e: TaskError) -> Self {
        match e {
            TaskError::NotFound(msg) => RestError::not_found(msg),
            TaskError::DuplicateTitle(msg) => RestError::bad_request(msg),
            TaskError::Storage(e) => RestError::with_internal(500, "Internal error", e),
        }
    }
}
```

Rules:
- **Client errors (4xx)** — include a meaningful message the caller can act on
- **Server errors (5xx)** — use `RestError::with_internal()` to log the real error while returning a generic message
- **Never leak internals** — stack traces, SQL queries, and file paths stay in logs
- Domain logic returns `Result<T, TaskError>`, handlers convert to `RestResult<T>` via `?` with the `From` impl

### JSON-RPC Errors

JSON-RPC uses standard error codes. Map domain errors to appropriate codes:

```rust
use ras_jsonrpc_types::JsonRpcError;

impl From<TaskError> for JsonRpcError {
    fn from(e: TaskError) -> Self {
        match e {
            TaskError::NotFound(msg) => JsonRpcError::new(-32001, msg, None),
            TaskError::DuplicateTitle(msg) => JsonRpcError::new(-32002, msg, None),
            TaskError::Storage(_) => JsonRpcError::internal_error(),
        }
    }
}
```

## Observability

RAS provides two hooks on every service builder: `UsageTracker` (counts requests) and `MethodDurationTracker` (measures latency). The `ras-observability-otel` crate provides a production-ready implementation backed by OpenTelemetry + Prometheus.

### Quick Start

```rust
use ras_observability_otel::standard_setup;

let otel = standard_setup("my-service")?;

let router = TaskServiceBuilder::new(service_impl)
    .auth_provider(auth)
    .with_usage_tracker({
        let tracker = otel.usage_tracker();
        move |headers, user, method, path| {
            let context = RequestContext::rest(method, path);
            let tracker = tracker.clone();
            async move { tracker.track_request(&headers, user.as_ref(), &context).await; }
        }
    })
    .with_method_duration_tracker({
        let tracker = otel.method_duration_tracker();
        move |method, path, user, duration| {
            let context = RequestContext::rest(method, path);
            let tracker = tracker.clone();
            async move { tracker.track_duration(&context, user.as_ref(), duration).await; }
        }
    })
    .build();

// Add Prometheus metrics endpoint
let app = Router::new()
    .merge(router)
    .merge(otel.metrics_router());  // exposes /metrics
```

### Exposed Metrics

| Metric | Type | Labels |
|--------|------|--------|
| `requests_started` | Counter | `method`, `protocol` |
| `requests_completed` | Counter | `method`, `protocol`, `success` |
| `method_duration_milliseconds` | Histogram | `method`, `protocol` |

Labels are kept minimal to prevent cardinality explosion. Use structured logs (not metric labels) for per-user or per-request data.

Read `references/observability-config.md` for complete setup snippets including Prometheus scrape config and Grafana queries.

## Generated Rust Client

Each RAS macro generates a type-safe async client alongside the server trait. Both live in the API crate and share the same request/response types — compile-time type safety across service boundaries.

### Using the Client

```rust
use my_api::{TaskServiceClient, CreateTaskRequest};
use std::time::Duration;

// Build client pointing at the target service
let mut client = TaskServiceClient::builder("http://localhost:3000/api/v1").build();

// Set auth token for protected endpoints
client.set_bearer_token(Some("jwt-token"));

// Methods mirror the service trait — same types, same names
let tasks = client.get_tasks().await?;
let task = client.get_tasks_by_id("task-123".into()).await?;
let new_task = client.post_tasks(CreateTaskRequest {
    title: "New task".into(),
    description: "Details".into(),
}).await?;

// Custom timeout for slow endpoints
let result = client.get_tasks_with_timeout(Some(Duration::from_secs(10))).await?;
```

### Service-to-Service Communication

The generated client is the primary way to call RAS services from other Rust crates. In a multi-service architecture, add the API crate as a dependency with only the `client` feature:

```toml
[dependencies]
task-api = { path = "../task-api", default-features = false, features = ["client"] }
```

This pulls in only the client code and shared types — no server-side code generation.

## Testing RAS Services

Follow the **rust-testing** skill's approach: hand-written fakes, `TestApp` pattern, `axum-test` for in-process HTTP.

### Hand-Written `FakeAuthProvider`

```rust
use ras_auth_core::{AuthProvider, AuthenticatedUser, AuthResult, AuthError};
use std::sync::Mutex;

pub struct FakeAuthProvider {
    users: Mutex<Vec<(String, AuthenticatedUser)>>,  // token → user
}

impl FakeAuthProvider {
    pub fn new() -> Self {
        Self { users: Mutex::new(Vec::new()) }
    }

    pub fn add_user(&self, token: &str, user_id: &str, permissions: Vec<String>) {
        self.users.lock().unwrap().push((
            token.into(),
            AuthenticatedUser {
                user_id: user_id.into(),
                permissions,
                ..Default::default()
            },
        ));
    }
}

#[async_trait::async_trait]
impl AuthProvider for FakeAuthProvider {
    async fn authenticate(&self, token: String) -> AuthResult<AuthenticatedUser> {
        self.users.lock().unwrap()
            .iter()
            .find(|(t, _)| *t == token)
            .map(|(_, u)| u.clone())
            .ok_or(AuthError::InvalidToken)
    }

    async fn check_permissions(
        &self,
        user: &AuthenticatedUser,
        required: &[String],
    ) -> AuthResult<()> {
        if required.iter().all(|p| user.permissions.contains(p)) {
            Ok(())
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }
}
```

### Integration Testing with `axum-test`

Build the full Axum router with fakes, exercise the HTTP stack in-process:

```rust
use axum_test::TestServer;
use std::sync::Arc;

struct TestApp {
    server: TestServer,
    auth: Arc<FakeAuthProvider>,
}

impl TestApp {
    fn new() -> Self {
        let auth = Arc::new(FakeAuthProvider::new());
        let service = TaskServiceImpl::new(/* inject domain fakes */);

        let router = TaskServiceBuilder::new(service)
            .auth_provider(Arc::clone(&auth) as Arc<dyn AuthProvider>)
            .build();

        let server = TestServer::new(router).unwrap();
        Self { server, auth }
    }
}

#[tokio::test]
async fn create_task_requires_auth() {
    let app = TestApp::new();

    // Unauthenticated — should fail
    let response = app.server
        .post("/api/v1/tasks")
        .json(&json!({ "title": "Test", "description": "" }))
        .await;
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_task_with_valid_token() {
    let app = TestApp::new();
    app.auth.add_user("test-token", "alice", vec!["user".into()]);

    let response = app.server
        .post("/api/v1/tasks")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({ "title": "Test", "description": "" }))
        .await;
    response.assert_status(StatusCode::CREATED);
}
```

For the full fake pattern (builders, `Mutex`-based storage, configurable failures), see the **rust-testing** skill.

## Production Checklist

- **Structured logging** — use `tracing` with JSON output, include request IDs
- **Health check endpoint** — `GET UNAUTHORIZED health() -> HealthStatus` in every service
- **Graceful shutdown** — handle `SIGTERM` with `tokio::signal` before stopping the listener
- **CORS** — configure `tower-http::cors::CorsLayer` for browser clients
- **Request size limits** — set `body_limit` in `file_service!`, use Tower middleware for REST
- **Protect `/metrics`** — require a bearer token or restrict to internal network

## Related Skills

For workspace setup and crate layout, see the **ras-setup** skill.
For macro syntax and endpoint definition, see the **ras-api-design** skill.
For auth provider implementation and permission design, see the **ras-security** skill.
For hand-written fake patterns and test organization, see the **rust-testing** skill.
For DI and trait boundary patterns used throughout, see the **rust-architecture** skill.
