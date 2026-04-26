---
name: scaffold-fullstack
description: Use when the user asks to scaffold, bootstrap, create, or start a new full-stack Rust project from scratch — including backend API, frontend UI, domain layer, and test infrastructure. Also use when they want a working starter template, a reference implementation, or to generate a new app following marketplace best practices.
version: 1.0.0
---

# Full-Stack Scaffold — RAS Backend + dwind Frontend

A compilable, tested full-stack Rust application template. Copy it, rename the `app-` prefix to your project name, and replace the Item domain with your own.

## Architecture

```
template/
├── Cargo.toml                     # Workspace root (edition 2024, resolver 3)
├── .rustfmt.toml                  # max_width = 100
├── justfile                       # fmt, check, test, ci targets
├── .github/workflows/ci.yml      # GitHub Actions: check + test + frontend build
│
├── crates/
│   ├── app-core/                  # Domain layer — pure, no IO deps
│   │   └── src/
│   │       ├── domain/mod.rs      # Item, ItemId (Uuid-backed)
│   │       ├── dto.rs             # CreateItemRequest, ItemResponse, ItemListResponse (shared by frontend + backend)
│   │       ├── error.rs           # ItemError (thiserror): NotFound, AlreadyExists, Storage
│   │       └── ports/mod.rs       # ItemRepository trait (async_trait, Send + Sync)
│   │
│   ├── app-api/                   # RAS API definition (rest_service! macro)
│   │   └── src/
│   │       ├── endpoints.rs       # GET/POST/DELETE /items with auth levels
│   │       └── types.rs           # API types with JsonSchema (re-exports core dto + adds schemars)
│   │
│   ├── app-adapters/              # Trait implementations
│   │   └── src/
│   │       └── in_memory.rs       # InMemoryItemRepository (Mutex<HashMap>)
│   │
│   ├── app-service/               # Backend binary
│   │   └── src/
│   │       ├── main.rs            # DI wiring, CORS, graceful shutdown
│   │       └── handlers.rs        # Implements ItemServiceTrait, error conversion, tests
│   │
│   ├── app-frontend/              # dwind WASM app (cdylib)
│   │   └── src/
│   │       ├── lib.rs             # wasm_bindgen entry, dwind stylesheet init
│   │       └── components/
│   │           ├── app.rs         # Root layout
│   │           └── items.rs       # Item list + create form (web_sys fetch, shared types from core)
│   │
│   └── app-testutils/             # Test support crate
│       └── src/
│           ├── fakes.rs           # FakeItemRepository (configurable failure), FakeAuthProvider (Clone + shared state)
│           └── builders.rs        # ItemBuilder with an_item() convenience
│
└── frontend/                      # Frontend build config
    ├── index.html                 # Minimal HTML shell with gradient background
    ├── package.json               # rollup + @wasm-tool/rollup-plugin-rust
    └── rollup.config.js           # Rust → WASM → JS bundle pipeline
```

## How to Use This Scaffold

1. **Read the template directory** to understand the complete file structure
2. **Copy the template** into the user's target directory
3. **Rename** all `app-` prefixes to the user's project name (e.g., `app-core` → `myapp-core`)
4. **Replace the domain** — swap `Item`/`ItemId`/`ItemRepository` with the user's domain types
5. **Update API endpoints** in `endpoints.rs` to match the new domain
6. **Update the frontend** components to display the new domain
7. **Run `just ci`** to verify everything compiles and tests pass

## Key Patterns Demonstrated

- **Trait-as-Interface DI** — domain traits in core, implementations in adapters, wiring in service binary (see **rust-architecture** skill)
- **Workspace-first layout** — all crates under `crates/`, shared deps in `[workspace.dependencies]` (see **rust-project-setup** skill)
- **RAS macro-driven API** — `rest_service!` generates trait, builder, client, and OpenAPI spec (see **ras-api-design** skill)
- **Hosted API explorer** — `serve_docs: true` exposes `/api/v1/docs` and `/api/v1/docs/openapi.json`
- **Hand-written fakes** — `FakeItemRepository` and `FakeAuthProvider` with `Mutex` for `Send + Sync` (see **rust-testing** skill)
- **TestApp pattern** — full Axum router in-process via `axum-test` (see **ras-best-practices** skill)
- **Shared types** — request/response DTOs in `app-core::dto`, used by both frontend and backend
- **Type-safe frontend** — dwind WASM app shares domain types with backend via `app-core`
- **thiserror/anyhow split** — `thiserror` for domain errors, `anyhow` only in the binary crate
- **Clippy pedantic** — workspace-level lints, `unwrap_used` warning (see **rust-ci-tooling** skill)

## Build & Test Commands

```bash
# Backend
just ci                          # Full CI: fmt + clippy + test (all feature combos)
just test                        # Run backend tests
just test-wasm                   # Run frontend wasm-bindgen-test (needs wasm-pack)
cargo run -p app-service         # Start the backend on :3000

# Frontend
rustup target add wasm32-unknown-unknown
cd frontend && npm install && npm start    # Dev server on :8080 (proxies /api/* to :3000)

# Or just build the WASM
cargo build --target wasm32-unknown-unknown -p app-frontend
```

## Authentication Flow

The scaffold uses real JWT authentication via RAS identity crates:

1. **Login** — `POST /api/auth/login` with `{"username":"demo","password":"demo"}` returns a JWT token
2. **Token storage** — Frontend stores the JWT in reactive state (`Mutable<Option<String>>`)
3. **Authenticated requests** — Frontend passes `Authorization: Bearer <token>` on POST/DELETE
4. **Validation** — `JwtAuthProvider` validates the JWT on every protected endpoint

The API explorer at `/api/v1/docs` can also call protected endpoints with a bearer token. It stores the entered token in browser `sessionStorage`, not persistent `localStorage`.

### Endpoint Auth Levels

| Endpoint | Auth | Why |
|----------|------|-----|
| `GET /api/v1/items` | Public (`UNAUTHORIZED`) | Read access is open |
| `GET /api/v1/items/{id}` | Public (`UNAUTHORIZED`) | Read access is open |
| `POST /api/v1/items` | Protected (`items:write`) | Mutations require auth |
| `DELETE /api/v1/items/{id}` | Protected (`items:write`) | Mutations require auth |
| `POST /api/auth/login` | Public (custom handler) | Login endpoint |

To make all endpoints require auth, change `UNAUTHORIZED` to `WITH_PERMISSIONS(["items:read"])` in `endpoints.rs`.

### Default Credentials

A demo user is created on startup in `main.rs`:
- **Username:** `demo`
- **Password:** `demo`
- **Permissions:** `items:write`

## Deployment

### Docker Compose

```bash
docker compose up --build
# Frontend: http://localhost:8080
# Backend:  http://localhost:3000
```

The frontend nginx proxies `/api/*` to the backend, so the WASM app uses relative URLs.

### Local Development

```bash
# Terminal 1 — backend
cargo run -p app-service

# Terminal 2 — frontend (proxies /api/* to :3000)
cd frontend && npm install && npm start
```

## Notes

- The frontend uses `web_sys::fetch` to call the backend API, sharing types from `app-core::dto`
- For the RAS-generated native Rust client (useful for service-to-service calls), depend on `app-api` with `features = ["client"]`
- The `app-api` crate has `server` and `client` features — use only what you need
- Frontend component tests use `wasm-bindgen-test` — run with `wasm-pack test --headless --chrome crates/app-frontend`
