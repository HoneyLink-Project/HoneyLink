//! # HoneyLink Crypto & Trust Anchor
//!
//! Cryptographic operations and key management for HoneyLink.
//!
//! ## Features
//!
//! - X25519 key agreement
//! - ChaCha20-Poly1305 AEAD encryption
//! - HKDF-SHA512 key derivation
//! - Ed25519 signatures
//! - Hierarchical key management (k_root → k_service → k_session → k_stream)
//!
//! ## Security
//!
//! - 100% Rust implementation (no C/C++)
//! - Secure memory handling with zeroize
//! - Key lifecycle management

pub mod aead;
pub mod key_agreement;
pub mod key_derivation;
pub mod key_management;
pub mod pop_token;
pub mod rotation;
pub mod signing;

#[cfg(feature = "vault")]
pub mod rotation_scheduler;

#[cfg(feature = "vault")]
pub mod vault;

#[cfg(feature = "vault")]
pub mod lifecycle;

pub use aead::{ChaCha20Poly1305Cipher, EncryptionKey, MAX_PLAINTEXT_SIZE, NONCE_SIZE, TAG_SIZE};
pub use key_agreement::{KeyAgreement, SecretKey, SharedSecret};
pub use key_derivation::{DeriveContext, KeyDerivation};
pub use key_management::{KeyHierarchy, KeyScope};
pub use pop_token::{PopClaims, PopToken, PopTokenGenerator, MAX_TOKEN_TTL_SECONDS};
pub use rotation::{KeyRotationManager, RotationPolicy, RotationStatus, VersionedKey, KeyVersion};

#[cfg(feature = "vault")]
pub use vault::{VaultClient, VaultError, KeyMaterial, KeyMetadata};

#[cfg(feature = "vault")]
pub use lifecycle::{VaultKeyLifecycle, LifecycleError};

#[cfg(feature = "vault")]
pub use rotation_scheduler::{RotationScheduler, RotationTrigger, RotationEvent, SchedulerConfig, SchedulerError};
