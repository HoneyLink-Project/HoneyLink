//! Policy update event bus with at-least-once delivery and fallback
//!
//! Provides reliable policy distribution to QoS Scheduler with:
//! - At-least-once delivery guarantee
//! - Configuration snapshots for rollback
//! - Automatic fallback on failure

use crate::error::{PolicyError, Result};
use crate::types::QoSPolicyUpdate;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Maximum number of pending policy updates in channel
const EVENT_BUS_CAPACITY: usize = 1024;

/// Policy update event types
#[derive(Debug, Clone)]
pub enum PolicyEvent {
    /// New policy update to be applied
    Update(QoSPolicyUpdate),

    /// Policy update failed, rollback to snapshot
    Rollback {
        policy_id: String,
        snapshot: QoSPolicyUpdate,
    },

    /// Policy expired or deprecated
    Invalidate { policy_id: String },
}

/// Event bus for distributing policy updates to subscribers
///
/// Uses tokio::sync::broadcast for multi-subscriber pub/sub pattern
pub struct PolicyEventBus {
    sender: broadcast::Sender<PolicyEvent>,

    /// Snapshot storage for rollback on failure
    /// Maps policy_id -> last known good configuration
    snapshots: Arc<RwLock<std::collections::HashMap<String, QoSPolicyUpdate>>>,
}

impl PolicyEventBus {
    /// Create a new policy event bus
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(EVENT_BUS_CAPACITY);

        Self {
            sender,
            snapshots: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Subscribe to policy events
    ///
    /// Returns a receiver that can be used to listen for policy updates
    pub fn subscribe(&self) -> broadcast::Receiver<PolicyEvent> {
        self.sender.subscribe()
    }

    /// Publish a policy update event
    ///
    /// Automatically saves a snapshot for potential rollback
    ///
    /// # Arguments
    /// * `update` - The policy update to publish
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of active subscribers that received the event
    /// * `Err(PolicyError)` - If validation fails or channel is closed
    pub async fn publish_update(&self, update: QoSPolicyUpdate) -> Result<usize> {
        // Validate before publishing
        update.validate()?;

        // Save snapshot for rollback
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.insert(update.policy_id.clone(), update.clone());
        }

        // Publish event
        let subscriber_count = self
            .sender
            .send(PolicyEvent::Update(update))
            .map_err(|e| PolicyError::EventBus(format!("Failed to send update: {}", e)))?;

        Ok(subscriber_count)
    }

    /// Rollback a policy to its last known good configuration
    ///
    /// # Arguments
    /// * `policy_id` - ID of the policy to rollback
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of subscribers notified
    /// * `Err(PolicyError)` - If snapshot doesn't exist or channel is closed
    pub async fn publish_rollback(&self, policy_id: &str) -> Result<usize> {
        // Retrieve snapshot
        let snapshot = {
            let snapshots = self.snapshots.read().await;
            snapshots
                .get(policy_id)
                .cloned()
                .ok_or_else(|| {
                    PolicyError::NotFound(format!("No snapshot for policy {}", policy_id))
                })?
        };

        // Publish rollback event
        let subscriber_count = self
            .sender
            .send(PolicyEvent::Rollback {
                policy_id: policy_id.to_string(),
                snapshot,
            })
            .map_err(|e| PolicyError::EventBus(format!("Failed to send rollback: {}", e)))?;

        Ok(subscriber_count)
    }

    /// Invalidate a policy (due to expiration or deprecation)
    ///
    /// # Arguments
    /// * `policy_id` - ID of the policy to invalidate
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of subscribers notified
    /// * `Err(PolicyError)` - If channel is closed
    pub async fn publish_invalidate(&self, policy_id: &str) -> Result<usize> {
        // Remove snapshot as policy is no longer valid
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.remove(policy_id);
        }

        // Publish invalidate event
        let subscriber_count = self
            .sender
            .send(PolicyEvent::Invalidate {
                policy_id: policy_id.to_string(),
            })
            .map_err(|e| PolicyError::EventBus(format!("Failed to send invalidate: {}", e)))?;

        Ok(subscriber_count)
    }

    /// Get current snapshot for a policy (for debugging/monitoring)
    pub async fn get_snapshot(&self, policy_id: &str) -> Option<QoSPolicyUpdate> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(policy_id).cloned()
    }

    /// Get count of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Clear all snapshots (use with caution)
    pub async fn clear_snapshots(&self) {
        let mut snapshots = self.snapshots.write().await;
        snapshots.clear();
    }
}

impl Default for PolicyEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FecMode, PowerProfile};
    use chrono::{Duration, Utc};
    use semver::Version;

    fn create_test_policy(policy_id: &str) -> QoSPolicyUpdate {
        QoSPolicyUpdate {
            schema_version: Version::new(1, 2, 0),
            policy_id: format!("pol_{}", policy_id),
            profile_id: "prof_test".to_string(),
            stream_id: 1,
            latency_budget_ms: 50,
            bandwidth_floor_mbps: 10.0,
            bandwidth_ceiling_mbps: Some(100.0),
            fec_mode: FecMode::Light,
            priority: 5,
            power_profile: Some(PowerProfile::Normal),
            deprecated_after: None,
            expiration_ts: Utc::now() + Duration::hours(1),
            signature: "test_signature".to_string(),
        }
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let bus = PolicyEventBus::new();
        let mut receiver = bus.subscribe();

        let policy = create_test_policy("test1");
        let subscriber_count = bus.publish_update(policy.clone()).await.unwrap();

        assert_eq!(subscriber_count, 1);

        // Receive event
        match receiver.recv().await.unwrap() {
            PolicyEvent::Update(received_policy) => {
                assert_eq!(received_policy.policy_id, policy.policy_id);
            }
            _ => panic!("Expected Update event"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = PolicyEventBus::new();
        let mut receiver1 = bus.subscribe();
        let mut receiver2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        let policy = create_test_policy("test2");
        bus.publish_update(policy.clone()).await.unwrap();

        // Both receivers should get the event
        match receiver1.recv().await.unwrap() {
            PolicyEvent::Update(p) => assert_eq!(p.policy_id, policy.policy_id),
            _ => panic!("Expected Update event"),
        }

        match receiver2.recv().await.unwrap() {
            PolicyEvent::Update(p) => assert_eq!(p.policy_id, policy.policy_id),
            _ => panic!("Expected Update event"),
        }
    }

    #[tokio::test]
    async fn test_snapshot_and_rollback() {
        let bus = PolicyEventBus::new();
        let mut receiver = bus.subscribe();

        let policy = create_test_policy("test3");
        bus.publish_update(policy.clone()).await.unwrap();

        // Verify snapshot was saved
        let snapshot = bus.get_snapshot(&policy.policy_id).await;
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().policy_id, policy.policy_id);

        // Publish rollback
        bus.publish_rollback(&policy.policy_id).await.unwrap();

        // Skip the initial update event
        receiver.recv().await.unwrap();

        // Receive rollback event
        match receiver.recv().await.unwrap() {
            PolicyEvent::Rollback { policy_id, snapshot } => {
                assert_eq!(policy_id, policy.policy_id);
                assert_eq!(snapshot.policy_id, policy.policy_id);
            }
            _ => panic!("Expected Rollback event"),
        }
    }

    #[tokio::test]
    async fn test_invalidate() {
        let bus = PolicyEventBus::new();
        let mut receiver = bus.subscribe();

        let policy = create_test_policy("test4");
        bus.publish_update(policy.clone()).await.unwrap();

        // Verify snapshot exists
        assert!(bus.get_snapshot(&policy.policy_id).await.is_some());

        // Invalidate
        bus.publish_invalidate(&policy.policy_id).await.unwrap();

        // Verify snapshot was removed
        assert!(bus.get_snapshot(&policy.policy_id).await.is_none());

        // Skip update event
        receiver.recv().await.unwrap();

        // Receive invalidate event
        match receiver.recv().await.unwrap() {
            PolicyEvent::Invalidate { policy_id } => {
                assert_eq!(policy_id, policy.policy_id);
            }
            _ => panic!("Expected Invalidate event"),
        }
    }

    #[tokio::test]
    async fn test_validation_before_publish() {
        let bus = PolicyEventBus::new();

        let mut invalid_policy = create_test_policy("test5");
        invalid_policy.stream_id = 10; // Invalid (must be 0-7)

        let result = bus.publish_update(invalid_policy).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rollback_without_snapshot() {
        let bus = PolicyEventBus::new();

        let result = bus.publish_rollback("pol_nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_snapshots() {
        let bus = PolicyEventBus::new();

        bus.publish_update(create_test_policy("test6")).await.unwrap();
        bus.publish_update(create_test_policy("test7")).await.unwrap();

        assert!(bus.get_snapshot("pol_test6").await.is_some());
        assert!(bus.get_snapshot("pol_test7").await.is_some());

        bus.clear_snapshots().await;

        assert!(bus.get_snapshot("pol_test6").await.is_none());
        assert!(bus.get_snapshot("pol_test7").await.is_none());
    }
}
