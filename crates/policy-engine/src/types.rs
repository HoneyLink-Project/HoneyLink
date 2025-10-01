//! Core type definitions for Policy & Profile Engine
//!
//! Implements the QoSPolicyUpdate schema and related types according to
//! spec/architecture/interfaces.md and spec/modules/policy-profile-engine.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use semver::Version;

/// FEC (Forward Error Correction) mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FecMode {
    /// No FEC applied
    None,
    /// Light FEC (5-10% overhead) for < 5% loss
    Light,
    /// Heavy FEC (20-30% overhead) for 5-15% loss
    Heavy,
}

impl Default for FecMode {
    fn default() -> Self {
        Self::None
    }
}

/// Stream priority level (0 = lowest, 7 = highest)
pub type Priority = u8;

/// Power consumption profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerProfile {
    /// Ultra low power (< 5mA average) for IoT
    UltraLow,
    /// Low power (5-20mA) for sensors
    Low,
    /// Normal power (20-100mA) for smartphones
    Normal,
    /// High power (> 100mA) for AR/VR/8K
    High,
}

impl Default for PowerProfile {
    fn default() -> Self {
        Self::Normal
    }
}

/// Use case classification for profile templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseCase {
    /// IoT sensor/actuator
    #[serde(rename = "IoT")]
    IoT,
    /// Augmented/Virtual Reality
    #[serde(rename = "AR_VR")]
    ArVr,
    /// 8K media streaming
    #[serde(rename = "Media8K")]
    Media8K,
    /// Low-latency gaming input
    Gaming,
    /// Custom user-defined profile
    Custom,
}

/// QoS Policy Update message sent to QoS Scheduler
///
/// This is the primary event type for policy distribution according to
/// spec/architecture/interfaces.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSPolicyUpdate {
    /// Schema version (SemVer) for compatibility checking
    pub schema_version: Version,

    /// Unique policy instance ID (prefix: pol_)
    pub policy_id: String,

    /// Profile template ID this policy is based on
    pub profile_id: String,

    /// Stream ID (0-7) to apply this policy to
    pub stream_id: u8,

    /// Target latency budget in milliseconds
    pub latency_budget_ms: u16,

    /// Minimum guaranteed bandwidth in Mbps
    pub bandwidth_floor_mbps: f64,

    /// Maximum bandwidth ceiling in Mbps (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_ceiling_mbps: Option<f64>,

    /// FEC mode selection
    pub fec_mode: FecMode,

    /// Stream priority (0-7)
    pub priority: Priority,

    /// Power consumption profile hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_profile: Option<PowerProfile>,

    /// Timestamp after which this policy version is deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_after: Option<DateTime<Utc>>,

    /// Policy expiration timestamp (TTL)
    pub expiration_ts: DateTime<Utc>,

    /// Ed25519 signature (base64-encoded) for integrity verification
    pub signature: String,
}

impl QoSPolicyUpdate {
    /// Validate policy update structure and constraints
    ///
    /// # Errors
    /// Returns `PolicyError::Validation` if:
    /// - policy_id doesn't start with "pol_"
    /// - profile_id doesn't start with "prof_"
    /// - stream_id > 7
    /// - priority > 7
    /// - latency_budget_ms == 0
    /// - bandwidth constraints are invalid
    pub fn validate(&self) -> crate::error::Result<()> {
        use crate::error::PolicyError;

        // Policy ID must have correct prefix
        if !self.policy_id.starts_with("pol_") {
            return Err(PolicyError::Validation(
                "policy_id must start with 'pol_'".to_string(),
            ));
        }

        // Profile ID must have correct prefix
        if !self.profile_id.starts_with("prof_") {
            return Err(PolicyError::Validation(
                "profile_id must start with 'prof_'".to_string(),
            ));
        }

        // Stream ID must be 0-7
        if self.stream_id > 7 {
            return Err(PolicyError::Validation(format!(
                "stream_id must be 0-7, got {}",
                self.stream_id
            )));
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

        // Bandwidth floor must be positive
        if self.bandwidth_floor_mbps <= 0.0 {
            return Err(PolicyError::Validation(
                "bandwidth_floor_mbps must be > 0".to_string(),
            ));
        }

        // If ceiling is set, it must be >= floor
        if let Some(ceiling) = self.bandwidth_ceiling_mbps {
            if ceiling < self.bandwidth_floor_mbps {
                return Err(PolicyError::Validation(
                    "bandwidth_ceiling_mbps must be >= bandwidth_floor_mbps".to_string(),
                ));
            }
        }

        // Check if policy is expired
        if self.expiration_ts < Utc::now() {
            return Err(PolicyError::Validation(
                "Policy has already expired".to_string(),
            ));
        }

        // Check if policy is deprecated
        if let Some(deprecated_after) = self.deprecated_after {
            if deprecated_after < Utc::now() {
                return Err(PolicyError::Deprecated(format!(
                    "Policy deprecated after {}",
                    deprecated_after
                )));
            }
        }

        Ok(())
    }

    /// Check if this policy is compatible with a given schema version
    ///
    /// Uses SemVer rules: major version must match, minor/patch can be newer
    pub fn is_compatible_with(&self, target_version: &Version) -> bool {
        self.schema_version.major == target_version.major
            && self.schema_version >= *target_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_valid_policy() -> QoSPolicyUpdate {
        QoSPolicyUpdate {
            schema_version: Version::new(1, 2, 0),
            policy_id: "pol_test123".to_string(),
            profile_id: "prof_iot_lowpower_v2".to_string(),
            stream_id: 3,
            latency_budget_ms: 50,
            bandwidth_floor_mbps: 0.5,
            bandwidth_ceiling_mbps: Some(2.0),
            fec_mode: FecMode::Light,
            priority: 2,
            power_profile: Some(PowerProfile::UltraLow),
            deprecated_after: Some(Utc::now() + Duration::days(365)),
            expiration_ts: Utc::now() + Duration::hours(12),
            signature: "ed25519:AAAA".to_string(),
        }
    }

    #[test]
    fn test_valid_policy() {
        let policy = create_valid_policy();
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_invalid_policy_id_prefix() {
        let mut policy = create_valid_policy();
        policy.policy_id = "invalid_id".to_string();
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_invalid_stream_id() {
        let mut policy = create_valid_policy();
        policy.stream_id = 8;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_invalid_priority() {
        let mut policy = create_valid_policy();
        policy.priority = 8;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_zero_latency_budget() {
        let mut policy = create_valid_policy();
        policy.latency_budget_ms = 0;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_invalid_bandwidth_ceiling() {
        let mut policy = create_valid_policy();
        policy.bandwidth_ceiling_mbps = Some(0.1); // Less than floor
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_version_compatibility() {
        let policy = create_valid_policy();

        // Same major, older minor - compatible
        assert!(policy.is_compatible_with(&Version::new(1, 0, 0)));

        // Same version - compatible
        assert!(policy.is_compatible_with(&Version::new(1, 2, 0)));

        // Different major - incompatible
        assert!(!policy.is_compatible_with(&Version::new(2, 0, 0)));

        // Newer minor required - incompatible
        assert!(!policy.is_compatible_with(&Version::new(1, 3, 0)));
    }

    #[test]
    fn test_fec_mode_serialization() {
        let json = serde_json::to_string(&FecMode::Light).unwrap();
        assert_eq!(json, r#""LIGHT""#);

        let mode: FecMode = serde_json::from_str(r#""HEAVY""#).unwrap();
        assert_eq!(mode, FecMode::Heavy);
    }
}
