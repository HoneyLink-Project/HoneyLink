//! Property-based tests for cryptographic primitives.
//!
//! Uses proptest to verify cryptographic invariants across randomized inputs.

use honeylink_crypto::{
    aead::ChaCha20Poly1305Cipher,
    key_agreement::KeyAgreement,
    key_derivation::{DeriveContext, KeyDerivation},
    pop_token::{PopClaims, PopTokenGenerator},
};
use proptest::prelude::*;

// Property: X25519 key agreement is commutative
proptest! {
    #[test]
    fn prop_x25519_commutativity(seed1: u64, seed2: u64) {
        let (alice_secret, alice_public) = KeyAgreement::generate_keypair();
        let (bob_secret, bob_public) = KeyAgreement::generate_keypair();

        let alice_shared = KeyAgreement::derive_shared_secret(&alice_secret, &bob_public).unwrap();
        let bob_shared = KeyAgreement::derive_shared_secret(&bob_secret, &alice_public).unwrap();

        prop_assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }
}

// Property: ChaCha20-Poly1305 encrypt-decrypt roundtrip preserves data
proptest! {
    #[test]
    fn prop_aead_roundtrip(plaintext in prop::collection::vec(any::<u8>(), 0..1024)) {
        let cipher = ChaCha20Poly1305Cipher::new_random();
        let aad = b"test_aad";

        let (nonce, ciphertext) = cipher.encrypt(&plaintext, aad).unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext, aad).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    }
}

// Property: HKDF key derivation is deterministic
proptest! {
    #[test]
    fn prop_hkdf_deterministic(parent_key in prop::collection::vec(any::<u8>(), 32..33)) {
        let context = DeriveContext::session("device-1", "session-1");

        let key1 = KeyDerivation::derive_with_context(&parent_key, &context, 32).unwrap();
        let key2 = KeyDerivation::derive_with_context(&parent_key, &context, 32).unwrap();

        prop_assert_eq!(key1.as_slice(), key2.as_slice());
    }
}

// Property: Different contexts produce different keys
proptest! {
    #[test]
    fn prop_hkdf_context_separation(parent_key in prop::collection::vec(any::<u8>(), 32..33), session1 in "[a-z0-9]{10}", session2 in "[a-z0-9]{10}") {
        prop_assume!(session1 != session2);

        let ctx1 = DeriveContext::session("device-1", &session1);
        let ctx2 = DeriveContext::session("device-1", &session2);

        let key1 = KeyDerivation::derive_with_context(&parent_key, &ctx1, 32).unwrap();
        let key2 = KeyDerivation::derive_with_context(&parent_key, &ctx2, 32).unwrap();

        prop_assert_ne!(key1.as_slice(), key2.as_slice());
    }
}

// Property: PoP token verification succeeds with correct key
proptest! {
    #[test]
    fn prop_pop_token_verification(session_key in prop::collection::vec(any::<u8>(), 32..33)) {
        let generator = PopTokenGenerator::new();
        let claims = PopClaims::new("session-123", "device-456", 300);

        let token = generator.generate(&session_key, &claims).unwrap();
        let verified = generator.verify(&token, &session_key).unwrap();

        prop_assert_eq!(verified.session_id, claims.session_id);
        prop_assert_eq!(verified.device_id, claims.device_id);
    }
}

// Property: PoP token verification fails with wrong key
proptest! {
    #[test]
    fn prop_pop_token_wrong_key(
        session_key in prop::collection::vec(any::<u8>(), 32..33),
        wrong_key in prop::collection::vec(any::<u8>(), 32..33)
    ) {
        prop_assume!(session_key != wrong_key);

        let generator = PopTokenGenerator::new();
        let claims = PopClaims::new("session-123", "device-456", 300);

        let token = generator.generate(&session_key, &claims).unwrap();
        let result = generator.verify(&token, &wrong_key);

        prop_assert!(result.is_err());
    }
}

// Property: ChaCha20-Poly1305 tampered ciphertext fails decryption
proptest! {
    #[test]
    fn prop_aead_tamper_detection(
        plaintext in prop::collection::vec(any::<u8>(), 1..1024),
        tamper_index: usize,
        tamper_byte: u8
    ) {
        let cipher = ChaCha20Poly1305Cipher::new_random();
        let aad = b"test_aad";

        let (nonce, mut ciphertext) = cipher.encrypt(&plaintext, aad).unwrap();

        // Tamper with ciphertext
        if !ciphertext.is_empty() {
            let idx = tamper_index % ciphertext.len();
            ciphertext[idx] ^= tamper_byte.wrapping_add(1); // Ensure it's different
        }

        let result = cipher.decrypt(&nonce, &ciphertext, aad);

        prop_assert!(result.is_err());
    }
}

// Property: X25519 public key serialization roundtrip
proptest! {
    #[test]
    fn prop_x25519_serialization_roundtrip(_seed: u64) {
        let (_secret, public) = KeyAgreement::generate_keypair();

        let serialized = KeyAgreement::serialize_public_key(&public);
        let deserialized = KeyAgreement::deserialize_public_key(&serialized).unwrap();

        prop_assert_eq!(public.as_bytes(), deserialized.as_bytes());
    }
}

// Property: HKDF output length is respected
proptest! {
    #[test]
    fn prop_hkdf_output_length(
        parent_key in prop::collection::vec(any::<u8>(), 32..33),
        output_len in 16usize..128
    ) {
        let context = DeriveContext::session("device-1", "session-1");

        let key = KeyDerivation::derive_with_context(&parent_key, &context, output_len).unwrap();

        prop_assert_eq!(key.len(), output_len);
    }
}

// Property: PoP token compact encoding roundtrip
proptest! {
    #[test]
    fn prop_pop_token_compact_roundtrip(session_key in prop::collection::vec(any::<u8>(), 32..33)) {
        let generator = PopTokenGenerator::new();
        let claims = PopClaims::new("session-123", "device-456", 300);

        let token = generator.generate(&session_key, &claims).unwrap();
        let compact = token.to_compact().unwrap();
        let decoded = honeylink_crypto::pop_token::PopToken::from_compact(&compact).unwrap();

        prop_assert_eq!(decoded.claims.session_id, token.claims.session_id);
        prop_assert_eq!(decoded.claims.device_id, token.claims.device_id);
        prop_assert_eq!(decoded.claims.nonce, token.claims.nonce);
    }
}
