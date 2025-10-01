//! Protocol version negotiation using SemVer
//!
//! Implements version compatibility checking per spec/modules/session-orchestrator.md
//! Supports version range: 1.0.0 ~ 2.9.99

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Supported protocol version range
const MIN_VERSION: &str = "1.0.0";
const MAX_VERSION: &str = "2.9.99";

/// Protocol version negotiation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiatedVersion {
    /// Client requested version
    pub client_version: String,

    /// Server supported version
    pub server_version: String,

    /// Negotiated version (highest compatible)
    pub negotiated_version: String,

    /// Whether fallback was required
    pub is_fallback: bool,
}

/// Protocol version negotiator
pub struct VersionNegotiator {
    /// Server's supported version range
    supported_range: VersionReq,

    /// Server's preferred version
    preferred_version: Version,
}

impl VersionNegotiator {
    /// Create negotiator with server's supported version range
    ///
    /// # Arguments
    /// * `preferred_version` - Server's preferred protocol version (e.g., "1.2.3")
    ///
    /// # Errors
    /// Returns error if version is outside supported range
    pub fn new(preferred_version: &str) -> Result<Self> {
        let version = Version::parse(preferred_version)
            .map_err(|e| Error::UnsupportedVersion(format!("Invalid version: {}", e)))?;

        // Check if within supported range
        let min = Version::parse(MIN_VERSION).unwrap();
        let max = Version::parse(MAX_VERSION).unwrap();

        if version < min || version > max {
            return Err(Error::UnsupportedVersion(format!(
                "Version {} outside supported range {}-{}",
                preferred_version, MIN_VERSION, MAX_VERSION
            )));
        }

        // Create version requirement: >=1.0.0, <3.0.0
        let supported_range = VersionReq::parse(">=1.0.0, <3.0.0")
            .expect("Valid version requirement");

        Ok(Self {
            supported_range,
            preferred_version: version,
        })
    }

    /// Negotiate version with client
    ///
    /// # Returns
    /// - `Ok(NegotiatedVersion)` if compatible version found
    /// - `Err(Error::VersionNegotiationFailed)` if incompatible
    pub fn negotiate(&self, client_version: &str) -> Result<NegotiatedVersion> {
        let client_ver = Version::parse(client_version).map_err(|e| {
            Error::UnsupportedVersion(format!("Invalid client version: {}", e))
        })?;

        // Check if client version is within supported range
        if !self.supported_range.matches(&client_ver) {
            return Err(Error::VersionNegotiationFailed {
                client: client_version.to_string(),
                server: self.preferred_version.to_string(),
            });
        }

        // Negotiate: use lower of (client, server) for major.minor
        // This ensures backward compatibility
        let (negotiated, is_fallback) = if client_ver >= self.preferred_version {
            // Client is newer or equal: use server version
            (self.preferred_version.clone(), false)
        } else {
            // Client is older: use client version (fallback)
            (client_ver.clone(), true)
        };

        Ok(NegotiatedVersion {
            client_version: client_version.to_string(),
            server_version: self.preferred_version.to_string(),
            negotiated_version: negotiated.to_string(),
            is_fallback,
        })
    }

    /// Check if version is supported
    pub fn is_supported(&self, version: &str) -> bool {
        Version::parse(version)
            .map(|v| self.supported_range.matches(&v))
            .unwrap_or(false)
    }

    /// Get server's preferred version
    pub fn preferred_version(&self) -> &Version {
        &self.preferred_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiator_creation() {
        let negotiator = VersionNegotiator::new("1.2.3").unwrap();
        assert_eq!(negotiator.preferred_version().to_string(), "1.2.3");
    }

    #[test]
    fn test_negotiator_invalid_version() {
        // Version outside supported range
        let result = VersionNegotiator::new("3.0.0");
        assert!(result.is_err());

        // Version below minimum
        let result = VersionNegotiator::new("0.9.9");
        assert!(result.is_err());
    }

    #[test]
    fn test_negotiate_exact_match() {
        let negotiator = VersionNegotiator::new("1.2.3").unwrap();
        let result = negotiator.negotiate("1.2.3").unwrap();

        assert_eq!(result.client_version, "1.2.3");
        assert_eq!(result.server_version, "1.2.3");
        assert_eq!(result.negotiated_version, "1.2.3");
        assert!(!result.is_fallback);
    }

    #[test]
    fn test_negotiate_client_newer() {
        let negotiator = VersionNegotiator::new("1.2.0").unwrap();
        let result = negotiator.negotiate("1.5.0").unwrap();

        // Should use server version (lower)
        assert_eq!(result.negotiated_version, "1.2.0");
        assert!(!result.is_fallback);
    }

    #[test]
    fn test_negotiate_client_older() {
        let negotiator = VersionNegotiator::new("1.5.0").unwrap();
        let result = negotiator.negotiate("1.2.0").unwrap();

        // Should use client version (fallback)
        assert_eq!(result.negotiated_version, "1.2.0");
        assert!(result.is_fallback);
    }

    #[test]
    fn test_negotiate_major_version_mismatch() {
        let negotiator = VersionNegotiator::new("1.5.0").unwrap();
        let result = negotiator.negotiate("3.0.0");

        // Major version 3 is outside supported range
        assert!(result.is_err());
    }

    #[test]
    fn test_negotiate_invalid_client_version() {
        let negotiator = VersionNegotiator::new("1.2.3").unwrap();
        let result = negotiator.negotiate("invalid");

        assert!(result.is_err());
    }

    #[test]
    fn test_is_supported() {
        let negotiator = VersionNegotiator::new("1.5.0").unwrap();

        assert!(negotiator.is_supported("1.0.0"));
        assert!(negotiator.is_supported("1.5.0"));
        assert!(negotiator.is_supported("2.9.99"));
        assert!(!negotiator.is_supported("3.0.0"));
        assert!(!negotiator.is_supported("0.9.0"));
        assert!(!negotiator.is_supported("invalid"));
    }

    #[test]
    fn test_backward_compatibility() {
        let negotiator = VersionNegotiator::new("2.0.0").unwrap();

        // All 1.x versions should be compatible
        for minor in 0..10 {
            for patch in 0..10 {
                let version = format!("1.{}.{}", minor, patch);
                let result = negotiator.negotiate(&version);
                assert!(result.is_ok(), "Version {} should be compatible", version);
            }
        }
    }
}
