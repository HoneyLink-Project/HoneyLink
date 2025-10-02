//! ChaCha20-Poly1305 Authenticated Encryption with Associated Data (AEAD).
//!
//! Implements the ChaCha20-Poly1305 AEAD cipher as specified in RFC 8439.
//! Used for encrypting HoneyLink session data with integrity protection.
//!
//! # Security Properties
//! - 256-bit key strength
//! - 96-bit nonces (must never be reused with the same key)
//! - 128-bit authentication tag (prevents tampering)
//! - Constant-time operations (timing-attack resistant)
//! - Associated Data (AAD) support for context binding
//!
//! # Example
//! ```
//! use honeylink_crypto::aead::ChaCha20Poly1305Cipher;
//!
//! let key = [42u8; 32];
//! let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
//!
//! let plaintext = b"Hello, HoneyLink!";
//! let aad = b"session_12345";
//!
//! // Encrypt
//! let (nonce, ciphertext) = cipher.encrypt(plaintext, aad).unwrap();
//!
//! // Decrypt
//! let decrypted = cipher.decrypt(&nonce, &ciphertext, aad).unwrap();
//! assert_eq!(plaintext, &decrypted[..]);
//! ```

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use honeylink_core::Result;
use rand::Rng;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Maximum plaintext size: 1 MB (as per spec/modules/crypto-trust-anchor.md)
pub const MAX_PLAINTEXT_SIZE: usize = 1024 * 1024;

/// Nonce size in bytes (96 bits)
pub const NONCE_SIZE: usize = 12;

/// Authentication tag size in bytes (128 bits)
pub const TAG_SIZE: usize = 16;

/// A zeroizing wrapper for ChaCha20-Poly1305 encryption keys.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct EncryptionKey {
    bytes: [u8; 32],
}

impl EncryptionKey {
    /// Creates a new encryption key from raw bytes.
    ///
    /// # Arguments
    /// - `bytes`: 32-byte key material
    ///
    /// # Errors
    /// Returns an error if the input is not exactly 32 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(honeylink_core::Error::Crypto(format!(
                "Invalid encryption key length: expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);

        Ok(Self { bytes: array })
    }

    /// Generates a new random encryption key.
    ///
    /// # Security
    /// Uses OS-provided cryptographically secure randomness.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill(&mut bytes);

        Self { bytes }
    }

    /// Returns the key as a byte slice.
    fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionKey")
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// ChaCha20-Poly1305 cipher for authenticated encryption.
///
/// Provides encryption and decryption with integrity protection.
/// Nonces are randomly generated for each encryption operation.
pub struct ChaCha20Poly1305Cipher {
    key: EncryptionKey,
}

impl ChaCha20Poly1305Cipher {
    /// Creates a new cipher with the given key.
    ///
    /// # Arguments
    /// - `key`: 32-byte encryption key
    ///
    /// # Errors
    /// Returns an error if the key is not exactly 32 bytes.
    pub fn new(key: &[u8]) -> Result<Self> {
        let key = EncryptionKey::from_bytes(key)?;
        Ok(Self { key })
    }

    /// Creates a new cipher with a randomly generated key.
    pub fn new_random() -> Self {
        Self {
            key: EncryptionKey::generate(),
        }
    }

    /// Encrypts plaintext with optional associated data.
    ///
    /// # Arguments
    /// - `plaintext`: Data to encrypt (max 1 MB)
    /// - `aad`: Additional Authenticated Data (not encrypted, but authenticated)
    ///
    /// # Returns
    /// - `(nonce, ciphertext)`: The nonce and encrypted data (includes authentication tag)
    ///
    /// # Security
    /// - Nonce is randomly generated for each call (must never be reused)
    /// - AAD is typically the session ID to prevent cross-session replay attacks
    /// - Ciphertext includes a 16-byte authentication tag appended
    ///
    /// # Errors
    /// Returns an error if:
    /// - Plaintext exceeds MAX_PLAINTEXT_SIZE
    /// - Encryption fails (extremely rare, indicates hardware/memory issues)
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<([u8; NONCE_SIZE], Vec<u8>)> {
        if plaintext.len() > MAX_PLAINTEXT_SIZE {
            return Err(honeylink_core::Error::Crypto(format!(
                "Plaintext too large: {} bytes (max {} bytes)",
                plaintext.len(),
                MAX_PLAINTEXT_SIZE
            )));
        }

        // Generate a random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher instance
        let cipher = ChaCha20Poly1305::new(self.key.as_bytes().into());

        // Prepare payload with AAD
        let payload = Payload {
            msg: plaintext,
            aad,
        };

        // Encrypt and authenticate
        let ciphertext = cipher
            .encrypt(nonce, payload)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Encryption failed: {}", e)))?;

        Ok((nonce_bytes, ciphertext))
    }

    /// Decrypts ciphertext with optional associated data.
    ///
    /// # Arguments
    /// - `nonce`: The nonce used during encryption (12 bytes)
    /// - `ciphertext`: Encrypted data (includes authentication tag)
    /// - `aad`: Additional Authenticated Data (must match encryption AAD)
    ///
    /// # Returns
    /// - `Vec<u8>`: Decrypted plaintext
    ///
    /// # Security
    /// - Authentication is verified before decryption (AEAD property)
    /// - If AAD doesn't match, decryption fails (context binding)
    /// - If ciphertext is tampered, decryption fails (integrity protection)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Nonce is not exactly 12 bytes
    /// - Authentication tag verification fails (tampering or wrong key)
    /// - Decryption fails
    pub fn decrypt(&self, nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        if nonce.len() != NONCE_SIZE {
            return Err(honeylink_core::Error::Crypto(format!(
                "Invalid nonce length: expected {} bytes, got {}",
                NONCE_SIZE,
                nonce.len()
            )));
        }

        let nonce = Nonce::from_slice(nonce);

        // Create cipher instance
        let cipher = ChaCha20Poly1305::new(self.key.as_bytes().into());

        // Prepare payload with AAD
        let payload = Payload {
            msg: ciphertext,
            aad,
        };

        // Decrypt and verify authentication
        let plaintext = cipher
            .decrypt(nonce, payload)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Encrypts data in-place (overwrites the buffer).
    ///
    /// # Arguments
    /// - `buffer`: Mutable slice containing plaintext (will be overwritten with ciphertext + tag)
    /// - `aad`: Additional Authenticated Data
    ///
    /// # Returns
    /// - `nonce`: The randomly generated nonce
    ///
    /// # Note
    /// This is a zero-copy optimization for performance-critical paths.
    /// The buffer must have space for the plaintext + 16 bytes for the authentication tag.
    pub fn encrypt_in_place(
        &self,
        buffer: &mut dyn chacha20poly1305::aead::Buffer,
        aad: &[u8],
    ) -> Result<[u8; NONCE_SIZE]> {
        // Generate a random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher instance
        let cipher = ChaCha20Poly1305::new(self.key.as_bytes().into());

        // Encrypt in-place
        cipher
            .encrypt_in_place(nonce, aad, buffer)
            .map_err(|e| honeylink_core::Error::Crypto(format!("In-place encryption failed: {}", e)))?;

        Ok(nonce_bytes)
    }

    /// Decrypts data in-place (overwrites the buffer).
    ///
    /// # Arguments
    /// - `nonce`: The nonce used during encryption
    /// - `buffer`: Mutable slice containing ciphertext + tag (will be overwritten with plaintext)
    /// - `aad`: Additional Authenticated Data
    ///
    /// # Errors
    /// Returns an error if authentication or decryption fails.
    pub fn decrypt_in_place(
        &self,
        nonce: &[u8],
        buffer: &mut dyn chacha20poly1305::aead::Buffer,
        aad: &[u8],
    ) -> Result<()> {
        if nonce.len() != NONCE_SIZE {
            return Err(honeylink_core::Error::Crypto(format!(
                "Invalid nonce length: expected {} bytes, got {}",
                NONCE_SIZE,
                nonce.len()
            )));
        }

        let nonce = Nonce::from_slice(nonce);

        // Create cipher instance
        let cipher = ChaCha20Poly1305::new(self.key.as_bytes().into());

        // Decrypt in-place
        cipher
            .decrypt_in_place(nonce, aad, buffer)
            .map_err(|e| honeylink_core::Error::Crypto(format!("In-place decryption failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [42u8; 32];
        let cipher = ChaCha20Poly1305Cipher::new(&key).expect("Failed to create cipher");

        let plaintext = b"Hello, HoneyLink!";
        let aad = b"session_12345";

        // Encrypt
        let (nonce, ciphertext) = cipher
            .encrypt(plaintext, aad)
            .expect("Encryption failed");

        // Verify ciphertext is different from plaintext
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);

        // Verify ciphertext includes authentication tag
        assert_eq!(ciphertext.len(), plaintext.len() + TAG_SIZE);

        // Decrypt
        let decrypted = cipher
            .decrypt(&nonce, &ciphertext, aad)
            .expect("Decryption failed");

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_random_nonce_generation() {
        let cipher = ChaCha20Poly1305Cipher::new_random();

        let plaintext = b"Test message";
        let aad = b"session_1";

        let (nonce1, _) = cipher.encrypt(plaintext, aad).unwrap();
        let (nonce2, _) = cipher.encrypt(plaintext, aad).unwrap();

        // Nonces should be different for each encryption
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_tampered_ciphertext_rejected() {
        let cipher = ChaCha20Poly1305Cipher::new_random();

        let plaintext = b"Secret message";
        let aad = b"session_1";

        let (nonce, mut ciphertext) = cipher.encrypt(plaintext, aad).unwrap();

        // Tamper with the ciphertext
        ciphertext[0] ^= 0xFF;

        // Decryption should fail due to authentication failure
        let result = cipher.decrypt(&nonce, &ciphertext, aad);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Decryption failed"));
    }

    #[test]
    fn test_wrong_aad_rejected() {
        let cipher = ChaCha20Poly1305Cipher::new_random();

        let plaintext = b"Secret message";
        let correct_aad = b"session_1";
        let wrong_aad = b"session_2";

        let (nonce, ciphertext) = cipher.encrypt(plaintext, correct_aad).unwrap();

        // Decryption with wrong AAD should fail
        let result = cipher.decrypt(&nonce, &ciphertext, wrong_aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let cipher = ChaCha20Poly1305Cipher::new_random();

        let plaintext = b"";
        let aad = b"session_1";

        let (nonce, ciphertext) = cipher.encrypt(plaintext, aad).unwrap();

        // Ciphertext should only contain the authentication tag
        assert_eq!(ciphertext.len(), TAG_SIZE);

        let decrypted = cipher.decrypt(&nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted.len(), 0);
    }

    #[test]
    fn test_max_plaintext_size() {
        let cipher = ChaCha20Poly1305Cipher::new_random();

        // Create plaintext at the maximum allowed size
        let plaintext = vec![0u8; MAX_PLAINTEXT_SIZE];
        let aad = b"session_1";

        let result = cipher.encrypt(&plaintext, aad);
        assert!(result.is_ok());

        // Exceed the maximum size
        let oversized = vec![0u8; MAX_PLAINTEXT_SIZE + 1];
        let result = cipher.encrypt(&oversized, aad);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_invalid_nonce_length() {
        let cipher = ChaCha20Poly1305Cipher::new_random();

        let ciphertext = vec![0u8; 32];
        let aad = b"session_1";

        // Nonce too short
        let result = cipher.decrypt(&[0u8; 11], &ciphertext, aad);
        assert!(result.is_err());

        // Nonce too long
        let result = cipher.decrypt(&[0u8; 13], &ciphertext, aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let result = ChaCha20Poly1305Cipher::new(&[0u8; 31]);
        assert!(result.is_err());

        let result = ChaCha20Poly1305Cipher::new(&[0u8; 33]);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_generation() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();

        // Generated keys should be different
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_debug_redaction() {
        let key = EncryptionKey::generate();
        let debug_str = format!("{:?}", key);

        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains(&format!("{:?}", key.as_bytes())));
    }

    #[test]
    fn test_encryption_determinism() {
        let key = [42u8; 32];
        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

        let plaintext = b"Test message";
        let aad = b"session_1";

        let (nonce1, ciphertext1) = cipher.encrypt(plaintext, aad).unwrap();
        let (nonce2, ciphertext2) = cipher.encrypt(plaintext, aad).unwrap();

        // Same plaintext should produce different ciphertext due to random nonces
        assert_ne!(nonce1, nonce2);
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_large_aad() {
        let cipher = ChaCha20Poly1305Cipher::new_random();

        let plaintext = b"Short message";
        let aad = vec![0xABu8; 1024]; // Large AAD

        let (nonce, ciphertext) = cipher.encrypt(plaintext, &aad).unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext, &aad).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }
}
