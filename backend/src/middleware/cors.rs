// CORS middleware configuration

use crate::config::CorsConfig;
use tower_http::cors::{Any, CorsLayer};
use std::time::Duration;

/// Create CORS layer from configuration
pub fn create_cors_layer(config: &CorsConfig) -> CorsLayer {
    let mut cors = CorsLayer::new();

    // Configure allowed origins
    if config.allowed_origins.is_empty() || config.allowed_origins.contains(&"*".to_string()) {
        cors = cors.allow_origin(Any);
    } else {
        let origins: Vec<_> = config
            .allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        cors = cors.allow_origin(origins);
    }

    // Configure allowed methods
    cors = cors.allow_methods([
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::PUT,
        axum::http::Method::DELETE,
        axum::http::Method::OPTIONS,
        axum::http::Method::PATCH,
    ]);

    // Configure allowed headers
    cors = cors.allow_headers([
        axum::http::header::AUTHORIZATION,
        axum::http::header::CONTENT_TYPE,
        axum::http::header::ACCEPT,
        axum::http::HeaderName::from_static("traceparent"),
        axum::http::HeaderName::from_static("tracestate"),
    ]);

    // Configure exposed headers
    cors = cors.expose_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderName::from_static("traceparent"),
        axum::http::HeaderName::from_static("x-request-id"),
    ]);

    // Configure credentials
    if config.allow_credentials {
        cors = cors.allow_credentials(true);
    }

    // Configure max age for preflight cache
    cors = cors.max_age(Duration::from_secs(config.max_age_seconds));

    cors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_layer_creation() {
        let config = CorsConfig {
            allowed_origins: vec!["https://console.honeylink.local".to_string()],
            allow_credentials: true,
            max_age_seconds: 3600,
        };

        let cors_layer = create_cors_layer(&config);
        // Layer is created successfully
        // Actual CORS behavior testing requires integration tests
    }

    #[test]
    fn test_cors_wildcard() {
        let config = CorsConfig {
            allowed_origins: vec!["*".to_string()],
            allow_credentials: false,
            max_age_seconds: 1800,
        };

        let cors_layer = create_cors_layer(&config);
        // Wildcard origin should work
    }
}
