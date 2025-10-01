//! SDK API

use honeylink_core::Result;

/// Main SDK interface
pub struct HoneyLinkSdk {
    initialized: bool,
}

impl HoneyLinkSdk {
    /// Create a new SDK instance
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the SDK
    pub async fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    /// Check if SDK is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Shutdown the SDK
    pub async fn shutdown(&mut self) -> Result<()> {
        self.initialized = false;
        Ok(())
    }
}

impl Default for HoneyLinkSdk {
    fn default() -> Self {
        Self::new()
    }
}
