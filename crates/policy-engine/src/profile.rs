//! Profile management and preset templates
//!
//! Implements profile CRUD operations, validation, and Ed25519 signature
//! verification according to spec/modules/policy-profile-engine.md

use crate::error::{PolicyError, Result};
use crate::types::{FecMode, PowerProfile, Priority, UseCase};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use semver::Version;
use std::collections::HashMap;

/// Policy Profile template definition
///
/// Profiles define reusable QoS configurations for specific use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyProfile {
    /// Unique profile identifier (prefix: prof_)
    pub profile_id: String,

    /// Human-readable profile name
    pub profile_name: String,

    /// SemVer version of this profile
    pub profile_version: Version,

    /// Use case classification
    pub use_case: UseCase,

    /// Target latency budget in milliseconds
    pub latency_budget_ms: u16,

    /// Minimum bandwidth floor in Mbps
    pub bandwidth_floor_mbps: f64,

    /// Maximum bandwidth ceiling in Mbps
    pub bandwidth_ceiling_mbps: f64,

    /// FEC mode
    pub fec_mode: FecMode,

    /// Default priority (0-7)
    pub priority: Priority,

    /// Power consumption profile
    pub power_profile: PowerProfile,

    /// Timestamp after which this profile version is deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_after: Option<DateTime<Utc>>,

    /// Ed25519 signature for integrity verification (base64-encoded)
    pub signature: String,

    /// Profile creation timestamp
    pub created_at: DateTime<Utc>,

    /// Profile last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Optional custom metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl PolicyProfile {
    /// Validate profile structure and constraints
    pub fn validate(&self) -> Result<()> {
        // Profile ID must have correct prefix
        if !self.profile_id.starts_with("prof_") {
            return Err(PolicyError::Validation(
                "profile_id must start with 'prof_'".to_string(),
            ));
        }

        // Priority must be 0-7
        if self.priority > 7 {
            return Err(PolicyError::Validation(format!(
                "priority must be 0-7, got {}",
                self.priority
            )));
        }

        // Latency budget must be positive
        if self.latency_budget_ms == 0 {
            return Err(PolicyError::Validation(
                "latency_budget_ms must be > 0".to_string(),
            ));
        }

        // Bandwidth constraints validation
        if self.bandwidth_floor_mbps <= 0.0 {
            return Err(PolicyError::Validation(
                "bandwidth_floor_mbps must be > 0".to_string(),
            ));
        }

        if self.bandwidth_ceiling_mbps < self.bandwidth_floor_mbps {
            return Err(PolicyError::Validation(
                "bandwidth_ceiling_mbps must be >= bandwidth_floor_mbps".to_string(),
            ));
        }

        // Check deprecation
        if let Some(deprecated_after) = self.deprecated_after {
            if deprecated_after < Utc::now() {
                return Err(PolicyError::Deprecated(format!(
                    "Profile deprecated after {}",
                    deprecated_after
                )));
            }
        }

        Ok(())
    }

    /// Verify Ed25519 signature using provided public key
    ///
    /// # Arguments
    /// * `public_key_bytes` - 32-byte Ed25519 public key
    ///
    /// # Returns
    /// Ok(()) if signature is valid, Err otherwise
    pub fn verify_signature(&self, public_key_bytes: &[u8; 32]) -> Result<()> {
        // Decode base64 signature
        use base64::Engine;
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.signature)
            .map_err(|e| {
                PolicyError::SignatureInvalid(format!("Base64 decode failed: {}", e))
            })?;

        if signature_bytes.len() != 64 {
            return Err(PolicyError::SignatureInvalid(
                "Signature must be 64 bytes".to_string(),
            ));
        }

        let signature = Signature::from_bytes(
            signature_bytes
                .as_slice()
                .try_into()
                .map_err(|_| PolicyError::SignatureInvalid("Invalid signature bytes".to_string()))?,
        );

        let public_key = VerifyingKey::from_bytes(public_key_bytes)
            .map_err(|e| PolicyError::SignatureInvalid(format!("Invalid public key: {}", e)))?;

        // Create canonical message to verify
        let message = self.create_canonical_message();

        public_key
            .verify(message.as_bytes(), &signature)
            .map_err(|e| PolicyError::SignatureInvalid(format!("Signature verification failed: {}", e)))
    }

    /// Create canonical message for signing/verification
    ///
    /// This creates a deterministic string representation of the profile
    /// for signature generation and verification
    fn create_canonical_message(&self) -> String {
        format!(
            "{}|{}|{}|{:?}|{}|{}|{}|{:?}|{}|{:?}",
            self.profile_id,
            self.profile_name,
            self.profile_version,
            self.use_case,
            self.latency_budget_ms,
            self.bandwidth_floor_mbps,
            self.bandwidth_ceiling_mbps,
            self.fec_mode,
            self.priority,
            self.power_profile
        )
    }
}

/// Profile storage trait for abstracting persistence layer
///
/// Implementations can use CockroachDB, in-memory storage, or other backends
pub trait ProfileStorage: Send + Sync {
    /// Create a new profile
    fn create(&mut self, profile: PolicyProfile) -> Result<()>;

    /// Retrieve a profile by ID
    fn get(&self, profile_id: &str) -> Result<PolicyProfile>;

    /// Update an existing profile
    fn update(&mut self, profile: PolicyProfile) -> Result<()>;

    /// Delete a profile (soft delete by setting deprecated_after)
    fn delete(&mut self, profile_id: &str) -> Result<()>;

    /// List all profiles, optionally filtered by use case
    fn list(&self, use_case: Option<UseCase>) -> Result<Vec<PolicyProfile>>;
}

/// In-memory profile storage for testing and development
#[derive(Default)]
pub struct InMemoryProfileStorage {
    profiles: HashMap<String, PolicyProfile>,
}

impl InMemoryProfileStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProfileStorage for InMemoryProfileStorage {
    fn create(&mut self, profile: PolicyProfile) -> Result<()> {
        profile.validate()?;

        if self.profiles.contains_key(&profile.profile_id) {
            return Err(PolicyError::Conflict(format!(
                "Profile {} already exists",
                profile.profile_id
            )));
        }

        self.profiles.insert(profile.profile_id.clone(), profile);
        Ok(())
    }

    fn get(&self, profile_id: &str) -> Result<PolicyProfile> {
        self.profiles
            .get(profile_id)
            .cloned()
            .ok_or_else(|| PolicyError::NotFound(format!("Profile {} not found", profile_id)))
    }

    fn update(&mut self, profile: PolicyProfile) -> Result<()> {
        profile.validate()?;

        if !self.profiles.contains_key(&profile.profile_id) {
            return Err(PolicyError::NotFound(format!(
                "Profile {} not found",
                profile.profile_id
            )));
        }

        self.profiles.insert(profile.profile_id.clone(), profile);
        Ok(())
    }

    fn delete(&mut self, profile_id: &str) -> Result<()> {
        let mut profile = self.get(profile_id)?;
        profile.deprecated_after = Some(Utc::now());
        self.profiles.insert(profile_id.to_string(), profile);
        Ok(())
    }

    fn list(&self, use_case: Option<UseCase>) -> Result<Vec<PolicyProfile>> {
        let profiles: Vec<PolicyProfile> = self
            .profiles
            .values()
            .filter(|p| use_case.map_or(true, |uc| p.use_case == uc))
            .cloned()
            .collect();
        Ok(profiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile(id: &str, use_case: UseCase) -> PolicyProfile {
        PolicyProfile {
            profile_id: format!("prof_{}", id),
            profile_name: format!("Test Profile {}", id),
            profile_version: Version::new(1, 0, 0),
            use_case,
            latency_budget_ms: 50,
            bandwidth_floor_mbps: 1.0,
            bandwidth_ceiling_mbps: 10.0,
            fec_mode: FecMode::Light,
            priority: 5,
            power_profile: PowerProfile::Normal,
            deprecated_after: None,
            signature: {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(&[0u8; 64])
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_profile_validation() {
        let profile = create_test_profile("test", UseCase::IoT);
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_invalid_profile_id() {
        let mut profile = create_test_profile("test", UseCase::IoT);
        profile.profile_id = "invalid_id".to_string();
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_invalid_priority() {
        let mut profile = create_test_profile("test", UseCase::IoT);
        profile.priority = 8;
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_storage_crud() {
        let mut storage = InMemoryProfileStorage::new();
        let profile = create_test_profile("test1", UseCase::IoT);

        // Create
        assert!(storage.create(profile.clone()).is_ok());

        // Duplicate create should fail
        assert!(storage.create(profile.clone()).is_err());

        // Read
        let retrieved = storage.get(&profile.profile_id).unwrap();
        assert_eq!(retrieved.profile_id, profile.profile_id);

        // Update
        let mut updated = profile.clone();
        updated.latency_budget_ms = 100;
        assert!(storage.update(updated.clone()).is_ok());

        let retrieved = storage.get(&profile.profile_id).unwrap();
        assert_eq!(retrieved.latency_budget_ms, 100);

        // Delete (soft)
        assert!(storage.delete(&profile.profile_id).is_ok());
        let retrieved = storage.get(&profile.profile_id).unwrap();
        assert!(retrieved.deprecated_after.is_some());

        // List
        storage.create(create_test_profile("test2", UseCase::Gaming)).unwrap();
        storage.create(create_test_profile("test3", UseCase::IoT)).unwrap();

        let all_profiles = storage.list(None).unwrap();
        assert_eq!(all_profiles.len(), 3);

        let iot_profiles = storage.list(Some(UseCase::IoT)).unwrap();
        assert_eq!(iot_profiles.len(), 2);
    }

    #[test]
    fn test_canonical_message() {
        let profile = create_test_profile("test", UseCase::IoT);
        let msg = profile.create_canonical_message();
        assert!(msg.contains("prof_test"));
        assert!(msg.contains("IoT"));
    }
}
