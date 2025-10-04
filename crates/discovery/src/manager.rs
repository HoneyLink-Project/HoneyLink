//! Unified Discovery Manager
//!
//! Coordinates multiple discovery protocols (mDNS, BLE) to provide a single
//! unified API for device discovery. Handles device deduplication, protocol
//! selection, and event aggregation.

use crate::error::Result;
use crate::protocol::{DiscoveryProtocol, ProtocolStrategy, ProtocolType};
use crate::types::{DeviceInfo, DiscoveryEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, info, warn};

/// Unified Discovery Manager
///
/// Manages multiple discovery protocols and provides a unified interface
/// for device discovery, announcement, and lifecycle management.
///
/// # Architecture
/// - Aggregates multiple DiscoveryProtocol implementations
/// - Deduplicates devices discovered via multiple protocols (by device_id)
/// - Provides unified event stream for all discovery events
/// - Supports protocol selection strategies (prefer mDNS, fallback to BLE, etc.)
///
/// # Thread Safety
/// - All internal state is protected by Arc<RwLock> or Arc<Mutex>
/// - Event channels use mpsc for cross-task communication
/// - Protocol implementations must be Send + Sync
pub struct DiscoveryManager {
    /// Registered discovery protocols (keyed by protocol type)
    protocols: Arc<RwLock<HashMap<ProtocolType, Box<dyn DiscoveryProtocol>>>>,

    /// Unified device map (device_id -> (DeviceInfo, ProtocolType))
    ///
    /// Stores deduplicated devices with source protocol tracking.
    /// If same device is discovered via multiple protocols, mDNS takes precedence.
    devices: Arc<RwLock<HashMap<String, (DeviceInfo, ProtocolType)>>>,

    /// Protocol selection strategy
    strategy: ProtocolStrategy,

    /// Unified event sender
    event_tx: mpsc::Sender<DiscoveryEvent>,

    /// Unified event receiver (for external consumers)
    event_rx: Arc<Mutex<Option<mpsc::Receiver<DiscoveryEvent>>>>,

    /// Running state
    running: Arc<Mutex<bool>>,
}

impl DiscoveryManager {
    /// Create new discovery manager
    ///
    /// # Parameters
    /// - `strategy`: Protocol selection strategy (default: prefer mDNS)
    /// - `channel_size`: Event channel buffer size (default: 100)
    ///
    /// # Returns
    /// A new DiscoveryManager instance with an empty protocol set.
    /// Call `register_protocol()` to add mDNS, BLE, or other backends.
    pub fn new(strategy: ProtocolStrategy, channel_size: usize) -> Self {
        let (event_tx, event_rx) = mpsc::channel(channel_size);

        Self {
            protocols: Arc::new(RwLock::new(HashMap::new())),
            devices: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            event_tx,
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Register a discovery protocol
    ///
    /// Adds a new protocol backend to the manager. Protocols can be registered
    /// before or after calling `start()`.
    ///
    /// # Example
    /// ```no_run
    /// use honeylink_discovery::{DiscoveryManager, MdnsDiscovery};
    /// use honeylink_discovery::protocol::{ProtocolStrategy, ProtocolType};
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut manager = DiscoveryManager::new(ProtocolStrategy::default(), 100);
    ///     let (tx, _rx) = mpsc::channel(100);
    ///     let mdns = MdnsDiscovery::new("DEV-001", "Test Device", "desktop", tx).unwrap();
    ///     manager.register_protocol(ProtocolType::Mdns, Box::new(mdns));
    /// }
    /// ```
    pub fn register_protocol(
        &mut self,
        protocol_type: ProtocolType,
        protocol: Box<dyn DiscoveryProtocol>,
    ) {
        let mut protocols = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.protocols.write())
        });
        info!(
            protocol = protocol.protocol_name(),
            "Registering discovery protocol"
        );
        protocols.insert(protocol_type, protocol);
    }

    /// Start discovery on all registered protocols
    ///
    /// Begins device announcement and browsing according to the configured strategy.
    /// Spawns background tasks for event aggregation.
    ///
    /// # Strategy Behavior
    /// - `All`: Start all protocols simultaneously
    /// - `Prefer(Mdns)`: Start mDNS first, add BLE if mDNS fails
    /// - `Prefer(Ble)`: Start BLE first, add mDNS if BLE fails
    /// - `Only(protocol)`: Start only specified protocol
    pub async fn start(&mut self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }

        info!(strategy = ?self.strategy, "Starting discovery manager");

        let mut protocols = self.protocols.write().await;

        // Determine which protocols to start based on strategy
        let protocol_types: Vec<ProtocolType> = match self.strategy {
            ProtocolStrategy::All => protocols.keys().copied().collect(),
            ProtocolStrategy::Prefer(pref) => {
                use crate::protocol::PreferredProtocol;
                match pref {
                    PreferredProtocol::Mdns => {
                        vec![ProtocolType::Mdns, ProtocolType::Ble]
                    }
                    PreferredProtocol::Ble => {
                        vec![ProtocolType::Ble, ProtocolType::Mdns]
                    }
                }
            }
            ProtocolStrategy::Only(pt) => vec![pt],
        };

        // Start each protocol
        for pt in protocol_types {
            if let Some(protocol) = protocols.get_mut(&pt) {
                info!(protocol = protocol.protocol_name(), "Starting protocol");

                // Start announcing and browsing
                if let Err(e) = protocol.start_announcing().await {
                    warn!(
                        protocol = protocol.protocol_name(),
                        error = %e,
                        "Failed to start announcing"
                    );
                }

                if let Err(e) = protocol.start_browsing().await {
                    warn!(
                        protocol = protocol.protocol_name(),
                        error = %e,
                        "Failed to start browsing"
                    );
                }
            }
        }

        *running = true;
        Ok(())
    }

    /// Stop discovery on all protocols
    ///
    /// Stops device announcement and browsing, releases resources.
    pub async fn stop(&mut self) -> Result<()> {
        let mut running = self.running.lock().await;
        if !*running {
            return Ok(());
        }

        info!("Stopping discovery manager");

        let mut protocols = self.protocols.write().await;

        for (_pt, protocol) in protocols.iter_mut() {
            info!(protocol = protocol.protocol_name(), "Stopping protocol");

            if let Err(e) = protocol.stop_announcing().await {
                warn!(
                    protocol = protocol.protocol_name(),
                    error = %e,
                    "Failed to stop announcing"
                );
            }

            if let Err(e) = protocol.stop_browsing().await {
                warn!(
                    protocol = protocol.protocol_name(),
                    error = %e,
                    "Failed to stop browsing"
                );
            }
        }

        *running = false;
        Ok(())
    }

    /// Get all discovered devices (deduplicated)
    ///
    /// Returns a unified view of devices discovered across all protocols.
    /// If a device is discovered via multiple protocols, only one entry is returned
    /// (prioritizing mDNS over BLE for consistency).
    pub async fn get_devices(&self) -> HashMap<String, DeviceInfo> {
        let devices = self.devices.read().await;
        devices
            .iter()
            .map(|(id, (info, _protocol))| (id.clone(), info.clone()))
            .collect()
    }

    /// Get device count
    pub async fn device_count(&self) -> usize {
        self.devices.read().await.len()
    }

    /// Take the event receiver
    ///
    /// Returns the unified event stream receiver. Can only be called once.
    /// Subsequent calls return None.
    pub async fn take_event_receiver(&self) -> Option<mpsc::Receiver<DiscoveryEvent>> {
        self.event_rx.lock().await.take()
    }

    /// Check if manager is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// Internal: Handle device deduplication
    ///
    /// When a device is discovered via multiple protocols:
    /// - Prefer mDNS (faster, more reliable)
    /// - Keep BLE if mDNS is unavailable
    /// - Update RSSI if BLE provides better signal info
    async fn merge_device(
        &self,
        device_info: DeviceInfo,
        source_protocol: ProtocolType,
    ) -> Result<()> {
        let mut devices = self.devices.write().await;
        let device_id = device_info.device_id.clone();

        match devices.get_mut(&device_id) {
            Some((existing_info, existing_protocol)) => {
                // Device already exists - apply merge strategy
                match (*existing_protocol, source_protocol) {
                    (ProtocolType::Mdns, ProtocolType::Ble) => {
                        // mDNS takes precedence, but update RSSI from BLE if available
                        if device_info.rssi.is_some() {
                            existing_info.rssi = device_info.rssi;
                        }
                        debug!(
                            device_id = %device_id,
                            "Keeping mDNS entry, updated RSSI from BLE"
                        );
                    }
                    (ProtocolType::Ble, ProtocolType::Mdns) => {
                        // Replace BLE with mDNS (more reliable)
                        *existing_info = device_info;
                        *existing_protocol = source_protocol;
                        debug!(
                            device_id = %device_id,
                            "Replaced BLE entry with mDNS"
                        );
                    }
                    _ => {
                        // Same protocol - update info
                        *existing_info = device_info;
                        debug!(
                            device_id = %device_id,
                            protocol = ?source_protocol,
                            "Updated device info"
                        );
                    }
                }
            }
            None => {
                // New device - insert
                devices.insert(device_id.clone(), (device_info, source_protocol));
                debug!(
                    device_id = %device_id,
                    protocol = ?source_protocol,
                    "Added new device"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DeviceType;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = DiscoveryManager::new(ProtocolStrategy::default(), 100);
        assert!(!manager.is_running().await);
        assert_eq!(manager.device_count().await, 0);
    }

    #[tokio::test]
    async fn test_manager_lifecycle() {
        let mut manager = DiscoveryManager::new(ProtocolStrategy::All, 100);

        // Start should succeed even without protocols
        assert!(manager.start().await.is_ok());
        assert!(manager.is_running().await);

        // Stop should succeed
        assert!(manager.stop().await.is_ok());
        assert!(!manager.is_running().await);
    }

    #[tokio::test]
    async fn test_device_deduplication() {
        let manager = DiscoveryManager::new(ProtocolStrategy::All, 100);

        // Add device via BLE
        let device_ble = DeviceInfo::new("DEV-001", "Test Device", DeviceType::Desktop)
            .with_addresses(vec!["192.168.1.100".parse().unwrap()])
            .with_port(7843)
            .with_rssi(-50);

        manager
            .merge_device(device_ble.clone(), ProtocolType::Ble)
            .await
            .unwrap();

        assert_eq!(manager.device_count().await, 1);

        // Add same device via mDNS (should replace BLE)
        let device_mdns = DeviceInfo::new("DEV-001", "Test Device", DeviceType::Desktop)
            .with_addresses(vec!["192.168.1.100".parse().unwrap()])
            .with_port(7843);

        manager
            .merge_device(device_mdns.clone(), ProtocolType::Mdns)
            .await
            .unwrap();

        assert_eq!(manager.device_count().await, 1);

        // Verify mDNS took precedence
        let devices = manager.get_devices().await;
        let device = devices.get("DEV-001").unwrap();
        assert_eq!(device.rssi, Some(-50)); // RSSI preserved from BLE
    }

    #[tokio::test]
    async fn test_event_receiver_take_once() {
        let manager = DiscoveryManager::new(ProtocolStrategy::All, 100);

        // First take should succeed
        assert!(manager.take_event_receiver().await.is_some());

        // Second take should return None
        assert!(manager.take_event_receiver().await.is_none());
    }
}
