// Input validation for device management API

use crate::error::ApiError;
use crate::types::DeviceId;
use base64::Engine;
use semver::Version;
use x25519_dalek::PublicKey;

/// Validates X25519 public key
/// Must be exactly 32 bytes
pub fn validate_x25519_public_key(key_base64: &str) -> Result<Vec<u8>, ApiError> {
    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(key_base64)
        .map_err(|e| ApiError::Validation(format!("Invalid base64 encoding: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(ApiError::Validation(format!(
            "X25519 public key must be exactly 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    // Verify it's a valid X25519 public key
    PublicKey::from(<[u8; 32]>::try_from(key_bytes.as_slice()).unwrap());

    Ok(key_bytes)
}

/// Validates firmware version (must be valid SemVer)
pub fn validate_firmware_version(version: &str) -> Result<Version, ApiError> {
    Version::parse(version).map_err(|e| {
        ApiError::Validation(format!("Invalid firmware version (must be SemVer): {}", e))
    })
}

/// Validates attestation format
pub fn validate_attestation_format(format: &str) -> Result<(), ApiError> {
    if format != "remote-attestation-v1" {
        return Err(ApiError::Validation(format!(
            "Unsupported attestation format: '{}', expected 'remote-attestation-v1'",
            format
        )));
    }
    Ok(())
}

/// Validates device capabilities
pub fn validate_capabilities(capabilities: &[String]) -> Result<(), ApiError> {
    const VALID_CAPABILITIES: &[&str] = &["telemetry", "control", "diagnostics", "ota-update"];

    for cap in capabilities {
        if !VALID_CAPABILITIES.contains(&cap.as_str()) {
            return Err(ApiError::Validation(format!(
                "Invalid capability: '{}'. Valid capabilities are: {}",
                cap,
                VALID_CAPABILITIES.join(", ")
            )));
        }
    }

    Ok(())
}

/// Generates a secure device token (32 bytes, base64url encoded)
pub fn generate_device_token() -> String {
    let mut token_bytes = [0u8; 32];
    rand::random::<[u8; 32]>().iter().enumerate().for_each(|(i, &b)| {
        token_bytes[i] = b;
    });
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token_bytes)
}

/// Validates CSR (Certificate Signing Request) format
/// Returns parsed CSR if valid
pub fn validate_csr(csr_base64: &str) -> Result<Vec<u8>, ApiError> {
    let csr_bytes = base64::engine::general_purpose::STANDARD
        .decode(csr_base64)
        .map_err(|e| ApiError::Validation(format!("Invalid CSR base64 encoding: {}", e)))?;

    // Basic PEM format check
    let csr_str = String::from_utf8(csr_bytes.clone())
        .map_err(|e| ApiError::Validation(format!("CSR is not valid UTF-8: {}", e)))?;

    if !csr_str.contains("-----BEGIN CERTIFICATE REQUEST-----")
        || !csr_str.contains("-----END CERTIFICATE REQUEST-----")
    {
        return Err(ApiError::Validation(
            "Invalid CSR format: missing PEM headers".to_string(),
        ));
    }

    Ok(csr_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_x25519_public_key_valid() {
        // Generate a valid 32-byte key
        let key = [42u8; 32];
        let key_base64 = base64::engine::general_purpose::STANDARD.encode(key);

        let result = validate_x25519_public_key(&key_base64);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_validate_x25519_public_key_invalid_length() {
        let key = [42u8; 16]; // Wrong length
        let key_base64 = base64::engine::general_purpose::STANDARD.encode(key);

        let result = validate_x25519_public_key(&key_base64);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 bytes"));
    }

    #[test]
    fn test_validate_x25519_public_key_invalid_base64() {
        let result = validate_x25519_public_key("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_firmware_version_valid() {
        assert!(validate_firmware_version("1.2.3").is_ok());
        assert!(validate_firmware_version("0.1.0").is_ok());
        assert!(validate_firmware_version("2.0.0-beta.1").is_ok());
    }

    #[test]
    fn test_validate_firmware_version_invalid() {
        assert!(validate_firmware_version("1.2").is_err());
        assert!(validate_firmware_version("invalid").is_err());
        assert!(validate_firmware_version("").is_err());
    }

    #[test]
    fn test_validate_attestation_format() {
        assert!(validate_attestation_format("remote-attestation-v1").is_ok());
        assert!(validate_attestation_format("invalid-format").is_err());
    }

    #[test]
    fn test_validate_capabilities_valid() {
        let caps = vec!["telemetry".to_string(), "control".to_string()];
        assert!(validate_capabilities(&caps).is_ok());
    }

    #[test]
    fn test_validate_capabilities_invalid() {
        let caps = vec!["invalid-capability".to_string()];
        assert!(validate_capabilities(&caps).is_err());
    }

    #[test]
    fn test_generate_device_token_format() {
        let token = generate_device_token();

        // Should be valid base64url
        assert!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&token).is_ok());

        // Decoded should be 32 bytes
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&token).unwrap();
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn test_generate_device_token_uniqueness() {
        let tokens: std::collections::HashSet<_> = (0..100)
            .map(|_| generate_device_token())
            .collect();

        // Should generate unique tokens
        assert_eq!(tokens.len(), 100);
    }

    #[test]
    fn test_validate_csr_valid() {
        let csr_pem = "-----BEGIN CERTIFICATE REQUEST-----\nMIICvDCCAaQCAQAwdzELMAkGA1UEBhMCVVMxDTALBgNVBAgMBFRlc3QxDTALBgNV\n-----END CERTIFICATE REQUEST-----";
        let csr_base64 = base64::engine::general_purpose::STANDARD.encode(csr_pem);

        let result = validate_csr(&csr_base64);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_csr_missing_headers() {
        let invalid_csr = "NOT A VALID CSR";
        let csr_base64 = base64::engine::general_purpose::STANDARD.encode(invalid_csr);

        let result = validate_csr(&csr_base64);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing PEM headers"));
    }
}
