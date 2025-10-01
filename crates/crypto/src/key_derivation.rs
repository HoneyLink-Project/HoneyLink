//! Key derivation using HKDF-SHA512

use hkdf::Hkdf;
use honeylink_core::Result;
use sha2::Sha512;
use zeroize::Zeroizing;

pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a key using HKDF-SHA512
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
}
