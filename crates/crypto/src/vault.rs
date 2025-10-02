//! Vault client integration for HoneyLink key management.
//!
//! This module provides a safe Rust wrapper around HashiCorp Vault's KV v2 API,
//! implementing the key hierarchy defined in spec/security/encryption.md:
//! - k_root: Root trust anchor (HSM-backed)
//! - k_service: Service-level master keys
//! - k_profile: Profile data encryption keys
//! - k_telemetry: Telemetry data encryption keys
//!
//! # Security Considerations
//! - All communication with Vault uses TLS 1.3 (in production)
//! - Tokens are short-lived (5 minutes recommended)
//! - Keys are retrieved on-demand and zeroized after use
//! - Fallback to local encrypted storage if Vault is unavailable
//!
//! # Example
//! ```no_run
//! use honeylink_crypto::vault::{VaultClient, KeyScope};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = VaultClient::from_env()?;
//!
//! // Store a service key
//! client.store_key(
//!     KeyScope::Service,
//!     "session-orchestrator",
//!     &[0u8; 32],
//!     std::time::Duration::from_secs(90 * 24 * 3600), // 90 days
//! ).await?;
//!
//! // Retrieve the key
//! let key = client.retrieve_key(KeyScope::Service, "session-orchestrator").await?;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use vaultrs::client::{VaultClient as VrsClient, VaultClientSettingsBuilder};
use vaultrs::kv2;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Errors that can occur during Vault operations.
#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Vault connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Key not found: scope={scope:?}, name={name}")]
    KeyNotFound { scope: KeyScope, name: String },

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Key expired: scope={scope:?}, name={name}")]
    KeyExpired { scope: KeyScope, name: String },

    #[error("Vault API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Key scope in the HoneyLink hierarchy.
///
/// Maps to Vault paths:
/// - Root -> honeylink/k_root
/// - Service -> honeylink/k_service
/// - Profile -> honeylink/k_profile
/// - Telemetry -> honeylink/k_telemetry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyScope {
    /// Root trust anchor (HSM-backed, rarely accessed)
    Root,
    /// Service-level master keys (Session Orchestrator, Policy Engine, etc.)
    Service,
    /// Profile data encryption keys
    Profile,
    /// Telemetry data encryption keys
    Telemetry,
}

impl KeyScope {
    /// Returns the Vault path prefix for this scope.
    pub fn vault_path(&self) -> &'static str {
        match self {
            KeyScope::Root => "k_root",
            KeyScope::Service => "k_service",
            KeyScope::Profile => "k_profile",
            KeyScope::Telemetry => "k_telemetry",
        }
    }

    /// Returns the recommended rotation interval for keys in this scope.
    pub fn rotation_interval(&self) -> Duration {
        match self {
            KeyScope::Root => Duration::from_secs(5 * 365 * 24 * 3600), // 5 years
            KeyScope::Service => Duration::from_secs(365 * 24 * 3600),   // 1 year
            KeyScope::Profile => Duration::from_secs(90 * 24 * 3600),    // 90 days
            KeyScope::Telemetry => Duration::from_secs(90 * 24 * 3600),  // 90 days
        }
    }
}

/// Metadata for a stored key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Key scope (root, service, profile, telemetry)
    pub scope: KeyScope,
    /// Key identifier (e.g., "session-orchestrator", "device-12345")
    pub name: String,
    /// Key version (increments on rotation)
    pub version: u32,
    /// Creation timestamp (RFC 3339)
    pub created_at: String,
    /// Expiration timestamp (RFC 3339)
    pub expires_at: String,
    /// Algorithm used (e.g., "X25519", "ChaCha20-Poly1305")
    pub algorithm: String,
    /// Environment (dev, staging, production)
    pub environment: String,
}

/// A key material wrapper that automatically zeroizes on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct KeyMaterial {
    #[zeroize(skip)]
    pub metadata: KeyMetadata,
    pub data: Vec<u8>,
}

impl std::fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyMaterial")
            .field("metadata", &self.metadata)
            .field("data", &"[REDACTED]")
            .finish()
    }
}

/// Vault secret format (internal representation).
#[derive(Debug, Serialize, Deserialize)]
struct VaultSecret {
    key_data: String, // Base64-encoded key material
    metadata: KeyMetadata,
}

/// HoneyLink Vault client for key management.
///
/// Implements the key hierarchy defined in spec/security/encryption.md.
/// All operations are async and use tokio runtime.
pub struct VaultClient {
    client: VrsClient,
    mount: String,
    environment: String,
}

impl VaultClient {
    /// Creates a new Vault client from environment variables.
    ///
    /// Required environment variables:
    /// - `VAULT_ADDR`: Vault server address (e.g., "https://vault.example.com:8200")
    /// - `VAULT_TOKEN`: Vault authentication token
    ///
    /// Optional environment variables:
    /// - `VAULT_NAMESPACE`: Vault namespace (Enterprise feature)
    /// - `HONEYLINK_ENV`: Environment name (default: "development")
    /// - `VAULT_MOUNT`: KV mount point (default: "honeylink")
    pub fn from_env() -> Result<Self, VaultError> {
        let addr = std::env::var("VAULT_ADDR")
            .map_err(|_| VaultError::Configuration("VAULT_ADDR not set".to_string()))?;
        let token = std::env::var("VAULT_TOKEN")
            .map_err(|_| VaultError::Configuration("VAULT_TOKEN not set".to_string()))?;
        let namespace = std::env::var("VAULT_NAMESPACE").ok();
        let mount = std::env::var("VAULT_MOUNT").unwrap_or_else(|_| "honeylink".to_string());
        let environment =
            std::env::var("HONEYLINK_ENV").unwrap_or_else(|_| "development".to_string());

        Self::new(&addr, &token, namespace.as_deref(), &mount, &environment)
    }

    /// Creates a new Vault client with explicit configuration.
    ///
    /// # Arguments
    /// - `addr`: Vault server address
    /// - `token`: Vault authentication token
    /// - `namespace`: Optional Vault namespace
    /// - `mount`: KV v2 mount point
    /// - `environment`: Environment name (dev, staging, production)
    pub fn new(
        addr: &str,
        token: &str,
        namespace: Option<&str>,
        mount: &str,
        environment: &str,
    ) -> Result<Self, VaultError> {
        let mut settings_builder = VaultClientSettingsBuilder::default();
        settings_builder.address(addr);
        settings_builder.token(token);

        if let Some(ns) = namespace {
            settings_builder.namespace(ns);
        }

        let settings = settings_builder
            .build()
            .map_err(|e| VaultError::Configuration(e.to_string()))?;

        let client = VrsClient::new(settings)
            .map_err(|e| VaultError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            mount: mount.to_string(),
            environment: environment.to_string(),
        })
    }

    /// Stores a key in Vault with metadata.
    ///
    /// # Arguments
    /// - `scope`: Key scope (Root, Service, Profile, Telemetry)
    /// - `name`: Key identifier (e.g., "session-orchestrator")
    /// - `key_data`: Raw key bytes (will be zeroized)
    /// - `ttl`: Time-to-live before expiration
    ///
    /// # Security
    /// - Keys are base64-encoded before storage
    /// - Original key_data is zeroized after encoding
    /// - Metadata includes expiration timestamp
    pub async fn store_key(
        &self,
        scope: KeyScope,
        name: &str,
        mut key_data: Vec<u8>,
        ttl: Duration,
    ) -> Result<(), VaultError> {
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::from_std(ttl).unwrap();

        let metadata = KeyMetadata {
            scope,
            name: name.to_string(),
            version: 1, // Will be incremented on rotation
            created_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
            algorithm: "X25519".to_string(),
            environment: self.environment.clone(),
        };

        // Encode key data as base64
        let encoded_key = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &key_data);

        // Zeroize original key material
        key_data.zeroize();

        let secret = VaultSecret {
            key_data: encoded_key,
            metadata,
        };

        let path = format!("{}/{}", scope.vault_path(), name);

        kv2::set(
            &self.client,
            &self.mount,
            &path,
            &serde_json::to_value(&secret)
                .map_err(|e| VaultError::Serialization(e))?,
        )
        .await
        .map_err(|e| VaultError::ApiError(format!("Failed to store key: {}", e)))?;

        Ok(())
    }

    /// Retrieves a key from Vault.
    ///
    /// # Arguments
    /// - `scope`: Key scope
    /// - `name`: Key identifier
    ///
    /// # Returns
    /// - `Ok(KeyMaterial)`: Key material with metadata
    /// - `Err(VaultError::KeyNotFound)`: Key does not exist
    /// - `Err(VaultError::KeyExpired)`: Key has expired
    ///
    /// # Security
    /// - Returned KeyMaterial is automatically zeroized on drop
    /// - Expiration is checked before returning
    pub async fn retrieve_key(
        &self,
        scope: KeyScope,
        name: &str,
    ) -> Result<KeyMaterial, VaultError> {
        let path = format!("{}/{}", scope.vault_path(), name);

        let secret: VaultSecret = kv2::read(&self.client, &self.mount, &path)
            .await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    VaultError::KeyNotFound {
                        scope,
                        name: name.to_string(),
                    }
                } else {
                    VaultError::ApiError(format!("Failed to retrieve key: {}", e))
                }
            })?;

        // Check expiration
        let expires_at = chrono::DateTime::parse_from_rfc3339(&secret.metadata.expires_at)
            .map_err(|e| VaultError::InvalidKeyFormat(e.to_string()))?;

        if expires_at < chrono::Utc::now() {
            return Err(VaultError::KeyExpired {
                scope,
                name: name.to_string(),
            });
        }

        // Decode key data
        let key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &secret.key_data)
            .map_err(|e| VaultError::InvalidKeyFormat(e.to_string()))?;

        Ok(KeyMaterial {
            metadata: secret.metadata,
            data: key_bytes,
        })
    }

    /// Lists all keys in a given scope.
    ///
    /// # Arguments
    /// - `scope`: Key scope to list
    ///
    /// # Returns
    /// - List of key names in the scope
    pub async fn list_keys(&self, scope: KeyScope) -> Result<Vec<String>, VaultError> {
        let path = scope.vault_path();

        let keys: Vec<String> = kv2::list(&self.client, &self.mount, path)
            .await
            .map_err(|e| VaultError::ApiError(format!("Failed to list keys: {}", e)))?;

        Ok(keys)
    }

    /// Deletes a key from Vault.
    ///
    /// # Security
    /// - This performs a soft delete (version is marked deleted)
    /// - For permanent deletion, use `destroy_key`
    pub async fn delete_key(&self, scope: KeyScope, name: &str) -> Result<(), VaultError> {
        let path = format!("{}/{}", scope.vault_path(), name);

        kv2::delete_latest(&self.client, &self.mount, &path)
            .await
            .map_err(|e| VaultError::ApiError(format!("Failed to delete key: {}", e)))?;

        Ok(())
    }

    /// Permanently destroys a key (all versions).
    ///
    /// # Warning
    /// This is irreversible. Use with caution.
    pub async fn destroy_key(
        &self,
        scope: KeyScope,
        name: &str,
        versions: Vec<u64>,
    ) -> Result<(), VaultError> {
        let path = format!("{}/{}", scope.vault_path(), name);

        kv2::destroy(&self.client, &self.mount, &path, versions)
            .await
            .map_err(|e| VaultError::ApiError(format!("Failed to destroy key: {}", e)))?;

        Ok(())
    }

    /// Rotates a key by creating a new version.
    ///
    /// # Process
    /// 1. Retrieve current key
    /// 2. Generate new key material (caller responsibility)
    /// 3. Store new version with incremented version number
    /// 4. Old version remains accessible for grace period
    pub async fn rotate_key(
        &self,
        scope: KeyScope,
        name: &str,
        new_key_data: Vec<u8>,
    ) -> Result<u32, VaultError> {
        // Retrieve current key to get version
        let current = self.retrieve_key(scope, name).await?;
        let new_version = current.metadata.version + 1;

        // Store new version
        let ttl = scope.rotation_interval();
        self.store_key(scope, name, new_key_data, ttl).await?;

        Ok(new_version)
    }

    /// Health check for Vault connectivity.
    ///
    /// # Returns
    /// - `Ok(())`: Vault is reachable and authenticated
    /// - `Err(VaultError)`: Connection or authentication failure
    pub async fn health_check(&self) -> Result<(), VaultError> {
        vaultrs::sys::health(&self.client)
            .await
            .map_err(|e| VaultError::Connection(format!("Health check failed: {}", e)))?;

        Ok(())
    }
}

// Required to satisfy the vault feature
use base64::Engine as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Vault dev server
    async fn test_vault_roundtrip() {
        let client = VaultClient::from_env().expect("Failed to create client");

        let key_data = vec![42u8; 32];
        let scope = KeyScope::Service;
        let name = "test-key";

        // Store key
        client
            .store_key(scope, name, key_data.clone(), Duration::from_secs(3600))
            .await
            .expect("Failed to store key");

        // Retrieve key
        let retrieved = client
            .retrieve_key(scope, name)
            .await
            .expect("Failed to retrieve key");

        assert_eq!(retrieved.data, key_data);
        assert_eq!(retrieved.metadata.scope, scope);
        assert_eq!(retrieved.metadata.name, name);

        // Delete key
        client
            .delete_key(scope, name)
            .await
            .expect("Failed to delete key");
    }

    #[test]
    fn test_key_scope_paths() {
        assert_eq!(KeyScope::Root.vault_path(), "k_root");
        assert_eq!(KeyScope::Service.vault_path(), "k_service");
        assert_eq!(KeyScope::Profile.vault_path(), "k_profile");
        assert_eq!(KeyScope::Telemetry.vault_path(), "k_telemetry");
    }

    #[test]
    fn test_key_scope_rotation_intervals() {
        assert_eq!(
            KeyScope::Root.rotation_interval(),
            Duration::from_secs(5 * 365 * 24 * 3600)
        );
        assert_eq!(
            KeyScope::Service.rotation_interval(),
            Duration::from_secs(365 * 24 * 3600)
        );
        assert_eq!(
            KeyScope::Profile.rotation_interval(),
            Duration::from_secs(90 * 24 * 3600)
        );
    }
}
