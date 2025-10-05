// crates/transport/src/cert_pinning.rs
//
// Certificate Pinning Implementation
//
// Provides SHA-256 fingerprint-based certificate pinning for QUIC/TLS connections.
// Prevents MITM attacks by validating server certificates against pre-configured pins.
//
// Design:
// - Custom rustls ServerCertVerifier with pinning logic
// - SHA-256 fingerprint matching against configured pins
// - Support for multiple pins (enables certificate rotation)
// - Optional pinning (backward compatible)
//
// Security model:
// - Pins are SHA-256 hashes of DER-encoded certificates (industry standard)
// - Pin mismatch results in connection rejection
// - Empty pin list = no pinning (allows any valid cert)
// - Multiple pins = OR logic (any match succeeds, for rotation)

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error as TlsError, SignatureScheme};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

/// Certificate pinning verifier for rustls
///
/// Validates server certificates against a list of SHA-256 fingerprints.
/// If the pin set is empty, delegates to default WebPKI verification.
/// If pins are configured, the certificate MUST match at least one pin.
///
/// # Example
/// ```no_run
/// use honeylink_transport::cert_pinning::PinnedCertVerifier;
///
/// let pins = vec![
///     "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
/// ];
/// let verifier = PinnedCertVerifier::new(pins);
/// // Use verifier in rustls ClientConfig
/// ```
pub struct PinnedCertVerifier {
    /// Set of allowed SHA-256 fingerprints (hex-encoded, lowercase)
    pinned_fingerprints: HashSet<String>,
    /// Fallback WebPKI verifier for when no pins are configured
    webpki_verifier: Arc<dyn ServerCertVerifier>,
}

impl PinnedCertVerifier {
    /// Create a new pinned certificate verifier
    ///
    /// # Arguments
    /// - `pins`: List of SHA-256 fingerprints (hex strings, case-insensitive)
    ///
    /// # Panics
    /// Panics if the default WebPKI verifier cannot be constructed (should never happen)
    pub fn new(pins: Vec<String>) -> Self {
        // Normalize pins to lowercase hex
        let pinned_fingerprints = pins
            .into_iter()
            .map(|p| p.to_lowercase())
            .collect::<HashSet<_>>();

        // Create default WebPKI verifier for fallback
        // Use webpki-roots for system trust anchors
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let webpki_verifier = rustls::client::WebPkiServerVerifier::builder(Arc::new(root_store))
            .build()
            .expect("Failed to build WebPKI verifier");

        Self {
            pinned_fingerprints,
            webpki_verifier,
        }
    }

    /// Check if pinning is enabled (i.e., pins are configured)
    pub fn is_pinning_enabled(&self) -> bool {
        !self.pinned_fingerprints.is_empty()
    }

    /// Compute SHA-256 fingerprint of a certificate
    ///
    /// Returns hex-encoded lowercase fingerprint.
    fn compute_fingerprint(cert: &CertificateDer<'_>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert.as_ref());
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    /// Validate certificate against pinned fingerprints
    ///
    /// Returns true if the certificate matches any pin, or if no pins are configured.
    fn validate_pin(&self, cert: &CertificateDer<'_>) -> Result<(), PinError> {
        // If no pins configured, allow any cert (backward compat)
        if self.pinned_fingerprints.is_empty() {
            return Ok(());
        }

        let fingerprint = Self::compute_fingerprint(cert);
        if self.pinned_fingerprints.contains(&fingerprint) {
            Ok(())
        } else {
            Err(PinError::Mismatch {
                expected: self.pinned_fingerprints.clone(),
                actual: fingerprint,
            })
        }
    }
}

impl fmt::Debug for PinnedCertVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PinnedCertVerifier")
            .field("pinned_count", &self.pinned_fingerprints.len())
            .field("pinning_enabled", &self.is_pinning_enabled())
            .finish()
    }
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        // First, validate the pin if pinning is enabled
        if self.is_pinning_enabled() {
            self.validate_pin(end_entity)
                .map_err(|e| TlsError::General(e.to_string()))?;
        }

        // Then delegate to WebPKI for standard validation (expiry, chain, etc.)
        // NOTE: In production, you may want to skip WebPKI if pinning is enabled
        // (pin-only validation). For now, we do both for defense-in-depth.
        self.webpki_verifier.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        self.webpki_verifier
            .verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        self.webpki_verifier
            .verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.webpki_verifier.supported_verify_schemes()
    }
}

/// Certificate pinning errors
#[derive(Debug, Clone)]
pub enum PinError {
    /// Certificate fingerprint does not match any configured pin
    Mismatch {
        expected: HashSet<String>,
        actual: String,
    },
}

impl fmt::Display for PinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PinError::Mismatch { expected, actual } => {
                write!(
                    f,
                    "Certificate pin mismatch: expected one of {:?}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for PinError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinned_verifier_creation() {
        let pins = vec![
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        ];
        let verifier = PinnedCertVerifier::new(pins.clone());
        assert!(verifier.is_pinning_enabled());
        assert_eq!(verifier.pinned_fingerprints.len(), 1);
    }

    #[test]
    fn test_empty_pins_disables_pinning() {
        let verifier = PinnedCertVerifier::new(vec![]);
        assert!(!verifier.is_pinning_enabled());
    }

    #[test]
    fn test_pin_normalization() {
        let pins = vec![
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855".to_string(), // uppercase
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(), // lowercase
        ];
        let verifier = PinnedCertVerifier::new(pins);
        // Should deduplicate to 1 pin (case-insensitive)
        assert_eq!(verifier.pinned_fingerprints.len(), 1);
    }

    #[test]
    fn test_compute_fingerprint() {
        // Empty certificate for testing
        let cert = CertificateDer::from(vec![]);
        let fp = PinnedCertVerifier::compute_fingerprint(&cert);
        // SHA-256 of empty input
        assert_eq!(
            fp,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_validate_pin_match() {
        let pins = vec![
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        ];
        let verifier = PinnedCertVerifier::new(pins);
        let cert = CertificateDer::from(vec![]); // SHA-256 = e3b0c44...
        assert!(verifier.validate_pin(&cert).is_ok());
    }

    #[test]
    fn test_validate_pin_mismatch() {
        let pins = vec![
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        ];
        let verifier = PinnedCertVerifier::new(pins);
        let cert = CertificateDer::from(vec![0x01]); // Different hash
        assert!(verifier.validate_pin(&cert).is_err());
    }

    #[test]
    fn test_validate_pin_no_pins() {
        let verifier = PinnedCertVerifier::new(vec![]);
        let cert = CertificateDer::from(vec![0x01, 0x02, 0x03]);
        // No pins = allow any cert
        assert!(verifier.validate_pin(&cert).is_ok());
    }

    #[test]
    fn test_multiple_pins_rotation() {
        let pins = vec![
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(), // old pin (empty cert)
            "4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a".to_string(), // new pin ([0x01] cert)
        ];
        let verifier = PinnedCertVerifier::new(pins);

        // Old cert should match
        let old_cert = CertificateDer::from(vec![]);
        assert!(verifier.validate_pin(&old_cert).is_ok());

        // New cert should also match (rotation support)
        let new_cert = CertificateDer::from(vec![0x01]);
        assert!(verifier.validate_pin(&new_cert).is_ok());
    }

    #[test]
    fn test_debug_impl() {
        let pins = vec![
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        ];
        let verifier = PinnedCertVerifier::new(pins);
        let debug_str = format!("{:?}", verifier);
        assert!(debug_str.contains("PinnedCertVerifier"));
        assert!(debug_str.contains("pinned_count"));
    }

    #[test]
    fn test_pin_error_display() {
        let expected = vec!["abc123".to_string()].into_iter().collect();
        let error = PinError::Mismatch {
            expected,
            actual: "def456".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("pin mismatch"));
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
    }
}
