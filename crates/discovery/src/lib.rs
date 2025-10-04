//! HoneyLink P2P Device Discovery
//!
//! Pure Rust implementation of device discovery using mDNS-SD and BLE.
//! Provides Bluetooth-compatible UX: nearby devices discovered in 3-5 seconds.
//!
//! # Architecture
//!
//! - **No servers**: Devices discover each other via local multicast (mDNS) or BLE
//! - **Automatic announcement**: Devices announce themselves on startup
//! - **Graceful shutdown**: Unregister on app close
//! - **Network resilience**: Re-announce on network changes
//! - **Mobile support**: BLE discovery for devices without mDNS
//!
//! # Examples
//!
//! ```no_run
//! use honeylink_discovery::{DiscoveryService, DeviceInfo};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Start discovery service
//!     let mut service = DiscoveryService::new("DEV-001", "My Laptop", "desktop")?;
//!     service.start().await?;
//!
//!     // Discover nearby devices
//!     let devices = service.discover_devices(5).await?;
//!     for device in devices {
//!         println!("Found: {} ({})", device.device_name, device.device_id);
//!     }
//!
//!     // Graceful shutdown
//!     service.stop().await?;
//!     Ok(())
//! }
//! ```

pub mod ble;
pub mod error;
pub mod gatt;
pub mod manager;
pub mod mdns;
pub mod network_monitor;
pub mod protocol;
pub mod types;

pub use ble::BleDiscovery;
pub use error::{DiscoveryError, Result};
pub use gatt::{
    GattDeviceInfo, GattPairingState, PairingState, DEVICE_INFO_CHAR_UUID,
    HONEYLINK_SERVICE_UUID, MAX_GATT_VALUE_SIZE, PAIRING_STATE_CHAR_UUID,
};
pub use manager::DiscoveryManager;
pub use mdns::MdnsDiscovery;
pub use network_monitor::{NetworkEvent, NetworkMonitor};
pub use types::{DeviceInfo, DeviceType, DiscoveryEvent};

use tokio::sync::mpsc;
use tracing::info;

/// P2P Discovery Service (mDNS + BLE)
///
/// Provides automatic device discovery with Bluetooth-compatible UX.
pub struct DiscoveryService {
    mdns: MdnsDiscovery,
    ble: Option<BleDiscovery>,
    event_rx: mpsc::Receiver<DiscoveryEvent>,
}

impl DiscoveryService {
    /// Create new discovery service
    ///
    /// # Arguments
    ///
    /// * `device_id` - Unique device identifier (e.g., "DEV-001")
    /// * `device_name` - Human-readable name (e.g., "Alice's Laptop")
    /// * `device_type` - Device category ("desktop", "mobile", "iot", "server")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use honeylink_discovery::DiscoveryService;
    /// let service = DiscoveryService::new("DEV-001", "My Device", "desktop")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(device_id: &str, device_name: &str, device_type: &str) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(100);
        let mdns = MdnsDiscovery::new(device_id, device_name, device_type, event_tx.clone())?;
        let ble = BleDiscovery::new(device_id, device_name, device_type, event_tx)?;

        Ok(Self {
            mdns,
            ble: Some(ble),
            event_rx,
        })
    }

    /// Start discovery service
    ///
    /// - Announces device via mDNS (_honeylink._tcp.local)
    /// - Starts browsing for nearby devices
    /// - Spawns background task for network monitoring
    /// - Optionally starts BLE advertising/scanning (if enabled)
    pub async fn start(&mut self) -> Result<()> {
        self.mdns.announce().await?;
        self.mdns.start_browsing().await?;
        self.mdns.start_network_monitoring().await?;
        Ok(())
    }

    /// Enable BLE discovery (mobile device support)
    ///
    /// Starts BLE advertising and scanning for devices where mDNS is unavailable
    pub async fn enable_ble(&mut self) -> Result<()> {
        if let Some(ble) = &mut self.ble {
            ble.start_advertising().await?;
            ble.start_scanning().await?;
            info!("BLE discovery enabled");
        }
        Ok(())
    }

    /// Discover nearby devices (blocking for specified duration)
    ///
    /// # Arguments
    ///
    /// * `timeout_secs` - Discovery timeout in seconds (recommended: 5)
    ///
    /// # Returns
    ///
    /// List of discovered devices (typically 3-5 seconds for Bluetooth-compatible UX)
    pub async fn discover_devices(&mut self, timeout_secs: u64) -> Result<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        let timeout = tokio::time::Duration::from_secs(timeout_secs);
        let start = tokio::time::Instant::now();

        while start.elapsed() < timeout {
            match tokio::time::timeout(timeout - start.elapsed(), self.event_rx.recv()).await {
                Ok(Some(DiscoveryEvent::DeviceFound(device))) => {
                    devices.push(device);
                }
                Ok(Some(DiscoveryEvent::DeviceLost(device_id))) => {
                    devices.retain(|d| d.device_id != device_id);
                }
                Ok(Some(DiscoveryEvent::NetworkChanged)) => {
                    tracing::debug!("Network changed, continuing discovery");
                }
                Ok(None) => break, // Channel closed
                Err(_) => break,   // Timeout
            }
        }

        Ok(devices)
    }

    /// Stop discovery service (graceful shutdown)
    ///
    /// - Unregisters mDNS service
    /// - Stops browsing
    /// - Stops BLE (if enabled)
    /// - Cleans up resources
    pub async fn stop(&mut self) -> Result<()> {
        self.mdns.stop().await?;

        if let Some(ble) = &mut self.ble {
            ble.stop().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_service_creation() {
        let service = DiscoveryService::new("DEV-TEST-001", "Test Device", "desktop");
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_discovery_lifecycle() {
        let mut service =
            DiscoveryService::new("DEV-TEST-002", "Lifecycle Test", "mobile").unwrap();

        // Start service
        let start_result = service.start().await;
        assert!(start_result.is_ok());

        // Stop service
        let stop_result = service.stop().await;
        assert!(stop_result.is_ok());
    }
}
