---
name: ras-api-design
description: Use when the user asks about defining REST endpoints, JSON-RPC methods, file service routes, or WebSocket services with RAS macros, designing request/response types, path parameters, query parameters, macro syntax for rest_service!, jsonrpc_service!, file_service!, or jsonrpc_bidirectional_service!, or asks about OpenAPI/OpenRPC generation.
version: 1.0.0
---

# RAS API Design — Macro Syntax & Endpoint Definition

RAS macros generate a service trait, builder, Axum router, and spec (OpenAPI/OpenRPC) from a single declarative block. You define the contract; the macro generates the plumbing. All four macros share the same auth-level syntax and type requirements.

## `rest_service!` — REST APIs

```rust
use ras_rest_macro::rest_service;

rest_service!({
    service_name: TaskService,
    base_path: "/api/v1",
    openapi: true,
    serve_docs: true,
    docs_path: "/docs",
    endpoints: [
        // Public — no auth
        GET UNAUTHORIZED tasks() -> TasksResponse,
        GET UNAUTHORIZED tasks/{id: String}() -> Task,

        // Query parameters
        GET UNAUTHORIZED search/tasks ? q: String & limit: Option<u32> & offset: Option<u32> () -> TasksResponse,

        // Authenticated — requires "user" permission
        POST WITH_PERMISSIONS(["user"]) tasks(CreateTaskRequest) -> Task,

        // Multiple path params
        PUT WITH_PERMISSIONS(["user"]) users/{user_id: String}/tasks/{task_id: String}(UpdateTaskRequest) -> Task,

        // OR permissions — either "owner" OR "admin" suffices
        DELETE WITH_PERMISSIONS(["owner"] | ["admin"]) tasks/{id: String}() -> (),
    ]
});
```

### Endpoint Syntax

```
METHOD AUTH_LEVEL path/{param: Type}/segments ? query: Type & query2: Type (RequestBody) -> ResponseType
```

| Component | Options |
|-----------|---------|
| **Method** | `GET`, `POST`, `PUT`, `DELETE`, `PATCH` |
| **Auth level** | `UNAUTHORIZED`, `WITH_PERMISSIONS(["perm1", "perm2"])` |
| **Path params** | `{name: Type}` inline in the path |
| **Query params** | `? param: Type & param2: Option<Type>` after the path |
| **Request body** | `(RequestType)` — omit the type for GET/DELETE: `()` |
| **Response** | `-> ResponseType` — use `()` for empty responses |

### Path Parameters

Parameters are extracted from the URL path. Multiple params supported:

```rust
GET UNAUTHORIZED users/{user_id: String}/posts/{post_id: i32}() -> Post,
PUT WITH_PERMISSIONS(["user"]) posts/{post_id: i32}/comments/{comment_id: i32}(UpdateCommentRequest) -> Comment,
```

### Query Parameters

Appended after `?`, separated by `&`. Use `Option<T>` for optional params:

```rust
GET UNAUTHORIZED search ? q: String & limit: Option<u32> & offset: Option<u32> () -> SearchResults,
```

### Macro Configuration

```rust
rest_service!({
    service_name: ServiceName,           // Required: generates trait, builder, client names
    base_path: "/api/v1",               // Required: URL prefix for all endpoints
    openapi: true,                      // Optional: generate OpenAPI 3.0 spec
    openapi: { output: "custom.json" }, // Optional: custom output path
    serve_docs: true,                   // Optional: host the built-in API explorer
    docs_path: "/docs",                 // Optional: explorer path (default: "/docs")
    endpoints: [ ... ]
});
```

## Request & Response Types

All types used in macro invocations must derive three traits:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}
```

- `Serialize` + `Deserialize` — serde, for JSON encoding
- `JsonSchema` — schemars, for OpenAPI spec generation

Missing `JsonSchema` causes a compile error when `openapi: true`.

### Hosted REST Explorer

When `serve_docs: true` is set, the generated router serves the built-in RAS API explorer at `base_path + docs_path` and the OpenAPI document at `base_path + docs_path + "/openapi.json"`. For example, `base_path: "/api/v1"` and `docs_path: "/docs"` serve:

- `GET /api/v1/docs` — interactive API explorer
- `GET /api/v1/docs/openapi.json` — generated OpenAPI JSON

The explorer has built-in bearer-token entry for trying protected endpoints. Tokens are stored in `sessionStorage` for the current browser session, not `localStorage`; only non-secret UI preferences such as theme are stored persistently.

### Error Responses

Use `RestResult<T>` (alias for `Result<RestResponse<T>, RestError>`) in handler implementations:

```rust
use ras_rest_core::{RestResult, RestResponse, RestError};

async fn get_task_by_id(&self, id: String) -> RestResult<Task> {
    // Success variants
    Ok(RestResponse::ok(task))          // 200
    Ok(RestResponse::created(task))     // 201
    Ok(RestResponse::with_status(202, task))  // custom

    // Error variants
    Err(RestError::not_found("Task not found"))
    Err(RestError::bad_request("Invalid task ID"))
    Err(RestError::unauthorized("Invalid token"))
    Err(RestError::forbidden("Insufficient permissions"))

    // Internal error — logged but message not sent to client
    Err(RestError::with_internal(500, "Database error", db_error))
}
```

For domain-specific errors, define a `thiserror` enum and convert to `RestError`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("task not found: {0}")]
    NotFound(String),
    #[error("duplicate title: {0}")]
    DuplicateTitle(String),
    #[error("storage error: {0}")]
    Storage(String),
}

impl From<TaskError> for RestError {
    fn from(e: TaskError) -> Self {
        match e {
            TaskError::NotFound(msg) => RestError::not_found(msg),
            TaskError::DuplicateTitle(msg) => RestError::bad_request(msg),
            TaskError::Storage(msg) => {
                RestError::with_internal(500, "Internal error", std::io::Error::other(msg))
            }
        }
    }
}
```

## Generated Code

Each macro generates four things:

### 1. Service Trait

```rust
#[async_trait]
pub trait TaskServiceTrait: Send + Sync + 'static {
    // UNAUTHORIZED — no user parameter
    async fn get_tasks(&self) -> RestResult<TasksResponse>;
    async fn get_tasks_by_id(&self, id: String) -> RestResult<Task>;

    // WITH_PERMISSIONS — receives &AuthenticatedUser
    async fn post_tasks(&self, user: &AuthenticatedUser, req: CreateTaskRequest) -> RestResult<Task>;
    async fn delete_tasks_by_id(&self, user: &AuthenticatedUser, id: String) -> RestResult<()>;
}
```

Method names are generated from HTTP method + path segments: `get_tasks`, `post_tasks`, `get_tasks_by_id`, `delete_tasks_by_id`.

### 2. Service Builder

```rust
let router = TaskServiceBuilder::new(service_impl)
    .auth_provider(auth)                    // Arc<dyn AuthProvider>
    .with_usage_tracker(|headers, user, method, path| async move { ... })
    .with_method_duration_tracker(|method, path, user, duration| async move { ... })
    .build();  // Returns axum::Router
```

### 3. Native Rust Client

The macro generates a type-safe async client with the same method signatures as the service trait. Enable with `client` feature flag.

```rust
use my_api::TaskServiceClient;

// Build client with server URL
let mut client = TaskServiceClient::builder("http://localhost:3000/api/v1").build();

// Set auth token for protected endpoints
client.set_bearer_token(Some("jwt-token"));

// Methods mirror the service trait — same types, same names
let tasks: TasksResponse = client.get_tasks().await?;
let task: Task = client.get_tasks_by_id("task-123".into()).await?;
let new_task: Task = client.post_tasks(CreateTaskRequest {
    title: "New task".into(),
    description: "Details".into(),
}).await?;

// Methods with custom timeout
let tasks = client.get_tasks_with_timeout(Some(Duration::from_secs(5))).await?;
```

The client is generated in the API crate alongside the server trait — both sides share the same request/response types, ensuring compile-time type safety across service boundaries. This is the primary way to consume RAS services from other Rust crates.

## `jsonrpc_service!` — JSON-RPC

```rust
use ras_jsonrpc_macro::jsonrpc_service;

jsonrpc_service!({
    service_name: ChatService,
    openrpc: true,
    explorer: true,
    methods: [
        UNAUTHORIZED health_check(()) -> HealthStatus,
        WITH_PERMISSIONS(["user"]) send_message(SendMessageRequest) -> SendMessageResponse,
        WITH_PERMISSIONS(["admin"]) delete_channel(DeleteChannelRequest) -> (),
    ]
});
```

JSON-RPC methods map to JSON-RPC 2.0 `method` strings. Like REST, `UNAUTHORIZED` methods receive only the request, while `WITH_PERMISSIONS` methods also receive `&AuthenticatedUser`.

When `explorer: true` is used with `openrpc: true`, the macro generates `{service}_explorer_routes(base_path)`. Merge those routes into your Axum app to serve the same built-in explorer at `/explorer` by default, plus `/explorer/openrpc.json`. A custom path can be configured with `explorer: { path: "/api/docs" }`.

## `file_service!` — File Upload/Download

```rust
use ras_file_macro::file_service;

file_service!({
    service_name: DocumentService,
    base_path: "/api/files",
    body_limit: 52428800,  // 50MB
    endpoints: [
        UPLOAD WITH_PERMISSIONS(["user"]) upload() -> FileMetadata,
        DOWNLOAD UNAUTHORIZED download/{file_id: String}(),
    ]
});
```

- `UPLOAD` endpoints accept streaming multipart bodies
- `DOWNLOAD` endpoints return streaming responses
- `body_limit` sets the maximum upload size in bytes

## `jsonrpc_bidirectional_service!` — WebSocket

```rust
use ras_jsonrpc_bidirectional_macro::jsonrpc_bidirectional_service;

jsonrpc_bidirectional_service!({
    service_name: RealtimeService,
    client_to_server: [
        WITH_PERMISSIONS(["user"]) send_message(SendMessageRequest) -> SendMessageResponse,
        WITH_PERMISSIONS(["user"]) subscribe_channel(SubscribeRequest) -> (),
    ],
    server_to_client: [
        message_received(MessageNotification),
        user_joined(UserJoinedNotification),
    ],
    server_to_client_calls: [
        ping(PingRequest) -> PongResponse,
    ]
});
```

- `client_to_server` — methods the client can call on the server (request/response)
- `server_to_client` — notifications the server pushes to clients (fire-and-forget, no response)
- `server_to_client_calls` — methods the server can call on the client (request/response, bidirectional)

Read `references/macro-syntax-reference.md` for a compact cheat sheet of all four macros.

## Auth Level Details

Auth levels are shared across all macros:

| Auth Level | Handler Signature | Meaning |
|-----------|------------------|---------|
| `UNAUTHORIZED` | No user param | No authentication required |
| `WITH_PERMISSIONS(["a"])` | `user: &AuthenticatedUser` | Requires permission "a" |
| `WITH_PERMISSIONS(["a", "b"])` | `user: &AuthenticatedUser` | Requires "a" AND "b" |
| `WITH_PERMISSIONS(["a"] \| ["b"])` | `user: &AuthenticatedUser` | Requires "a" OR "b" |
| `WITH_PERMISSIONS(["a"] \| ["b", "c"])` | `user: &AuthenticatedUser` | Requires "a" OR ("b" AND "c") |

The macro enforces auth at the router level — unauthenticated requests to protected endpoints are rejected before your handler runs.

## Related Skills

For project scaffolding and where macros live in the crate layout, see the **ras-setup** skill.
For `AuthProvider` implementation and permission design, see the **ras-security** skill.
For error handling patterns and observability wiring, see the **ras-best-practices** skill.
