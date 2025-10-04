//! Device information types for discovery

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Device type categories (Bluetooth-compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    /// Desktop/Laptop computer
    Desktop,
    /// Mobile phone/tablet
    Mobile,
    /// IoT sensor/actuator
    Iot,
    /// Server/NAS
    Server,
    /// Unknown device type
    Unknown,
}

impl DeviceType {
    /// Parse device type from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "desktop" => Self::Desktop,
            "mobile" => Self::Mobile,
            "iot" => Self::Iot,
            "server" => Self::Server,
            _ => Self::Unknown,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
            Self::Iot => "iot",
            Self::Server => "server",
            Self::Unknown => "unknown",
        }
    }

    /// Convert to u8 for binary serialization (GATT protocol)
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Desktop => 1,
            Self::Mobile => 2,
            Self::Iot => 3,
            Self::Server => 4,
            Self::Unknown => 0,
        }
    }

    /// Convert from u8 (with fallback to Unknown)
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Desktop,
            2 => Self::Mobile,
            3 => Self::Iot,
            4 => Self::Server,
            _ => Self::Unknown,
        }
    }
}

/// Discovered device information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier (e.g., "DEV-001")
    pub device_id: String,

    /// Human-readable device name (e.g., "Alice's Laptop")
    pub device_name: String,

    /// Device type category
    pub device_type: DeviceType,

    /// HoneyLink protocol version (SemVer)
    pub version: String,

    /// Device IP addresses (IPv4/IPv6)
    pub addresses: Vec<IpAddr>,

    /// QUIC port (default: 7843)
    pub port: u16,

    /// Signal strength (RSSI in dBm, for BLE)
    pub rssi: Option<i16>,

    /// Discovery timestamp (Unix epoch milliseconds)
    pub discovered_at: u64,
}

impl DeviceInfo {
    /// Create new device info
    pub fn new(
        device_id: impl Into<String>,
        device_name: impl Into<String>,
        device_type: DeviceType,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            device_name: device_name.into(),
            device_type,
            version: env!("CARGO_PKG_VERSION").to_string(),
            addresses: Vec::new(),
            port: 7843, // Default QUIC port
            rssi: None,
            discovered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Set IP addresses
    pub fn with_addresses(mut self, addresses: Vec<IpAddr>) -> Self {
        self.addresses = addresses;
        self
    }

    /// Set port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set RSSI (signal strength)
    pub fn with_rssi(mut self, rssi: i16) -> Self {
        self.rssi = Some(rssi);
        self
    }
}

/// Discovery events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryEvent {
    /// New device discovered
    DeviceFound(DeviceInfo),

    /// Device lost (went offline)
    DeviceLost(String), // device_id

    /// Network interface changed
    NetworkChanged,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_parsing() {
        assert_eq!(DeviceType::from_str("desktop"), DeviceType::Desktop);
        assert_eq!(DeviceType::from_str("MOBILE"), DeviceType::Mobile);
        assert_eq!(DeviceType::from_str("unknown"), DeviceType::Unknown);
    }

    #[test]
    fn test_device_info_creation() {
        let device = DeviceInfo::new("DEV-001", "Test Device", DeviceType::Desktop);
        assert_eq!(device.device_id, "DEV-001");
        assert_eq!(device.device_name, "Test Device");
        assert_eq!(device.device_type, DeviceType::Desktop);
        assert_eq!(device.port, 7843);
    }

    #[test]
    fn test_device_info_builder() {
        let device = DeviceInfo::new("DEV-002", "Builder Test", DeviceType::Mobile)
            .with_port(8080)
            .with_rssi(-50);

        assert_eq!(device.port, 8080);
        assert_eq!(device.rssi, Some(-50));
    }
}
