//! Policy Engine orchestration layer
//!
//! Coordinates policy updates, profile management, and event distribution

use crate::error::Result;
use crate::event_bus::PolicyEventBus;
use crate::profile::{PolicyProfile, ProfileStorage};
use crate::types::{QoSPolicyUpdate, UseCase};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Main Policy Engine coordinator
///
/// Orchestrates policy lifecycle:
/// 1. Profile management (CRUD operations)
/// 2. Policy instance creation from profiles
/// 3. Event distribution to QoS Scheduler
/// 4. Rollback on failure
pub struct PolicyEngine<S: ProfileStorage> {
    /// Profile storage backend
    storage: Arc<RwLock<S>>,

    /// Event bus for policy distribution
    event_bus: Arc<PolicyEventBus>,
}

impl<S: ProfileStorage> PolicyEngine<S> {
    /// Create a new PolicyEngine instance
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            event_bus: Arc::new(PolicyEventBus::new()),
        }
    }

    /// Get reference to event bus for subscribers
    pub fn event_bus(&self) -> Arc<PolicyEventBus> {
        Arc::clone(&self.event_bus)
    }

    /// Create a new profile
    pub async fn create_profile(&self, profile: PolicyProfile) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.create(profile)
    }

    /// Get a profile by ID
    pub async fn get_profile(&self, profile_id: &str) -> Result<PolicyProfile> {
        let storage = self.storage.read().await;
        storage.get(profile_id)
    }

    /// Update an existing profile
    pub async fn update_profile(&self, profile: PolicyProfile) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.update(profile)
    }

    /// Delete a profile (soft delete)
    pub async fn delete_profile(&self, profile_id: &str) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.delete(profile_id)
    }

    /// List all profiles, optionally filtered by use case
    pub async fn list_profiles(&self, use_case: Option<UseCase>) -> Result<Vec<PolicyProfile>> {
        let storage = self.storage.read().await;
        storage.list(use_case)
    }

    /// Create a QoS policy instance from a profile
    ///
    /// # Arguments
    /// * `profile_id` - Profile template to instantiate
    /// * `stream_id` - Stream ID (0-7) to apply policy to
    /// * `device_id` - Device this policy applies to
    /// * `ttl_hours` - Time-to-live in hours (default: 12h per spec)
    ///
    /// # Returns
    /// Generated policy update that can be published to event bus
    pub async fn create_policy_from_profile(
        &self,
        profile_id: &str,
        stream_id: u8,
        device_id: &str,
        ttl_hours: Option<u32>,
    ) -> Result<QoSPolicyUpdate> {
        // Fetch profile
        let profile = self.get_profile(profile_id).await?;

        // Generate unique policy ID
        let policy_id = format!("pol_{}", Uuid::now_v7());

        // Calculate expiration (default 12h per spec)
        let ttl = ttl_hours.unwrap_or(12);
        let expiration_ts = Utc::now() + chrono::Duration::hours(ttl as i64);

        // Create policy update from profile
        let policy = QoSPolicyUpdate {
            schema_version: profile.profile_version.clone(),
            policy_id,
            profile_id: profile.profile_id.clone(),
            stream_id,
            latency_budget_ms: profile.latency_budget_ms,
            bandwidth_floor_mbps: profile.bandwidth_floor_mbps,
            bandwidth_ceiling_mbps: Some(profile.bandwidth_ceiling_mbps),
            fec_mode: profile.fec_mode,
            priority: profile.priority,
            power_profile: Some(profile.power_profile),
            deprecated_after: profile.deprecated_after,
            expiration_ts,
            signature: format!("policy:{}:device:{}", profile.signature, device_id),
        };

        // Validate before returning
        policy.validate()?;

        Ok(policy)
    }

    /// Apply a policy update by publishing to event bus
    ///
    /// Automatically saves snapshot for rollback
    pub async fn apply_policy(&self, policy: QoSPolicyUpdate) -> Result<usize> {
        self.event_bus.publish_update(policy).await
    }

    /// Rollback a policy to its last known good configuration
    pub async fn rollback_policy(&self, policy_id: &str) -> Result<usize> {
        self.event_bus.publish_rollback(policy_id).await
    }

    /// Invalidate a policy (due to expiration or deprecation)
    pub async fn invalidate_policy(&self, policy_id: &str) -> Result<usize> {
        self.event_bus.publish_invalidate(policy_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::create_iot_lowpower_preset;
    use crate::profile::InMemoryProfileStorage;

    #[tokio::test]
    async fn test_policy_engine_lifecycle() {
        let storage = InMemoryProfileStorage::new();
        let engine = PolicyEngine::new(storage);

        // Create profile
        let profile = create_iot_lowpower_preset();
        engine.create_profile(profile.clone()).await.unwrap();

        // Get profile
        let retrieved = engine.get_profile(&profile.profile_id).await.unwrap();
        assert_eq!(retrieved.profile_id, profile.profile_id);

        // List profiles
        let profiles = engine.list_profiles(None).await.unwrap();
        assert_eq!(profiles.len(), 1);

        // Create policy from profile
        let policy = engine
            .create_policy_from_profile(&profile.profile_id, 3, "dev_test123", Some(24))
            .await
            .unwrap();

        assert_eq!(policy.stream_id, 3);
        assert_eq!(policy.profile_id, profile.profile_id);
        assert!(policy.policy_id.starts_with("pol_"));

        // Subscribe to events
        let mut receiver = engine.event_bus().subscribe();

        // Apply policy
        let subscriber_count = engine.apply_policy(policy.clone()).await.unwrap();
        assert_eq!(subscriber_count, 1);

        // Receive event
        match receiver.recv().await.unwrap() {
            crate::event_bus::PolicyEvent::Update(received) => {
                assert_eq!(received.policy_id, policy.policy_id);
            }
            _ => panic!("Expected Update event"),
        }

        // Rollback
        engine.rollback_policy(&policy.policy_id).await.unwrap();

        match receiver.recv().await.unwrap() {
            crate::event_bus::PolicyEvent::Rollback { policy_id, .. } => {
                assert_eq!(policy_id, policy.policy_id);
            }
            _ => panic!("Expected Rollback event"),
        }

        // Invalidate
        engine.invalidate_policy(&policy.policy_id).await.unwrap();

        match receiver.recv().await.unwrap() {
            crate::event_bus::PolicyEvent::Invalidate { policy_id } => {
                assert_eq!(policy_id, policy.policy_id);
            }
            _ => panic!("Expected Invalidate event"),
        }

        // Delete profile
        engine.delete_profile(&profile.profile_id).await.unwrap();
        let deleted = engine.get_profile(&profile.profile_id).await.unwrap();
        assert!(deleted.deprecated_after.is_some());
    }

    #[tokio::test]
    async fn test_create_policy_validation() {
        let storage = InMemoryProfileStorage::new();
        let engine = PolicyEngine::new(storage);

        let profile = create_iot_lowpower_preset();
        engine.create_profile(profile.clone()).await.unwrap();

        // Invalid stream ID should fail
        let result = engine
            .create_policy_from_profile(&profile.profile_id, 10, "dev_test", Some(1))
            .await;

        assert!(result.is_err());
    }
}
