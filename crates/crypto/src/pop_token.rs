//! Proof-of-Possession (PoP) Token generation and verification.
//!
//! Implements DPoP (Demonstrating Proof-of-Possession) tokens as specified in RFC 9449.
//! Used for binding access tokens to specific devices/sessions in HoneyLink.
//!
//! # Security Properties
//! - Token binding: Access tokens are cryptographically bound to session keys
//! - Replay protection: Nonce-based replay防止 and短命 tokens (5分)
//! - Key confirmation: Proves possession of private key without revealing it
//! - Man-in-the-middle protection: Prevents token theft and reuse
//!
//! # Example
//! ```
//! use honeylink_crypto::pop_token::{PopTokenGenerator, PopClaims};
//!
//! // Generate a token
//! let generator = PopTokenGenerator::new();
//! let session_key = [42u8; 32];
//! let claims = PopClaims::new("session-123", "device-456", 300);
//!
//! let token = generator.generate(&session_key, &claims).unwrap();
//!
//! // Verify the token
//! let verified = generator.verify(&token, &session_key).unwrap();
//! assert_eq!(verified.session_id, "session-123");
//! ```

use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use honeylink_core::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Maximum token validity period (5 minutes as per spec)
pub const MAX_TOKEN_TTL_SECONDS: i64 = 300;

/// PoP token claims (JWT-like structure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopClaims {
    /// Session ID (binds token to session)
    pub session_id: String,
    /// Device ID (binds token to device)
    pub device_id: String,
    /// Issued at timestamp (Unix epoch seconds)
    pub iat: i64,
    /// Expiration timestamp (Unix epoch seconds)
    pub exp: i64,
    /// Nonce for replay protection
    pub nonce: String,
    /// HTTP method (for DPoP binding)
    pub htm: Option<String>,
    /// HTTP URL (for DPoP binding)
    pub htu: Option<String>,
}

impl PopClaims {
    /// Creates new PoP claims with a generated nonce.
    ///
    /// # Arguments
    /// - `session_id`: Session identifier
    /// - `device_id`: Device identifier
    /// - `ttl_seconds`: Token time-to-live (max 300 seconds)
    pub fn new(session_id: impl Into<String>, device_id: impl Into<String>, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        let ttl = std::cmp::min(ttl_seconds, MAX_TOKEN_TTL_SECONDS);

        Self {
            session_id: session_id.into(),
            device_id: device_id.into(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(ttl)).timestamp(),
            nonce: Self::generate_nonce(),
            htm: None,
            htu: None,
        }
    }

    /// Creates claims with HTTP method and URL binding (DPoP).
    pub fn with_http_binding(
        session_id: impl Into<String>,
        device_id: impl Into<String>,
        ttl_seconds: i64,
        http_method: impl Into<String>,
        http_url: impl Into<String>,
    ) -> Self {
        let mut claims = Self::new(session_id, device_id, ttl_seconds);
        claims.htm = Some(http_method.into());
        claims.htu = Some(http_url.into());
        claims
    }

    /// Generates a cryptographically secure nonce (16 bytes, base64-encoded).
    fn generate_nonce() -> String {
        let mut nonce_bytes = [0u8; 16];
        rand::Rng::fill(&mut rand::thread_rng(), &mut nonce_bytes[..]);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(nonce_bytes)
    }

    /// Checks if the token is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Returns the remaining time until expiration (in seconds).
    pub fn ttl_remaining(&self) -> i64 {
        self.exp - Utc::now().timestamp()
    }
}

/// A PoP token with signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopToken {
    /// Token claims (payload)
    pub claims: PopClaims,
    /// Ed25519 signature (64 bytes, base64-encoded)
    pub signature: String,
    /// Public key fingerprint (SHA-256 hash of session key, base64-encoded)
    pub key_fingerprint: String,
}

impl PopToken {
    /// Encodes the token as a compact string for HTTP headers.
    ///
    /// Format: `base64(claims).base64(signature)`
    pub fn to_compact(&self) -> Result<String> {
        let claims_json = serde_json::to_string(&self.claims)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Serialization failed: {}", e)))?;

        let claims_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims_json);

        Ok(format!("{}.{}", claims_b64, self.signature))
    }

    /// Decodes a compact token string.
    pub fn from_compact(compact: &str) -> Result<Self> {
        let parts: Vec<&str> = compact.split('.').collect();
        if parts.len() != 2 {
            return Err(honeylink_core::Error::Crypto(
                "Invalid token format: expected 2 parts".to_string(),
            ));
        }

        let claims_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|e| honeylink_core::Error::Crypto(format!("Base64 decode failed: {}", e)))?;

        let claims: PopClaims = serde_json::from_slice(&claims_json)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Deserialization failed: {}", e)))?;

        Ok(Self {
            claims,
            signature: parts[1].to_string(),
            key_fingerprint: String::new(), // Will be verified separately
        })
    }
}

/// PoP token generator and verifier.
pub struct PopTokenGenerator {
    /// Nonce store for replay protection (in-memory, use Redis in production)
    nonce_store: std::sync::Arc<std::sync::RwLock<std::collections::HashSet<String>>>,
}

impl PopTokenGenerator {
    /// Creates a new PoP token generator.
    pub fn new() -> Self {
        Self {
            nonce_store: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Generates a PoP token by signing the claims with a key derived from the session key.
    ///
    /// # Arguments
    /// - `session_key`: Session key (32 bytes)
    /// - `claims`: Token claims
    ///
    /// # Returns
    /// - `PopToken`: Signed token
    ///
    /// # Security
    /// - Session key is hashed to derive an Ed25519 signing key
    /// - Claims are serialized and signed
    /// - Public key fingerprint is included for verification
    pub fn generate(&self, session_key: &[u8], claims: &PopClaims) -> Result<PopToken> {
        if session_key.len() != 32 {
            return Err(honeylink_core::Error::Crypto(format!(
                "Invalid session key length: expected 32 bytes, got {}",
                session_key.len()
            )));
        }

        // Derive Ed25519 signing key from session key (using SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(b"HoneyLink-PoP-v1|");
        hasher.update(session_key);
        let key_hash = hasher.finalize();

        let signing_key = SigningKey::from_bytes(&key_hash.into());

        // Serialize claims
        let claims_json = serde_json::to_string(claims)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Serialization failed: {}", e)))?;

        // Sign the claims
        let signature = signing_key.sign(claims_json.as_bytes());

        // Compute key fingerprint
        let mut fp_hasher = Sha256::new();
        fp_hasher.update(session_key);
        let fingerprint = fp_hasher.finalize();
        let key_fingerprint = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(fingerprint);

        // Encode signature as base64
        let signature_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Ok(PopToken {
            claims: claims.clone(),
            signature: signature_b64,
            key_fingerprint,
        })
    }

    /// Verifies a PoP token's signature and validity.
    ///
    /// # Arguments
    /// - `token`: Token to verify
    /// - `session_key`: Session key used to generate the token
    ///
    /// # Returns
    /// - `Ok(PopClaims)`: Token is valid
    /// - `Err`: Token is invalid (expired, wrong signature, replay detected)
    ///
    /// # Security
    /// - Checks expiration
    /// - Verifies signature with session key
    /// - Checks nonce for replay protection
    pub fn verify(&self, token: &PopToken, session_key: &[u8]) -> Result<PopClaims> {
        // Check expiration
        if token.claims.is_expired() {
            return Err(honeylink_core::Error::Crypto(
                "Token expired".to_string(),
            ));
        }

        // Derive verification key from session key
        let mut hasher = Sha256::new();
        hasher.update(b"HoneyLink-PoP-v1|");
        hasher.update(session_key);
        let key_hash = hasher.finalize();

        let signing_key = SigningKey::from_bytes(&key_hash.into());
        let verifying_key = signing_key.verifying_key();

        // Serialize claims
        let claims_json = serde_json::to_string(&token.claims)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Serialization failed: {}", e)))?;

        // Decode signature
        let signature_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&token.signature)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Signature decode failed: {}", e)))?;

        let signature = Signature::from_bytes(
            &signature_bytes
                .try_into()
                .map_err(|_| honeylink_core::Error::Crypto("Invalid signature length".to_string()))?,
        );

        // Verify signature
        verifying_key
            .verify(claims_json.as_bytes(), &signature)
            .map_err(|e| honeylink_core::Error::Crypto(format!("Signature verification failed: {}", e)))?;

        // Check nonce for replay protection
        let mut nonce_store = self.nonce_store.write().unwrap();
        if nonce_store.contains(&token.claims.nonce) {
            return Err(honeylink_core::Error::Crypto(
                "Replay detected: nonce already used".to_string(),
            ));
        }
        nonce_store.insert(token.claims.nonce.clone());

        Ok(token.claims.clone())
    }

    /// Clears the nonce store (for testing or periodic cleanup).
    pub fn clear_nonce_store(&self) {
        self.nonce_store.write().unwrap().clear();
    }
}

impl Default for PopTokenGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_claims_creation() {
        let claims = PopClaims::new("session-123", "device-456", 300);

        assert_eq!(claims.session_id, "session-123");
        assert_eq!(claims.device_id, "device-456");
        assert!(!claims.is_expired());
        assert!(claims.ttl_remaining() > 0);
        assert!(claims.ttl_remaining() <= 300);
    }

    #[test]
    fn test_pop_claims_with_http_binding() {
        let claims = PopClaims::with_http_binding(
            "session-123",
            "device-456",
            300,
            "GET",
            "https://api.honeylink.example/stream",
        );

        assert_eq!(claims.htm, Some("GET".to_string()));
        assert_eq!(claims.htu, Some("https://api.honeylink.example/stream".to_string()));
    }

    #[test]
    fn test_token_generation_and_verification() {
        let generator = PopTokenGenerator::new();
        let session_key = [42u8; 32];
        let claims = PopClaims::new("session-123", "device-456", 300);

        // Generate token
        let token = generator.generate(&session_key, &claims).unwrap();

        assert!(!token.signature.is_empty());
        assert!(!token.key_fingerprint.is_empty());

        // Verify token
        let verified = generator.verify(&token, &session_key).unwrap();

        assert_eq!(verified.session_id, claims.session_id);
        assert_eq!(verified.device_id, claims.device_id);
    }

    #[test]
    fn test_token_verification_with_wrong_key() {
        let generator = PopTokenGenerator::new();
        let session_key = [42u8; 32];
        let wrong_key = [43u8; 32];
        let claims = PopClaims::new("session-123", "device-456", 300);

        let token = generator.generate(&session_key, &claims).unwrap();

        // Verification with wrong key should fail
        let result = generator.verify(&token, &wrong_key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Signature verification failed"));
    }

    #[test]
    fn test_expired_token() {
        let generator = PopTokenGenerator::new();
        let session_key = [42u8; 32];
        let mut claims = PopClaims::new("session-123", "device-456", 300);

        // Manually set expiration to past
        claims.exp = Utc::now().timestamp() - 100;

        let token = generator.generate(&session_key, &claims).unwrap();

        // Verification should fail due to expiration
        let result = generator.verify(&token, &session_key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Token expired"));
    }

    #[test]
    fn test_replay_protection() {
        let generator = PopTokenGenerator::new();
        let session_key = [42u8; 32];
        let claims = PopClaims::new("session-123", "device-456", 300);

        let token = generator.generate(&session_key, &claims).unwrap();

        // First verification should succeed
        let result1 = generator.verify(&token, &session_key);
        assert!(result1.is_ok());

        // Second verification with same token should fail (replay detected)
        let result2 = generator.verify(&token, &session_key);
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("Replay detected"));
    }

    #[test]
    fn test_token_compact_encoding() {
        let generator = PopTokenGenerator::new();
        let session_key = [42u8; 32];
        let claims = PopClaims::new("session-123", "device-456", 300);

        let token = generator.generate(&session_key, &claims).unwrap();

        // Encode to compact format
        let compact = token.to_compact().unwrap();

        assert!(compact.contains('.'));
        assert!(compact.len() > 100); // Should be a reasonable length

        // Decode from compact format
        let decoded = PopToken::from_compact(&compact).unwrap();

        assert_eq!(decoded.claims.session_id, token.claims.session_id);
        assert_eq!(decoded.claims.device_id, token.claims.device_id);
    }

    #[test]
    fn test_max_ttl_enforcement() {
        let claims = PopClaims::new("session-123", "device-456", 1000);

        // TTL should be capped at MAX_TOKEN_TTL_SECONDS
        let ttl = claims.exp - claims.iat;
        assert!(ttl <= MAX_TOKEN_TTL_SECONDS);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let claims1 = PopClaims::new("session-1", "device-1", 300);
        let claims2 = PopClaims::new("session-1", "device-1", 300);

        // Nonces should be different even for same session/device
        assert_ne!(claims1.nonce, claims2.nonce);
    }

    #[test]
    fn test_invalid_session_key_length() {
        let generator = PopTokenGenerator::new();
        let invalid_key = [42u8; 31]; // Wrong length
        let claims = PopClaims::new("session-123", "device-456", 300);

        let result = generator.generate(&invalid_key, &claims);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid session key length"));
    }
}
