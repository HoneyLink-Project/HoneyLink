//! # HoneyLink Keychain Integration
//!
//! Platform-agnostic secure credential storage using OS-native keychains.
//!
//! This module provides a unified interface for storing and retrieving sensitive
//! credentials across Windows, macOS, and Linux platforms. Credentials are stored
//! securely using:
//!
//! - **Windows**: Credential Manager (Data Protection API)
//! - **macOS**: Keychain Services
//! - **Linux**: Secret Service API (GNOME Keyring, KWallet)
//!
//! # Design Principles
//!
//! - **Platform-agnostic**: Single API works across all platforms
//! - **Pure Rust**: No C/C++ dependencies (uses keyring crate)
//! - **Secure by default**: Credentials never touch disk unencrypted
//! - **Zero-copy**: Minimal memory allocations for sensitive data
//! - **Zeroization**: Sensitive data cleared from memory on drop
//!
//! # Security Model
//!
//! - OS-level encryption enforced by platform keychain
//! - Access controlled by OS permissions (user/system level)
//! - No plaintext storage (all encryption handled by OS)
//! - Memory safety guaranteed by Rust ownership
//!
//! # Example
//!
//! ```no_run
//! use honeylink_keychain::{KeychainProvider, SystemKeychain};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let keychain = SystemKeychain::new("honeylink", "my-app")?;
//!
//! // Store credential
//! keychain.set_credential("api_key", b"secret-key-data")?;
//!
//! // Retrieve credential
//! let key = keychain.get_credential("api_key")?;
//! assert_eq!(key, b"secret-key-data");
//!
//! // Delete credential
//! keychain.delete_credential("api_key")?;
//! # Ok(())
//! # }
//! ```

use std::fmt;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Errors that can occur during keychain operations
#[derive(Debug, Error)]
pub enum KeychainError {
    /// Credential not found in keychain
    #[error("Credential '{0}' not found in keychain")]
    CredentialNotFound(String),

    /// Failed to access keychain (permissions, unavailable, etc.)
    #[error("Failed to access keychain: {0}")]
    AccessDenied(String),

    /// Invalid credential name or value
    #[error("Invalid credential: {0}")]
    InvalidCredential(String),

    /// Platform-specific error
    #[error("Platform error: {0}")]
    PlatformError(String),

    /// Keychain service unavailable (e.g., Linux without Secret Service)
    #[error("Keychain service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Result type for keychain operations
pub type Result<T> = std::result::Result<T, KeychainError>;

/// Secure credential storage with automatic zeroization
///
/// Wraps credential data and ensures it's cleared from memory on drop.
/// Use this for any sensitive data retrieved from the keychain.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureCredential {
    data: Vec<u8>,
}

impl SecureCredential {
    /// Create a new secure credential from bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get a reference to the credential data
    ///
    /// # Security Warning
    /// The returned slice should not be copied or persisted.
    /// It will be zeroized when the SecureCredential is dropped.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Convert to a UTF-8 string (if valid)
    ///
    /// Returns None if the data is not valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }

    /// Get the length of the credential data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the credential is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl fmt::Debug for SecureCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureCredential")
            .field("len", &self.len())
            .field("data", &"<redacted>")
            .finish()
    }
}

impl PartialEq for SecureCredential {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for SecureCredential {}

/// Platform-agnostic keychain provider trait
///
/// Implementations provide secure credential storage using OS-native keychains.
/// All methods are synchronous (keychain operations are typically fast).
///
/// # Thread Safety
/// Implementations should be Send + Sync for use in async contexts.
pub trait KeychainProvider: Send + Sync {
    /// Store a credential in the keychain
    ///
    /// # Arguments
    /// - `key`: Unique identifier for the credential
    /// - `value`: Credential data (will be encrypted by OS)
    ///
    /// # Errors
    /// - `AccessDenied`: Permission denied or keychain locked
    /// - `InvalidCredential`: Invalid key name or value
    /// - `PlatformError`: Platform-specific error
    fn set_credential(&self, key: &str, value: &[u8]) -> Result<()>;

    /// Retrieve a credential from the keychain
    ///
    /// # Arguments
    /// - `key`: Unique identifier for the credential
    ///
    /// # Returns
    /// SecureCredential that will be zeroized on drop
    ///
    /// # Errors
    /// - `CredentialNotFound`: Key doesn't exist
    /// - `AccessDenied`: Permission denied or keychain locked
    /// - `PlatformError`: Platform-specific error
    fn get_credential(&self, key: &str) -> Result<SecureCredential>;

    /// Delete a credential from the keychain
    ///
    /// # Arguments
    /// - `key`: Unique identifier for the credential
    ///
    /// # Errors
    /// - `CredentialNotFound`: Key doesn't exist (not an error, returns Ok)
    /// - `AccessDenied`: Permission denied
    /// - `PlatformError`: Platform-specific error
    fn delete_credential(&self, key: &str) -> Result<()>;

    /// Check if a credential exists in the keychain
    ///
    /// # Arguments
    /// - `key`: Unique identifier for the credential
    ///
    /// # Returns
    /// true if the credential exists, false otherwise
    fn has_credential(&self, key: &str) -> bool;
}

/// System keychain implementation using the `keyring` crate
///
/// Provides secure credential storage using OS-native keychains:
/// - Windows: Credential Manager
/// - macOS: Keychain Services
/// - Linux: Secret Service (GNOME Keyring, KWallet)
///
/// # Example
/// ```no_run
/// use honeylink_keychain::{KeychainProvider, SystemKeychain};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let keychain = SystemKeychain::new("honeylink", "production")?;
/// keychain.set_credential("db_password", b"super-secret")?;
/// # Ok(())
/// # }
/// ```
pub struct SystemKeychain {
    service: String,
    username: String,
}

impl SystemKeychain {
    /// Create a new system keychain accessor
    ///
    /// # Arguments
    /// - `service`: Service name (e.g., "honeylink", "my-app")
    /// - `username`: Username/namespace (e.g., "production", "user@example.com")
    ///
    /// # Design Rationale
    /// The service + username combination creates a unique namespace for credentials.
    /// This allows multiple applications or environments to coexist without conflicts.
    ///
    /// # Platform Mapping
    /// - Windows: Target = service, Username = username
    /// - macOS: Service = service, Account = username
    /// - Linux: Collection = service, Label = username
    pub fn new(service: impl Into<String>, username: impl Into<String>) -> Result<Self> {
        let service = service.into();
        let username = username.into();

        // Validate service and username (must be non-empty)
        if service.is_empty() || username.is_empty() {
            return Err(KeychainError::InvalidCredential(
                "Service and username must be non-empty".to_string(),
            ));
        }

        Ok(Self { service, username })
    }

    /// Build a full key name for the keyring crate
    ///
    /// Combines service, username, and key into a unique identifier.
    /// Format: "{service}:{username}:{key}"
    fn build_key_name(&self, key: &str) -> String {
        format!("{}:{}:{}", self.service, self.username, key)
    }

    /// Create a keyring Entry for the given key
    fn create_entry(&self, key: &str) -> keyring::Entry {
        let full_key = self.build_key_name(key);
        keyring::Entry::new(&self.service, &full_key)
            .expect("Failed to create keyring entry (should never fail)")
    }
}

impl KeychainProvider for SystemKeychain {
    fn set_credential(&self, key: &str, value: &[u8]) -> Result<()> {
        let entry = self.create_entry(key);

        // Convert bytes to string for keyring storage
        // Note: keyring crate only supports UTF-8 strings
        let value_str = std::str::from_utf8(value).map_err(|_| {
            KeychainError::InvalidCredential("Credential value must be valid UTF-8".to_string())
        })?;

        entry.set_password(value_str).map_err(|e| {
            KeychainError::PlatformError(format!("Failed to set credential: {}", e))
        })
    }

    fn get_credential(&self, key: &str) -> Result<SecureCredential> {
        let entry = self.create_entry(key);

        let password = entry.get_password().map_err(|e| match e {
            keyring::Error::NoEntry => KeychainError::CredentialNotFound(key.to_string()),
            keyring::Error::PlatformFailure(msg) => {
                KeychainError::PlatformError(format!("Platform error: {}", msg))
            }
            _ => KeychainError::PlatformError(format!("Failed to get credential: {}", e)),
        })?;

        Ok(SecureCredential::new(password.into_bytes()))
    }

    fn delete_credential(&self, key: &str) -> Result<()> {
        let entry = self.create_entry(key);

        match entry.delete_password() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => {
                // Not an error if credential doesn't exist
                Ok(())
            }
            Err(keyring::Error::PlatformFailure(msg)) => {
                Err(KeychainError::PlatformError(format!("Platform error: {}", msg)))
            }
            Err(e) => Err(KeychainError::PlatformError(format!(
                "Failed to delete credential: {}",
                e
            ))),
        }
    }

    fn has_credential(&self, key: &str) -> bool {
        self.get_credential(key).is_ok()
    }
}

impl fmt::Debug for SystemKeychain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemKeychain")
            .field("service", &self.service)
            .field("username", &self.username)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_credential_creation() {
        let cred = SecureCredential::new(vec![0x01, 0x02, 0x03]);
        assert_eq!(cred.as_bytes(), &[0x01, 0x02, 0x03]);
        assert_eq!(cred.len(), 3);
        assert!(!cred.is_empty());
    }

    #[test]
    fn test_secure_credential_as_str() {
        let cred = SecureCredential::new(b"hello".to_vec());
        assert_eq!(cred.as_str(), Some("hello"));

        let invalid = SecureCredential::new(vec![0xFF, 0xFE]);
        assert_eq!(invalid.as_str(), None);
    }

    #[test]
    fn test_secure_credential_debug() {
        let cred = SecureCredential::new(b"secret".to_vec());
        let debug = format!("{:?}", cred);
        assert!(debug.contains("SecureCredential"));
        assert!(debug.contains("len"));
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn test_secure_credential_equality() {
        let cred1 = SecureCredential::new(b"test".to_vec());
        let cred2 = SecureCredential::new(b"test".to_vec());
        let cred3 = SecureCredential::new(b"other".to_vec());

        assert_eq!(cred1, cred2);
        assert_ne!(cred1, cred3);
    }

    #[test]
    fn test_system_keychain_creation() {
        let keychain = SystemKeychain::new("test-service", "test-user").unwrap();
        assert_eq!(keychain.service, "test-service");
        assert_eq!(keychain.username, "test-user");
    }

    #[test]
    fn test_system_keychain_empty_service() {
        let result = SystemKeychain::new("", "user");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KeychainError::InvalidCredential(_)
        ));
    }

    #[test]
    fn test_system_keychain_empty_username() {
        let result = SystemKeychain::new("service", "");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KeychainError::InvalidCredential(_)
        ));
    }

    #[test]
    fn test_build_key_name() {
        let keychain = SystemKeychain::new("honeylink", "prod").unwrap();
        let full_key = keychain.build_key_name("api_key");
        assert_eq!(full_key, "honeylink:prod:api_key");
    }

    #[test]
    #[ignore] // Requires OS keychain access
    fn test_set_and_get_credential() {
        let keychain = SystemKeychain::new("honeylink-test", "test-user").unwrap();
        let test_key = "test-credential";
        let test_value = b"test-secret-value";

        // Clean up any existing credential
        let _ = keychain.delete_credential(test_key);

        // Set credential
        keychain.set_credential(test_key, test_value).unwrap();

        // Get credential
        let retrieved = keychain.get_credential(test_key).unwrap();
        assert_eq!(retrieved.as_bytes(), test_value);

        // Clean up
        keychain.delete_credential(test_key).unwrap();
    }

    #[test]
    #[ignore] // Requires OS keychain access
    fn test_delete_credential() {
        let keychain = SystemKeychain::new("honeylink-test", "test-user").unwrap();
        let test_key = "test-delete";

        // Set credential
        keychain.set_credential(test_key, b"value").unwrap();

        // Verify it exists
        assert!(keychain.has_credential(test_key));

        // Delete it
        keychain.delete_credential(test_key).unwrap();

        // Verify it's gone
        assert!(!keychain.has_credential(test_key));

        // Deleting non-existent credential should not error
        keychain.delete_credential(test_key).unwrap();
    }

    #[test]
    #[ignore] // Requires OS keychain access
    fn test_has_credential() {
        let keychain = SystemKeychain::new("honeylink-test", "test-user").unwrap();
        let test_key = "test-exists";

        // Clean up
        let _ = keychain.delete_credential(test_key);

        // Should not exist
        assert!(!keychain.has_credential(test_key));

        // Set credential
        keychain.set_credential(test_key, b"value").unwrap();

        // Should exist
        assert!(keychain.has_credential(test_key));

        // Clean up
        keychain.delete_credential(test_key).unwrap();
    }

    #[test]
    #[ignore] // Requires OS keychain access
    fn test_get_nonexistent_credential() {
        let keychain = SystemKeychain::new("honeylink-test", "test-user").unwrap();
        let result = keychain.get_credential("nonexistent-key-xyz");

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KeychainError::CredentialNotFound(_)
        ));
    }

    #[test]
    fn test_keychain_error_display() {
        let err = KeychainError::CredentialNotFound("test".to_string());
        assert!(err.to_string().contains("not found"));

        let err = KeychainError::AccessDenied("permission denied".to_string());
        assert!(err.to_string().contains("access keychain"));
    }
}
