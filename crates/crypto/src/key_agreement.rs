//! X25519 Elliptic Curve Diffie-Hellman key agreement.
//!
//! Implements the X25519 key exchange protocol as specified in RFC 7748.
//! Used for establishing shared secrets between HoneyLink devices during session setup.
//!
//! # Security Properties
//! - Forward secrecy: Each session uses ephemeral keys
//! - 128-bit security level (equivalent to AES-256 post-quantum adjusted)
//! - Side-channel resistant (constant-time operations)
//! - Automatic key zeroization on drop
//!
//! # Example
//! ```
//! use honeylink_crypto::key_agreement::KeyAgreement;
//!
//! // Alice generates a key pair
//! let (alice_secret, alice_public) = KeyAgreement::generate_keypair();
//!
//! // Bob generates a key pair
//! let (bob_secret, bob_public) = KeyAgreement::generate_keypair();
//!
//! // Both derive the same shared secret
//! let alice_shared = KeyAgreement::derive_shared_secret(&alice_secret, &bob_public).unwrap();
//! let bob_shared = KeyAgreement::derive_shared_secret(&bob_secret, &alice_public).unwrap();
//!
//! assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
//! ```

use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::Scalar;
use honeylink_core::Result;
use rand::RngCore;
use x25519_dalek::PublicKey;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A zeroizing wrapper for X25519 static secret keys.
///
/// Automatically zeroes memory on drop to prevent key material leakage.
///
/// Note: x25519-dalek 2.0 removed StaticSecret. We use curve25519-dalek's
/// Scalar directly for reusable keys.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey {
    scalar: Scalar,
}

impl SecretKey {
    /// Creates a new secret key from raw bytes.
    ///
    /// # Arguments
    /// - `bytes`: 32-byte secret key material
    ///
    /// # Errors
    /// Returns an error if the input is not exactly 32 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(honeylink_core::Error::Crypto(format!(
                "Invalid secret key length: expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        
        // Apply clamping for X25519
        array[0] &= 248;
        array[31] &= 127;
        array[31] |= 64;
        
        let scalar = Scalar::from_bytes_mod_order(array);

        // Zeroize the temporary array
        array.zeroize();

        Ok(Self { scalar })
    }

    /// Returns the public key corresponding to this secret key.
    pub fn public_key(&self) -> PublicKey {
        let public_point = &self.scalar * &curve25519_dalek::constants::X25519_BASEPOINT;
        PublicKey::from(*public_point.as_bytes())
    }

    /// Returns a reference to the inner scalar for cryptographic operations.
    ///
    /// # Security
    /// This method is internal and should not expose the secret beyond this module.
    pub(crate) fn as_scalar(&self) -> &Scalar {
        &self.scalar
    }
}

/// A wrapper for the shared secret derived from X25519 key agreement.
///
/// Automatically zeroes memory on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SharedSecret {
    bytes: [u8; 32],
}

impl SharedSecret {
    /// Returns the shared secret as a byte slice.
    ///
    /// # Security
    /// Caller must ensure the bytes are used in constant-time operations
    /// and zeroized after use.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Consumes the shared secret and returns the raw bytes.
    ///
    /// # Security
    /// Caller is responsible for zeroizing the returned array.
    pub fn into_bytes(self) -> [u8; 32] {
        let bytes = self.bytes;
        // Prevent double-zeroization (moved out)
        std::mem::forget(self);
        bytes
    }
}

impl std::fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedSecret")
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// X25519 key agreement operations.
pub struct KeyAgreement;

impl KeyAgreement {
    /// Generates a new X25519 keypair using cryptographically secure randomness.
    ///
    /// # Returns
    /// - `(SecretKey, PublicKey)`: A tuple of the secret and public keys
    ///
    /// # Security
    /// - Uses OS-provided randomness (`getrandom` crate)
    /// - Secret key is automatically zeroized on drop
    /// - Public key can be safely shared
    ///
    /// # Example
    /// ```
    /// use honeylink_crypto::key_agreement::KeyAgreement;
    ///
    /// let (secret, public) = KeyAgreement::generate_keypair();
    /// // Send `public` to peer, keep `secret` private
    /// ```
    pub fn generate_keypair() -> (SecretKey, PublicKey) {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        
        // Apply X25519 clamping
        bytes[0] &= 248;
        bytes[31] &= 127;
        bytes[31] |= 64;
        
        let scalar = Scalar::from_bytes_mod_order(bytes);
        let secret = SecretKey { scalar };
        let public = secret.public_key();

        // Zeroize the temporary array
        bytes.zeroize();

        (secret, public)
    }

    /// Derives a shared secret from your secret key and the peer's public key.
    ///
    /// # Arguments
    /// - `our_secret`: Your secret key (will not be consumed or zeroized)
    /// - `their_public`: The peer's public key
    ///
    /// # Returns
    /// - `Ok(SharedSecret)`: The 32-byte shared secret
    /// - `Err`: If the key agreement fails (e.g., low-order point)
    ///
    /// # Security
    /// - Constant-time operation (resistant to timing attacks)
    /// - Rejects low-order points (contributory behavior attack mitigation)
    /// - Returned shared secret is automatically zeroized on drop
    ///
    /// # Example
    /// ```
    /// use honeylink_crypto::key_agreement::KeyAgreement;
    ///
    /// let (alice_secret, alice_public) = KeyAgreement::generate_keypair();
    /// let (bob_secret, bob_public) = KeyAgreement::generate_keypair();
    ///
    /// let alice_shared = KeyAgreement::derive_shared_secret(&alice_secret, &bob_public).unwrap();
    /// let bob_shared = KeyAgreement::derive_shared_secret(&bob_secret, &alice_public).unwrap();
    ///
    /// assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    /// ```
    pub fn derive_shared_secret(
        our_secret: &SecretKey,
        their_public: &PublicKey,
    ) -> Result<SharedSecret> {
        let their_point = MontgomeryPoint(*their_public.as_bytes());
        let shared_point = our_secret.as_scalar() * their_point;
        let shared_bytes = *shared_point.as_bytes();

        // Check for low-order points (all-zeros shared secret indicates contributory behavior attack)
        if shared_bytes == [0u8; 32] {
            return Err(honeylink_core::Error::Crypto(
                "Key agreement failed: low-order point detected (possible attack)".to_string(),
            ));
        }

        Ok(SharedSecret {
            bytes: shared_bytes,
        })
    }

    /// Serializes a public key to bytes for transmission.
    ///
    /// # Returns
    /// - 32-byte array representing the public key
    pub fn serialize_public_key(public: &PublicKey) -> [u8; 32] {
        *public.as_bytes()
    }

    /// Deserializes a public key from bytes received from a peer.
    ///
    /// # Arguments
    /// - `bytes`: 32-byte public key
    ///
    /// # Errors
    /// Returns an error if the input is not exactly 32 bytes.
    pub fn deserialize_public_key(bytes: &[u8]) -> Result<PublicKey> {
        if bytes.len() != 32 {
            return Err(honeylink_core::Error::Crypto(format!(
                "Invalid public key length: expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(PublicKey::from(array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (secret1, public1) = KeyAgreement::generate_keypair();
        let (secret2, public2) = KeyAgreement::generate_keypair();

        // Different keypairs should have different public keys
        assert_ne!(public1.as_bytes(), public2.as_bytes());

        // Public key should match the derived one from secret
        assert_eq!(public1.as_bytes(), secret1.public_key().as_bytes());
        assert_eq!(public2.as_bytes(), secret2.public_key().as_bytes());
    }

    #[test]
    fn test_key_agreement() {
        // Alice and Bob generate keypairs
        let (alice_secret, alice_public) = KeyAgreement::generate_keypair();
        let (bob_secret, bob_public) = KeyAgreement::generate_keypair();

        // Both derive the same shared secret
        let alice_shared = KeyAgreement::derive_shared_secret(&alice_secret, &bob_public)
            .expect("Alice key agreement failed");
        let bob_shared = KeyAgreement::derive_shared_secret(&bob_secret, &alice_public)
            .expect("Bob key agreement failed");

        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());

        // Shared secret should be non-zero
        assert_ne!(alice_shared.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_public_key_serialization() {
        let (_secret, public) = KeyAgreement::generate_keypair();

        let serialized = KeyAgreement::serialize_public_key(&public);
        assert_eq!(serialized.len(), 32);

        let deserialized =
            KeyAgreement::deserialize_public_key(&serialized).expect("Deserialization failed");

        assert_eq!(public.as_bytes(), deserialized.as_bytes());
    }

    #[test]
    fn test_invalid_public_key_length() {
        let result = KeyAgreement::deserialize_public_key(&[0u8; 31]);
        assert!(result.is_err());

        let result = KeyAgreement::deserialize_public_key(&[0u8; 33]);
        assert!(result.is_err());
    }

    #[test]
    fn test_secret_key_from_bytes() {
        let bytes = [42u8; 32];
        let secret = SecretKey::from_bytes(&bytes).expect("Failed to create secret key");

        // Verify the secret key can derive a public key
        let _public = secret.public_key();

        // Test invalid length
        let result = SecretKey::from_bytes(&[0u8; 31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_low_order_point_detection() {
        // Create a secret key
        let (secret, _public) = KeyAgreement::generate_keypair();

        // All-zero public key is a low-order point
        let bad_public = PublicKey::from([0u8; 32]);

        let result = KeyAgreement::derive_shared_secret(&secret, &bad_public);

        // Should detect and reject low-order point
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("low-order point"));
    }

    #[test]
    fn test_shared_secret_zeroization() {
        let (alice_secret, alice_public) = KeyAgreement::generate_keypair();
        let (_bob_secret, bob_public) = KeyAgreement::generate_keypair();

        let shared = KeyAgreement::derive_shared_secret(&alice_secret, &bob_public).unwrap();

        // Shared secret should be non-zero
        assert_ne!(shared.as_bytes(), &[0u8; 32]);

        // Drop the shared secret (zeroization happens automatically)
        drop(shared);

        // Cannot verify zeroization directly (memory is freed),
        // but we trust the Zeroize trait implementation
    }

    #[test]
    fn test_debug_redaction() {
        let (alice_secret, _) = KeyAgreement::generate_keypair();
        let (_, bob_public) = KeyAgreement::generate_keypair();

        let shared = KeyAgreement::derive_shared_secret(&alice_secret, &bob_public).unwrap();

        let debug_str = format!("{:?}", shared);
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains(&format!("{:?}", shared.as_bytes())));
    }
}
