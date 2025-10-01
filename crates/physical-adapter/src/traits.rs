//! Physical layer trait definition

use honeylink_core::Result;

use async_trait::async_trait;

/// Physical layer interface
#[async_trait]
pub trait PhysicalLayer: Send + Sync {
    /// Send data through the physical layer
    async fn send(&mut self, data: &[u8]) -> Result<usize>;

    /// Receive data from the physical layer
    async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize>;

    /// Get current link status
    fn status(&self) -> Result<LinkStatus>;

    /// Configure the physical layer
    async fn configure(&mut self, config: LayerConfig) -> Result<()>;
}

/// Link status
#[derive(Debug, Clone)]
pub struct LinkStatus {
    pub connected: bool,
    pub signal_strength: i8,
}

/// Layer configuration
#[derive(Debug, Clone)]
pub struct LayerConfig {
    pub max_bandwidth_mbps: u32,
}
