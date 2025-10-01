//! Event bus for session lifecycle events
//!
//! Publishes events to subscribers (Crypto, Policy Engine, Telemetry)
//! using tokio mpsc channels per spec/modules/session-orchestrator.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::state_machine::SessionState;

/// Maximum event channel capacity (backpressure threshold)
const EVENT_CHANNEL_CAPACITY: usize = 1024;

/// Session lifecycle event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum SessionEvent {
    /// Session established (Pending → Paired → Active)
    SessionEstablished {
        session_id: Uuid,
        device_a_id: String,
        device_b_id: String,
        negotiated_version: String,
        shared_key_id: String,
        ttl_seconds: i64,
        created_at: DateTime<Utc>,
        trace_id: String,
    },

    /// Session state changed
    SessionStateChanged {
        session_id: Uuid,
        from_state: SessionState,
        to_state: SessionState,
        timestamp: DateTime<Utc>,
        trace_id: String,
    },

    /// Session activity (for TTL sliding window)
    SessionActivity {
        session_id: Uuid,
        activity_type: String,
        timestamp: DateTime<Utc>,
    },

    /// Session closed
    SessionClosed {
        session_id: Uuid,
        reason: String,
        timestamp: DateTime<Utc>,
        trace_id: String,
    },

    /// Session error
    SessionError {
        session_id: Uuid,
        error_type: String,
        error_message: String,
        timestamp: DateTime<Utc>,
        trace_id: String,
    },
}

impl SessionEvent {
    /// Get session ID from event
    pub fn session_id(&self) -> Uuid {
        match self {
            Self::SessionEstablished { session_id, .. }
            | Self::SessionStateChanged { session_id, .. }
            | Self::SessionActivity { session_id, .. }
            | Self::SessionClosed { session_id, .. }
            | Self::SessionError { session_id, .. } => *session_id,
        }
    }

    /// Get event timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::SessionEstablished { created_at, .. } => *created_at,
            Self::SessionStateChanged { timestamp, .. }
            | Self::SessionActivity { timestamp, .. }
            | Self::SessionClosed { timestamp, .. }
            | Self::SessionError { timestamp, .. } => *timestamp,
        }
    }

    /// Get event type as string
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::SessionEstablished { .. } => "session.established",
            Self::SessionStateChanged { .. } => "session.state_changed",
            Self::SessionActivity { .. } => "session.activity",
            Self::SessionClosed { .. } => "session.closed",
            Self::SessionError { .. } => "session.error",
        }
    }
}

/// Event bus for publishing session events
pub struct EventBus {
    sender: broadcast::Sender<SessionEvent>,
}

impl EventBus {
    /// Create new event bus with default capacity
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Create event bus with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish event to all subscribers
    ///
    /// # Errors
    /// Returns `Error::EventBusError` if no subscribers are listening
    pub fn publish(&self, event: SessionEvent) -> Result<()> {
        self.sender.send(event).map_err(|e| {
            Error::EventBusError(format!("No subscribers listening: {}", e))
        })?;
        Ok(())
    }

    /// Subscribe to events
    ///
    /// Returns a broadcast receiver for event consumption
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.sender.subscribe()
    }

    /// Get current subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let mut subscriber = bus.subscribe();

        let event = SessionEvent::SessionEstablished {
            session_id: Uuid::now_v7(),
            device_a_id: "DEV-A".to_string(),
            device_b_id: "DEV-B".to_string(),
            negotiated_version: "1.0.0".to_string(),
            shared_key_id: "key123".to_string(),
            ttl_seconds: 43200,
            created_at: Utc::now(),
            trace_id: "trace123".to_string(),
        };

        bus.publish(event.clone()).unwrap();

        let received = subscriber.recv().await.unwrap();
        assert_eq!(received.session_id(), event.session_id());
        assert_eq!(received.event_type(), "session.established");
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::new();
        let mut sub1 = bus.subscribe();
        let mut sub2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        let event = SessionEvent::SessionStateChanged {
            session_id: Uuid::now_v7(),
            from_state: SessionState::Pending,
            to_state: SessionState::Paired,
            timestamp: Utc::now(),
            trace_id: "trace123".to_string(),
        };

        bus.publish(event.clone()).unwrap();

        let recv1 = sub1.recv().await.unwrap();
        let recv2 = sub2.recv().await.unwrap();

        assert_eq!(recv1.session_id(), event.session_id());
        assert_eq!(recv2.session_id(), event.session_id());
    }

    #[tokio::test]
    async fn test_event_bus_no_subscribers() {
        let bus = EventBus::new();

        let event = SessionEvent::SessionClosed {
            session_id: Uuid::now_v7(),
            reason: "test".to_string(),
            timestamp: Utc::now(),
            trace_id: "trace123".to_string(),
        };

        // Should not panic, but return error
        let result = bus.publish(event);
        assert!(result.is_err());
    }

    #[test]
    fn test_session_event_accessors() {
        let session_id = Uuid::now_v7();
        let timestamp = Utc::now();

        let event = SessionEvent::SessionEstablished {
            session_id,
            device_a_id: "DEV-A".to_string(),
            device_b_id: "DEV-B".to_string(),
            negotiated_version: "1.0.0".to_string(),
            shared_key_id: "key123".to_string(),
            ttl_seconds: 43200,
            created_at: timestamp,
            trace_id: "trace123".to_string(),
        };

        assert_eq!(event.session_id(), session_id);
        assert_eq!(event.timestamp(), timestamp);
        assert_eq!(event.event_type(), "session.established");
    }

    #[test]
    fn test_event_serialization() {
        let event = SessionEvent::SessionStateChanged {
            session_id: Uuid::now_v7(),
            from_state: SessionState::Pending,
            to_state: SessionState::Active,
            timestamp: Utc::now(),
            trace_id: "trace123".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SessionEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.session_id(), event.session_id());
        assert_eq!(deserialized.event_type(), "session.state_changed");
    }
}
