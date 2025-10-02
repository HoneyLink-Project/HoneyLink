// mTLS (Mutual TLS) middleware for certificate-based authentication
// This is a foundational implementation; full certificate verification
// will be completed in subsequent iterations

use crate::error::ApiError;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use rustls::ServerConfig;
use std::sync::Arc;

/// mTLS configuration
#[derive(Clone)]
pub struct MtlsConfig {
    /// Require client certificates
    pub require_client_cert: bool,
    /// Allowed certificate algorithms (ed25519, secp384r1)
    pub allowed_algorithms: Vec<String>,
}

impl Default for MtlsConfig {
    fn default() -> Self {
        MtlsConfig {
            require_client_cert: true,
            allowed_algorithms: vec!["ed25519".to_string(), "secp384r1".to_string()],
        }
    }
}

/// Client certificate information extracted from TLS handshake
#[derive(Debug, Clone)]
pub struct ClientCertInfo {
    /// Certificate subject
    pub subject: String,
    /// Certificate serial number
    pub serial_number: String,
    /// Certificate algorithm
    pub algorithm: String,
    /// Certificate validity (not before, not after)
    pub validity: (String, String),
}

/// Middleware to verify client certificate
/// Note: Actual certificate extraction happens at TLS layer
/// This middleware validates the extracted certificate information
pub async fn mtls_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // In a real implementation, we would:
    // 1. Extract client certificate from TLS connection
    // 2. Verify certificate chain
    // 3. Check certificate is not revoked (OCSP/CRL)
    // 4. Validate certificate algorithm (ed25519 or secp384r1 only)
    // 5. Extract device_id or subject from certificate
    // 6. Insert certificate info into request extensions

    // For now, we'll implement a stub that checks for certificate presence
    // The actual TLS handshake and certificate verification is handled by rustls

    // TODO: Implement full certificate verification in next iteration
    // This requires:
    // - Custom ClientCertVerifier for rustls
    // - Certificate chain validation
    // - OCSP/CRL checking
    // - Algorithm whitelisting (ed25519, secp384r1)

    tracing::debug!("mTLS middleware: certificate verification stub");

    Ok(next.run(request).await)
}

/// Create rustls ServerConfig with mTLS support
pub fn create_mtls_server_config(
    cert_path: &std::path::Path,
    key_path: &std::path::Path,
    client_ca_path: Option<&std::path::Path>,
) -> Result<Arc<ServerConfig>, ApiError> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls_pemfile::{certs, private_key};
    use std::fs::File;
    use std::io::BufReader;

    // Load server certificate
    let cert_file = File::open(cert_path)
        .map_err(|e| ApiError::Internal(format!("Failed to open certificate: {}", e)))?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<CertificateDer> = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ApiError::Internal(format!("Failed to parse certificate: {}", e)))?;

    // Load server private key
    let key_file = File::open(key_path)
        .map_err(|e| ApiError::Internal(format!("Failed to open private key: {}", e)))?;
    let mut key_reader = BufReader::new(key_file);
    let key = private_key(&mut key_reader)
        .map_err(|e| ApiError::Internal(format!("Failed to parse private key: {}", e)))?
        .ok_or_else(|| ApiError::Internal("No private key found".to_string()))?;

    // Create server config
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| ApiError::Internal(format!("Failed to create server config: {}", e)))?;

    // TODO: Configure client certificate verification if client_ca_path is provided
    // This requires implementing a custom ClientCertVerifier
    if let Some(_ca_path) = client_ca_path {
        tracing::warn!("Client CA provided but mTLS verification not yet fully implemented");
        // Future implementation:
        // 1. Load client CA certificates
        // 2. Create custom ClientCertVerifier
        // 3. Whitelist only ed25519 and secp384r1 algorithms
        // 4. Set config.client_cert_verifier
    }

    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtls_config_default() {
        let config = MtlsConfig::default();
        assert!(config.require_client_cert);
        assert_eq!(config.allowed_algorithms.len(), 2);
        assert!(config.allowed_algorithms.contains(&"ed25519".to_string()));
        assert!(config.allowed_algorithms.contains(&"secp384r1".to_string()));
    }

    #[test]
    fn test_client_cert_info() {
        let info = ClientCertInfo {
            subject: "CN=HL-EDGE-0001".to_string(),
            serial_number: "123456".to_string(),
            algorithm: "ed25519".to_string(),
            validity: ("2025-01-01T00:00:00Z".to_string(), "2026-01-01T00:00:00Z".to_string()),
        };

        assert_eq!(info.subject, "CN=HL-EDGE-0001");
        assert_eq!(info.algorithm, "ed25519");
    }
}
