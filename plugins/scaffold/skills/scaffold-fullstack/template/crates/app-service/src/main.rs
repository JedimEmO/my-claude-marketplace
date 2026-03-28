use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Json;
use axum::extract::State;
use axum::routing::post;
use ras_identity_core::StaticPermissions;
use ras_identity_local::LocalUserProvider;
use ras_identity_session::{JwtAuthProvider, SessionConfig, SessionService};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use app_adapters::in_memory::InMemoryItemRepository;
use app_core::dto::{LoginRequest, LoginResponse};
use app_core::ports::ItemRepository;

mod handlers;

use handlers::ItemServiceHandler;

#[derive(Clone)]
struct AppState {
    session_service: Arc<SessionService>,
}

async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, axum::http::StatusCode> {
    let token = state
        .session_service
        .begin_session(
            "local",
            serde_json::json!({
                "username": req.username,
                "password": req.password
            }),
        )
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

    let claims = state
        .session_service
        .verify_session(&token)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token,
        user_id: claims.sub,
        permissions: claims.permissions.into_iter().collect(),
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Domain
    let items: Arc<dyn ItemRepository> = Arc::new(InMemoryItemRepository::new());

    // Identity: local username/password provider with a demo user
    let local_provider = LocalUserProvider::new();
    local_provider
        .add_user(
            "demo".into(),
            "demo".into(),
            Some("demo@example.com".into()),
            Some("Demo User".into()),
        )
        .await
        .context("failed to create demo user")?;

    // Session: JWT creation + validation
    let permissions = Arc::new(StaticPermissions::new(vec!["items:write".into()]));
    let session_service =
        Arc::new(SessionService::new(SessionConfig::default()).with_permissions(permissions));
    session_service
        .register_provider(Box::new(local_provider))
        .await;

    // Auth: validates JWT tokens on protected RAS endpoints
    let auth = JwtAuthProvider::new(Arc::clone(&session_service));

    let handler = ItemServiceHandler::new(items);

    let ras_router = app_api::ItemServiceBuilder::new(handler)
        .auth_provider(auth)
        .build();

    let state = AppState { session_service };

    let app = axum::Router::new()
        .route("/api/auth/login", post(login_handler))
        .with_state(state)
        .merge(ras_router)
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3000";
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind")?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    tracing::info!("shutdown signal received");
}
