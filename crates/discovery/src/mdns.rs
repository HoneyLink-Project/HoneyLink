//! mDNS-SD device discovery implementation
//!
//! Service: `_honeylink._tcp.local`
//! TXT Records: device_id, device_name, device_type, version

use crate::error::{DiscoveryError, Result};
use crate::types::{DeviceInfo, DeviceType, DiscoveryEvent};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

/// mDNS service type for HoneyLink
const SERVICE_TYPE: &str = "_honeylink._tcp.local.";

/// Default QUIC port
const DEFAULT_PORT: u16 = 7843;

/// mDNS Discovery implementation
pub struct MdnsDiscovery {
    /// Own device information
    device_id: String,
    device_name: String,
    device_type: DeviceType,

    /// mDNS daemon (wrapped in Arc<Mutex> for async access)
    daemon: Arc<Mutex<Option<ServiceDaemon>>>,

    /// Discovered devices (device_id -> DeviceInfo)
    devices: Arc<Mutex<HashMap<String, DeviceInfo>>>,

    /// Event sender
    event_tx: mpsc::Sender<DiscoveryEvent>,

    /// Running state
    running: Arc<Mutex<bool>>,
}

impl MdnsDiscovery {
    /// Create new mDNS discovery service
    pub fn new(
        device_id: &str,
        device_name: &str,
        device_type: &str,
        event_tx: mpsc::Sender<DiscoveryEvent>,
    ) -> Result<Self> {
        let device_type = DeviceType::from_str(device_type);

        Ok(Self {
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            device_type,
            daemon: Arc::new(Mutex::new(None)),
            devices: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Announce device via mDNS
    ///
    /// Registers service `_honeylink._tcp.local` with TXT records:
    /// - device_id: Unique identifier
    /// - device_name: Human-readable name
    /// - device_type: Device category
    /// - version: HoneyLink protocol version
    pub async fn announce(&mut self) -> Result<()> {
        info!(
            device_id = %self.device_id,
            device_name = %self.device_name,
            "Announcing device via mDNS"
        );

        // Create mDNS daemon
        let daemon = ServiceDaemon::new()
            .map_err(|e| DiscoveryError::MdnsError(format!("Failed to create daemon: {}", e)))?;

        // Get local IP addresses
        let host_ipv4 = local_ip_address::local_ip()
            .unwrap_or_else(|_| "127.0.0.1".parse().unwrap());

        // Create TXT records
        let mut properties = HashMap::new();
        properties.insert("device_id".to_string(), self.device_id.clone());
        properties.insert("device_name".to_string(), self.device_name.clone());
        properties.insert(
            "device_type".to_string(),
            self.device_type.as_str().to_string(),
        );
        properties.insert(
            "version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        // Create service info
        let service_hostname = format!("{}.local.", self.device_id.replace('-', ""));
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &self.device_id,
            &service_hostname,
            host_ipv4,
            DEFAULT_PORT,
            Some(properties),
        )
        .map_err(|e| DiscoveryError::MdnsError(format!("Failed to create service: {}", e)))?;

        // Register service
        daemon
            .register(service_info)
            .map_err(|e| DiscoveryError::MdnsError(format!("Failed to register: {}", e)))?;

        *self.daemon.lock().await = Some(daemon);
        *self.running.lock().await = true;

        info!(device_id = %self.device_id, "mDNS announcement successful");
        Ok(())
    }

    /// Start browsing for nearby devices
    pub async fn start_browsing(&mut self) -> Result<()> {
        let daemon_guard = self.daemon.lock().await;
        let daemon = daemon_guard
            .as_ref()
            .ok_or(DiscoveryError::NotStarted)?
            .clone();
        drop(daemon_guard);

        info!("Starting mDNS browsing for {}", SERVICE_TYPE);

        // Browse for HoneyLink devices
        let receiver = daemon
            .browse(SERVICE_TYPE)
            .map_err(|e| DiscoveryError::MdnsError(format!("Failed to browse: {}", e)))?;

        // Spawn background task to process events
        let devices = Arc::clone(&self.devices);
        let event_tx = self.event_tx.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running.lock().await {
                match receiver.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(event) => {
                        if let Err(e) =
                            Self::handle_service_event(event, &devices, &event_tx).await
                        {
                            error!("Error handling service event: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Normal timeout, continue
                        continue;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("mDNS receiver disconnected");
                        break;
                    }
                }
            }
            debug!("mDNS browsing task stopped");
        });

        Ok(())
    }

    /// Handle mDNS service event
    async fn handle_service_event(
        event: ServiceEvent,
        devices: &Arc<Mutex<HashMap<String, DeviceInfo>>>,
        event_tx: &mpsc::Sender<DiscoveryEvent>,
    ) -> Result<()> {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                debug!(
                    service_name = %info.get_fullname(),
                    "Service resolved"
                );

                if let Some(device) = Self::parse_service_info(&info) {
                    let device_id = device.device_id.clone();

                    // Add to devices map
                    devices.lock().await.insert(device_id.clone(), device.clone());

                    // Send event
                    let _ = event_tx.send(DiscoveryEvent::DeviceFound(device)).await;

                    info!(device_id = %device_id, "Device discovered");
                }
            }
            ServiceEvent::ServiceRemoved(_, fullname) => {
                // Extract device_id from fullname (format: "device_id._honeylink._tcp.local.")
                if let Some(device_id) = fullname.split('.').next() {
                    devices.lock().await.remove(device_id);
                    let _ = event_tx
                        .send(DiscoveryEvent::DeviceLost(device_id.to_string()))
                        .await;

                    info!(device_id = %device_id, "Device lost");
                }
            }
            ServiceEvent::SearchStarted(_) => {
                debug!("mDNS search started");
            }
            ServiceEvent::SearchStopped(_) => {
                debug!("mDNS search stopped");
            }
            _ => {}
        }

        Ok(())
    }

    /// Parse ServiceInfo into DeviceInfo
    fn parse_service_info(info: &ServiceInfo) -> Option<DeviceInfo> {
        let properties = info.get_properties();

        let device_id = properties.get("device_id")?.as_str()?;
        let device_name = properties.get("device_name")?.as_str()?;
        let device_type_str = properties.get("device_type")?.as_str()?;
        let version = properties.get("version")?.as_str()?;

        let device_type = DeviceType::from_str(device_type_str);

        let addresses: Vec<IpAddr> = info
            .get_addresses()
            .iter()
            .copied()
            .collect();

        Some(
            DeviceInfo::new(device_id, device_name, device_type)
                .with_addresses(addresses)
                .with_port(info.get_port()),
        )
    }

    /// Stop mDNS service (graceful shutdown)
    pub async fn stop(&mut self) -> Result<()> {
        info!(device_id = %self.device_id, "Stopping mDNS service");

        *self.running.lock().await = false;

        if let Some(daemon) = self.daemon.lock().await.take() {
            daemon.shutdown().map_err(|e| {
                DiscoveryError::MdnsError(format!("Failed to shutdown daemon: {}", e))
            })?;
        }

        self.devices.lock().await.clear();

        info!("mDNS service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mdns_creation() {
        let (tx, _rx) = mpsc::channel(10);
        let mdns = MdnsDiscovery::new("DEV-TEST-001", "Test Device", "desktop", tx);
        assert!(mdns.is_ok());
    }

    #[tokio::test]
    async fn test_device_type_conversion() {
        assert_eq!(DeviceType::from_str("desktop"), DeviceType::Desktop);
        assert_eq!(DeviceType::from_str("mobile"), DeviceType::Mobile);
    }
}
