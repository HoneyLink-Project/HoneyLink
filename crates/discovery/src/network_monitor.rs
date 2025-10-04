//! Network interface monitoring for mDNS re-announcement
//!
//! Monitors network interface changes (IP address changes) and triggers
//! mDNS service re-announcement when changes are detected.

use crate::error::{DiscoveryError, Result};
use std::collections::HashSet;
use std::net::IpAddr;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Network change event
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// IP addresses changed
    AddressesChanged {
        added: Vec<IpAddr>,
        removed: Vec<IpAddr>,
    },
}

/// Network monitor that detects IP address changes
pub struct NetworkMonitor {
    /// Current known IP addresses
    current_addresses: HashSet<IpAddr>,
    
    /// Event sender for network changes
    event_tx: mpsc::Sender<NetworkEvent>,
    
    /// Running state
    running: bool,
}

impl NetworkMonitor {
    /// Create new network monitor
    pub fn new(event_tx: mpsc::Sender<NetworkEvent>) -> Self {
        Self {
            current_addresses: HashSet::new(),
            event_tx,
            running: false,
        }
    }

    /// Get current network interfaces' IP addresses
    fn get_current_addresses() -> Result<HashSet<IpAddr>> {
        let interfaces = if_addrs::get_if_addrs()
            .map_err(|e| DiscoveryError::NetworkError(format!("Failed to get interfaces: {}", e)))?;

        let addresses: HashSet<IpAddr> = interfaces
            .into_iter()
            .filter(|iface| !iface.is_loopback()) // Exclude loopback
            .map(|iface| iface.addr.ip())
            .collect();

        Ok(addresses)
    }

    /// Start monitoring network changes
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(DiscoveryError::AlreadyRunning);
        }

        // Initialize with current addresses
        self.current_addresses = Self::get_current_addresses()?;
        info!("Network monitor started with {} addresses", self.current_addresses.len());
        debug!("Current addresses: {:?}", self.current_addresses);

        self.running = true;
        Ok(())
    }

    /// Check for network changes (call periodically)
    pub async fn check_changes(&mut self) -> Result<()> {
        if !self.running {
            return Err(DiscoveryError::NotStarted);
        }

        let new_addresses = Self::get_current_addresses()?;

        // Calculate differences
        let added: Vec<IpAddr> = new_addresses
            .difference(&self.current_addresses)
            .copied()
            .collect();

        let removed: Vec<IpAddr> = self.current_addresses
            .difference(&new_addresses)
            .copied()
            .collect();

        // If there are changes, notify
        if !added.is_empty() || !removed.is_empty() {
            info!(
                "Network change detected: +{} addresses, -{} addresses",
                added.len(),
                removed.len()
            );
            debug!("Added: {:?}", added);
            debug!("Removed: {:?}", removed);

            self.current_addresses = new_addresses;

            // Send event
            let event = NetworkEvent::AddressesChanged {
                added: added.clone(),
                removed: removed.clone(),
            };

            if let Err(e) = self.event_tx.send(event).await {
                warn!("Failed to send network event: {}", e);
            }
        }

        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&mut self) {
        self.running = false;
        info!("Network monitor stopped");
    }

    /// Spawn a background task that monitors network changes periodically
    pub fn spawn_monitor_task(
        mut self,
        check_interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while self.running {
                if let Err(e) = self.check_changes().await {
                    warn!("Error checking network changes: {}", e);
                }

                tokio::time::sleep(check_interval).await;
            }
            debug!("Network monitor task stopped");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_monitor_creation() {
        let (tx, _rx) = mpsc::channel(10);
        let monitor = NetworkMonitor::new(tx);
        assert!(!monitor.running);
    }

    #[tokio::test]
    async fn test_get_addresses() {
        let addresses = NetworkMonitor::get_current_addresses().unwrap();
        // Should have at least one non-loopback address in most environments
        println!("Found {} network addresses", addresses.len());
    }

    #[tokio::test]
    async fn test_monitor_start_stop() {
        let (tx, _rx) = mpsc::channel(10);
        let mut monitor = NetworkMonitor::new(tx);
        
        monitor.start().await.unwrap();
        assert!(monitor.running);
        
        monitor.stop();
        assert!(!monitor.running);
    }
}
