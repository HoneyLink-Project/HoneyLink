// HoneyLink Control Plane API Server
// Main entry point for the Control Plane REST API

mod config;
mod db;
mod error;
mod middleware;
mod routes;
mod types;
mod validation;
mod vault;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use config::AppConfig;
use error::ApiError;
use middleware::{
    auth::JwtValidator,
    cors::create_cors_layer,
    rate_limit::PerIpRateLimiter,
    tracing::otel_trace_middleware,
};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub jwt_validator: Arc<JwtValidator>,
    pub rate_limiter: Arc<PerIpRateLimiter>,
    pub db_pool: sqlx::PgPool,
    pub vault_config: vault::VaultConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::default_with_env()
        .expect("Failed to load configuration");

    // Initialize telemetry
    init_telemetry(&config)?;

    // Create database connection pool
    let db_config = db::DatabaseConfig::default();
    let db_pool = db::create_pool(&db_config).await?;

    // Run database migrations
    db::run_migrations(&db_pool).await?;
    tracing::info!("Database migrations completed");

    // Create JWT validator
    let jwt_validator = Arc::new(
        JwtValidator::new(config.jwt.clone())
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to initialize JWT validator: {:?}. Running in development mode.", e);
                // In development, we might want to continue without JWT validation
                // In production, this should be a fatal error
                panic!("JWT validator initialization failed: {:?}", e);
            })
    );

    // Create rate limiter
    let rate_limiter = Arc::new(PerIpRateLimiter::new(config.rate_limit.clone()));

    // Create Vault configuration
    let vault_config = vault::VaultConfig::default();

    // Create application state
    let app_state = AppState {
        config: config.clone(),
        jwt_validator,
        rate_limiter,
        db_pool,
        vault_config,
    };

    // Build router
    let app = create_router(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting HoneyLink Control Plane API on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// Create application router with all routes and middleware
fn create_router(state: AppState) -> Router {
    // Create CORS layer
    let cors_layer = create_cors_layer(&state.config.cors);

    // Build middleware stack
    let middleware_stack = ServiceBuilder::new()
        .layer(cors_layer)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(otel_trace_middleware));

    // Define routes
    let api_routes = Router::new()
        .route("/health", get(health_handler))
        .nest("/api/v1/devices", routes::devices::routes())
        .route("/api/v1/sessions", post(sessions_stub))
        // Additional routes will be added in Task 3.3-3.5
        .with_state(state.clone());

    // Combine with middleware
    Router::new()
        .merge(api_routes)
        .layer(middleware_stack)
        .fallback(not_found_handler)
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    let response = types::HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    (StatusCode::OK, Json(response))
}

/// Stub handler for session creation (to be implemented in Task 3.3)
async fn sessions_stub() -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::Internal("Not yet implemented - Task 3.3".to_string()))
}

/// 404 handler
async fn not_found_handler() -> impl IntoResponse {
    let error = ApiError::NotFound("Endpoint not found".to_string());
    error.into_response()
}

/// Initialize OpenTelemetry and tracing
fn init_telemetry(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create OTLP exporter
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.otel.endpoint)
        )
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", config.otel.service_name.clone()),
                    opentelemetry::KeyValue::new("service.version", config.otel.service_version.clone()),
                    opentelemetry::KeyValue::new("deployment.environment", config.otel.environment.clone()),
                ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // Create tracing subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,honeylink_control_plane=debug".into()))
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("honeylink-control-plane")))
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require JWT keys to be set up
    // They are disabled by default and should be run in CI/CD with proper key fixtures

    #[tokio::test]
    #[ignore = "Requires JWT key setup"]
    async fn test_health_endpoint() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let config = AppConfig::default();
        let jwt_validator = Arc::new(JwtValidator::new(config.jwt.clone()).expect("JWT validator required"));
        let rate_limiter = Arc::new(PerIpRateLimiter::new(config.rate_limit.clone()));

        let app_state = AppState {
            config,
            jwt_validator,
            rate_limiter,
        };

        let app = create_router(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "Requires JWT key setup"]
    async fn test_not_found() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let config = AppConfig::default();
        let jwt_validator = Arc::new(JwtValidator::new(config.jwt.clone()).expect("JWT validator required"));
        let rate_limiter = Arc::new(PerIpRateLimiter::new(config.rate_limit.clone()));

        let app_state = AppState {
            config,
            jwt_validator,
            rate_limiter,
        };

        let app = create_router(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
