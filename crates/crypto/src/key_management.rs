//! Key hierarchy management

use honeylink_core::Result;
use zeroize::Zeroizing;

/// Key scopes in the hierarchy
///
/// Follows spec/security/key-management.md:
/// - Root: Trust anchor (5 year lifetime)
/// - DeviceMaster: Device identification (90 day rotation)
/// - Session: Per-session encryption (24 hour lifetime)
/// - Stream: Per-stream keys (connection lifetime)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KeyScope {
    Root,
    DeviceMaster,
    Session,
    Stream,
}

/// Hierarchical key management
pub struct KeyHierarchy {
    root_key: Zeroizing<Vec<u8>>,
}

impl KeyHierarchy {
    /// Create a new key hierarchy with a root key (`Vec<u8>`)
    pub fn new(root_key: Vec<u8>) -> Self {
        Self {
            root_key: Zeroizing::new(root_key),
        }
    }

    /// Create a new key hierarchy with a fixed-size root key
    pub fn from_bytes(root_key: [u8; 32]) -> Self {
        Self {
            root_key: Zeroizing::new(root_key.to_vec()),
        }
    }

    /// Derive a key for a specific scope with optional context
    pub fn derive(&self, scope: KeyScope, context: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        let info = format!("honeylink:{}:{}", scope_to_string(scope), hex::encode(context));
        crate::key_derivation::KeyDerivation::derive(&self.root_key, None, info.as_bytes(), 32)
    }

    /// Derive a key for a specific scope without context (returns fixed 32 bytes)
    pub fn derive_simple(&self, scope: KeyScope) -> Result<[u8; 32]> {
        let derived = self.derive(scope, b"")?;
        let mut output = [0u8; 32];
        output.copy_from_slice(&derived[..32]);
        Ok(output)
    }
}

fn scope_to_string(scope: KeyScope) -> &'static str {
    match scope {
        KeyScope::Root => "root",
        KeyScope::DeviceMaster => "device-master",
        KeyScope::Session => "session",
        KeyScope::Stream => "stream",
    }
}

// Temporary hex encoding until we add hex dependency to this crate
mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_hierarchy() {
        let root = vec![0u8; 32];
        let hierarchy = KeyHierarchy::new(root);
        let result = hierarchy.derive(KeyScope::Session, b"test_session");
        assert!(result.is_ok());
    }
}
