//! Policy Engine Error Types

use thiserror::Error;

/// Policy Engine operation errors
#[derive(Debug, Error)]
pub enum PolicyError {
    /// Version parsing or comparison failed
    #[error("Version error: {0}")]
    Version(#[from] semver::Error),

    /// Policy validation failed
    #[error("Validation error: {0}")]
    Validation(String),

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureInvalid(String),

    /// Policy not found
    #[error("Policy not found: {0}")]
    NotFound(String),

    /// Policy conflict (e.g., duplicate ID)
    #[error("Policy conflict: {0}")]
    Conflict(String),

    /// Policy has been deprecated
    #[error("Policy deprecated after: {0}")]
    Deprecated(String),

    /// Storage operation failed
    #[error("Storage error: {0}")]
    Storage(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid timestamp
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// Event bus error
    #[error("Event bus error: {0}")]
    EventBus(String),
}

pub type Result<T> = std::result::Result<T, PolicyError>;
