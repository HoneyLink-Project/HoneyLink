// HashiCorp Vault PKI integration for certificate issuance

use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::pki;

/// Vault configuration
#[derive(Debug, Clone)]
pub struct VaultConfig {
    pub address: String,
    pub token: String,
    pub pki_mount: String,
    pub pki_role: String,
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            address: std::env::var("VAULT_ADDR")
                .unwrap_or_else(|_| "http://127.0.0.1:8200".to_string()),
            token: std::env::var("VAULT_TOKEN").unwrap_or_default(),
            pki_mount: std::env::var("VAULT_PKI_MOUNT")
                .unwrap_or_else(|_| "pki".to_string()),
            pki_role: std::env::var("VAULT_PKI_ROLE")
                .unwrap_or_else(|_| "honeylink-device".to_string()),
        }
    }
}

/// Certificate response from Vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedCertificate {
    pub certificate: String,
    pub issuing_ca: String,
    pub ca_chain: Vec<String>,
    pub serial_number: String,
}

/// Creates a Vault client
pub fn create_vault_client(config: &VaultConfig) -> Result<VaultClient, ApiError> {
    let settings = VaultClientSettingsBuilder::default()
        .address(&config.address)
        .token(&config.token)
        .build()
        .map_err(|e| ApiError::Dependency(format!("Failed to configure Vault client: {}", e)))?;

    Ok(VaultClient::new(settings)
        .map_err(|e| ApiError::Dependency(format!("Failed to create Vault client: {}", e)))?)
}

/// Issues a certificate using Vault PKI
pub async fn issue_certificate(
    client: &VaultClient,
    config: &VaultConfig,
    common_name: &str,
    csr_pem: &str,
) -> Result<IssuedCertificate, ApiError> {
    // Sign the CSR using Vault PKI
    let sign_response = pki::cert::sign(
        client,
        &config.pki_mount,
        &config.pki_role,
        Some(pki::csr::SignCertificateRequest::builder()
            .csr(csr_pem)
            .common_name(common_name)
            .ttl("8760h") // 1 year
            .build()
            .map_err(|e| ApiError::Internal(format!("Failed to build sign request: {}", e)))?),
    )
    .await
    .map_err(|e| ApiError::Dependency(format!("Vault PKI sign failed: {}", e)))?;

    Ok(IssuedCertificate {
        certificate: sign_response.certificate,
        issuing_ca: sign_response.issuing_ca,
        ca_chain: sign_response.ca_chain.unwrap_or_default(),
        serial_number: sign_response.serial_number,
    })
}

/// Revokes a certificate by serial number
pub async fn revoke_certificate(
    client: &VaultClient,
    config: &VaultConfig,
    serial_number: &str,
) -> Result<(), ApiError> {
    pki::cert::revoke(
        client,
        &config.pki_mount,
        serial_number,
    )
    .await
    .map_err(|e| ApiError::Dependency(format!("Vault PKI revoke failed: {}", e)))?;

    Ok(())
}

/// Retrieves the CA certificate chain
pub async fn get_ca_chain(
    client: &VaultClient,
    config: &VaultConfig,
) -> Result<Vec<String>, ApiError> {
    let ca_pem = pki::cert::read_ca_certificate(
        client,
        &config.pki_mount,
        "pem",
    )
    .await
    .map_err(|e| ApiError::Dependency(format!("Failed to read CA certificate: {}", e)))?;

    Ok(vec![ca_pem])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_vault_config() {
        let config = VaultConfig::default();
        assert_eq!(config.pki_mount, "pki");
        assert_eq!(config.pki_role, "honeylink-device");
    }

    #[test]
    fn test_vault_config_custom() {
        std::env::set_var("VAULT_ADDR", "https://vault.example.com");
        std::env::set_var("VAULT_TOKEN", "test-token");
        std::env::set_var("VAULT_PKI_MOUNT", "custom-pki");
        std::env::set_var("VAULT_PKI_ROLE", "custom-role");

        let config = VaultConfig::default();
        assert_eq!(config.address, "https://vault.example.com");
        assert_eq!(config.token, "test-token");
        assert_eq!(config.pki_mount, "custom-pki");
        assert_eq!(config.pki_role, "custom-role");

        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
        std::env::remove_var("VAULT_PKI_MOUNT");
        std::env::remove_var("VAULT_PKI_ROLE");
    }
}
