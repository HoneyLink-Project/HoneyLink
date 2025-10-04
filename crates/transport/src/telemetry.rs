//! Telemetry trait for transport layer
//!
//! Defines trait for recording transport metrics to avoid circular dependency.
//! Implementation is provided in honeylink-telemetry crate.

use async_trait::async_trait;
use crate::LinkQualityMetrics;

/// Transport telemetry recorder trait
///
/// Abstraction for recording transport-specific metrics:
/// - packet_loss_rate (SLI)
/// - qos_packet_drop_rate (SLI)
/// - Link quality metrics (RSSI, SNR, RTT)
/// - FEC strategy changes
/// - WFQ queue depth
#[async_trait]
pub trait TransportTelemetry: Send + Sync {
    /// Create new transport telemetry recorder
    fn new_transport_telemetry() -> Self where Self: Sized;

    /// Record packet loss rate (SLI metric)
    ///
    /// This is a key metric per spec/testing/metrics.md:
    /// - Green: < 0.1%
    /// - Yellow: 0.1-0.5%
    /// - Orange: 0.5-1.0%
    /// - Red: > 1.0%
    /// - SLO: P95 < 0.5%
    ///
    /// # Arguments
    /// * `loss_rate` - Packet loss rate (0.0 to 1.0)
    /// * `physical_layer` - Physical layer identifier (e.g., "lora", "ble", "wifi")
    async fn record_packet_loss_rate(
        &self,
        loss_rate: f32,
        physical_layer: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Record QoS packet drop rate (SLI metric)
    ///
    /// Tracks packets dropped by WFQ scheduler due to buffer overflow.
    /// This is a key metric per spec/testing/metrics.md:
    /// - Green: < 0.01%
    /// - Yellow: 0.01-0.1%
    /// - Orange: 0.1-0.5%
    /// - Red: > 0.5%
    /// - SLO: P95 < 0.1%
    ///
    /// # Arguments
    /// * `drop_rate` - Drop rate (0.0 to 1.0)
    /// * `priority` - QoS priority level (0-7)
    async fn record_qos_packet_drop_rate(
        &self,
        drop_rate: f32,
        priority: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Record link quality metrics
    ///
    /// Records RSSI, SNR, RTT, and bandwidth for physical layer monitoring.
    ///
    /// # Arguments
    /// * `metrics` - Link quality metrics snapshot
    /// * `physical_layer` - Physical layer identifier
    async fn record_link_quality(
        &self,
        metrics: &LinkQualityMetrics,
        physical_layer: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Record FEC strategy change event
    ///
    /// Tracks when FEC encoding strategy is switched (e.g., RS(32,24) → RS(64,48)).
    ///
    /// # Arguments
    /// * `from_strategy` - Previous FEC strategy
    /// * `to_strategy` - New FEC strategy
    /// * `reason` - Reason for change (e.g., "high_loss_rate", "low_snr")
    async fn record_fec_strategy_change(
        &self,
        from_strategy: &str,
        to_strategy: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Record WFQ queue depth
    ///
    /// Tracks current queue depth for each priority level.
    ///
    /// # Arguments
    /// * `queue_depth` - Current number of packets in queue
    /// * `priority` - QoS priority level (0-7)
    async fn record_wfq_queue_depth(
        &self,
        queue_depth: usize,
        priority: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Record transport throughput
    ///
    /// Tracks bytes transmitted per second through transport layer.
    ///
    /// # Arguments
    /// * `bytes_per_sec` - Throughput in bytes per second
    /// * `physical_layer` - Physical layer identifier
    async fn record_throughput(
        &self,
        bytes_per_sec: u64,
        physical_layer: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
