//! Session persistence trait and in-memory implementation
//!
//! Defines DB abstraction for session storage. Production should implement
//! with CockroachDB or other distributed SQL per spec/modules/session-orchestrator.md

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::session::Session;

/// Session persistence trait
///
/// Abstraction for session storage. Implementations should provide:
/// - CockroachDB for production (distributed SQL, multi-region)
/// - PostgreSQL for single-region deployments
/// - In-memory for testing
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Store new session
    async fn create(&mut self, session: Session) -> Result<()>;

    /// Retrieve session by ID
    async fn get(&self, session_id: Uuid) -> Result<Option<Session>>;

    /// Update existing session
    async fn update(&mut self, session: Session) -> Result<()>;

    /// Delete session (for cleanup)
    async fn delete(&mut self, session_id: Uuid) -> Result<()>;

    /// List all sessions for a device
    async fn list_by_device(&self, device_id: &str) -> Result<Vec<Session>>;

    /// List sessions by state
    async fn list_by_state(&self, state: crate::state_machine::SessionState) -> Result<Vec<Session>>;

    /// Count active sessions
    async fn count_active(&self) -> Result<usize>;

    /// Cleanup expired sessions
    ///
    /// Returns number of sessions deleted
    async fn cleanup_expired(&mut self) -> Result<usize>;
}

/// In-memory session store for testing and single-instance deployments
///
/// NOTE: Not suitable for production multi-instance setups (no shared state)
pub struct InMemorySessionStore {
    sessions: HashMap<Uuid, Session>,
}

impl InMemorySessionStore {
    /// Create new in-memory store
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(&mut self, session: Session) -> Result<()> {
        if self.sessions.contains_key(&session.session_id) {
            return Err(Error::PersistenceError(format!(
                "Session {} already exists",
                session.session_id
            )));
        }

        self.sessions.insert(session.session_id, session);
        Ok(())
    }

    async fn get(&self, session_id: Uuid) -> Result<Option<Session>> {
        Ok(self.sessions.get(&session_id).cloned())
    }

    async fn update(&mut self, session: Session) -> Result<()> {
        if !self.sessions.contains_key(&session.session_id) {
            return Err(Error::SessionNotFound(session.session_id.to_string()));
        }

        self.sessions.insert(session.session_id, session);
        Ok(())
    }

    async fn delete(&mut self, session_id: Uuid) -> Result<()> {
        self.sessions
            .remove(&session_id)
            .ok_or_else(|| Error::SessionNotFound(session_id.to_string()))?;
        Ok(())
    }

    async fn list_by_device(&self, device_id: &str) -> Result<Vec<Session>> {
        Ok(self
            .sessions
            .values()
            .filter(|s| s.device_a_id == device_id || s.device_b_id == device_id)
            .cloned()
            .collect())
    }

    async fn list_by_state(
        &self,
        state: crate::state_machine::SessionState,
    ) -> Result<Vec<Session>> {
        Ok(self
            .sessions
            .values()
            .filter(|s| s.state == state)
            .cloned()
            .collect())
    }

    async fn count_active(&self) -> Result<usize> {
        Ok(self
            .sessions
            .values()
            .filter(|s| s.state == crate::state_machine::SessionState::Active)
            .count())
    }

    async fn cleanup_expired(&mut self) -> Result<usize> {
        let initial_count = self.sessions.len();
        self.sessions.retain(|_, session| !session.is_expired());
        Ok(initial_count - self.sessions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::SessionState;

    #[tokio::test]
    async fn test_create_and_get() {
        let mut store = InMemorySessionStore::new();
        let session = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );
        let session_id = session.session_id;

        store.create(session.clone()).await.unwrap();

        let retrieved = store.get(session_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session_id);
    }

    #[tokio::test]
    async fn test_create_duplicate() {
        let mut store = InMemorySessionStore::new();
        let session = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );

        store.create(session.clone()).await.unwrap();
        let result = store.create(session).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let mut store = InMemorySessionStore::new();
        let mut session = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );
        let session_id = session.session_id;

        store.create(session.clone()).await.unwrap();

        session.set_state(SessionState::Active);
        store.update(session).await.unwrap();

        let retrieved = store.get(session_id).await.unwrap().unwrap();
        assert_eq!(retrieved.state, SessionState::Active);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut store = InMemorySessionStore::new();
        let session = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );
        let session_id = session.session_id;

        store.create(session).await.unwrap();
        store.delete(session_id).await.unwrap();

        let retrieved = store.get(session_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_by_device() {
        let mut store = InMemorySessionStore::new();

        let session1 = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );
        let session2 = Session::new(
            "DEV-A".to_string(),
            "DEV-C".to_string(),
            "1.0.0".to_string(),
            12,
        );
        let session3 = Session::new(
            "DEV-D".to_string(),
            "DEV-E".to_string(),
            "1.0.0".to_string(),
            12,
        );

        store.create(session1).await.unwrap();
        store.create(session2).await.unwrap();
        store.create(session3).await.unwrap();

        let sessions = store.list_by_device("DEV-A").await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_state() {
        let mut store = InMemorySessionStore::new();

        let mut session1 = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );
        session1.set_state(SessionState::Active);

        let session2 = Session::new(
            "DEV-C".to_string(),
            "DEV-D".to_string(),
            "1.0.0".to_string(),
            12,
        );

        store.create(session1).await.unwrap();
        store.create(session2).await.unwrap();

        let active_sessions = store.list_by_state(SessionState::Active).await.unwrap();
        assert_eq!(active_sessions.len(), 1);

        let pending_sessions = store.list_by_state(SessionState::Pending).await.unwrap();
        assert_eq!(pending_sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_count_active() {
        let mut store = InMemorySessionStore::new();

        let mut session1 = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );
        session1.set_state(SessionState::Active);

        let mut session2 = Session::new(
            "DEV-C".to_string(),
            "DEV-D".to_string(),
            "1.0.0".to_string(),
            12,
        );
        session2.set_state(SessionState::Active);

        let session3 = Session::new(
            "DEV-E".to_string(),
            "DEV-F".to_string(),
            "1.0.0".to_string(),
            12,
        );

        store.create(session1).await.unwrap();
        store.create(session2).await.unwrap();
        store.create(session3).await.unwrap();

        let count = store.count_active().await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let mut store = InMemorySessionStore::new();

        let mut session1 = Session::new(
            "DEV-A".to_string(),
            "DEV-B".to_string(),
            "1.0.0".to_string(),
            12,
        );
        // Expire session1
        session1.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);

        let session2 = Session::new(
            "DEV-C".to_string(),
            "DEV-D".to_string(),
            "1.0.0".to_string(),
            12,
        );

        store.create(session1).await.unwrap();
        store.create(session2).await.unwrap();

        let removed = store.cleanup_expired().await.unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.sessions.len(), 1);
    }
}
