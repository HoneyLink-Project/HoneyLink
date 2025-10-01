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

pub mod key_derivation;
pub mod key_management;
pub mod rotation;
pub mod signing;

pub use key_derivation::KeyDerivation;
pub use key_management::{KeyHierarchy, KeyScope};
pub use rotation::{KeyRotationManager, RotationPolicy, RotationStatus, VersionedKey, KeyVersion};
