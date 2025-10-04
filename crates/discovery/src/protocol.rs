//! Discovery protocol trait abstraction
//!
//! Defines a common interface for different discovery backends (mDNS, BLE, etc.)
//! to enable pluggable protocol implementations and unified device management.

use crate::error::Result;
use crate::types::{DeviceInfo, DiscoveryEvent};
use async_trait::async_trait;
use std::collections::HashMap;

/// Discovery protocol trait
///
/// This trait defines the common interface for all discovery backends.
/// Implementations must provide methods for:
/// - Starting/stopping discovery
/// - Announcing own device
/// - Browsing for remote devices
/// - Managing device lifecycle
///
/// # Design Rationale
/// - Trait-based design enables Phase 2 to plug in real BLE implementation without API changes
/// - Async methods support non-blocking I/O for network operations
/// - Unified event stream simplifies multi-protocol coordination
/// - Device lifetime management (start/stop) is explicit for resource cleanup
#[async_trait]
pub trait DiscoveryProtocol: Send + Sync {
    /// Protocol name (e.g., "mDNS", "BLE")
    fn protocol_name(&self) -> &'static str;

    /// Start announcing own device
    ///
    /// Makes the device discoverable by other peers on this protocol.
    /// Should be idempotent (calling twice has no additional effect).
    async fn start_announcing(&mut self) -> Result<()>;

    /// Stop announcing own device
    ///
    /// Removes device from discoverable list.
    /// Should be idempotent (calling twice has no additional effect).
    async fn stop_announcing(&mut self) -> Result<()>;

    /// Start browsing for remote devices
    ///
    /// Begins active discovery of peers on this protocol.
    /// Discovered devices are reported via the event channel.
    async fn start_browsing(&mut self) -> Result<()>;

    /// Stop browsing for remote devices
    ///
    /// Stops active discovery and releases resources.
    async fn stop_browsing(&mut self) -> Result<()>;

    /// Get currently discovered devices
    ///
    /// Returns a snapshot of devices discovered via this protocol.
    /// Key: device_id, Value: DeviceInfo
    async fn get_devices(&self) -> HashMap<String, DeviceInfo>;

    /// Check if protocol is currently running
    async fn is_running(&self) -> bool;
}

/// Protocol-specific event wrapper
///
/// Wraps DiscoveryEvent with protocol metadata for source tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolEvent {
    /// Source protocol (e.g., "mDNS", "BLE")
    pub protocol: String,
    /// Underlying discovery event
    pub event: DiscoveryEvent,
}

impl ProtocolEvent {
    /// Create new protocol event
    pub fn new(protocol: impl Into<String>, event: DiscoveryEvent) -> Self {
        Self {
            protocol: protocol.into(),
            event,
        }
    }
}

/// Protocol selection strategy
///
/// Determines which protocols to use based on network conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolStrategy {
    /// Use all available protocols simultaneously
    All,
    /// Prefer mDNS, fallback to BLE if network unavailable
    Prefer(PreferredProtocol),
    /// Use only specified protocol
    Only(ProtocolType),
}

/// Preferred protocol for fallback strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferredProtocol {
    /// Prefer mDNS (faster, more reliable on LAN)
    Mdns,
    /// Prefer BLE (works without network)
    Ble,
}

/// Protocol type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    /// mDNS-SD (Multicast DNS Service Discovery)
    Mdns,
    /// BLE (Bluetooth Low Energy)
    Ble,
}

impl ProtocolType {
    /// Get protocol name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mdns => "mDNS",
            Self::Ble => "BLE",
        }
    }
}

impl Default for ProtocolStrategy {
    /// Default strategy: prefer mDNS with BLE fallback
    ///
    /// Rationale: mDNS is faster and more reliable on LAN,
    /// BLE works in scenarios without network infrastructure
    fn default() -> Self {
        Self::Prefer(PreferredProtocol::Mdns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DeviceType;

    #[test]
    fn test_protocol_type_as_str() {
        assert_eq!(ProtocolType::Mdns.as_str(), "mDNS");
        assert_eq!(ProtocolType::Ble.as_str(), "BLE");
    }

    #[test]
    fn test_protocol_event_creation() {
        use crate::types::DeviceInfo;
        let device_info = DeviceInfo::new("DEV-001", "Test Device", DeviceType::Desktop)
            .with_addresses(vec!["192.168.1.100".parse().unwrap()])
            .with_port(7843);

        let event = ProtocolEvent::new("mDNS", DiscoveryEvent::DeviceFound(device_info.clone()));
        assert_eq!(event.protocol, "mDNS");
        assert_eq!(
            event.event,
            DiscoveryEvent::DeviceFound(device_info)
        );
    }

    #[test]
    fn test_default_strategy() {
        let strategy = ProtocolStrategy::default();
        assert_eq!(strategy, ProtocolStrategy::Prefer(PreferredProtocol::Mdns));
    }
}
