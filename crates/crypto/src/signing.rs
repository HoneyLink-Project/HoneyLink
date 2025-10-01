//! Ed25519 signing operations

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use honeylink_core::Result;

/// Sign a message with Ed25519
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// Verify an Ed25519 signature
pub fn verify(verifying_key: &VerifyingKey, message: &[u8], signature: &Signature) -> Result<()> {
    verifying_key
        .verify(message, signature)
        .map_err(|e| honeylink_core::Error::Crypto(format!("Signature verification failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn test_sign_verify() {
        // Generate random signing key using from_bytes
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut OsRng, &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let message = b"test message";

        let signature = sign(&signing_key, message);
        assert!(verify(&verifying_key, message, &signature).is_ok());
    }
}
