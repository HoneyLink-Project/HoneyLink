//! Session entity model and metadata

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state_machine::SessionState;

/// Session entity with metadata
///
/// Represents a bidirectional communication session between two devices.
/// Includes TTL management, protocol version, and cryptographic key references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// UUIDv7 session identifier (monotonic, time-ordered)
    pub session_id: Uuid,

    /// Device A identifier
    pub device_a_id: String,

    /// Device B identifier
    pub device_b_id: String,

    /// Current session state
    pub state: SessionState,

    /// Negotiated protocol version (SemVer format)
    pub protocol_version: String,

    /// Reference to shared session key in KMS (NOT the key itself)
    pub shared_key_id: String,

    /// Session creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Session expiration timestamp (TTL enforcement: 12h default)
    pub expires_at: DateTime<Utc>,

    /// Last activity timestamp (for 30min sliding window)
    pub last_activity_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session in Pending state
    ///
    /// # Arguments
    /// * `device_a_id` - First device identifier
    /// * `device_b_id` - Second device identifier
    /// * `protocol_version` - Negotiated protocol version (SemVer)
    /// * `ttl_hours` - Session TTL in hours (default: 12)
    pub fn new(
        device_a_id: String,
        device_b_id: String,
        protocol_version: String,
        ttl_hours: i64,
    ) -> Self {
        let now = Utc::now();
        let session_id = Uuid::now_v7();
        let expires_at = now + Duration::hours(ttl_hours);

        Self {
            session_id,
            device_a_id,
            device_b_id,
            state: SessionState::Pending,
            protocol_version,
            shared_key_id: String::new(), // Set later by Crypto module
            created_at: now,
            updated_at: now,
            expires_at,
            last_activity_at: now,
        }
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session requires sliding window refresh (>30min since last activity)
    pub fn needs_activity_refresh(&self) -> bool {
        Utc::now() - self.last_activity_at > Duration::minutes(30)
    }

    /// Update last activity timestamp (sliding window)
    pub fn touch(&mut self) {
        self.last_activity_at = Utc::now();
        self.updated_at = Utc::now();
    }

    /// Update session state
    pub fn set_state(&mut self, new_state: SessionState) {
        self.state = new_state;
        self.updated_at = Utc::now();
    }

    /// Set shared key ID after Crypto module generates session key
    pub fn set_shared_key_id(&mut self, key_id: String) {
        self.shared_key_id = key_id;
        self.updated_at = Utc::now();
    }

    /// Get session age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get time until expiration in seconds
    pub fn ttl_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );

        assert_eq!(session.state, SessionState::Pending);
        assert_eq!(session.device_a_id, "DEV-A");
        assert_eq!(session.device_b_id, "DEV-B");
        assert!(!session.is_expired());
        assert!(!session.needs_activity_refresh());
    }

    #[test]
    fn test_session_expiration() {
        let mut session = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );

        // Set expiration to past
        session.expires_at = Utc::now() - Duration::hours(1);

        assert!(session.is_expired());
    }

    #[test]
    fn test_activity_refresh() {
        let mut session = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );

        // Set last activity to 31 minutes ago
        session.last_activity_at = Utc::now() - Duration::minutes(31);

        assert!(session.needs_activity_refresh());

        session.touch();
        assert!(!session.needs_activity_refresh());
    }

    #[test]
    fn test_uuidv7_monotonicity() {
        let session1 = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );

        std::thread::sleep(std::time::Duration::from_millis(10));

        let session2 = Session::new(
            "DEV-C".to_string(),
            "DEV-D".to_string(),
            "1.0.0".to_string(),
            12,
        );

        // UUIDv7 should be monotonically increasing
        assert!(session1.session_id < session2.session_id);
    }
}
