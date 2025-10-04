//! # Transport Abstraction Layer
//!
//! Transport abstraction with FEC and WFQ support.
//!
//! This module provides:
//! - **Protocol layer**: QUIC/WebRTC transport protocols (Phase 4)
//! - **Physical layer**: Low-level adapter abstraction (BLE/WiFi)
//! - **FEC**: Forward Error Correction strategies
//! - **WFQ**: Weighted Fair Queuing scheduling
//! - **Telemetry**: Link quality monitoring and power management

use async_trait::async_trait;
use std::time::Duration;
use thiserror::Error;

// Phase 4: Transport protocol abstraction (QUIC/WebRTC)
pub mod protocol;

// Existing modules (Physical layer, FEC, WFQ, Telemetry)
pub mod fec;
pub mod retry;
pub mod wfq;
pub mod telemetry;

// Phase 4 exports
pub use protocol::{
    Connection, ConnectionStats, ProtocolStrategy, ProtocolType, Stream, TransportProtocol,
    TransportStats,
};

// Existing exports
pub use fec::{FecEncoder, FecStrategy};
pub use retry::{CircuitBreaker, CircuitState, RetryExecutor, RetryPolicy};
pub use wfq::WeightedFairQueuing;
pub use telemetry::TransportTelemetry;

/// Error types for transport operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// Timeout occurred during send/receive operation (default: 5 seconds)
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),

    /// Physical layer connection is down
    #[error("Physical layer link is down")]
    LinkDown,

    /// WFQ queue buffer is full (max: 10000 packets)
    #[error("Buffer overflow: queue depth exceeds {0} packets")]
    BufferOverflow(usize),

    /// Reed-Solomon FEC decoding failed
    #[error("FEC decoding failed: {0}")]
    FecDecodingFailed(String),

    /// Invalid priority value (must be 0-7)
    #[error("Invalid priority: {0} (valid range: 0-7)")]
    InvalidPriority(u8),

    /// General I/O error
    #[error("I/O error: {0}")]
    Io(String),

    /// Adapter-specific error
    #[error("Adapter error: {0}")]
    AdapterError(String),
}

/// Power consumption modes for physical layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerMode {
    /// Ultra-low power mode: ~5mA consumption
    UltraLow,
    /// Low power mode: ~50mA consumption
    Low,
    /// Normal power mode: ~200mA consumption
    Normal,
    /// High power mode: ~500mA+ consumption
    High,
}

impl PowerMode {
    /// Returns typical current consumption in milliamperes
    pub fn current_ma(&self) -> u16 {
        match self {
            Self::UltraLow => 5,
            Self::Low => 50,
            Self::Normal => 200,
            Self::High => 500,
        }
    }
}

/// Link quality metrics for physical layers
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LinkQualityMetrics {
    /// Received Signal Strength Indicator in dBm
    pub rssi_dbm: i16,
    /// Signal-to-Noise Ratio in dB
    pub snr_db: f32,
    /// Packet loss rate (0.0 to 1.0)
    pub loss_rate: f32,
    /// Current bandwidth in Mbps
    pub bandwidth_mbps: f32,
    /// Round-trip time in milliseconds
    pub rtt_ms: u32,
}

impl LinkQualityMetrics {
    /// Creates a new LinkQualityMetrics with default values
    pub fn new() -> Self {
        Self {
            rssi_dbm: -50,
            snr_db: 30.0,
            loss_rate: 0.0,
            bandwidth_mbps: 100.0,
            rtt_ms: 10,
        }
    }

    /// Returns true if link quality is considered good
    /// Criteria: RSSI > -70 dBm, SNR > 15 dB, loss_rate < 5%
    pub fn is_good(&self) -> bool {
        self.rssi_dbm > -70 && self.snr_db > 15.0 && self.loss_rate < 0.05
    }

    /// Returns true if link quality is degraded and requires intervention
    /// Criteria: RSSI < -80 dBm OR SNR < 10 dB OR loss_rate > 15%
    pub fn is_degraded(&self) -> bool {
        self.rssi_dbm < -80 || self.snr_db < 10.0 || self.loss_rate > 0.15
    }
}

impl Default for LinkQualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Packet to be transmitted over physical layer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    /// Raw packet data
    pub data: Vec<u8>,
    /// Priority level (0-7, higher is more urgent)
    pub priority: u8,
    /// Timestamp when packet was created (Unix epoch milliseconds)
    pub timestamp_ms: u64,
}

impl Packet {
    /// Creates a new packet with given data and priority
    pub fn new(data: Vec<u8>, priority: u8) -> Result<Self, TransportError> {
        if priority > 7 {
            return Err(TransportError::InvalidPriority(priority));
        }
        Ok(Self {
            data,
            priority,
            timestamp_ms: current_time_ms(),
        })
    }

    /// Returns the size of the packet in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Physical layer abstraction trait
///
/// This trait must be implemented by all physical layer adapters (Wi-Fi, 5G, THz, etc.)
/// to provide a unified interface for the transport layer.
///
/// # Design Rationale
/// - Async methods for non-blocking I/O
/// - Error handling via TransportError
/// - Metrics collection for observability
/// - Power management for energy efficiency
#[async_trait]
pub trait PhysicalLayer: Send + Sync {
    /// Sends a packet over the physical layer
    ///
    /// # Arguments
    /// * `packet` - The packet to send
    ///
    /// # Returns
    /// * `Ok(())` if sent successfully
    /// * `Err(TransportError)` on failure (timeout, link down, etc.)
    ///
    /// # Performance
    /// * Target: P95 < 30ms (as per MOD-007 spec)
    async fn send_packet(&self, packet: &Packet) -> Result<(), TransportError>;

    /// Receives a packet from the physical layer
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for a packet
    ///
    /// # Returns
    /// * `Ok(Packet)` if received successfully
    /// * `Err(TransportError::Timeout)` if no packet within timeout
    /// * `Err(TransportError)` on other failures
    async fn recv_packet(&self, timeout: Duration) -> Result<Packet, TransportError>;

    /// Retrieves current link quality metrics
    ///
    /// # Returns
    /// * `Ok(LinkQualityMetrics)` with current measurements
    /// * `Err(TransportError)` if metrics unavailable
    ///
    /// # Polling Interval
    /// * Recommended: 5 seconds (as per MOD-007 spec)
    async fn get_link_quality(&self) -> Result<LinkQualityMetrics, TransportError>;

    /// Sets the power consumption mode
    ///
    /// # Arguments
    /// * `mode` - Desired power mode
    ///
    /// # Returns
    /// * `Ok(())` if mode set successfully
    /// * `Err(TransportError)` if mode change failed
    ///
    /// # Performance
    /// * Target: P95 < 500ms (as per MOD-007 spec)
    async fn set_power_mode(&self, mode: PowerMode) -> Result<(), TransportError>;

    /// Returns the physical layer type identifier
    fn layer_type(&self) -> &str;
}

/// Returns current Unix epoch time in milliseconds
fn current_time_ms() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
