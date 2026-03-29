---
name: ras-security
description: Use when the user asks about authentication, authorization, permissions, JWT, OAuth2, session management, AuthProvider, IdentityProvider, securing RAS endpoints, token validation, RBAC, permission guards, or the UNAUTHORIZED/WITH_PERMISSIONS auth levels in RAS services.
version: 1.0.0
---

# RAS Security — Auth, Permissions & Identity

RAS uses pluggable authentication via two port traits: `AuthProvider` (validates tokens, checks permissions) and `IdentityProvider` (verifies credentials, returns identity). Both follow the trait-as-interface pattern from the **rust-architecture** skill — define in core, implement as adapters, wire via `Arc<dyn Trait>`.

## Auth Levels in Macros

Every endpoint declares its auth requirement. The macro enforces it at the router level — unauthenticated requests to protected endpoints are rejected before your handler runs.

```rust
endpoints: [
    // No auth — handler has no user parameter
    GET UNAUTHORIZED health() -> HealthStatus,

    // Requires authentication + "user" permission
    POST WITH_PERMISSIONS(["user"]) tasks(CreateTaskRequest) -> Task,

    // AND: requires both "moderator" AND "editor"
    PUT WITH_PERMISSIONS(["moderator", "editor"]) posts/{id: String}(UpdatePostRequest) -> Post,

    // OR: requires "admin" OR ("moderator" AND "editor")
    DELETE WITH_PERMISSIONS(["admin"] | ["moderator", "editor"]) posts/{id: String}() -> (),
]
```

How auth level affects the generated handler signature:

```rust
// UNAUTHORIZED — no user parameter
async fn get_health(&self) -> RestResult<HealthStatus> { ... }

// WITH_PERMISSIONS — receives &AuthenticatedUser
async fn post_tasks(&self, user: &AuthenticatedUser, req: CreateTaskRequest) -> RestResult<Task> {
    // user.user_id, user.permissions available here
    ...
}
```

## The `AuthProvider` Trait

`AuthProvider` is the port that RAS macros use to validate incoming requests. It extracts the bearer token, validates it, and returns an `AuthenticatedUser` with permissions.

```rust
use ras_auth_core::{AuthProvider, AuthenticatedUser, AuthResult, AuthFuture};

pub trait AuthProvider: Send + Sync + 'static {
    fn authenticate(&self, token: String) -> AuthFuture<'_>;
    fn check_permissions(
        &self,
        user: &AuthenticatedUser,
        required_permissions: &[String],
    ) -> AuthResult<()>;
}
```

Note: `authenticate` returns `AuthFuture` (a pinned boxed future), not an `async fn`. `check_permissions` is synchronous.

Wire it via `Arc<dyn AuthProvider>` in the service builder:

```rust
let auth: Arc<dyn AuthProvider> = Arc::new(JwtAuthProvider::new(session_service));

let router = TaskServiceBuilder::new(service_impl)
    .auth_provider(auth)
    .build();
```

## The `IdentityProvider` Trait

`IdentityProvider` verifies credentials (username/password, OAuth2 code) and returns a `VerifiedIdentity`. Multiple providers can be registered with a `SessionService`.

```rust
use ras_identity_core::{IdentityProvider, VerifiedIdentity, IdentityError};

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    async fn verify(&self, payload: serde_json::Value) -> Result<VerifiedIdentity, IdentityError>;
}
```

### Built-in Providers

**Local (username/password):**

```rust
use ras_identity_local::LocalUserProvider;

let provider = LocalUserProvider::new();
provider.add_user("alice".into(), "secure_password".into(), Some("alice@example.com".into()), Some("Alice".into())).await?;
```

Security features: Argon2 password hashing, timing attack resistance, username enumeration prevention, rate limiting (5 concurrent auth attempts).

**OAuth2 (external IdP):**

```rust
use ras_identity_oauth2::{OAuth2Provider, OAuth2Config, ProviderConfig};

let google_config = OAuth2ProviderConfig {
    provider_id: "google".into(),
    client_id: "your-client-id".into(),
    client_secret: "your-client-secret".into(),
    authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".into(),
    token_endpoint: "https://oauth2.googleapis.com/token".into(),
    userinfo_endpoint: Some("https://www.googleapis.com/oauth2/v2/userinfo".into()),
    redirect_uri: "http://localhost:3000/auth/callback".into(),
    scopes: vec!["openid".into(), "email".into(), "profile".into()],
    use_pkce: true,
    auth_params: Default::default(),
    user_info_mapping: None,
};

let oauth_provider = OAuth2Provider::new(
    OAuth2Config { providers: vec![google_config] },
    state_store,  // Arc<dyn OAuth2StateStore>
);
```

PKCE is used by default for authorization code flow.

### Custom Provider

Implement `IdentityProvider` for any auth backend:

```rust
struct LdapProvider { /* config */ }

#[async_trait]
impl IdentityProvider for LdapProvider {
    fn provider_id(&self) -> &str { "ldap" }

    async fn verify(&self, payload: serde_json::Value) -> Result<VerifiedIdentity, IdentityError> {
        let username = payload["username"].as_str().ok_or(IdentityError::InvalidCredentials)?;
        let password = payload["password"].as_str().ok_or(IdentityError::InvalidCredentials)?;
        // LDAP verification...
        Ok(VerifiedIdentity { subject: username.into(), ..Default::default() })
    }
}
```

## Session Management

`SessionService` orchestrates identity verification and JWT session creation:

```rust
use ras_identity_session::{SessionService, SessionConfig, JwtAuthProvider};
use std::time::Duration;

use jsonwebtoken::Algorithm;

let config = SessionConfig {
    jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
    jwt_ttl: Duration::from_secs(3600),  // 1 hour
    algorithm: Algorithm::HS256,
    refresh_enabled: false,
    enforce_active_sessions: true,
};

let session_service = Arc::new(SessionService::new(config));

// Register identity providers
session_service.register_provider(Box::new(local_provider)).await;
session_service.register_provider(Box::new(oauth_provider)).await;

// Create JWT auth provider for use with service macros
let auth: Arc<dyn AuthProvider> = Arc::new(JwtAuthProvider::new(session_service.clone()));
```

Session lifecycle:

```rust
// Authenticate and create session
let jwt = session_service.begin_session("local", json!({
    "username": "alice",
    "password": "secure_password"
})).await?;

// Verify session (used internally by JwtAuthProvider)
let user = session_service.verify_session(&jwt).await?;

// End session (logout, revokes token by JTI claim)
session_service.end_session(&jti).await;
```

## Permission Design (RBAC)

Implement `UserPermissions` to map identities to permissions:

```rust
use ras_identity_core::{UserPermissions, VerifiedIdentity, IdentityResult};

struct RoleBasedPermissions { /* role store */ }

#[async_trait]
impl UserPermissions for RoleBasedPermissions {
    async fn get_permissions(&self, identity: &VerifiedIdentity) -> IdentityResult<Vec<String>> {
        Ok(match identity.subject.as_str() {
            "admin" => vec!["user".into(), "admin".into()],
            _ => vec!["user".into()],
        })
    }
}

session_service.with_permissions(Arc::new(RoleBasedPermissions { /* ... */ }));
```

### Permission Naming

Use `resource:action` format for fine-grained control:

- `tasks:read`, `tasks:write`, `tasks:delete`
- `users:manage`, `admin:*`

In macros, check specific permissions rather than roles:

```rust
// Good — checks capability
POST WITH_PERMISSIONS(["tasks:write"]) tasks(CreateTaskRequest) -> Task,

// Avoid — checks role (less flexible)
POST WITH_PERMISSIONS(["admin"]) tasks(CreateTaskRequest) -> Task,
```

## Auth Error Handling

Define auth errors with `thiserror`, never leak internal details:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("authentication required")]
    AuthenticationRequired,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("insufficient permissions")]
    InsufficientPermissions,
    #[error("internal authentication error")]
    Internal(#[source] anyhow::Error),
}
```

The `Internal` variant logs the source error but only returns "internal authentication error" to the client.

## Security Checklist

- **HTTPS in production** — terminate TLS at the load balancer or use `axum-server` with rustls
- **Strong JWT secrets** — generate cryptographically secure secrets, never hardcode
- **Short token TTL** — 1 hour for access tokens, rotate via refresh tokens if needed
- **Environment config** — secrets from env vars or secret manager, never in code or config files
- **Rate limit auth endpoints** — the local provider limits to 5 concurrent attempts; add your own for login routes
- **Audit auth failures** — log failed auth attempts with request metadata (IP, user-agent) for incident response
- **Validate all inputs** — request types get serde deserialization, but validate business constraints in handlers
- **Sanitize error responses** — use `RestError::with_internal()` to log details without exposing them

## Related Skills

For the trait-as-interface pattern behind `AuthProvider`, see the **rust-architecture** skill.
For auth level syntax and endpoint definition, see the **ras-api-design** skill.
For testing with `FakeAuthProvider`, see the **ras-best-practices** skill.
