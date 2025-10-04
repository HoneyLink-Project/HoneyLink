

//! Telemetry integration for session orchestrator
//!
//! Integrates with honeylink-telemetry crate to record:
//! - session_establishment_latency_p95 (SLI)
//! - Session lifecycle events
//! - State transition metrics

use honeylink_telemetry::{Metric, MetricType, TelemetryCollector};
use std::sync::Arc;
use std::time::Instant;
use crate::state_machine::SessionState;
use crate::error::Result;

/// Session orchestrator telemetry recorder
///
/// Wraps TelemetryCollector to provide session-specific metrics.
/// Thread-safe via Arc (TelemetryCollector is already thread-safe).
#[derive(Clone)]
pub struct SessionTelemetry {
    collector: Arc<TelemetryCollector>,
}

impl SessionTelemetry {
    /// Create new session telemetry recorder
    ///
    /// # Arguments
    /// * `collector` - Shared telemetry collector instance
    pub fn new(collector: Arc<TelemetryCollector>) -> Self {
        Self { collector }
    }

    /// Record session establishment timing
    ///
    /// Records `session_establishment_latency_p95` SLI metric.
    /// This is a key metric per spec/testing/metrics.md:
    /// - Green: < 400ms
    /// - Yellow: 400-500ms
    /// - Orange: 500-800ms
    /// - Red: > 800ms
    /// - SLO: P95 < 500ms
    ///
    /// # Arguments
    /// * `start_time` - When session establishment started (from `Instant::now()`)
    /// * `success` - Whether establishment succeeded
    pub async fn record_session_establishment(
        &self,
        start_time: Instant,
        success: bool,
    ) -> Result<()> {
        let duration_ms = start_time.elapsed().as_millis() as f64;

        let labels = vec![("success".to_string(), success.to_string())];

        let metric = Metric::new(
            "session_establishment_latency_p95".to_string(),
            MetricType::Histogram,
            duration_ms,
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::EventBusError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record state transition event
    ///
    /// Records state machine transitions for monitoring.
    ///
    /// # Arguments
    /// * `from_state` - Previous state
    /// * `to_state` - New state
    /// * `session_id` - Session identifier
    pub async fn record_state_transition(
        &self,
        from_state: &SessionState,
        to_state: &SessionState,
        session_id: &str,
    ) -> Result<()> {
        let labels = vec![
            ("from_state".to_string(), format!("{:?}", from_state)),
            ("to_state".to_string(), format!("{:?}", to_state)),
            ("session_id".to_string(), session_id.to_string()),
        ];

        let metric = Metric::new(
            "session_state_transitions_total".to_string(),
            MetricType::Counter,
            1.0,
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::EventBusError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record active session count gauge
    ///
    /// # Arguments
    /// * `count` - Current number of active sessions
    pub async fn record_active_sessions(&self, count: u64) -> Result<()> {
        let metric = Metric::new(
            "session_active_count".to_string(),
            MetricType::Gauge,
            count as f64,
            vec![],
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::EventBusError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record session failure
    ///
    /// Increments failure counter with reason label.
    ///
    /// # Arguments
    /// * `reason` - Failure reason (e.g., "timeout", "protocol_mismatch", "crypto_error")
    pub async fn record_session_failure(&self, reason: &str) -> Result<()> {
        let labels = vec![("reason".to_string(), reason.to_string())];

        let metric = Metric::new(
            "session_failures_total".to_string(),
            MetricType::Counter,
            1.0,
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::EventBusError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }

    /// Record session activity (heartbeat)
    ///
    /// Records when session is touched (sliding window refresh).
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    pub async fn record_session_activity(&self, session_id: &str) -> Result<()> {
        let labels = vec![("session_id".to_string(), session_id.to_string())];

        let metric = Metric::new(
            "session_activity_heartbeats_total".to_string(),
            MetricType::Counter,
            1.0,
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| crate::error::Error::EventBusError(format!("Telemetry error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeylink_telemetry::TelemetryConfig;

    #[tokio::test]
    async fn test_record_session_establishment_success() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = SessionTelemetry::new(Arc::new(collector));

        let start = Instant::now();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let result = telemetry.record_session_establishment(start, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_state_transition() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = SessionTelemetry::new(Arc::new(collector));

        let result = telemetry
            .record_state_transition(&SessionState::Pending, &SessionState::Paired, "test-session")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_active_sessions() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = SessionTelemetry::new(Arc::new(collector));

        let result = telemetry.record_active_sessions(42).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_session_failure() {
        let mut collector = TelemetryCollector::new();
        collector
            .initialize(TelemetryConfig::default())
            .await
            .unwrap();
        let telemetry = SessionTelemetry::new(Arc::new(collector));

        let result = telemetry.record_session_failure("timeout").await;
        assert!(result.is_ok());
    }
}
