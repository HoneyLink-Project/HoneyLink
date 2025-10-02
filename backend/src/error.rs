// Error types for HoneyLink Control Plane API
// Implements unified error model as per spec/api/control-plane.md

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Unified error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    pub trace_id: Option<String>,
}

/// API error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("State transition error: {0}")]
    State(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Dependency unavailable: {0}")]
    Dependency(String),
}

impl ApiError {
    /// Returns the error code string as per spec
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::Validation(_) => "ERR_VALIDATION",
            ApiError::Authentication(_) => "ERR_AUTH",
            ApiError::Authorization(_) => "ERR_AUTHZ",
            ApiError::NotFound(_) => "ERR_NOT_FOUND",
            ApiError::Conflict(_) => "ERR_CONFLICT",
            ApiError::State(_) => "ERR_STATE",
            ApiError::Internal(_) => "ERR_INTERNAL",
            ApiError::Dependency(_) => "ERR_DEPENDENCY",
        }
    }

    /// Maps error to appropriate HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::Authentication(_) => StatusCode::UNAUTHORIZED,
            ApiError::Authorization(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::State(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Dependency(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Extract trace_id from current span (if available)
        let trace_id = extract_trace_id();

        let error_response = ErrorResponse {
            error_code: self.error_code().to_string(),
            message: self.to_string(),
            trace_id,
        };

        // Log error with appropriate level
        match &self {
            ApiError::Internal(_) | ApiError::Dependency(_) => {
                tracing::error!(error = ?self, "API error occurred");
            }
            _ => {
                tracing::warn!(error = ?self, "API error occurred");
            }
        }

        (self.status_code(), Json(error_response)).into_response()
    }
}

/// Extract trace_id from current OpenTelemetry span
fn extract_trace_id() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let span = tracing::Span::current();
    let context = span.context();
    let span_context = context.span().span_context();

    if span_context.is_valid() {
        Some(span_context.trace_id().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ApiError::Validation("test".into()).error_code(), "ERR_VALIDATION");
        assert_eq!(ApiError::Authentication("test".into()).error_code(), "ERR_AUTH");
        assert_eq!(ApiError::Authorization("test".into()).error_code(), "ERR_AUTHZ");
        assert_eq!(ApiError::NotFound("test".into()).error_code(), "ERR_NOT_FOUND");
        assert_eq!(ApiError::Conflict("test".into()).error_code(), "ERR_CONFLICT");
        assert_eq!(ApiError::State("test".into()).error_code(), "ERR_STATE");
        assert_eq!(ApiError::Internal("test".into()).error_code(), "ERR_INTERNAL");
        assert_eq!(ApiError::Dependency("test".into()).error_code(), "ERR_DEPENDENCY");
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(ApiError::Validation("test".into()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::Authentication("test".into()).status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::Authorization("test".into()).status_code(), StatusCode::FORBIDDEN);
        assert_eq!(ApiError::NotFound("test".into()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::Conflict("test".into()).status_code(), StatusCode::CONFLICT);
        assert_eq!(ApiError::State("test".into()).status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(ApiError::Internal("test".into()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ApiError::Dependency("test".into()).status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_error_response_format() {
        let err = ApiError::Validation("invalid device_id".to_string());
        let response = ErrorResponse {
            error_code: err.error_code().to_string(),
            message: err.to_string(),
            trace_id: Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("ERR_VALIDATION"));
        assert!(json.contains("invalid device_id"));
    }
}
