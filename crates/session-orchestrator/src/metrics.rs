//! OpenTelemetry metrics for session orchestrator
//!
//! Implements SLI metrics per spec/modules/session-orchestrator.md#8.1
//! and spec/testing/metrics.md

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Session orchestrator metrics collector
///
/// Thread-safe metrics using atomic operations. In production, these should
/// be exported to OpenTelemetry collector via OTLP.
#[derive(Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    // Counters
    sessions_established_total: AtomicU64,
    sessions_failed_total: AtomicU64,
    state_transitions_total: AtomicU64,
    errors_total: AtomicU64,

    // Gauges
    active_sessions: AtomicU64,
    pending_sessions: AtomicU64,
    suspended_sessions: AtomicU64,

    // Histograms (simplified: store sum and count for average calculation)
    establishment_duration_sum_ms: AtomicU64,
    establishment_duration_count: AtomicU64,
    transition_duration_sum_ms: AtomicU64,
    transition_duration_count: AtomicU64,
}

impl Metrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                sessions_established_total: AtomicU64::new(0),
                sessions_failed_total: AtomicU64::new(0),
                state_transitions_total: AtomicU64::new(0),
                errors_total: AtomicU64::new(0),
                active_sessions: AtomicU64::new(0),
                pending_sessions: AtomicU64::new(0),
                suspended_sessions: AtomicU64::new(0),
                establishment_duration_sum_ms: AtomicU64::new(0),
                establishment_duration_count: AtomicU64::new(0),
                transition_duration_sum_ms: AtomicU64::new(0),
                transition_duration_count: AtomicU64::new(0),
            }),
        }
    }

    // Counters

    /// Increment session establishment counter (success)
    pub fn inc_sessions_established(&self) {
        self.inner
            .sessions_established_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Increment session failure counter
    pub fn inc_sessions_failed(&self) {
        self.inner
            .sessions_failed_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Increment state transition counter
    pub fn inc_state_transitions(&self) {
        self.inner
            .state_transitions_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error counter
    pub fn inc_errors(&self) {
        self.inner.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    // Gauges

    /// Set active session count
    pub fn set_active_sessions(&self, count: u64) {
        self.inner.active_sessions.store(count, Ordering::Relaxed);
    }

    /// Set pending session count
    pub fn set_pending_sessions(&self, count: u64) {
        self.inner
            .pending_sessions
            .store(count, Ordering::Relaxed);
    }

    /// Set suspended session count
    pub fn set_suspended_sessions(&self, count: u64) {
        self.inner
            .suspended_sessions
            .store(count, Ordering::Relaxed);
    }

    // Histograms

    /// Record session establishment duration
    pub fn record_establishment_duration(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        self.inner
            .establishment_duration_sum_ms
            .fetch_add(ms, Ordering::Relaxed);
        self.inner
            .establishment_duration_count
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record state transition duration
    pub fn record_transition_duration(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        self.inner
            .transition_duration_sum_ms
            .fetch_add(ms, Ordering::Relaxed);
        self.inner
            .transition_duration_count
            .fetch_add(1, Ordering::Relaxed);
    }

    // Accessors for monitoring/testing

    /// Get total sessions established
    pub fn sessions_established_total(&self) -> u64 {
        self.inner
            .sessions_established_total
            .load(Ordering::Relaxed)
    }

    /// Get total sessions failed
    pub fn sessions_failed_total(&self) -> u64 {
        self.inner.sessions_failed_total.load(Ordering::Relaxed)
    }

    /// Get total state transitions
    pub fn state_transitions_total(&self) -> u64 {
        self.inner
            .state_transitions_total
            .load(Ordering::Relaxed)
    }

    /// Get total errors
    pub fn errors_total(&self) -> u64 {
        self.inner.errors_total.load(Ordering::Relaxed)
    }

    /// Get active session count
    pub fn active_sessions(&self) -> u64 {
        self.inner.active_sessions.load(Ordering::Relaxed)
    }

    /// Get pending session count
    pub fn pending_sessions(&self) -> u64 {
        self.inner.pending_sessions.load(Ordering::Relaxed)
    }

    /// Get suspended session count
    pub fn suspended_sessions(&self) -> u64 {
        self.inner.suspended_sessions.load(Ordering::Relaxed)
    }

    /// Get average establishment duration in milliseconds
    pub fn avg_establishment_duration_ms(&self) -> f64 {
        let sum = self
            .inner
            .establishment_duration_sum_ms
            .load(Ordering::Relaxed);
        let count = self
            .inner
            .establishment_duration_count
            .load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            sum as f64 / count as f64
        }
    }

    /// Get average transition duration in milliseconds
    pub fn avg_transition_duration_ms(&self) -> f64 {
        let sum = self
            .inner
            .transition_duration_sum_ms
            .load(Ordering::Relaxed);
        let count = self
            .inner
            .transition_duration_count
            .load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            sum as f64 / count as f64
        }
    }

    /// Calculate session establishment success rate (0.0 - 1.0)
    pub fn establishment_success_rate(&self) -> f64 {
        let success = self.sessions_established_total();
        let failure = self.sessions_failed_total();
        let total = success + failure;
        if total == 0 {
            1.0
        } else {
            success as f64 / total as f64
        }
    }

    /// Reset all metrics (for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        self.inner
            .sessions_established_total
            .store(0, Ordering::Relaxed);
        self.inner
            .sessions_failed_total
            .store(0, Ordering::Relaxed);
        self.inner
            .state_transitions_total
            .store(0, Ordering::Relaxed);
        self.inner.errors_total.store(0, Ordering::Relaxed);
        self.inner.active_sessions.store(0, Ordering::Relaxed);
        self.inner.pending_sessions.store(0, Ordering::Relaxed);
        self.inner.suspended_sessions.store(0, Ordering::Relaxed);
        self.inner
            .establishment_duration_sum_ms
            .store(0, Ordering::Relaxed);
        self.inner
            .establishment_duration_count
            .store(0, Ordering::Relaxed);
        self.inner
            .transition_duration_sum_ms
            .store(0, Ordering::Relaxed);
        self.inner
            .transition_duration_count
            .store(0, Ordering::Relaxed);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counters() {
        let metrics = Metrics::new();

        assert_eq!(metrics.sessions_established_total(), 0);
        metrics.inc_sessions_established();
        assert_eq!(metrics.sessions_established_total(), 1);

        metrics.inc_sessions_failed();
        metrics.inc_sessions_failed();
        assert_eq!(metrics.sessions_failed_total(), 2);
    }

    #[test]
    fn test_gauges() {
        let metrics = Metrics::new();

        metrics.set_active_sessions(100);
        assert_eq!(metrics.active_sessions(), 100);

        metrics.set_pending_sessions(50);
        assert_eq!(metrics.pending_sessions(), 50);

        metrics.set_suspended_sessions(10);
        assert_eq!(metrics.suspended_sessions(), 10);
    }

    #[test]
    fn test_histograms() {
        let metrics = Metrics::new();

        metrics.record_establishment_duration(Duration::from_millis(100));
        metrics.record_establishment_duration(Duration::from_millis(200));
        metrics.record_establishment_duration(Duration::from_millis(300));

        assert_eq!(metrics.avg_establishment_duration_ms(), 200.0);
    }

    #[test]
    fn test_success_rate() {
        let metrics = Metrics::new();

        // No sessions
        assert_eq!(metrics.establishment_success_rate(), 1.0);

        // 80% success rate
        for _ in 0..8 {
            metrics.inc_sessions_established();
        }
        for _ in 0..2 {
            metrics.inc_sessions_failed();
        }

        assert_eq!(metrics.establishment_success_rate(), 0.8);
    }

    #[test]
    fn test_concurrent_updates() {
        use std::thread;

        let metrics = Metrics::new();
        let mut handles = vec![];

        // Spawn 10 threads, each incrementing counters 100 times
        for _ in 0..10 {
            let m = metrics.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    m.inc_sessions_established();
                    m.inc_state_transitions();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(metrics.sessions_established_total(), 1000);
        assert_eq!(metrics.state_transitions_total(), 1000);
    }
}
