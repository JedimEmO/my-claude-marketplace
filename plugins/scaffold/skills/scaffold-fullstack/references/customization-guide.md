# Customization Guide

## Renaming the Project

Replace all `app-` prefixes with your project name. For a project named `acme`:

1. Rename directories: `app-core` → `acme-core`, `app-api` → `acme-api`, etc.
2. Update `Cargo.toml` names and path references in all crates
3. Update `use` statements: `app_core` → `acme_core`, etc.
4. Update workspace `Cargo.toml` members if you changed the directory structure

## Replacing the Domain

The scaffold uses an Item/Inventory domain. To replace it:

1. **`app-core/src/domain/mod.rs`** — Replace `Item`, `ItemId` with your domain types
2. **`app-core/src/error.rs`** — Replace `ItemError` variants with your domain errors
3. **`app-core/src/ports/mod.rs`** — Replace `ItemRepository` with your domain trait(s)
4. **`app-core/src/dto.rs`** — Replace request/response types
5. **`app-api/src/types.rs`** — Update API types (add `JsonSchema` derive)
6. **`app-api/src/endpoints.rs`** — Rewrite the `rest_service!` macro with your endpoints
7. **`app-service/src/handlers.rs`** — Implement the new generated trait
8. **`app-adapters/src/in_memory.rs`** — Implement the new trait (or replace with a real adapter)
9. **`app-testutils/src/fakes.rs`** — Write fakes for your new traits
10. **`app-testutils/src/builders.rs`** — Write builders for your new domain types

## Adding a Real Database

Replace `InMemoryItemRepository` with a Diesel + SQLite adapter:

1. Add `diesel` and `diesel_migrations` to workspace deps
2. Create a `SqliteItemRepository` in `app-adapters` (see **rust-architecture** skill's trait-patterns reference)
3. Wrap `SqliteConnection` in `Mutex` for `Send + Sync`
4. Update `app-service/main.rs` to create the connection and wire the new adapter

## Customizing Authentication

The scaffold already includes real JWT auth via `SessionService` + `LocalUserProvider` + `JwtAuthProvider`. To customize:

1. **Add users** — call `local_provider.add_user()` in `main.rs`, or replace `LocalUserProvider` with a database-backed provider
2. **Per-user permissions** — replace `StaticPermissions` with a custom `UserPermissions` impl that looks up permissions per user
3. **OAuth2** — add `ras-identity-oauth2` and register an `OAuth2Provider` alongside (or instead of) the local provider
4. **Session config** — adjust `SessionConfig` in `main.rs` (JWT secret, TTL, algorithm)

## Adding Observability

1. Add `ras-observability-otel` to workspace deps
2. Use `standard_setup()` and wire `with_usage_tracker` / `with_method_duration_tracker` on the builder (see **ras-best-practices** skill)
3. Add a `/metrics` endpoint

## Adding More API Crates

For JSON-RPC or WebSocket APIs alongside REST:

1. Create a new `app-rpc-api` crate with `jsonrpc_service!` or `jsonrpc_bidirectional_service!`
2. Add it to the workspace
3. Merge its router into the main app (see **ras-api-design** skill)

## Splitting the Frontend

For a Tauri desktop app instead of (or alongside) the web app:

1. See the **dwind-tauri** skill for the separate frontend/backend workspace pattern
2. Use Trunk instead of Rollup for the WASM build
3. Add IPC bridge via `window.__TAURI__` bindings
