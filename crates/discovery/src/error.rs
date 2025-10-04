//! Error types for HoneyLink Discovery

use thiserror::Error;

/// Discovery error types
#[derive(Error, Debug)]
pub enum DiscoveryError {
    /// mDNS service error
    #[error("mDNS error: {0}")]
    MdnsError(String),

    /// BLE adapter error
    #[error("BLE error: {0}")]
    BleError(String),

    /// Invalid device information
    #[error("Invalid device info: {0}")]
    InvalidDeviceInfo(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout error
    #[error("Discovery timeout after {0} seconds")]
    Timeout(u64),

    /// Service already running
    #[error("Discovery service already running")]
    AlreadyRunning,

    /// Service not started
    #[error("Discovery service not started")]
    NotStarted,

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for discovery operations
pub type Result<T> = std::result::Result<T, DiscoveryError>;
