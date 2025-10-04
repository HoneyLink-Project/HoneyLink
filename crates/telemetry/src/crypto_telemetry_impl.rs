//! Crypto telemetry trait implementation
//!
//! Implements CryptoTelemetry trait defined in honeylink-crypto.

use honeylink_crypto::telemetry::CryptoTelemetry as CryptoTelemetryTrait;
use crate::crypto_metrics::CryptoMetrics;
use std::sync::Arc;

/// Crypto telemetry recorder implementation
///
/// Wraps CryptoMetrics to implement the CryptoTelemetry trait.
#[derive(Clone)]
pub struct CryptoTelemetryImpl {
    metrics: Arc<CryptoMetrics>,
}

impl CryptoTelemetryImpl {
    /// Create new crypto telemetry recorder
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(CryptoMetrics::new()),
        }
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> Vec<crate::Metric> {
        let mut metrics = Vec::new();
        metrics.extend(self.metrics.x25519_metrics());
        metrics.extend(self.metrics.chacha20_metrics());
        metrics.extend(self.metrics.hkdf_metrics());
        metrics.extend(self.metrics.rotation_metrics());
        metrics.extend(self.metrics.pop_token_metrics());
        metrics
    }
}

impl Default for CryptoTelemetryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoTelemetryTrait for CryptoTelemetryImpl {
    fn new_crypto_telemetry() -> Self {
        Self::new()
    }

    fn record_x25519_operation(&self, duration_ns: u64, success: bool) {
        self.metrics.record_x25519_operation(duration_ns, success);
    }

    fn record_chacha20_encryption(&self, duration_ns: u64, _bytes: usize, success: bool) {
        self.metrics.record_chacha20_encryption(duration_ns, success);
    }

    fn record_chacha20_decryption(&self, duration_ns: u64, _bytes: usize, success: bool) {
        self.metrics.record_chacha20_decryption(duration_ns, success);
    }

    fn record_hkdf_derivation(&self, duration_ns: u64, success: bool) {
        self.metrics.record_hkdf_derivation(duration_ns, success);
    }

    fn record_key_rotation(&self, duration_ms: u64, success: bool) {
        self.metrics.record_rotation_event(duration_ms, success);
    }

    fn record_pop_token_generation(&self) {
        self.metrics.record_pop_token_generation();
    }

    fn record_pop_token_verification(&self, success: bool) {
        self.metrics.record_pop_token_verification(success, false);
    }

    fn record_pop_token_replay_detection(&self) {
        self.metrics.record_pop_token_verification(false, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_telemetry_impl() {
        let telemetry = CryptoTelemetryImpl::new();
        
        // Record various operations
        telemetry.record_x25519_operation(1000, true);
        telemetry.record_chacha20_encryption(2000, 1024, true);
        telemetry.record_chacha20_decryption(1500, 1024, true);
        telemetry.record_hkdf_derivation(500, true);
        telemetry.record_key_rotation(10, true);
        telemetry.record_pop_token_generation();
        telemetry.record_pop_token_verification(true);
        telemetry.record_pop_token_replay_detection();

        // Get metrics snapshot
        let metrics = telemetry.get_metrics();
        assert!(!metrics.is_empty(), "Should have recorded metrics");
    }
}
