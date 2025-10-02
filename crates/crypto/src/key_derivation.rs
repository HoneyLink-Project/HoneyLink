//! Key derivation using HKDF-SHA512
//!
//! Implements HKDF (HMAC-based Extract-and-Expand Key Derivation Function) with SHA-512.
//! Used for deriving session and stream keys from master keys in the HoneyLink key hierarchy.
//!
//! # Key Hierarchy
//! ```text
//! Root Key (HSM)
//!   ↓ HKDF-SHA512
//! Device Master Key (90-day rotation)
//!   ↓ HKDF-SHA512
//! Session Key (per-session)
//!   ↓ HKDF-SHA512
//! Stream Key (per-stream, 24-hour rotation)
//! ```
//!
//! # Example
//! ```
//! use honeylink_crypto::key_derivation::{KeyDerivation, DeriveContext};
//!
//! let master_key = [42u8; 32];
//! let context = DeriveContext::session("device-12345", "session-abc");
//!
//! let session_key = KeyDerivation::derive_with_context(
//!     &master_key,
//!     &context,
//!     32,
//! ).unwrap();
//! ```

use hkdf::Hkdf;
use honeylink_core::Result;
use sha2::Sha512;
use zeroize::Zeroizing;

/// Context for key derivation, providing domain separation.
///
/// Each derivation context includes a scope identifier and additional
/// context information to ensure derived keys are unique to their purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveContext {
    /// Root → Device Master derivation
    DeviceMaster { device_id: String },
    /// Device Master → Session derivation
    Session {
        device_id: String,
        session_id: String,
    },
    /// Session → Stream derivation
    Stream {
        session_id: String,
        stream_id: String,
    },
    /// Custom derivation (use with caution)
    Custom { label: String, info: Vec<u8> },
}

impl DeriveContext {
    /// Creates a device master key derivation context.
    pub fn device_master(device_id: impl Into<String>) -> Self {
        Self::DeviceMaster {
            device_id: device_id.into(),
        }
    }

    /// Creates a session key derivation context.
    pub fn session(device_id: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self::Session {
            device_id: device_id.into(),
            session_id: session_id.into(),
        }
    }

    /// Creates a stream key derivation context.
    pub fn stream(session_id: impl Into<String>, stream_id: impl Into<String>) -> Self {
        Self::Stream {
            session_id: session_id.into(),
            stream_id: stream_id.into(),
        }
    }

    /// Creates a custom derivation context.
    pub fn custom(label: impl Into<String>, info: impl Into<Vec<u8>>) -> Self {
        Self::Custom {
            label: label.into(),
            info: info.into(),
        }
    }

    /// Encodes the context into the HKDF info field.
    ///
    /// Format: "HoneyLink-v1|<scope>|<context_data>"
    /// This provides domain separation to prevent key reuse across contexts.
    fn encode(&self) -> Vec<u8> {
        let prefix = "HoneyLink-v1";

        match self {
            Self::DeviceMaster { device_id } => {
                format!("{}|DeviceMaster|{}", prefix, device_id).into_bytes()
            }
            Self::Session {
                device_id,
                session_id,
            } => {
                format!("{}|Session|{}|{}", prefix, device_id, session_id).into_bytes()
            }
            Self::Stream {
                session_id,
                stream_id,
            } => {
                format!("{}|Stream|{}|{}", prefix, session_id, stream_id).into_bytes()
            }
            Self::Custom { label, info } => {
                let mut encoded = format!("{}|Custom|{}", prefix, label).into_bytes();
                encoded.extend_from_slice(info);
                encoded
            }
        }
    }
}

/// HKDF-SHA512 key derivation operations.
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derives a key using HKDF-SHA512 with explicit parameters.
    ///
    /// # Arguments
    /// - `parent_key`: Input Key Material (IKM) - the parent key
    /// - `salt`: Optional salt (recommended: 32+ random bytes, or None for default)
    /// - `info`: Context information for domain separation
    /// - `output_length`: Desired key length in bytes (max 8160 for SHA-512)
    ///
    /// # Returns
    /// - `Zeroizing<Vec<u8>>`: Derived key material (automatically zeroized on drop)
    ///
    /// # Security
    /// - Uses HKDF-Extract to generate a Pseudo-Random Key (PRK) from IKM and salt
    /// - Uses HKDF-Expand to generate Output Key Material (OKM) from PRK and info
    /// - Info field provides domain separation (different contexts → different keys)
    ///
    /// # Errors
    /// Returns an error if output_length is too large (> 8160 bytes for SHA-512)
    pub fn derive(
        parent_key: &[u8],
        salt: Option<&[u8]>,
        info: &[u8],
        output_length: usize,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let hkdf = Hkdf::<Sha512>::new(salt, parent_key);
        let mut output = Zeroizing::new(vec![0u8; output_length]);
        hkdf.expand(info, &mut output)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Key derivation failed: {}", e)))?;
        Ok(output)
    }

    /// Derives a key using a structured context (recommended).
    ///
    /// # Arguments
    /// - `parent_key`: Input Key Material
    /// - `context`: Derivation context (automatically encodes domain separation)
    /// - `output_length`: Desired key length in bytes
    ///
    /// # Example
    /// ```
    /// use honeylink_crypto::key_derivation::{KeyDerivation, DeriveContext};
    ///
    /// let device_master = [0u8; 32];
    /// let context = DeriveContext::session("device-123", "session-abc");
    ///
    /// let session_key = KeyDerivation::derive_with_context(
    ///     &device_master,
    ///     &context,
    ///     32,
    /// ).unwrap();
    /// ```
    pub fn derive_with_context(
        parent_key: &[u8],
        context: &DeriveContext,
        output_length: usize,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let info = context.encode();
        Self::derive(parent_key, None, &info, output_length)
    }

    /// Derives a session key from a device master key.
    ///
    /// Convenience method for the most common use case.
    pub fn derive_session_key(
        device_master: &[u8],
        device_id: &str,
        session_id: &str,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let context = DeriveContext::session(device_id, session_id);
        Self::derive_with_context(device_master, &context, 32)
    }

    /// Derives a stream key from a session key.
    ///
    /// Convenience method for stream-specific key derivation.
    pub fn derive_stream_key(
        session_key: &[u8],
        session_id: &str,
        stream_id: &str,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let context = DeriveContext::stream(session_id, stream_id);
        Self::derive_with_context(session_key, &context, 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let parent = b"test_parent_key_32_bytes_long!!!";
        let result = KeyDerivation::derive(parent, None, b"test_info", 32);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_context_encoding() {
        let ctx1 = DeriveContext::device_master("device-123");
        let encoded1 = ctx1.encode();
        assert!(String::from_utf8_lossy(&encoded1).contains("DeviceMaster"));
        assert!(String::from_utf8_lossy(&encoded1).contains("device-123"));

        let ctx2 = DeriveContext::session("device-123", "session-abc");
        let encoded2 = ctx2.encode();
        assert!(String::from_utf8_lossy(&encoded2).contains("Session"));
        assert!(String::from_utf8_lossy(&encoded2).contains("session-abc"));
    }

    #[test]
    fn test_derive_with_context() {
        let parent = [42u8; 32];

        let ctx1 = DeriveContext::session("device-1", "session-1");
        let key1 = KeyDerivation::derive_with_context(&parent, &ctx1, 32).unwrap();

        let ctx2 = DeriveContext::session("device-1", "session-2");
        let key2 = KeyDerivation::derive_with_context(&parent, &ctx2, 32).unwrap();

        // Different contexts should produce different keys
        assert_ne!(key1.as_slice(), key2.as_slice());
    }

    #[test]
    fn test_derive_session_key() {
        let device_master = [0xABu8; 32];

        let key1 = KeyDerivation::derive_session_key(&device_master, "device-1", "session-1")
            .unwrap();
        let key2 = KeyDerivation::derive_session_key(&device_master, "device-1", "session-2")
            .unwrap();

        // Different session IDs should produce different keys
        assert_ne!(key1.as_slice(), key2.as_slice());
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }

    #[test]
    fn test_derive_stream_key() {
        let session_key = [0xCDu8; 32];

        let key1 = KeyDerivation::derive_stream_key(&session_key, "session-1", "stream-1")
            .unwrap();
        let key2 = KeyDerivation::derive_stream_key(&session_key, "session-1", "stream-2")
            .unwrap();

        // Different stream IDs should produce different keys
        assert_ne!(key1.as_slice(), key2.as_slice());
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }

    #[test]
    fn test_key_hierarchy() {
        let root_key = [0x01u8; 32];

        // Derive device master key
        let ctx_device = DeriveContext::device_master("device-123");
        let device_master = KeyDerivation::derive_with_context(&root_key, &ctx_device, 32)
            .unwrap();

        // Derive session key from device master
        let session_key = KeyDerivation::derive_session_key(
            &device_master,
            "device-123",
            "session-abc",
        ).unwrap();

        // Derive stream key from session
        let stream_key = KeyDerivation::derive_stream_key(
            &session_key,
            "session-abc",
            "stream-001",
        ).unwrap();

        // All keys should be different
        assert_ne!(device_master.as_slice(), session_key.as_slice());
        assert_ne!(session_key.as_slice(), stream_key.as_slice());
        assert_ne!(device_master.as_slice(), stream_key.as_slice());
    }

    #[test]
    fn test_deterministic_derivation() {
        let parent = [42u8; 32];
        let context = DeriveContext::session("device-1", "session-1");

        // Same inputs should produce same output
        let key1 = KeyDerivation::derive_with_context(&parent, &context, 32).unwrap();
        let key2 = KeyDerivation::derive_with_context(&parent, &context, 32).unwrap();

        assert_eq!(key1.as_slice(), key2.as_slice());
    }

    #[test]
    fn test_custom_context() {
        let parent = [42u8; 32];
        let ctx = DeriveContext::custom("telemetry", b"region=us-west-2");

        let key = KeyDerivation::derive_with_context(&parent, &ctx, 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_variable_output_length() {
        let parent = [42u8; 32];
        let context = DeriveContext::session("device-1", "session-1");

        let key16 = KeyDerivation::derive_with_context(&parent, &context, 16).unwrap();
        let key32 = KeyDerivation::derive_with_context(&parent, &context, 32).unwrap();
        let key64 = KeyDerivation::derive_with_context(&parent, &context, 64).unwrap();

        assert_eq!(key16.len(), 16);
        assert_eq!(key32.len(), 32);
        assert_eq!(key64.len(), 64);

        // First 16 bytes of key32 should match key16 (HKDF property)
        assert_eq!(&key32[..16], key16.as_slice());
    }
}
