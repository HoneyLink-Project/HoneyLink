//! Key lifecycle management with Vault integration.
//!
//! This module extends the existing key generation functionality with Vault storage,
//! implementing rotation, revocation, and grace period management according to
//! spec/security/key-management.md.

#[cfg(feature = "vault")]
use crate::vault::{KeyScope as VaultKeyScope, KeyMaterial, VaultClient, VaultError};
use crate::key_management::KeyScope;
use crate::rotation::{KeyRotationManager, RotationPolicy};
use std::time::Duration;
use thiserror::Error;
use zeroize::Zeroizing;

/// Errors that can occur during key lifecycle operations.
#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("Key generation failed: {0}")]
    Generation(String),

    #[error("Vault integration error: {0}")]
    Vault(String),

    #[error("Key rotation failed: {0}")]
    Rotation(String),

    #[error("Key revocation failed: {0}")]
    Revocation(String),

    #[error("Invalid key scope: {0}")]
    InvalidScope(String),

    #[cfg(feature = "vault")]
    #[error("Vault error: {0}")]
    VaultError(#[from] VaultError),
}

/// Maps KeyScope to VaultKeyScope.
#[cfg(feature = "vault")]
fn to_vault_scope(scope: KeyScope) -> Result<VaultKeyScope, LifecycleError> {
    match scope {
        KeyScope::Root => Ok(VaultKeyScope::Root),
        KeyScope::DeviceMaster => Ok(VaultKeyScope::Service),
        KeyScope::Session => Ok(VaultKeyScope::Service),
        KeyScope::Stream => Ok(VaultKeyScope::Service),
    }
}

/// Key lifecycle manager integrating Vault storage.
#[cfg(feature = "vault")]
pub struct VaultKeyLifecycle {
    vault: VaultClient,
    rotation_manager: KeyRotationManager,
}

#[cfg(feature = "vault")]
impl VaultKeyLifecycle {
    /// Creates a new lifecycle manager from environment variables.
    pub fn from_env() -> Result<Self, LifecycleError> {
        let vault = VaultClient::from_env()
            .map_err(|e| LifecycleError::Vault(e.to_string()))?;

        let rotation_manager = KeyRotationManager::new();

        Ok(Self {
            vault,
            rotation_manager,
        })
    }

    /// Creates a new lifecycle manager with explicit Vault client.
    pub fn new(vault: VaultClient) -> Self {
        Self {
            vault,
            rotation_manager: KeyRotationManager::new(),
        }
    }

    /// Generates a new key and stores it in Vault.
    ///
    /// # Arguments
    /// - `scope`: Key scope (Root, DeviceMaster, Session, Stream)
    /// - `name`: Key identifier
    /// - `key_data`: Key material (will be zeroized)
    ///
    /// # Returns
    /// - `Ok(())`: Key generated and stored successfully
    /// - `Err(LifecycleError)`: Generation or storage failed
    pub async fn generate_and_store(
        &self,
        scope: KeyScope,
        name: &str,
        key_data: Vec<u8>,
    ) -> Result<(), LifecycleError> {
        let vault_scope = to_vault_scope(scope)?;
        let ttl = vault_scope.rotation_interval();

        self.vault
            .store_key(vault_scope, name, key_data, ttl)
            .await
            .map_err(|e| LifecycleError::Vault(e.to_string()))?;

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
    /// - `Err(LifecycleError)`: Key not found or retrieval failed
    pub async fn retrieve(
        &self,
        scope: KeyScope,
        name: &str,
    ) -> Result<KeyMaterial, LifecycleError> {
        let vault_scope = to_vault_scope(scope)?;

        self.vault
            .retrieve_key(vault_scope, name)
            .await
            .map_err(|e| LifecycleError::Vault(e.to_string()))
    }

    /// Rotates a key by generating a new version.
    ///
    /// # Process
    /// 1. Generate new key material
    /// 2. Store in Vault with incremented version
    /// 3. Old version remains accessible for grace period
    ///
    /// # Arguments
    /// - `scope`: Key scope
    /// - `name`: Key identifier
    /// - `new_key_data`: New key material
    ///
    /// # Returns
    /// - `Ok(u32)`: New version number
    /// - `Err(LifecycleError)`: Rotation failed
    pub async fn rotate(
        &self,
        scope: KeyScope,
        name: &str,
        new_key_data: Vec<u8>,
    ) -> Result<u32, LifecycleError> {
        let vault_scope = to_vault_scope(scope)?;

        self.vault
            .rotate_key(vault_scope, name, new_key_data)
            .await
            .map_err(|e| LifecycleError::Rotation(e.to_string()))
    }

    /// Revokes a key (soft delete).
    ///
    /// The key is marked as deleted but remains accessible for recovery.
    /// For permanent deletion, use `destroy`.
    ///
    /// # Arguments
    /// - `scope`: Key scope
    /// - `name`: Key identifier
    pub async fn revoke(&self, scope: KeyScope, name: &str) -> Result<(), LifecycleError> {
        let vault_scope = to_vault_scope(scope)?;

        self.vault
            .delete_key(vault_scope, name)
            .await
            .map_err(|e| LifecycleError::Revocation(e.to_string()))
    }

    /// Permanently destroys a key (all versions).
    ///
    /// # Warning
    /// This is irreversible. Use with extreme caution.
    ///
    /// # Arguments
    /// - `scope`: Key scope
    /// - `name`: Key identifier
    /// - `versions`: Version numbers to destroy
    pub async fn destroy(
        &self,
        scope: KeyScope,
        name: &str,
        versions: Vec<u64>,
    ) -> Result<(), LifecycleError> {
        let vault_scope = to_vault_scope(scope)?;

        self.vault
            .destroy_key(vault_scope, name, versions)
            .await
            .map_err(|e| LifecycleError::Revocation(e.to_string()))
    }

    /// Lists all keys in a scope.
    pub async fn list(&self, scope: KeyScope) -> Result<Vec<String>, LifecycleError> {
        let vault_scope = to_vault_scope(scope)?;

        self.vault
            .list_keys(vault_scope)
            .await
            .map_err(|e| LifecycleError::Vault(e.to_string()))
    }

    /// Checks if a key needs rotation based on policy.
    ///
    /// # Arguments
    /// - `scope`: Key scope
    /// - `name`: Key identifier
    ///
    /// # Returns
    /// - `Ok(true)`: Key should be rotated
    /// - `Ok(false)`: Key is still valid
    pub async fn should_rotate(
        &self,
        scope: KeyScope,
        name: &str,
    ) -> Result<bool, LifecycleError> {
        let material = self.retrieve(scope, name).await?;

        // Parse created_at timestamp
        let created_at = chrono::DateTime::parse_from_rfc3339(&material.metadata.created_at)
            .map_err(|e| LifecycleError::Vault(format!("Invalid timestamp: {}", e)))?;

        let age = chrono::Utc::now().signed_duration_since(created_at);
        let rotation_interval = to_vault_scope(scope)?.rotation_interval();

        Ok(age.num_seconds() as u64 >= rotation_interval.as_secs())
    }

    /// Health check for Vault connectivity.
    pub async fn health_check(&self) -> Result<(), LifecycleError> {
        self.vault
            .health_check()
            .await
            .map_err(|e| LifecycleError::Vault(e.to_string()))
    }
}

#[cfg(test)]
#[cfg(feature = "vault")]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Vault dev server
    async fn test_lifecycle_roundtrip() {
        let lifecycle = VaultKeyLifecycle::from_env()
            .expect("Failed to create lifecycle manager");

        let key_data = vec![42u8; 32];
        let scope = KeyScope::DeviceMaster;
        let name = "test-device";

        // Generate and store
        lifecycle
            .generate_and_store(scope, name, key_data.clone())
            .await
            .expect("Failed to generate and store");

        // Retrieve
        let retrieved = lifecycle
            .retrieve(scope, name)
            .await
            .expect("Failed to retrieve");

        assert_eq!(retrieved.data, key_data);

        // Revoke
        lifecycle
            .revoke(scope, name)
            .await
            .expect("Failed to revoke");
    }

    #[tokio::test]
    #[ignore] // Requires Vault dev server
    async fn test_rotation() {
        let lifecycle = VaultKeyLifecycle::from_env()
            .expect("Failed to create lifecycle manager");

        let original_key = vec![1u8; 32];
        let new_key = vec![2u8; 32];
        let scope = KeyScope::Session;
        let name = "test-session";

        // Initial key
        lifecycle
            .generate_and_store(scope, name, original_key.clone())
            .await
            .expect("Failed to store initial key");

        // Rotate
        let new_version = lifecycle
            .rotate(scope, name, new_key.clone())
            .await
            .expect("Failed to rotate");

        assert_eq!(new_version, 2);

        // Retrieve new version
        let retrieved = lifecycle
            .retrieve(scope, name)
            .await
            .expect("Failed to retrieve rotated key");

        assert_eq!(retrieved.data, new_key);
        assert_eq!(retrieved.metadata.version, 2);

        // Cleanup
        lifecycle.destroy(scope, name, vec![1, 2]).await.ok();
    }
}
