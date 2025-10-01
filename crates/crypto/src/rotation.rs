// Key rotation and versioning for hierarchical key management
//
// Supports versioned key storage with grace periods for rotation.
// Each key can have multiple versions with overlapping validity periods.

use crate::{KeyHierarchy, KeyScope};
use honeylink_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key version identifier (monotonically increasing)
pub type KeyVersion = u32;

/// Versioned key material with metadata
#[derive(Clone, Serialize, Deserialize)]
pub struct VersionedKey {
    /// Key version number (starts at 1)
    pub version: KeyVersion,

    /// 32-byte key material
    #[serde(with = "serde_bytes")]
    pub key_material: [u8; 32],

    /// Unix timestamp when this key was created
    pub created_at: i64,

    /// Unix timestamp when this key becomes active (can be future)
    pub active_from: i64,

    /// Unix timestamp when this key should be deprecated (grace period starts)
    pub deprecated_after: i64,

    /// Unix timestamp when this key is permanently invalid
    pub expires_at: i64,

    /// Scope this key belongs to
    pub scope: KeyScope,
}

impl VersionedKey {
    /// Check if this key is currently active (within active_from..deprecated_after)
    pub fn is_active(&self, now: i64) -> bool {
        now >= self.active_from && now < self.deprecated_after
    }

    /// Check if this key is in grace period (deprecated but not expired)
    pub fn is_in_grace_period(&self, now: i64) -> bool {
        now >= self.deprecated_after && now < self.expires_at
    }

    /// Check if this key can be used (active or in grace period)
    pub fn is_usable(&self, now: i64) -> bool {
        self.is_active(now) || self.is_in_grace_period(now)
    }

    /// Check if this key is expired
    pub fn is_expired(&self, now: i64) -> bool {
        now >= self.expires_at
    }
}

/// Key rotation manager for hierarchical keys
///
/// Manages multiple versions of keys with automatic rotation and grace periods.
/// Supports:
/// - Scheduled rotation (e.g., every 90 days)
/// - Event-triggered rotation (security incident)
/// - Grace period for key transition (old key still usable)
/// - Automatic expiration cleanup
#[derive(Serialize, Deserialize)]
pub struct KeyRotationManager {
    /// All key versions by scope
    keys: HashMap<KeyScope, Vec<VersionedKey>>,

    /// Rotation policy per scope
    policies: HashMap<KeyScope, RotationPolicy>,
}

/// Rotation policy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// How long a key remains active before deprecation (seconds)
    pub active_duration: i64,

    /// Grace period after deprecation before expiration (seconds)
    pub grace_period: i64,

    /// Scheduled rotation interval (seconds), None for manual rotation only
    pub rotation_interval: Option<i64>,
}

impl RotationPolicy {
    /// Default policy for Root keys (5 years active, 30 days grace)
    pub fn root_default() -> Self {
        Self {
            active_duration: 5 * 365 * 24 * 3600,
            grace_period: 30 * 24 * 3600,
            rotation_interval: None, // Manual only
        }
    }

    /// Default policy for Device keys (90 days active, 7 days grace)
    pub fn device_default() -> Self {
        Self {
            active_duration: 90 * 24 * 3600,
            grace_period: 7 * 24 * 3600,
            rotation_interval: Some(90 * 24 * 3600),
        }
    }

    /// Default policy for Session keys (24 hours active, 1 hour grace)
    pub fn session_default() -> Self {
        Self {
            active_duration: 24 * 3600,
            grace_period: 3600,
            rotation_interval: Some(24 * 3600),
        }
    }

    /// Default policy for Stream keys (connection lifetime, no grace)
    pub fn stream_default() -> Self {
        Self {
            active_duration: 3600, // 1 hour max
            grace_period: 0,
            rotation_interval: None, // Per-connection
        }
    }
}

impl KeyRotationManager {
    /// Create a new rotation manager with default policies
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        policies.insert(KeyScope::Root, RotationPolicy::root_default());
        policies.insert(KeyScope::DeviceMaster, RotationPolicy::device_default());
        policies.insert(KeyScope::Session, RotationPolicy::session_default());
        policies.insert(KeyScope::Stream, RotationPolicy::stream_default());

        Self {
            keys: HashMap::new(),
            policies,
        }
    }

    /// Set custom rotation policy for a scope
    pub fn set_policy(&mut self, scope: KeyScope, policy: RotationPolicy) {
        self.policies.insert(scope, policy);
    }

    /// Add a new key version for a scope
    ///
    /// # Arguments
    /// * `scope` - Key scope (Root, DeviceMaster, Session, Stream)
    /// * `key_material` - 32-byte key material
    /// * `now` - Current timestamp
    ///
    /// # Returns
    /// Version number of the newly created key
    pub fn add_key_version(
        &mut self,
        scope: KeyScope,
        key_material: [u8; 32],
        now: i64,
    ) -> Result<KeyVersion> {
        let policy = self.policies.get(&scope)
            .ok_or_else(|| honeylink_core::Error::Configuration("No policy for scope".into()))?;

        let versions = self.keys.entry(scope).or_default();
        let version = versions.len() as u32 + 1;

        let key = VersionedKey {
            version,
            key_material,
            created_at: now,
            active_from: now,
            deprecated_after: now + policy.active_duration,
            expires_at: now + policy.active_duration + policy.grace_period,
            scope,
        };

        versions.push(key);
        Ok(version)
    }

    /// Get the currently active key for a scope
    pub fn get_active_key(&self, scope: KeyScope, now: i64) -> Option<&VersionedKey> {
        self.keys.get(&scope)?
            .iter()
            .filter(|k| k.is_active(now))
            .max_by_key(|k| k.version)
    }

    /// Get all usable keys for a scope (active + grace period)
    pub fn get_usable_keys(&self, scope: KeyScope, now: i64) -> Vec<&VersionedKey> {
        self.keys.get(&scope)
            .map(|versions| {
                versions.iter()
                    .filter(|k| k.is_usable(now))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if rotation is needed for a scope
    pub fn needs_rotation(&self, scope: KeyScope, now: i64) -> bool {
        let Some(policy) = self.policies.get(&scope) else {
            return false;
        };

        let Some(interval) = policy.rotation_interval else {
            return false; // Manual rotation only
        };

        // Check if we have an active key
        let Some(active_key) = self.get_active_key(scope, now) else {
            return true; // No active key, need rotation
        };

        // Check if we're past the rotation interval
        now - active_key.created_at >= interval
    }

    /// Perform rotation for a scope using key derivation
    pub fn rotate(&mut self, hierarchy: &KeyHierarchy, scope: KeyScope, now: i64) -> Result<KeyVersion> {
        // Derive new key material from hierarchy
        let new_key = hierarchy.derive_simple(scope)?;

        // Add as new version
        self.add_key_version(scope, new_key, now)
    }

    /// Clean up expired keys
    pub fn cleanup_expired(&mut self, now: i64) -> usize {
        let mut removed = 0;

        for versions in self.keys.values_mut() {
            let original_len = versions.len();
            versions.retain(|k| !k.is_expired(now));
            removed += original_len - versions.len();
        }

        removed
    }

    /// Get rotation status summary for all scopes
    pub fn get_status(&self, now: i64) -> HashMap<KeyScope, RotationStatus> {
        let mut status = HashMap::new();

        for scope in [KeyScope::Root, KeyScope::DeviceMaster, KeyScope::Session, KeyScope::Stream] {
            let active_key = self.get_active_key(scope, now);
            let usable_count = self.get_usable_keys(scope, now).len();
            let needs_rotation = self.needs_rotation(scope, now);

            status.insert(scope, RotationStatus {
                active_version: active_key.map(|k| k.version),
                usable_versions: usable_count,
                needs_rotation,
                next_rotation: active_key.and_then(|k| {
                    self.policies.get(&scope)
                        .and_then(|p| p.rotation_interval)
                        .map(|interval| k.created_at + interval)
                }),
            });
        }

        status
    }
}

impl Default for KeyRotationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Rotation status for a key scope
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RotationStatus {
    /// Currently active key version (None if no active key)
    pub active_version: Option<KeyVersion>,

    /// Number of usable key versions (active + grace period)
    pub usable_versions: usize,

    /// Whether rotation is needed now
    pub needs_rotation: bool,

    /// Unix timestamp of next scheduled rotation (None for manual)
    pub next_rotation: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_key_lifecycle() {
        let now = 1000000;
        let key = VersionedKey {
            version: 1,
            key_material: [0u8; 32],
            created_at: now,
            active_from: now,
            deprecated_after: now + 100,
            expires_at: now + 200,
            scope: KeyScope::Session,
        };

        assert!(key.is_active(now));
        assert!(key.is_active(now + 50));
        assert!(!key.is_active(now + 100));

        assert!(!key.is_in_grace_period(now));
        assert!(key.is_in_grace_period(now + 150));
        assert!(!key.is_in_grace_period(now + 200));

        assert!(key.is_usable(now + 50));
        assert!(key.is_usable(now + 150));
        assert!(!key.is_usable(now + 200));

        assert!(!key.is_expired(now + 150));
        assert!(key.is_expired(now + 200));
    }

    #[test]
    fn test_rotation_manager_basic() {
        let mut manager = KeyRotationManager::new();
        let now = 1000000;

        // Add first version
        let v1 = manager.add_key_version(
            KeyScope::Session,
            [1u8; 32],
            now,
        ).unwrap();
        assert_eq!(v1, 1);

        // Active key should be v1
        let active = manager.get_active_key(KeyScope::Session, now).unwrap();
        assert_eq!(active.version, 1);

        // Add second version (simulating rotation)
        let v2 = manager.add_key_version(
            KeyScope::Session,
            [2u8; 32],
            now + 1000,
        ).unwrap();
        assert_eq!(v2, 2);

        // Active key should be v2
        let active = manager.get_active_key(KeyScope::Session, now + 1000).unwrap();
        assert_eq!(active.version, 2);

        // Both should be usable during grace period
        let usable = manager.get_usable_keys(KeyScope::Session, now + 1000);
        assert_eq!(usable.len(), 2);
    }

    #[test]
    fn test_rotation_policy() {
        let policy = RotationPolicy::session_default();
        assert_eq!(policy.active_duration, 24 * 3600);
        assert_eq!(policy.grace_period, 3600);
        assert_eq!(policy.rotation_interval, Some(24 * 3600));
    }
}
