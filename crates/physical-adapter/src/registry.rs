//! Adapter registry and Hot Swap manager
//!
//! This module manages multiple physical layer adapters and provides seamless switching
//! between them based on link quality metrics.
//!
//! # Hot Swap Algorithm (MOD-007 spec)
//! 1. Monitor link quality every 5 seconds
//! 2. If current link is degraded (RSSI < -80 dBm OR loss_rate > 15%):
//!    - Select best available adapter (highest RSSI)
//!    - Gracefully transition (no packet loss during switch)
//!    - Target: P95 < 2s switchover latency
//! 3. Fallback priority: WiFi6E > WiFi7 > 5G > Ethernet

use crate::adapter::{AdapterType, FiveGAdapter, ThzAdapter, WiFi6eAdapter, WiFi7Adapter};
use honeylink_transport::{LinkQualityMetrics, Packet, PhysicalLayer, PowerMode, TransportError};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Hot Swap strategy for selecting next adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotSwapStrategy {
    /// Select adapter with highest RSSI
    HighestRssi,
    /// Select adapter with lowest loss rate
    LowestLossRate,
    /// Select adapter with highest bandwidth
    HighestBandwidth,
    /// Manual selection (no automatic switching)
    Manual,
}

/// Adapter registry managing multiple physical layers
pub struct AdapterRegistry {
    /// Registered adapters
    adapters: Arc<RwLock<HashMap<AdapterType, Box<dyn PhysicalLayer>>>>,
    /// Currently active adapter type
    active_adapter: Arc<RwLock<Option<AdapterType>>>,
    /// Hot Swap strategy
    strategy: HotSwapStrategy,
    /// Link quality monitoring interval
    monitor_interval: Duration,
}

impl AdapterRegistry {
    /// Creates a new adapter registry
    pub fn new(strategy: HotSwapStrategy) -> Self {
        Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
            active_adapter: Arc::new(RwLock::new(None)),
            strategy,
            monitor_interval: Duration::from_secs(5), // MOD-007 spec
        }
    }

    /// Registers a WiFi 6E adapter
    pub async fn register_wifi6e(
        &self,
        base_url: String,
        bearer_token: Option<String>,
    ) -> Result<(), TransportError> {
        let adapter = WiFi6eAdapter::new(base_url, bearer_token);
        let mut adapters = self.adapters.write().await;
        adapters.insert(AdapterType::WiFi6E, Box::new(adapter));
        Ok(())
    }

    /// Registers a WiFi 7 adapter
    pub async fn register_wifi7(
        &self,
        base_url: String,
        bearer_token: Option<String>,
    ) -> Result<(), TransportError> {
        let adapter = WiFi7Adapter::new(base_url, bearer_token);
        let mut adapters = self.adapters.write().await;
        adapters.insert(AdapterType::WiFi7, Box::new(adapter));
        Ok(())
    }

    /// Registers a 5G adapter
    pub async fn register_5g(
        &self,
        base_url: String,
        bearer_token: Option<String>,
    ) -> Result<(), TransportError> {
        let adapter = FiveGAdapter::new(base_url, bearer_token);
        let mut adapters = self.adapters.write().await;
        adapters.insert(AdapterType::FiveG, Box::new(adapter));
        Ok(())
    }

    /// Registers a THz adapter (experimental)
    pub async fn register_thz(&self) -> Result<(), TransportError> {
        let adapter = ThzAdapter::new();
        let mut adapters = self.adapters.write().await;
        adapters.insert(AdapterType::THz, Box::new(adapter));
        Ok(())
    }

    /// Sets the active adapter
    pub async fn set_active(&self, adapter_type: AdapterType) -> Result<(), TransportError> {
        let adapters = self.adapters.read().await;
        if !adapters.contains_key(&adapter_type) {
            return Err(TransportError::AdapterError(format!(
                "Adapter {:?} not registered",
                adapter_type
            )));
        }
        *self.active_adapter.write().await = Some(adapter_type);
        Ok(())
    }

    /// Returns the currently active adapter type
    pub async fn active_adapter(&self) -> Option<AdapterType> {
        *self.active_adapter.read().await
    }

    /// Sends a packet through the active adapter
    pub async fn send_packet(&self, packet: &Packet) -> Result<(), TransportError> {
        let active = self
            .active_adapter
            .read()
            .await
            .ok_or_else(|| TransportError::AdapterError("No active adapter".into()))?;

        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(&active)
            .ok_or_else(|| TransportError::AdapterError("Active adapter not found".into()))?;

        adapter.send_packet(packet).await
    }

    /// Retrieves link quality from the active adapter
    pub async fn get_link_quality(&self) -> Result<LinkQualityMetrics, TransportError> {
        let active = self
            .active_adapter
            .read()
            .await
            .ok_or_else(|| TransportError::AdapterError("No active adapter".into()))?;

        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(&active)
            .ok_or_else(|| TransportError::AdapterError("Active adapter not found".into()))?;

        adapter.get_link_quality().await
    }

    /// Sets power mode for the active adapter
    pub async fn set_power_mode(&self, mode: PowerMode) -> Result<(), TransportError> {
        let active = self
            .active_adapter
            .read()
            .await
            .ok_or_else(|| TransportError::AdapterError("No active adapter".into()))?;

        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(&active)
            .ok_or_else(|| TransportError::AdapterError("Active adapter not found".into()))?;

        adapter.set_power_mode(mode).await
    }

    /// Performs Hot Swap evaluation and switches adapter if needed
    ///
    /// # Returns
    /// * `Ok(true)` if adapter was switched
    /// * `Ok(false)` if no switch was needed
    /// * `Err(TransportError)` on failure
    ///
    /// # Performance
    /// * Target: P95 < 2s (MOD-007 spec)
    pub async fn evaluate_hot_swap(&self) -> Result<bool, TransportError> {
        if self.strategy == HotSwapStrategy::Manual {
            return Ok(false);
        }

        // Get current link quality
        let current_quality = match self.get_link_quality().await {
            Ok(q) => q,
            Err(_) => {
                // Current link is down, force switch
                return self.switch_to_best_adapter().await.map(|_| true);
            }
        };

        // Check if current link is degraded (MOD-007 spec)
        if !current_quality.is_degraded() {
            return Ok(false); // Link is good, no need to switch
        }

        // Find best alternative adapter
        self.switch_to_best_adapter().await.map(|_| true)
    }

    /// Switches to the best available adapter based on strategy
    async fn switch_to_best_adapter(&self) -> Result<AdapterType, TransportError> {
        let adapters = self.adapters.read().await;
        let mut best_adapter: Option<(AdapterType, LinkQualityMetrics)> = None;

        for (adapter_type, adapter) in adapters.iter() {
            if let Ok(metrics) = adapter.get_link_quality().await {
                let is_better = match self.strategy {
                    HotSwapStrategy::HighestRssi => {
                        best_adapter
                            .as_ref()
                            .map(|(_, m)| metrics.rssi_dbm > m.rssi_dbm)
                            .unwrap_or(true)
                    }
                    HotSwapStrategy::LowestLossRate => {
                        best_adapter
                            .as_ref()
                            .map(|(_, m)| metrics.loss_rate < m.loss_rate)
                            .unwrap_or(true)
                    }
                    HotSwapStrategy::HighestBandwidth => {
                        best_adapter
                            .as_ref()
                            .map(|(_, m)| metrics.bandwidth_mbps > m.bandwidth_mbps)
                            .unwrap_or(true)
                    }
                    HotSwapStrategy::Manual => false,
                };

                if is_better {
                    best_adapter = Some((*adapter_type, metrics));
                }
            }
        }

        if let Some((adapter_type, _)) = best_adapter {
            *self.active_adapter.write().await = Some(adapter_type);
            Ok(adapter_type)
        } else {
            Err(TransportError::AdapterError(
                "No suitable adapter found".into(),
            ))
        }
    }

    /// Starts background link quality monitoring and automatic Hot Swap
    ///
    /// # Returns
    /// A JoinHandle that can be used to stop monitoring
    pub fn start_monitoring(
        self: Arc<Self>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(self.monitor_interval).await;

                if let Err(e) = self.evaluate_hot_swap().await {
                    eprintln!("Hot Swap evaluation failed: {}", e);
                }
            }
        })
    }

    /// Returns a list of registered adapter types
    pub async fn registered_adapters(&self) -> Vec<AdapterType> {
        let adapters = self.adapters.read().await;
        adapters.keys().copied().collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new(HotSwapStrategy::HighestRssi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = AdapterRegistry::new(HotSwapStrategy::HighestRssi);
        assert_eq!(registry.active_adapter().await, None);
    }

    #[tokio::test]
    async fn test_register_thz() {
        let registry = AdapterRegistry::new(HotSwapStrategy::Manual);
        registry.register_thz().await.unwrap();

        let adapters = registry.registered_adapters().await;
        assert!(adapters.contains(&AdapterType::THz));
    }

    #[tokio::test]
    async fn test_set_active_unregistered() {
        let registry = AdapterRegistry::new(HotSwapStrategy::Manual);
        let result = registry.set_active(AdapterType::WiFi6E).await;
        assert!(matches!(result, Err(TransportError::AdapterError(_))));
    }

    #[tokio::test]
    async fn test_set_active_registered() {
        let registry = AdapterRegistry::new(HotSwapStrategy::Manual);
        registry.register_thz().await.unwrap();
        registry.set_active(AdapterType::THz).await.unwrap();

        assert_eq!(registry.active_adapter().await, Some(AdapterType::THz));
    }
}
