# RAS Macro Syntax Reference

## `rest_service!`

```rust
rest_service!({
    service_name: Name,              // Required
    base_path: "/prefix",            // Required
    openapi: true,                   // Optional — or { output: "path.json" }
    serve_docs: true,                // Optional — built-in API explorer
    docs_path: "/docs",              // Optional — explorer path, default "/docs"
    endpoints: [
        METHOD AUTH path/{param: Type}/more ? query: Type & opt: Option<Type> (Body) -> Response,
    ]
});
```

**Methods:** `GET`, `POST`, `PUT`, `DELETE`, `PATCH`

**Generated names:** `{method}_{path_segments}` — e.g., `GET users/{id}` → `get_users_by_id`

**Trait:** `{ServiceName}Trait`
**Builder:** `{ServiceName}Builder`
**Client:** `{ServiceName}Client`

**Hosted docs:** `serve_docs: true` serves the explorer at `base_path + docs_path` and the OpenAPI JSON at `base_path + docs_path + "/openapi.json"`. Bearer tokens entered in the explorer are kept in `sessionStorage`, not `localStorage`.

## `jsonrpc_service!`

```rust
jsonrpc_service!({
    service_name: Name,              // Required
    openrpc: true,                   // Optional — OpenRPC spec
    explorer: true,                  // Optional — web explorer UI
    methods: [
        AUTH method_name(RequestType) -> ResponseType,
    ]
});
```

**Trait:** `{ServiceName}` (no Trait suffix)
**Builder:** `{ServiceName}Builder`

**Hosted explorer:** `explorer: true` requires `openrpc: true` and generates `{service}_explorer_routes(base_path)`, serving the explorer at `/explorer` by default and OpenRPC JSON at `/explorer/openrpc.json`.

## `file_service!`

```rust
file_service!({
    service_name: Name,              // Required
    base_path: "/prefix",            // Required
    body_limit: 52428800,            // Optional — bytes, default varies
    endpoints: [
        UPLOAD AUTH path() -> MetadataType,
        DOWNLOAD AUTH path/{param: Type}(),
    ]
});
```

## `jsonrpc_bidirectional_service!`

```rust
jsonrpc_bidirectional_service!({
    service_name: Name,              // Required
    client_to_server: [
        AUTH method_name(RequestType) -> ResponseType,
    ],
    server_to_client: [
        notification_name(NotificationType),
    ]
});
```

## Auth Levels (all macros)

| Syntax | Meaning |
|--------|---------|
| `UNAUTHORIZED` | No auth check, no user param in handler |
| `WITH_PERMISSIONS(["a"])` | Requires permission "a" |
| `WITH_PERMISSIONS(["a", "b"])` | Requires "a" AND "b" |
| `WITH_PERMISSIONS(["a"] \| ["b"])` | Requires "a" OR "b" |
| `WITH_PERMISSIONS(["a"] \| ["b", "c"])` | "a" OR ("b" AND "c") |

## Type Requirements

All request/response types must derive:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

- `Serialize` + `Deserialize` — from `serde`
- `JsonSchema` — from `schemars` (required for OpenAPI/OpenRPC generation)

## REST Path & Query Parameter Syntax

```
path/{name: Type}                    # path param
path ? key: Type                     # required query param
path ? key: Option<Type>             # optional query param
path ? a: Type & b: Option<Type>     # multiple query params
path/{id: String} ? detail: bool     # path + query combined
```

## RestResult Responses

```rust
Ok(RestResponse::ok(value))                    // 200
Ok(RestResponse::created(value))               // 201
Ok(RestResponse::with_status(202, value))      // custom status
Err(RestError::bad_request("msg"))             // 400
Err(RestError::unauthorized("msg"))            // 401
Err(RestError::forbidden("msg"))               // 403
Err(RestError::not_found("msg"))               // 404
Err(RestError::with_internal(500, "msg", err)) // 500 (err logged, not sent)
```

## Generated Rust Client

```rust
// Build client
let client = ServiceNameClient::builder("http://host:port/base").build();
client.set_bearer_token(Some("jwt-token"));

// Methods mirror the service trait
let result = client.get_things().await?;
let item = client.get_things_by_id("id".into()).await?;
let created = client.post_things(CreateRequest { ... }).await?;

// With custom timeout
let result = client.get_things_with_timeout(Some(Duration::from_secs(5))).await?;
```

## Feature Flags

```toml
[features]
default = ["server", "client"]
server = []    # server-side trait + builder + router
client = []    # native Rust client (async, reqwest-based)
```
