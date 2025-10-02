//! Transport and Physical Layer telemetry events
//!
//! This module defines telemetry events for:
//! - Link state changes (Hot Swap events)
//! - FEC strategy changes
//! - WFQ queue depth monitoring
//! - Packet transmission metrics
//! - Power mode changes

use honeylink_transport::{FecStrategy, LinkQualityMetrics, PowerMode};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Transport layer event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransportEvent {
    /// Link state changed (Hot Swap occurred)
    LinkStateChange(LinkStateChangeEvent),
    /// FEC strategy changed
    FecStrategyChange(FecStrategyChangeEvent),
    /// WFQ queue depth warning
    QueueDepthWarning(QueueDepthWarningEvent),
    /// Packet sent successfully
    PacketSent(PacketSentEvent),
    /// Packet receive failed
    PacketReceiveFailed(PacketReceiveFailedEvent),
    /// Power mode changed
    PowerModeChange(PowerModeChangeEvent),
}

/// Link state change event (Hot Swap)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkStateChangeEvent {
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp_ms: u64,
    /// Previous physical layer type
    pub from_type: String,
    /// New physical layer type
    pub to_type: String,
    /// Duration of the switch in milliseconds
    pub duration_ms: u64,
    /// Reason for the switch
    pub reason: String,
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,
}

impl LinkStateChangeEvent {
    /// Creates a new link state change event
    pub fn new(
        from_type: String,
        to_type: String,
        duration_ms: u64,
        reason: String,
    ) -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            from_type,
            to_type,
            duration_ms,
            reason,
            trace_id: None,
        }
    }
}

/// FEC strategy change event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FecStrategyChangeEvent {
    pub timestamp_ms: u64,
    pub from_strategy: String,
    pub to_strategy: String,
    pub packet_loss_rate: f32,
    pub trace_id: Option<String>,
}

impl FecStrategyChangeEvent {
    pub fn new(from: FecStrategy, to: FecStrategy, packet_loss_rate: f32) -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            from_strategy: format!("{:?}", from),
            to_strategy: format!("{:?}", to),
            packet_loss_rate,
            trace_id: None,
        }
    }
}

/// Queue depth warning event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueDepthWarningEvent {
    pub timestamp_ms: u64,
    pub priority: u8,
    pub queue_depth: usize,
    pub max_queue_depth: usize,
    pub trace_id: Option<String>,
}

impl QueueDepthWarningEvent {
    pub fn new(priority: u8, queue_depth: usize, max_queue_depth: usize) -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            priority,
            queue_depth,
            max_queue_depth,
            trace_id: None,
        }
    }
}

/// Packet sent event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PacketSentEvent {
    pub timestamp_ms: u64,
    pub physical_type: String,
    pub priority: u8,
    pub packet_size_bytes: usize,
    pub send_latency_ms: u32,
    pub fec_mode: String,
    pub trace_id: Option<String>,
}

impl PacketSentEvent {
    pub fn new(
        physical_type: String,
        priority: u8,
        packet_size_bytes: usize,
        send_latency_ms: u32,
        fec_mode: FecStrategy,
    ) -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            physical_type,
            priority,
            packet_size_bytes,
            send_latency_ms,
            fec_mode: format!("{:?}", fec_mode),
            trace_id: None,
        }
    }
}

/// Packet receive failed event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PacketReceiveFailedEvent {
    pub timestamp_ms: u64,
    pub physical_type: String,
    pub error: String,
    pub trace_id: Option<String>,
}

impl PacketReceiveFailedEvent {
    pub fn new(physical_type: String, error: String) -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            physical_type,
            error,
            trace_id: None,
        }
    }
}

/// Power mode change event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PowerModeChangeEvent {
    pub timestamp_ms: u64,
    pub physical_type: String,
    pub from_mode: String,
    pub to_mode: String,
    pub trace_id: Option<String>,
}

impl PowerModeChangeEvent {
    pub fn new(physical_type: String, from_mode: PowerMode, to_mode: PowerMode) -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            physical_type,
            from_mode: format!("{:?}", from_mode),
            to_mode: format!("{:?}", to_mode),
            trace_id: None,
        }
    }
}

/// Transport metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMetrics {
    pub timestamp_ms: u64,
    pub physical_type: String,
    pub link_quality: LinkQualityMetrics,
    pub packets_sent_total: u64,
    pub packets_received_total: u64,
    pub fec_encoding_duration_p95_ms: f32,
    pub wfq_queue_depth_high: usize,
    pub wfq_queue_depth_medium: usize,
    pub wfq_queue_depth_low: usize,
    pub fec_effectiveness_percent: f32,
}

impl TransportMetrics {
    pub fn new(physical_type: String, link_quality: LinkQualityMetrics) -> Self {
        Self {
            timestamp_ms: current_time_ms(),
            physical_type,
            link_quality,
            packets_sent_total: 0,
            packets_received_total: 0,
            fec_encoding_duration_p95_ms: 0.0,
            wfq_queue_depth_high: 0,
            wfq_queue_depth_medium: 0,
            wfq_queue_depth_low: 0,
            fec_effectiveness_percent: 0.0,
        }
    }
}

/// Returns current Unix epoch time in milliseconds
fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_state_change_event() {
        let event = LinkStateChangeEvent::new(
            "WiFi6E".into(),
            "5G".into(),
            1800,
            "Link degraded".into(),
        );
        assert_eq!(event.from_type, "WiFi6E");
        assert_eq!(event.to_type, "5G");
        assert_eq!(event.duration_ms, 1800);
    }

    #[test]
    fn test_fec_strategy_change_event() {
        let event =
            FecStrategyChangeEvent::new(FecStrategy::None, FecStrategy::Light, 0.07);
        assert_eq!(event.from_strategy, "None");
        assert_eq!(event.to_strategy, "Light");
        assert_eq!(event.packet_loss_rate, 0.07);
    }

    #[test]
    fn test_transport_event_serialization() {
        let event = TransportEvent::LinkStateChange(LinkStateChangeEvent::new(
            "WiFi".into(),
            "5G".into(),
            2000,
            "Test".into(),
        ));

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("link_state_change"));
        assert!(json.contains("WiFi"));
    }
}
