//! Crypto telemetry trait
//!
//! Defines trait for recording cryptographic operation metrics.
//! Implementation provided in honeylink-telemetry crate.

/// Crypto telemetry recorder trait
///
/// Abstraction for recording crypto operation metrics to avoid circular dependency.
pub trait CryptoTelemetry: Send + Sync {
    /// Create new crypto telemetry instance
    fn new_crypto_telemetry() -> Self where Self: Sized;

    /// Record X25519 key agreement operation
    fn record_x25519_operation(&self, duration_ns: u64, success: bool);

    /// Record ChaCha20-Poly1305 encryption
    fn record_chacha20_encryption(&self, duration_ns: u64, bytes: usize, success: bool);

    /// Record ChaCha20-Poly1305 decryption
    fn record_chacha20_decryption(&self, duration_ns: u64, bytes: usize, success: bool);

    /// Record HKDF key derivation
    fn record_hkdf_derivation(&self, duration_ns: u64, success: bool);

    /// Record key rotation event
    fn record_key_rotation(&self, duration_ms: u64, success: bool);

    /// Record PoP token generation
    fn record_pop_token_generation(&self);

    /// Record PoP token verification
    fn record_pop_token_verification(&self, success: bool);

    /// Record PoP token replay detection
    fn record_pop_token_replay_detection(&self);
}
