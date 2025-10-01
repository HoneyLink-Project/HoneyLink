//! Event definitions for the shared event bus

use crate::types::{DeviceId, SessionId, StreamId};
use serde::{Deserialize, Serialize};

/// Events emitted by various modules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Session state changed
    SessionStateChanged {
        session_id: SessionId,
        old_state: String,
        new_state: String,
    },

    /// Policy updated
    PolicyUpdated { device_id: DeviceId, version: String },

    /// Stream created
    StreamCreated {
        session_id: SessionId,
        stream_id: StreamId,
    },

    /// Link state changed
    LinkStateChanged { adapter: String, state: String },
}
