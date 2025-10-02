// Common types for Control Plane API

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Device identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

impl DeviceId {
    /// Validates device ID format: /^[A-Z0-9-]{4,64}$/
    pub fn validate(&self) -> Result<(), String> {
        let s = &self.0;
        if s.len() < 4 || s.len() > 64 {
            return Err("Device ID must be between 4 and 64 characters".to_string());
        }
        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err("Device ID must contain only A-Z, 0-9, and hyphens".to_string());
        }
        if !s.chars().next().unwrap().is_ascii_alphabetic() {
            return Err("Device ID must start with a letter".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session identifier (UUIDv7)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionId(pub Uuid);

impl SessionId {
    /// Generates a new UUIDv7-based session ID
    pub fn new() -> Self {
        SessionId(Uuid::now_v7())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sess_{}", &self.0.to_string()[0..8])
    }
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (typically device_id or user_id)
    pub sub: String,
    /// Expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// Not before timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Scopes/permissions
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl JwtClaims {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= self.exp
    }

    /// Check if token is valid (not before)
    pub fn is_valid_now(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        if let Some(nbf) = self.nbf {
            now >= nbf && now < self.exp
        } else {
            now < self.exp
        }
    }

    /// Check if token has required scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// W3C Trace Context
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub version: String,
    pub trace_id: String,
    pub parent_id: String,
    pub trace_flags: String,
}

impl TraceContext {
    /// Parse from traceparent header
    /// Format: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
    pub fn from_header(header: &str) -> Result<Self, String> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return Err("Invalid traceparent format".to_string());
        }

        Ok(TraceContext {
            version: parts[0].to_string(),
            trace_id: parts[1].to_string(),
            parent_id: parts[2].to_string(),
            trace_flags: parts[3].to_string(),
        })
    }

    /// Format as traceparent header
    pub fn to_header(&self) -> String {
        format!("{}-{}-{}-{}", self.version, self.trace_id, self.parent_id, self.trace_flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_validation() {
        // Valid IDs
        assert!(DeviceId("HL-EDGE-0001".to_string()).validate().is_ok());
        assert!(DeviceId("DEVICE123".to_string()).validate().is_ok());
        assert!(DeviceId("A-1".to_string()).validate().is_ok());

        // Invalid IDs
        assert!(DeviceId("abc".to_string()).validate().is_err()); // Too short
        assert!(DeviceId("1234".to_string()).validate().is_err()); // Starts with number
        assert!(DeviceId("dev@ice".to_string()).validate().is_err()); // Invalid char
        assert!(DeviceId("a".repeat(65)).validate().is_err()); // Too long
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);

        let display = format!("{}", id1);
        assert!(display.starts_with("sess_"));
    }

    #[test]
    fn test_jwt_claims_expiration() {
        let now = chrono::Utc::now().timestamp();

        let expired = JwtClaims {
            sub: "device1".to_string(),
            exp: now - 100,
            iat: now - 400,
            nbf: None,
            iss: "honeylink".to_string(),
            aud: "api".to_string(),
            scopes: vec![],
        };
        assert!(expired.is_expired());

        let valid = JwtClaims {
            sub: "device1".to_string(),
            exp: now + 300,
            iat: now,
            nbf: None,
            iss: "honeylink".to_string(),
            aud: "api".to_string(),
            scopes: vec!["read".to_string()],
        };
        assert!(!valid.is_expired());
        assert!(valid.is_valid_now());
        assert!(valid.has_scope("read"));
        assert!(!valid.has_scope("write"));
    }

    #[test]
    fn test_trace_context_parsing() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let ctx = TraceContext::from_header(header).unwrap();

        assert_eq!(ctx.version, "00");
        assert_eq!(ctx.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(ctx.parent_id, "00f067aa0ba902b7");
        assert_eq!(ctx.trace_flags, "01");

        assert_eq!(ctx.to_header(), header);
    }

    #[test]
    fn test_trace_context_invalid() {
        assert!(TraceContext::from_header("invalid").is_err());
        assert!(TraceContext::from_header("00-abc").is_err());
    }
}
