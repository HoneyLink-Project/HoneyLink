//! Physical adapter implementations
//!
//! This module provides concrete implementations of physical layer adapters:
//! - WiFi6eAdapter: Wi-Fi 6E via REST API
//! - WiFi7Adapter: Wi-Fi 7 via REST API
//! - FiveGAdapter: 5G modem via REST API
//! - ThzAdapter: THz experimental via gRPC (future)
//!
//! # C/C++ Dependency Avoidance (MOD-007 spec)
//! All adapters use process separation and network APIs (gRPC/REST) to communicate
//! with vendor driver services, eliminating direct C/C++ library dependencies.

use async_trait::async_trait;
use honeylink_transport::{LinkQualityMetrics, Packet, PhysicalLayer, PowerMode, TransportError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Adapter type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdapterType {
    /// Wi-Fi 6E (802.11ax, 6GHz band)
    WiFi6E,
    /// Wi-Fi 7 (802.11be)
    WiFi7,
    /// 5G cellular modem
    FiveG,
    /// Terahertz experimental
    THz,
    /// Bluetooth (future)
    Bluetooth,
    /// Ethernet (wired fallback)
    Ethernet,
}

impl AdapterType {
    /// Returns the string identifier for this adapter type
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WiFi6E => "WiFi6E",
            Self::WiFi7 => "WiFi7",
            Self::FiveG => "5G",
            Self::THz => "THz",
            Self::Bluetooth => "Bluetooth",
            Self::Ethernet => "Ethernet",
        }
    }

    /// Returns typical bandwidth in Mbps
    pub fn typical_bandwidth_mbps(&self) -> f32 {
        match self {
            Self::WiFi6E => 1200.0,
            Self::WiFi7 => 4800.0,
            Self::FiveG => 1000.0,
            Self::THz => 10000.0,
            Self::Bluetooth => 24.0,
            Self::Ethernet => 1000.0,
        }
    }
}

/// REST API client for HTTP-based adapters
struct RestClient {
    client: Client,
    base_url: String,
    bearer_token: Option<String>,
}

impl RestClient {
    fn new(base_url: String, bearer_token: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            base_url,
            bearer_token,
        }
    }

    async fn post_send(&self, data: &[u8]) -> Result<(), TransportError> {
        let url = format!("{}/send", self.base_url);
        let mut req = self.client.post(&url).body(data.to_vec());

        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| TransportError::Io(format!("REST send failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(TransportError::AdapterError(format!(
                "REST send failed: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    async fn get_metrics(&self) -> Result<RestMetricsResponse, TransportError> {
        let url = format!("{}/metrics", self.base_url);
        let mut req = self.client.get(&url);

        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| TransportError::Io(format!("REST metrics failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(TransportError::AdapterError(format!(
                "REST metrics failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| TransportError::Io(format!("JSON decode failed: {}", e)))
    }

    async fn post_power_mode(&self, mode: PowerMode) -> Result<(), TransportError> {
        let url = format!("{}/power_mode", self.base_url);
        let body = serde_json::json!({ "mode": format!("{:?}", mode).to_lowercase() });
        let mut req = self.client.post(&url).json(&body);

        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| TransportError::Io(format!("REST power_mode failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(TransportError::AdapterError(format!(
                "REST power_mode failed: {}",
                resp.status()
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct RestMetricsResponse {
    rssi: Option<i16>,
    snr: Option<f32>,
    loss_rate: Option<f32>,
}

/// Wi-Fi 6E adapter via REST API
pub struct WiFi6eAdapter {
    client: Arc<RwLock<RestClient>>,
    power_mode: Arc<RwLock<PowerMode>>,
}

impl WiFi6eAdapter {
    /// Creates a new WiFi 6E adapter
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the Wi-Fi controller service (e.g., "http://localhost:8080")
    /// * `bearer_token` - Optional OAuth2 bearer token for authentication
    pub fn new(base_url: String, bearer_token: Option<String>) -> Self {
        Self {
            client: Arc::new(RwLock::new(RestClient::new(base_url, bearer_token))),
            power_mode: Arc::new(RwLock::new(PowerMode::Normal)),
        }
    }
}

#[async_trait]
impl PhysicalLayer for WiFi6eAdapter {
    async fn send_packet(&self, packet: &Packet) -> Result<(), TransportError> {
        let client = self.client.read().await;
        client.post_send(&packet.data).await
    }

    async fn recv_packet(&self, timeout: Duration) -> Result<Packet, TransportError> {
        // Simplified: in real implementation, would poll /receive endpoint
        tokio::time::sleep(timeout).await;
        Err(TransportError::Timeout(timeout))
    }

    async fn get_link_quality(&self) -> Result<LinkQualityMetrics, TransportError> {
        let client = self.client.read().await;
        let metrics = client.get_metrics().await?;

        Ok(LinkQualityMetrics {
            rssi_dbm: metrics.rssi.unwrap_or(-50),
            snr_db: metrics.snr.unwrap_or(30.0),
            loss_rate: metrics.loss_rate.unwrap_or(0.0),
            bandwidth_mbps: AdapterType::WiFi6E.typical_bandwidth_mbps(),
            rtt_ms: 10,
        })
    }

    async fn set_power_mode(&self, mode: PowerMode) -> Result<(), TransportError> {
        let client = self.client.read().await;
        client.post_power_mode(mode).await?;
        *self.power_mode.write().await = mode;
        Ok(())
    }

    fn layer_type(&self) -> &str {
        AdapterType::WiFi6E.as_str()
    }
}

/// Wi-Fi 7 adapter via REST API
pub struct WiFi7Adapter {
    client: Arc<RwLock<RestClient>>,
    power_mode: Arc<RwLock<PowerMode>>,
}

impl WiFi7Adapter {
    pub fn new(base_url: String, bearer_token: Option<String>) -> Self {
        Self {
            client: Arc::new(RwLock::new(RestClient::new(base_url, bearer_token))),
            power_mode: Arc::new(RwLock::new(PowerMode::Normal)),
        }
    }
}

#[async_trait]
impl PhysicalLayer for WiFi7Adapter {
    async fn send_packet(&self, packet: &Packet) -> Result<(), TransportError> {
        let client = self.client.read().await;
        client.post_send(&packet.data).await
    }

    async fn recv_packet(&self, timeout: Duration) -> Result<Packet, TransportError> {
        tokio::time::sleep(timeout).await;
        Err(TransportError::Timeout(timeout))
    }

    async fn get_link_quality(&self) -> Result<LinkQualityMetrics, TransportError> {
        let client = self.client.read().await;
        let metrics = client.get_metrics().await?;

        Ok(LinkQualityMetrics {
            rssi_dbm: metrics.rssi.unwrap_or(-45),
            snr_db: metrics.snr.unwrap_or(35.0),
            loss_rate: metrics.loss_rate.unwrap_or(0.0),
            bandwidth_mbps: AdapterType::WiFi7.typical_bandwidth_mbps(),
            rtt_ms: 8,
        })
    }

    async fn set_power_mode(&self, mode: PowerMode) -> Result<(), TransportError> {
        let client = self.client.read().await;
        client.post_power_mode(mode).await?;
        *self.power_mode.write().await = mode;
        Ok(())
    }

    fn layer_type(&self) -> &str {
        AdapterType::WiFi7.as_str()
    }
}

/// 5G cellular modem adapter via REST API
pub struct FiveGAdapter {
    client: Arc<RwLock<RestClient>>,
    power_mode: Arc<RwLock<PowerMode>>,
}

impl FiveGAdapter {
    pub fn new(base_url: String, bearer_token: Option<String>) -> Self {
        Self {
            client: Arc::new(RwLock::new(RestClient::new(base_url, bearer_token))),
            power_mode: Arc::new(RwLock::new(PowerMode::Normal)),
        }
    }
}

#[async_trait]
impl PhysicalLayer for FiveGAdapter {
    async fn send_packet(&self, packet: &Packet) -> Result<(), TransportError> {
        let client = self.client.read().await;
        client.post_send(&packet.data).await
    }

    async fn recv_packet(&self, timeout: Duration) -> Result<Packet, TransportError> {
        tokio::time::sleep(timeout).await;
        Err(TransportError::Timeout(timeout))
    }

    async fn get_link_quality(&self) -> Result<LinkQualityMetrics, TransportError> {
        let client = self.client.read().await;
        let metrics = client.get_metrics().await?;

        Ok(LinkQualityMetrics {
            rssi_dbm: metrics.rssi.unwrap_or(-60),
            snr_db: metrics.snr.unwrap_or(25.0),
            loss_rate: metrics.loss_rate.unwrap_or(0.01),
            bandwidth_mbps: AdapterType::FiveG.typical_bandwidth_mbps(),
            rtt_ms: 20,
        })
    }

    async fn set_power_mode(&self, mode: PowerMode) -> Result<(), TransportError> {
        let client = self.client.read().await;
        client.post_power_mode(mode).await?;
        *self.power_mode.write().await = mode;
        Ok(())
    }

    fn layer_type(&self) -> &str {
        AdapterType::FiveG.as_str()
    }
}

/// THz experimental adapter (placeholder for future implementation)
pub struct ThzAdapter {
    power_mode: Arc<RwLock<PowerMode>>,
}

impl ThzAdapter {
    pub fn new() -> Self {
        Self {
            power_mode: Arc::new(RwLock::new(PowerMode::High)),
        }
    }
}

impl Default for ThzAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PhysicalLayer for ThzAdapter {
    async fn send_packet(&self, _packet: &Packet) -> Result<(), TransportError> {
        // Placeholder: experimental feature
        Err(TransportError::AdapterError(
            "THz adapter not yet implemented".into(),
        ))
    }

    async fn recv_packet(&self, timeout: Duration) -> Result<Packet, TransportError> {
        tokio::time::sleep(timeout).await;
        Err(TransportError::Timeout(timeout))
    }

    async fn get_link_quality(&self) -> Result<LinkQualityMetrics, TransportError> {
        Ok(LinkQualityMetrics {
            rssi_dbm: -40,
            snr_db: 40.0,
            loss_rate: 0.0,
            bandwidth_mbps: AdapterType::THz.typical_bandwidth_mbps(),
            rtt_ms: 5,
        })
    }

    async fn set_power_mode(&self, mode: PowerMode) -> Result<(), TransportError> {
        *self.power_mode.write().await = mode;
        Ok(())
    }

    fn layer_type(&self) -> &str {
        AdapterType::THz.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_type_str() {
        assert_eq!(AdapterType::WiFi6E.as_str(), "WiFi6E");
        assert_eq!(AdapterType::FiveG.as_str(), "5G");
    }

    #[test]
    fn test_adapter_bandwidth() {
        assert_eq!(AdapterType::WiFi6E.typical_bandwidth_mbps(), 1200.0);
        assert_eq!(AdapterType::WiFi7.typical_bandwidth_mbps(), 4800.0);
    }

    #[tokio::test]
    async fn test_thz_adapter_placeholder() {
        let adapter = ThzAdapter::new();
        let packet = Packet::new(vec![1, 2, 3], 5).unwrap();

        let result = adapter.send_packet(&packet).await;
        assert!(matches!(result, Err(TransportError::AdapterError(_))));
    }
}
