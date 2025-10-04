//! Telemetry integration for policy engine
//!
//! Integrates with honeylink-telemetry crate to record:
//! - policy_update_latency_p95 (SLI)
//! - Policy creation/update/validation events
//! - Profile template operations

use honeylink_telemetry::{Metric, MetricType, TelemetryCollector};
use std::sync::Arc;
use std::time::Instant;
use crate::error::Result;

/// Policy engine telemetry recorder
///
/// Wraps TelemetryCollector to provide policy-specific metrics.
/// Thread-safe via Arc (TelemetryCollector is already thread-safe).
#[derive(Clone)]
pub struct PolicyTelemetry {
    collector: Arc<TelemetryCollector>,
}

impl PolicyTelemetry {
    /// Create new policy telemetry recorder
    ///
    /// # Arguments
    /// * `collector` - Shared telemetry collector instance
    pub fn new(collector: Arc<TelemetryCollector>) -> Self {
        Self { collector }
    }

    /// Record policy update processing latency
    ///
    /// Records `policy_update_latency_p95` SLI metric.
    /// This is a key metric per spec/testing/metrics.md:
    /// - Green: < 250ms
    /// - Yellow: 250-300ms
    /// - Orange: 300-500ms
    /// - Red: > 500ms
    /// - SLO: P95 < 300ms
    ///
    /// # Arguments
    /// * `start_time` - When policy update started (from `Instant::now()`)
    /// * `success` - Whether update succeeded
    /// * `operation` - Operation type: "create", "update", "delete", "validate"
    pub async fn record_policy_update(
        &self,
        start_time: Instant,
        success: bool,
        operation: &str,
    ) -> Result<()> {
        let duration_ms = start_time.elapsed().as_millis() as f64;

        let labels = vec![
            ("success".to_string(), success.to_string()),
            ("operation".to_string(), operation.to_string()),
        ];

        let metric = Metric {
            name: "policy_update_latency_p95".to_string(),
            metric_type: MetricType::Histogram,
            value: duration_ms,
            labels,
            timestamp: chrono::Utc::now(),
        };

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::InternalError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record policy validation event
    ///
    /// # Arguments
    /// * `policy_id` - Policy identifier
    /// * `valid` - Whether validation passed
    /// * `error_count` - Number of validation errors found
    pub async fn record_policy_validation(
        &self,
        policy_id: &str,
        valid: bool,
        error_count: u32,
    ) -> Result<()> {
        let labels = vec![
            ("policy_id".to_string(), policy_id.to_string()),
            ("valid".to_string(), valid.to_string()),
        ];

        let metric = Metric {
            name: "policy_validation_errors_total".to_string(),
            metric_type: MetricType::Counter,
            value: error_count as f64,
            labels,
            timestamp: chrono::Utc::now(),
        };

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::InternalError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record active policies count gauge
    ///
    /// # Arguments
    /// * `count` - Current number of active policies
    pub async fn record_active_policies(&self, count: u64) -> Result<()> {
        let metric = Metric {
            name: "policy_active_count".to_string(),
            metric_type: MetricType::Gauge,
            value: count as f64,
            labels: vec![],
            timestamp: chrono::Utc::now(),
        };

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::InternalError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record profile template operation
    ///
    /// # Arguments
    /// * `operation` - Operation type: "create", "update", "delete", "get"
    /// * `profile_name` - Profile template name
    /// * `success` - Whether operation succeeded
    pub async fn record_profile_operation(
        &self,
        operation: &str,
        profile_name: &str,
        success: bool,
    ) -> Result<()> {
        let labels = vec![
            ("operation".to_string(), operation.to_string()),
            ("profile_name".to_string(), profile_name.to_string()),
            ("success".to_string(), success.to_string()),
        ];

        let metric = Metric {
            name: "policy_profile_operations_total".to_string(),
            metric_type: MetricType::Counter,
            value: 1.0,
            labels,
            timestamp: chrono::Utc::now(),
        };

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::InternalError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record policy event bus publish
    ///
    /// Tracks policy update events sent to QoS Scheduler.
    ///
    /// # Arguments
    /// * `policy_id` - Policy identifier
    /// * `event_type` - Event type: "update", "rollback", "create", "delete"
    /// * `success` - Whether publish succeeded
    pub async fn record_event_bus_publish(
        &self,
        policy_id: &str,
        event_type: &str,
        success: bool,
    ) -> Result<()> {
        let labels = vec![
            ("policy_id".to_string(), policy_id.to_string()),
            ("event_type".to_string(), event_type.to_string()),
            ("success".to_string(), success.to_string()),
        ];

        let metric = Metric {
            name: "policy_eventbus_publishes_total".to_string(),
            metric_type: MetricType::Counter,
            value: 1.0,
            labels,
            timestamp: chrono::Utc::now(),
        };

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::InternalError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record policy rollback event
    ///
    /// Tracks when policy updates are rolled back due to failure.
    ///
    /// # Arguments
    /// * `policy_id` - Policy identifier
    /// * `reason` - Rollback reason (e.g., "validation_failure", "qos_scheduler_reject")
    pub async fn record_policy_rollback(&self, policy_id: &str, reason: &str) -> Result<()> {
        let labels = vec![
            ("policy_id".to_string(), policy_id.to_string()),
            ("reason".to_string(), reason.to_string()),
        ];

        let metric = Metric {
            name: "policy_rollbacks_total".to_string(),
            metric_type: MetricType::Counter,
            value: 1.0,
            labels,
            timestamp: chrono::Utc::now(),
        };

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::InternalError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeylink_telemetry::TelemetryConfig;

    #[tokio::test]
    async fn test_record_policy_update_success() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = PolicyTelemetry::new(Arc::new(collector));

        let start = Instant::now();
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

        let result = telemetry
            .record_policy_update(start, true, "create")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_policy_validation() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = PolicyTelemetry::new(Arc::new(collector));

        let result = telemetry
            .record_policy_validation("policy-123", false, 3)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_active_policies() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = PolicyTelemetry::new(Arc::new(collector));

        let result = telemetry.record_active_policies(15).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_profile_operation() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = PolicyTelemetry::new(Arc::new(collector));

        let result = telemetry
            .record_profile_operation("create", "prof_iot_lowpower_v2", true)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_event_bus_publish() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = PolicyTelemetry::new(Arc::new(collector));

        let result = telemetry
            .record_event_bus_publish("policy-456", "update", true)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_policy_rollback() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = PolicyTelemetry::new(Arc::new(collector));

        let result = telemetry
            .record_policy_rollback("policy-789", "validation_failure")
            .await;
        assert!(result.is_ok());
    }
}
