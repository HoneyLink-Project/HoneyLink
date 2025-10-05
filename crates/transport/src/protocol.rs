//! Transport protocol trait abstraction
//!
//! Defines a common interface for different transport backends (QUIC, WebRTC, etc.)
//! to enable pluggable protocol implementations and unified connection management.
//!
//! # Architecture
//!
//! This follows the same trait-based design pattern as Phase 1's DiscoveryProtocol:
//! - Trait defines async API for connection lifecycle
//! - Multiple backends (QUIC, WebRTC) implement the same trait
//! - TransportManager coordinates multiple protocols
//! - Enables testing with mock implementations
//!
//! # Design Rationale
//!
//! - **Trait-based**: Enables Phase 4 to add WebRTC without changing QUIC code
//! - **Async methods**: Non-blocking I/O for network operations
//! - **Stream-based**: Supports multiple concurrent data streams per connection
//! - **Error handling**: Explicit Result types for connection failures
//! - **Protocol selection**: Manager can choose QUIC vs WebRTC based on network conditions

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;

/// Result type for transport operations
pub type Result<T> = std::result::Result<T, TransportError>;

/// Transport protocol errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// Connection establishment failed
    #[error("Failed to establish connection: {0}")]
    ConnectionFailed(String),

    /// Connection timeout (default: 5 seconds)
    #[error("Connection timed out after {0:?}")]
    ConnectionTimeout(Duration),

    /// Send operation failed
    #[error("Failed to send data: {0}")]
    SendFailed(String),

    /// Receive operation failed
    #[error("Failed to receive data: {0}")]
    ReceiveFailed(String),

    /// Connection closed by peer
    #[error("Connection closed by peer")]
    ConnectionClosed,

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Protocol not supported
    #[error("Protocol not supported: {0}")]
    ProtocolNotSupported(String),

    /// Resource exhausted (bandwidth or streams)
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// TLS/DTLS error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// NAT traversal failed
    #[error("NAT traversal failed: {0}")]
    NatTraversalFailed(String),
}

/// Transport protocol trait
///
/// This trait defines the common interface for all transport backends.
/// Implementations must provide methods for:
/// - Establishing connections (client mode)
/// - Listening for connections (server mode)
/// - Sending/receiving data on streams
/// - Managing connection lifecycle
///
/// # Design Rationale
/// - Trait-based design enables Phase 4 to plug in WebRTC implementation without API changes
/// - Async methods support non-blocking I/O for network operations
/// - Stream-based API supports multiple concurrent data transfers per connection
/// - Explicit connection handles enable multi-connection scenarios
#[async_trait]
pub trait TransportProtocol: Send + Sync {
    /// Protocol name (e.g., "QUIC", "WebRTC")
    fn protocol_name(&self) -> &'static str;

    /// Connect to a remote peer (client mode)
    ///
    /// Establishes a connection to the specified address.
    /// Returns a connection handle for sending/receiving data.
    ///
    /// # Arguments
    /// * `addr` - Remote peer address (IP:port)
    /// * `timeout` - Connection timeout duration
    ///
    /// # Returns
    /// * `Ok(Arc<dyn Connection>)` - Connection handle on success
    /// * `Err(TransportError)` - Connection failure
    async fn connect(&self, addr: SocketAddr, timeout: Duration) -> Result<Arc<dyn Connection>>;

    /// Listen for incoming connections (server mode)
    ///
    /// Starts listening on the specified address for peer connections.
    /// Incoming connections are sent to the returned channel.
    ///
    /// # Arguments
    /// * `addr` - Local address to bind (IP:port)
    ///
    /// # Returns
    /// * `Ok(mpsc::Receiver<Arc<dyn Connection>>)` - Channel of incoming connections
    /// * `Err(TransportError)` - Failed to start listener
    async fn listen(&self, addr: SocketAddr) -> Result<mpsc::Receiver<Arc<dyn Connection>>>;

    /// Stop listening for connections
    ///
    /// Shuts down the listener and closes all pending connections.
    /// Should be idempotent (calling twice has no additional effect).
    async fn stop_listening(&self) -> Result<()>;

    /// Check if protocol is currently listening
    async fn is_listening(&self) -> bool;

    /// Get protocol-specific statistics
    async fn stats(&self) -> TransportStats;
}

/// Connection handle for an established transport connection
///
/// Represents a single connection to a remote peer.
/// Supports multiple concurrent streams for data transfer.
#[async_trait]
pub trait Connection: Send + Sync {
    /// Get remote peer address
    fn remote_addr(&self) -> SocketAddr;

    /// Get local address
    fn local_addr(&self) -> SocketAddr;

    /// Send data on this connection
    ///
    /// Opens a new unidirectional stream and sends data.
    /// For bidirectional communication, use `open_stream()`.
    ///
    /// # Arguments
    /// * `data` - Bytes to send
    ///
    /// # Returns
    /// * `Ok(())` - Data sent successfully
    /// * `Err(TransportError)` - Send failed
    async fn send(&self, data: &[u8]) -> Result<()>;

    /// Receive data from this connection
    ///
    /// Blocks until data is available or connection is closed.
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Received data
    /// * `Err(TransportError)` - Receive failed or connection closed
    async fn receive(&self) -> Result<Vec<u8>>;

    /// Open a bidirectional stream
    ///
    /// Creates a new stream for bidirectional communication.
    /// Returns send/receive channels for the stream.
    ///
    /// # Returns
    /// * `Ok(Stream)` - Stream handle
    /// * `Err(TransportError)` - Failed to open stream
    async fn open_stream(&self) -> Result<Box<dyn Stream>>;

    /// Open a bidirectional stream with priority
    ///
    /// Creates a new stream with QoS priority for bandwidth allocation
    /// and latency optimization. Priority affects:
    /// - Bandwidth allocation (High priority gets more bandwidth)
    /// - Latency (High priority streams processed first)
    /// - Congestion control (High priority less affected by backpressure)
    ///
    /// # Arguments
    /// * `priority` - Stream priority level (High/Normal/Low)
    ///
    /// # Returns
    /// * `Ok(Stream)` - Stream handle with priority
    /// * `Err(TransportError)` - Failed to open stream
    ///
    /// # Default Implementation
    /// Falls back to `open_stream()` for protocols without QoS support.
    async fn open_stream_with_priority(&self, _priority: StreamPriority) -> Result<Box<dyn Stream>> {
        self.open_stream().await
    }

    /// Close the connection gracefully
    ///
    /// Sends close signal to peer and waits for acknowledgment.
    /// After this call, send/receive operations will fail.
    async fn close(&self) -> Result<()>;

    /// Check if connection is still alive
    fn is_connected(&self) -> bool;

    /// Get connection statistics
    fn stats(&self) -> ConnectionStats;
}

/// Bidirectional stream handle
///
/// Represents a single stream within a connection.
/// Enables multiple concurrent data transfers over one connection.
#[async_trait]
pub trait Stream: Send + Sync {
    /// Send data on this stream
    async fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Receive data from this stream
    async fn receive(&mut self) -> Result<Vec<u8>>;

    /// Close this stream
    async fn close(&mut self) -> Result<()>;
}

/// Transport protocol statistics
#[derive(Debug, Clone, Default)]
pub struct TransportStats {
    /// Total number of connections established
    pub connections_established: u64,
    /// Total number of connection failures
    pub connections_failed: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Current active connections
    pub active_connections: usize,
}

/// Connection-specific statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Connection start time (Unix timestamp)
    pub start_time: u64,
    /// Bytes sent on this connection
    pub bytes_sent: u64,
    /// Bytes received on this connection
    pub bytes_received: u64,
    /// Round-trip time in milliseconds
    pub rtt_ms: u32,
    /// Number of active streams
    pub active_streams: usize,
}

/// Protocol selection strategy
///
/// Determines which transport protocol to use based on network conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolStrategy {
    /// Use all available protocols simultaneously (fastest connection wins)
    All,
    /// Prefer QUIC, fallback to WebRTC if QUIC unavailable
    PreferQuic,
    /// Prefer WebRTC, fallback to QUIC if WebRTC unavailable
    PreferWebRtc,
    /// Use only QUIC
    QuicOnly,
    /// Use only WebRTC
    WebRtcOnly,
}

impl Default for ProtocolStrategy {
    /// Default strategy: prefer QUIC with WebRTC fallback
    ///
    /// Rationale: QUIC provides lower latency and better congestion control,
    /// WebRTC works better behind restrictive NATs/firewalls
    fn default() -> Self {
        Self::PreferQuic
    }
}

/// Protocol type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    /// QUIC (Quick UDP Internet Connections)
    Quic,
    /// WebRTC (Web Real-Time Communication)
    WebRtc,
}

impl ProtocolType {
    /// Get protocol name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quic => "QUIC",
            Self::WebRtc => "WebRTC",
        }
    }
}

/// Stream priority for QoS-aware stream allocation
///
/// Priority levels map to QoS Scheduler's QoSPriority:
/// - High: Burst traffic (video streaming, high bandwidth)
/// - Normal: Standard traffic (telemetry, control messages)
/// - Low: Background traffic (bulk transfers, low latency)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamPriority {
    /// High priority (burst traffic)
    High,
    /// Normal priority (standard traffic)
    Normal,
    /// Low priority (background traffic)
    Low,
}

impl Default for StreamPriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_type_as_str() {
        assert_eq!(ProtocolType::Quic.as_str(), "QUIC");
        assert_eq!(ProtocolType::WebRtc.as_str(), "WebRTC");
    }

    #[test]
    fn test_default_strategy() {
        let strategy = ProtocolStrategy::default();
        assert_eq!(strategy, ProtocolStrategy::PreferQuic);
    }

    #[test]
    fn test_transport_stats_default() {
        let stats = TransportStats::default();
        assert_eq!(stats.connections_established, 0);
        assert_eq!(stats.connections_failed, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_connection_stats_default() {
        let stats = ConnectionStats::default();
        assert_eq!(stats.start_time, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.rtt_ms, 0);
        assert_eq!(stats.active_streams, 0);
    }

    #[test]
    fn test_transport_error_display() {
        let err = TransportError::ConnectionFailed("timeout".to_string());
        assert_eq!(err.to_string(), "Failed to establish connection: timeout");

        let err = TransportError::ConnectionClosed;
        assert_eq!(err.to_string(), "Connection closed by peer");
    }
}
