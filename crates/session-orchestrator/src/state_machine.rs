//! Session state machine with validated transitions
//!
//! Implements the 5-state machine defined in spec/modules/session-orchestrator.md:
//! - Pending: Initial state, awaiting device authentication
//! - Paired: Devices authenticated, awaiting policy application
//! - Active: Session fully established and operational
//! - Suspended: Temporarily inactive (network loss, low battery)
//! - Closed: Session terminated (final state)

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Session states as defined in spec/modules/session-orchestrator.md#9.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionState {
    /// Initial state: HandshakeRequest received, awaiting authentication
    Pending,

    /// Devices authenticated: Crypto & Trust validation successful
    Paired,

    /// Fully operational: Policy applied, QoS active
    Active,

    /// Temporarily inactive: Network loss or low battery detected
    Suspended,

    /// Terminal state: Session ended (user disconnect, TTL expired, or error)
    Closed,
}

/// State transition event triggers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionEvent {
    /// Device authentication successful (Pending → Paired)
    DeviceAuthenticated,

    /// Authentication failed or timeout (Pending → Closed)
    AuthenticationFailed,

    /// Policy successfully applied (Paired → Active)
    PolicyApplied,

    /// Policy rejected by Policy Engine (Paired → Closed)
    PolicyRejected,

    /// Network loss detected (Active → Suspended)
    NetworkLoss,

    /// User-initiated disconnect (Active → Closed)
    UserDisconnect,

    /// TTL expired (Active → Closed)
    TtlExpired,

    /// Network restored (Suspended → Active)
    NetworkRestored,

    /// Suspend timeout exceeded (Suspended → Closed)
    SuspendTimeout,
}

/// Session state machine with validated transitions
///
/// Enforces the state transition table from spec/modules/session-orchestrator.md#9.2
/// Invalid transitions are rejected with `Error::InvalidStateTransition`
pub struct SessionStateMachine {
    current_state: SessionState,
}

impl SessionStateMachine {
    /// Create new state machine in Pending state
    pub fn new() -> Self {
        Self {
            current_state: SessionState::Pending,
        }
    }

    /// Create state machine from existing state (for restoration from DB)
    pub fn from_state(state: SessionState) -> Self {
        Self {
            current_state: state,
        }
    }

    /// Get current state
    pub fn state(&self) -> SessionState {
        self.current_state
    }

    /// Attempt state transition based on event
    ///
    /// # Errors
    /// Returns `Error::InvalidStateTransition` if transition is not allowed
    pub fn transition(&mut self, event: TransitionEvent) -> Result<SessionState> {
        let new_state = match (self.current_state, event) {
            // Pending transitions
            (SessionState::Pending, TransitionEvent::DeviceAuthenticated) => SessionState::Paired,
            (SessionState::Pending, TransitionEvent::AuthenticationFailed) => SessionState::Closed,

            // Paired transitions
            (SessionState::Paired, TransitionEvent::PolicyApplied) => SessionState::Active,
            (SessionState::Paired, TransitionEvent::PolicyRejected) => SessionState::Closed,

            // Active transitions
            (SessionState::Active, TransitionEvent::NetworkLoss) => SessionState::Suspended,
            (SessionState::Active, TransitionEvent::UserDisconnect) => SessionState::Closed,
            (SessionState::Active, TransitionEvent::TtlExpired) => SessionState::Closed,

            // Suspended transitions
            (SessionState::Suspended, TransitionEvent::NetworkRestored) => SessionState::Active,
            (SessionState::Suspended, TransitionEvent::SuspendTimeout) => SessionState::Closed,

            // Invalid transitions
            (current, _) => {
                return Err(Error::InvalidStateTransition {
                    from: current,
                    to: self.current_state, // Stays in current state
                });
            }
        };

        self.current_state = new_state;
        Ok(new_state)
    }

    /// Check if current state is terminal (Closed)
    pub fn is_terminal(&self) -> bool {
        self.current_state == SessionState::Closed
    }

    /// Check if transition event is valid for current state
    pub fn can_transition(&self, event: TransitionEvent) -> bool {
        matches!(
            (self.current_state, event),
            (SessionState::Pending, TransitionEvent::DeviceAuthenticated)
                | (SessionState::Pending, TransitionEvent::AuthenticationFailed)
                | (SessionState::Paired, TransitionEvent::PolicyApplied)
                | (SessionState::Paired, TransitionEvent::PolicyRejected)
                | (SessionState::Active, TransitionEvent::NetworkLoss)
                | (SessionState::Active, TransitionEvent::UserDisconnect)
                | (SessionState::Active, TransitionEvent::TtlExpired)
                | (SessionState::Suspended, TransitionEvent::NetworkRestored)
                | (SessionState::Suspended, TransitionEvent::SuspendTimeout)
        )
    }
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = SessionStateMachine::new();
        assert_eq!(sm.state(), SessionState::Pending);
        assert!(!sm.is_terminal());
    }

    #[test]
    fn test_valid_pending_to_paired() {
        let mut sm = SessionStateMachine::new();
        let result = sm.transition(TransitionEvent::DeviceAuthenticated);
        assert!(result.is_ok());
        assert_eq!(sm.state(), SessionState::Paired);
    }

    #[test]
    fn test_valid_pending_to_closed() {
        let mut sm = SessionStateMachine::new();
        let result = sm.transition(TransitionEvent::AuthenticationFailed);
        assert!(result.is_ok());
        assert_eq!(sm.state(), SessionState::Closed);
        assert!(sm.is_terminal());
    }

    #[test]
    fn test_valid_paired_to_active() {
        let mut sm = SessionStateMachine::from_state(SessionState::Paired);
        let result = sm.transition(TransitionEvent::PolicyApplied);
        assert!(result.is_ok());
        assert_eq!(sm.state(), SessionState::Active);
    }

    #[test]
    fn test_valid_active_to_suspended() {
        let mut sm = SessionStateMachine::from_state(SessionState::Active);
        let result = sm.transition(TransitionEvent::NetworkLoss);
        assert!(result.is_ok());
        assert_eq!(sm.state(), SessionState::Suspended);
    }

    #[test]
    fn test_valid_suspended_to_active() {
        let mut sm = SessionStateMachine::from_state(SessionState::Suspended);
        let result = sm.transition(TransitionEvent::NetworkRestored);
        assert!(result.is_ok());
        assert_eq!(sm.state(), SessionState::Active);
    }

    #[test]
    fn test_invalid_closed_to_active() {
        let mut sm = SessionStateMachine::from_state(SessionState::Closed);
        let result = sm.transition(TransitionEvent::PolicyApplied);
        assert!(result.is_err());
        match result {
            Err(Error::InvalidStateTransition { from, to }) => {
                assert_eq!(from, SessionState::Closed);
                assert_eq!(to, SessionState::Closed);
            }
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_invalid_pending_to_active() {
        let mut sm = SessionStateMachine::new();
        let result = sm.transition(TransitionEvent::PolicyApplied);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_transition() {
        let sm = SessionStateMachine::new();
        assert!(sm.can_transition(TransitionEvent::DeviceAuthenticated));
        assert!(sm.can_transition(TransitionEvent::AuthenticationFailed));
        assert!(!sm.can_transition(TransitionEvent::PolicyApplied));
        assert!(!sm.can_transition(TransitionEvent::NetworkLoss));
    }

    #[test]
    fn test_full_happy_path() {
        let mut sm = SessionStateMachine::new();

        // Pending → Paired
        assert!(sm.transition(TransitionEvent::DeviceAuthenticated).is_ok());
        assert_eq!(sm.state(), SessionState::Paired);

        // Paired → Active
        assert!(sm.transition(TransitionEvent::PolicyApplied).is_ok());
        assert_eq!(sm.state(), SessionState::Active);

        // Active → Suspended
        assert!(sm.transition(TransitionEvent::NetworkLoss).is_ok());
        assert_eq!(sm.state(), SessionState::Suspended);

        // Suspended → Active
        assert!(sm.transition(TransitionEvent::NetworkRestored).is_ok());
        assert_eq!(sm.state(), SessionState::Active);

        // Active → Closed
        assert!(sm.transition(TransitionEvent::UserDisconnect).is_ok());
        assert_eq!(sm.state(), SessionState::Closed);
        assert!(sm.is_terminal());
    }
}
