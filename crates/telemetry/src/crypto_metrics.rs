//! Crypto operations instrumentation
//!
//! Provides telemetry integration for cryptographic operations from Task 2.4.

use crate::types::{Metric, MetricType};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Crypto metrics collector
pub struct CryptoMetrics {
    // X25519 key agreement metrics
    x25519_operations_total: Arc<AtomicU64>,
    x25519_duration_ns_sum: Arc<AtomicU64>,
    x25519_failures_total: Arc<AtomicU64>,

    // ChaCha20-Poly1305 encryption metrics
    chacha20_encryptions_total: Arc<AtomicU64>,
    chacha20_decryptions_total: Arc<AtomicU64>,
    chacha20_encryption_duration_ns_sum: Arc<AtomicU64>,
    chacha20_decryption_duration_ns_sum: Arc<AtomicU64>,
    chacha20_failures_total: Arc<AtomicU64>,

    // HKDF key derivation metrics
    hkdf_derivations_total: Arc<AtomicU64>,
    hkdf_duration_ns_sum: Arc<AtomicU64>,
    hkdf_failures_total: Arc<AtomicU64>,

    // Key rotation metrics
    rotation_events_total: Arc<AtomicU64>,
    rotation_duration_ms_sum: Arc<AtomicU64>,
    rotation_failures_total: Arc<AtomicU64>,

    // PoP token metrics
    pop_token_generations_total: Arc<AtomicU64>,
    pop_token_verifications_total: Arc<AtomicU64>,
    pop_token_verification_failures_total: Arc<AtomicU64>,
    pop_token_replay_detections_total: Arc<AtomicU64>,
}

impl CryptoMetrics {
    /// Creates a new crypto metrics collector
    pub fn new() -> Self {
        Self {
            x25519_operations_total: Arc::new(AtomicU64::new(0)),
            x25519_duration_ns_sum: Arc::new(AtomicU64::new(0)),
            x25519_failures_total: Arc::new(AtomicU64::new(0)),

            chacha20_encryptions_total: Arc::new(AtomicU64::new(0)),
            chacha20_decryptions_total: Arc::new(AtomicU64::new(0)),
            chacha20_encryption_duration_ns_sum: Arc::new(AtomicU64::new(0)),
            chacha20_decryption_duration_ns_sum: Arc::new(AtomicU64::new(0)),
            chacha20_failures_total: Arc::new(AtomicU64::new(0)),

            hkdf_derivations_total: Arc::new(AtomicU64::new(0)),
            hkdf_duration_ns_sum: Arc::new(AtomicU64::new(0)),
            hkdf_failures_total: Arc::new(AtomicU64::new(0)),

            rotation_events_total: Arc::new(AtomicU64::new(0)),
            rotation_duration_ms_sum: Arc::new(AtomicU64::new(0)),
            rotation_failures_total: Arc::new(AtomicU64::new(0)),

            pop_token_generations_total: Arc::new(AtomicU64::new(0)),
            pop_token_verifications_total: Arc::new(AtomicU64::new(0)),
            pop_token_verification_failures_total: Arc::new(AtomicU64::new(0)),
            pop_token_replay_detections_total: Arc::new(AtomicU64::new(0)),
        }
    }

    // X25519 key agreement instrumentation

    /// Records an X25519 key agreement operation
    pub fn record_x25519_operation(&self, duration_ns: u64, success: bool) {
        self.x25519_operations_total.fetch_add(1, Ordering::Relaxed);
        self.x25519_duration_ns_sum
            .fetch_add(duration_ns, Ordering::Relaxed);

        if !success {
            self.x25519_failures_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Generates X25519 metrics
    pub fn x25519_metrics(&self) -> Vec<Metric> {
        let operations = self.x25519_operations_total.load(Ordering::Relaxed);
        let duration_sum = self.x25519_duration_ns_sum.load(Ordering::Relaxed);
        let failures = self.x25519_failures_total.load(Ordering::Relaxed);

        let avg_duration_ms = if operations > 0 {
            (duration_sum as f64 / operations as f64) / 1_000_000.0
        } else {
            0.0
        };

        vec![
            Metric::counter(
                "crypto_x25519_operations_total".to_string(),
                operations as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::gauge(
                "crypto_x25519_avg_duration_ms".to_string(),
                avg_duration_ms,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_x25519_failures_total".to_string(),
                failures as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
        ]
    }

    // ChaCha20-Poly1305 encryption instrumentation

    /// Records a ChaCha20-Poly1305 encryption
    pub fn record_chacha20_encryption(&self, duration_ns: u64, success: bool) {
        self.chacha20_encryptions_total
            .fetch_add(1, Ordering::Relaxed);
        self.chacha20_encryption_duration_ns_sum
            .fetch_add(duration_ns, Ordering::Relaxed);

        if !success {
            self.chacha20_failures_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Records a ChaCha20-Poly1305 decryption
    pub fn record_chacha20_decryption(&self, duration_ns: u64, success: bool) {
        self.chacha20_decryptions_total
            .fetch_add(1, Ordering::Relaxed);
        self.chacha20_decryption_duration_ns_sum
            .fetch_add(duration_ns, Ordering::Relaxed);

        if !success {
            self.chacha20_failures_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Generates ChaCha20-Poly1305 metrics
    pub fn chacha20_metrics(&self) -> Vec<Metric> {
        let encryptions = self.chacha20_encryptions_total.load(Ordering::Relaxed);
        let decryptions = self.chacha20_decryptions_total.load(Ordering::Relaxed);
        let enc_duration_sum = self
            .chacha20_encryption_duration_ns_sum
            .load(Ordering::Relaxed);
        let dec_duration_sum = self
            .chacha20_decryption_duration_ns_sum
            .load(Ordering::Relaxed);
        let failures = self.chacha20_failures_total.load(Ordering::Relaxed);

        let avg_enc_duration_ms = if encryptions > 0 {
            (enc_duration_sum as f64 / encryptions as f64) / 1_000_000.0
        } else {
            0.0
        };

        let avg_dec_duration_ms = if decryptions > 0 {
            (dec_duration_sum as f64 / decryptions as f64) / 1_000_000.0
        } else {
            0.0
        };

        vec![
            Metric::counter(
                "crypto_chacha20_encryptions_total".to_string(),
                encryptions as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_chacha20_decryptions_total".to_string(),
                decryptions as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::gauge(
                "crypto_chacha20_avg_encryption_duration_ms".to_string(),
                avg_enc_duration_ms,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::gauge(
                "crypto_chacha20_avg_decryption_duration_ms".to_string(),
                avg_dec_duration_ms,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_chacha20_failures_total".to_string(),
                failures as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
        ]
    }

    // HKDF key derivation instrumentation

    /// Records an HKDF key derivation
    pub fn record_hkdf_derivation(&self, duration_ns: u64, success: bool) {
        self.hkdf_derivations_total.fetch_add(1, Ordering::Relaxed);
        self.hkdf_duration_ns_sum
            .fetch_add(duration_ns, Ordering::Relaxed);

        if !success {
            self.hkdf_failures_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Generates HKDF metrics
    pub fn hkdf_metrics(&self) -> Vec<Metric> {
        let derivations = self.hkdf_derivations_total.load(Ordering::Relaxed);
        let duration_sum = self.hkdf_duration_ns_sum.load(Ordering::Relaxed);
        let failures = self.hkdf_failures_total.load(Ordering::Relaxed);

        let avg_duration_ms = if derivations > 0 {
            (duration_sum as f64 / derivations as f64) / 1_000_000.0
        } else {
            0.0
        };

        vec![
            Metric::counter(
                "crypto_hkdf_derivations_total".to_string(),
                derivations as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::gauge(
                "crypto_hkdf_avg_duration_ms".to_string(),
                avg_duration_ms,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_hkdf_failures_total".to_string(),
                failures as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
        ]
    }

    // Key rotation instrumentation

    /// Records a key rotation event
    pub fn record_rotation_event(&self, duration_ms: u64, success: bool) {
        self.rotation_events_total.fetch_add(1, Ordering::Relaxed);
        self.rotation_duration_ms_sum
            .fetch_add(duration_ms, Ordering::Relaxed);

        if !success {
            self.rotation_failures_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Generates key rotation metrics
    pub fn rotation_metrics(&self) -> Vec<Metric> {
        let rotations = self.rotation_events_total.load(Ordering::Relaxed);
        let duration_sum = self.rotation_duration_ms_sum.load(Ordering::Relaxed);
        let failures = self.rotation_failures_total.load(Ordering::Relaxed);

        let avg_duration_ms = if rotations > 0 {
            duration_sum as f64 / rotations as f64
        } else {
            0.0
        };

        vec![
            Metric::counter(
                "crypto_rotation_events_total".to_string(),
                rotations as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::gauge(
                "crypto_rotation_avg_duration_ms".to_string(),
                avg_duration_ms,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_rotation_failures_total".to_string(),
                failures as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
        ]
    }

    // PoP token instrumentation

    /// Records a PoP token generation
    pub fn record_pop_token_generation(&self) {
        self.pop_token_generations_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Records a PoP token verification
    pub fn record_pop_token_verification(&self, success: bool, replay_detected: bool) {
        self.pop_token_verifications_total
            .fetch_add(1, Ordering::Relaxed);

        if !success {
            self.pop_token_verification_failures_total
                .fetch_add(1, Ordering::Relaxed);
        }

        if replay_detected {
            self.pop_token_replay_detections_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Generates PoP token metrics
    pub fn pop_token_metrics(&self) -> Vec<Metric> {
        let generations = self.pop_token_generations_total.load(Ordering::Relaxed);
        let verifications = self.pop_token_verifications_total.load(Ordering::Relaxed);
        let failures = self
            .pop_token_verification_failures_total
            .load(Ordering::Relaxed);
        let replays = self
            .pop_token_replay_detections_total
            .load(Ordering::Relaxed);

        vec![
            Metric::counter(
                "crypto_pop_token_generations_total".to_string(),
                generations as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_pop_token_verifications_total".to_string(),
                verifications as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_pop_token_verification_failures_total".to_string(),
                failures as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
            Metric::counter(
                "crypto_pop_token_replay_detections_total".to_string(),
                replays as f64,
                vec![("module".to_string(), "crypto".to_string())],
            ),
        ]
    }

    /// Generates all crypto metrics
    pub fn all_metrics(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();

        metrics.extend(self.x25519_metrics());
        metrics.extend(self.chacha20_metrics());
        metrics.extend(self.hkdf_metrics());
        metrics.extend(self.rotation_metrics());
        metrics.extend(self.pop_token_metrics());

        metrics
    }
}

impl Default for CryptoMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_metrics_creation() {
        let metrics = CryptoMetrics::new();
        assert_eq!(
            metrics.x25519_operations_total.load(Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn test_x25519_metrics() {
        let metrics = CryptoMetrics::new();

        metrics.record_x25519_operation(10_000_000, true); // 10ms
        metrics.record_x25519_operation(20_000_000, true); // 20ms
        metrics.record_x25519_operation(5_000_000, false); // 5ms, failed

        let metric_list = metrics.x25519_metrics();
        assert_eq!(metric_list.len(), 3);

        // Check operations total
        assert_eq!(metric_list[0].value, 3.0);

        // Check average duration (should be around 11.67ms)
        assert!(metric_list[1].value > 11.0 && metric_list[1].value < 12.0);

        // Check failures
        assert_eq!(metric_list[2].value, 1.0);
    }

    #[test]
    fn test_chacha20_metrics() {
        let metrics = CryptoMetrics::new();

        metrics.record_chacha20_encryption(15_000_000, true); // 15ms
        metrics.record_chacha20_decryption(18_000_000, true); // 18ms
        metrics.record_chacha20_encryption(12_000_000, false); // 12ms, failed

        let metric_list = metrics.chacha20_metrics();
        assert_eq!(metric_list.len(), 5);

        // Check encryptions total
        assert_eq!(metric_list[0].value, 2.0);

        // Check decryptions total
        assert_eq!(metric_list[1].value, 1.0);

        // Check failures
        assert_eq!(metric_list[4].value, 1.0);
    }

    #[test]
    fn test_hkdf_metrics() {
        let metrics = CryptoMetrics::new();

        metrics.record_hkdf_derivation(5_000_000, true); // 5ms
        metrics.record_hkdf_derivation(3_000_000, true); // 3ms

        let metric_list = metrics.hkdf_metrics();
        assert_eq!(metric_list.len(), 3);

        // Check derivations total
        assert_eq!(metric_list[0].value, 2.0);

        // Check average duration (should be 4ms)
        assert!((metric_list[1].value - 4.0).abs() < 0.1);
    }

    #[test]
    fn test_rotation_metrics() {
        let metrics = CryptoMetrics::new();

        metrics.record_rotation_event(1500, true); // 1.5 seconds
        metrics.record_rotation_event(2000, false); // 2 seconds, failed

        let metric_list = metrics.rotation_metrics();
        assert_eq!(metric_list.len(), 3);

        // Check events total
        assert_eq!(metric_list[0].value, 2.0);

        // Check average duration (should be 1750ms)
        assert!((metric_list[1].value - 1750.0).abs() < 0.1);

        // Check failures
        assert_eq!(metric_list[2].value, 1.0);
    }

    #[test]
    fn test_pop_token_metrics() {
        let metrics = CryptoMetrics::new();

        metrics.record_pop_token_generation();
        metrics.record_pop_token_generation();
        metrics.record_pop_token_verification(true, false); // Success, no replay
        metrics.record_pop_token_verification(false, false); // Failed
        metrics.record_pop_token_verification(false, true); // Failed + replay

        let metric_list = metrics.pop_token_metrics();
        assert_eq!(metric_list.len(), 4);

        // Check generations
        assert_eq!(metric_list[0].value, 2.0);

        // Check verifications
        assert_eq!(metric_list[1].value, 3.0);

        // Check failures
        assert_eq!(metric_list[2].value, 2.0);

        // Check replay detections
        assert_eq!(metric_list[3].value, 1.0);
    }

    #[test]
    fn test_all_metrics() {
        let metrics = CryptoMetrics::new();

        metrics.record_x25519_operation(10_000_000, true);
        metrics.record_chacha20_encryption(15_000_000, true);
        metrics.record_hkdf_derivation(5_000_000, true);
        metrics.record_rotation_event(1500, true);
        metrics.record_pop_token_generation();

        let all = metrics.all_metrics();

        // Should have 3 (x25519) + 5 (chacha20) + 3 (hkdf) + 3 (rotation) + 4 (pop) = 18 metrics
        assert_eq!(all.len(), 18);
    }
}
