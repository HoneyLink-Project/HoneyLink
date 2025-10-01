//! Session Orchestrator error types

use honeylink_core::Error as CoreError;

/// Session Orchestrator specific errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: crate::state_machine::SessionState,
        to: crate::state_machine::SessionState,
    },

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session expired: {0}")]
    SessionExpired(String),

    #[error("Idempotency key already exists: {0}")]
    IdempotencyKeyExists(String),

    #[error("Protocol version negotiation failed: client={client}, server={server}")]
    VersionNegotiationFailed { client: String, server: String },

    #[error("Protocol version not supported: {0}")]
    UnsupportedVersion(String),

    #[error("Session TTL exceeded: {0}")]
    TtlExceeded(String),

    #[error("Device authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Policy rejected for session: {0}")]
    PolicyRejected(String),

    #[error("Network timeout: {0}")]
    NetworkTimeout(String),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Event bus error: {0}")]
    EventBusError(String),

    #[error(transparent)]
    Core(#[from] CoreError),
}

pub type Result<T> = std::result::Result<T, Error>;
