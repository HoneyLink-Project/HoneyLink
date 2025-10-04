//! # Session Orchestrator
//!
//! Manages session lifecycle, handshake, and state transitions per
//! spec/modules/session-orchestrator.md
//!
//! ## Features
//! - 5-state machine (Pending/Paired/Active/Suspended/Closed)
//! - UUIDv7-based session IDs (monotonic, time-ordered)
//! - Idempotency-key support (24h retention)
//! - TTL management (12h default + 30min sliding window)
//! - SemVer protocol version negotiation
//! - Event bus integration (tokio broadcast channels)
//! - OpenTelemetry metrics

pub mod error;
pub mod event_bus;
pub mod idempotency;
pub mod metrics;
pub mod persistence;
pub mod session;
pub mod state_machine;
pub mod telemetry;
pub mod versioning;

pub use error::{Error, Result};
pub use event_bus::{EventBus, SessionEvent};
pub use idempotency::{IdempotencyRecord, IdempotencyStore};
pub use metrics::Metrics;
pub use persistence::{InMemorySessionStore, SessionStore};
pub use session::Session;
pub use state_machine::{SessionState, SessionStateMachine, TransitionEvent};
pub use telemetry::SessionTelemetry;
pub use versioning::{NegotiatedVersion, VersionNegotiator};
