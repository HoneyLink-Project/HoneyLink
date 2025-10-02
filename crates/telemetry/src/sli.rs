//! SLI (Service Level Indicator) and SLO (Service Level Objective) monitoring
//!
//! This module implements the SLI/SLO framework according to spec/testing/metrics.md
//! with three-tier alerting (Yellow/Orange/Red).

use crate::types::{Metric, TelemetryError, TelemetryResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// SLI (Service Level Indicator) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliDefinition {
    /// SLI name (e.g., "session_establishment_latency_p95")
    pub name: String,
    /// Target module
    pub module: String,
    /// Yellow threshold (warning)
    pub yellow_threshold: f64,
    /// Orange threshold (requires action)
    pub orange_threshold: f64,
    /// Red threshold (critical, blocks release)
    pub red_threshold: f64,
    /// SLO target value
    pub slo_target: f64,
    /// Evaluation window duration
    pub evaluation_window: Duration,
    /// Consecutive breaches required for alert
    pub consecutive_breaches_required: u8,
    /// Whether higher values are better (false for latency, true for success rate)
    pub higher_is_better: bool,
}

impl SliDefinition {
    /// Creates the session establishment latency SLI
    pub fn session_establishment_latency() -> Self {
        Self {
            name: "session_establishment_latency_p95".to_string(),
            module: "session-orchestrator".to_string(),
            yellow_threshold: 400.0,
            orange_threshold: 500.0,
            red_threshold: 800.0,
            slo_target: 500.0,
            evaluation_window: Duration::from_secs(300), // 5 minutes
            consecutive_breaches_required: 3,
            higher_is_better: false,
        }
    }

    /// Creates the policy update latency SLI
    pub fn policy_update_latency() -> Self {
        Self {
            name: "policy_update_latency_p95".to_string(),
            module: "policy-engine".to_string(),
            yellow_threshold: 250.0,
            orange_threshold: 300.0,
            red_threshold: 500.0,
            slo_target: 300.0,
            evaluation_window: Duration::from_secs(300),
            consecutive_breaches_required: 3,
            higher_is_better: false,
        }
    }

    /// Creates the encryption latency SLI
    pub fn encryption_latency() -> Self {
        Self {
            name: "encryption_latency_p95".to_string(),
            module: "crypto".to_string(),
            yellow_threshold: 15.0,
            orange_threshold: 20.0,
            red_threshold: 50.0,
            slo_target: 20.0,
            evaluation_window: Duration::from_secs(300),
            consecutive_breaches_required: 3,
            higher_is_better: false,
        }
    }

    /// Creates the packet loss rate SLI
    pub fn packet_loss_rate() -> Self {
        Self {
            name: "packet_loss_rate".to_string(),
            module: "transport".to_string(),
            yellow_threshold: 0.05,
            orange_threshold: 0.10,
            red_threshold: 0.20,
            slo_target: 0.01,
            evaluation_window: Duration::from_secs(300),
            consecutive_breaches_required: 3,
            higher_is_better: false,
        }
    }

    /// Creates the QoS packet drop rate SLI
    pub fn qos_packet_drop_rate() -> Self {
        Self {
            name: "qos_packet_drop_rate".to_string(),
            module: "qos-scheduler".to_string(),
            yellow_threshold: 0.005,
            orange_threshold: 0.01,
            red_threshold: 0.05,
            slo_target: 0.01,
            evaluation_window: Duration::from_secs(300),
            consecutive_breaches_required: 3,
            higher_is_better: false,
        }
    }

    /// Returns all default SLI definitions
    pub fn all_defaults() -> Vec<Self> {
        vec![
            Self::session_establishment_latency(),
            Self::policy_update_latency(),
            Self::encryption_latency(),
            Self::packet_loss_rate(),
            Self::qos_packet_drop_rate(),
        ]
    }

    /// Evaluates a metric value against thresholds
    pub fn evaluate(&self, value: f64) -> ThresholdLevel {
        if self.higher_is_better {
            // For success rates, lower values are worse
            if value < self.red_threshold {
                ThresholdLevel::Red
            } else if value < self.orange_threshold {
                ThresholdLevel::Orange
            } else if value < self.yellow_threshold {
                ThresholdLevel::Yellow
            } else {
                ThresholdLevel::Green
            }
        } else {
            // For latency/loss rates, higher values are worse
            if value >= self.red_threshold {
                ThresholdLevel::Red
            } else if value >= self.orange_threshold {
                ThresholdLevel::Orange
            } else if value >= self.yellow_threshold {
                ThresholdLevel::Yellow
            } else {
                ThresholdLevel::Green
            }
        }
    }
}

/// Threshold level for alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ThresholdLevel {
    Green = 0,
    Yellow = 1,
    Orange = 2,
    Red = 3,
}

impl std::fmt::Display for ThresholdLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThresholdLevel::Green => write!(f, "Green"),
            ThresholdLevel::Yellow => write!(f, "Yellow"),
            ThresholdLevel::Orange => write!(f, "Orange"),
            ThresholdLevel::Red => write!(f, "Red"),
        }
    }
}

/// Breach detection state for a single SLI
#[derive(Debug, Clone)]
struct BreachState {
    consecutive_breaches: u8,
    last_breach_time: Option<SystemTime>,
    last_level: ThresholdLevel,
}

impl BreachState {
    fn new() -> Self {
        Self {
            consecutive_breaches: 0,
            last_breach_time: None,
            last_level: ThresholdLevel::Green,
        }
    }

    /// Records a new evaluation result
    fn record(&mut self, level: ThresholdLevel, now: SystemTime) -> bool {
        if level > ThresholdLevel::Green {
            if level == self.last_level {
                self.consecutive_breaches += 1;
            } else {
                self.consecutive_breaches = 1;
            }
            self.last_breach_time = Some(now);
            self.last_level = level;
            true
        } else {
            // Reset on Green
            self.consecutive_breaches = 0;
            self.last_breach_time = None;
            self.last_level = ThresholdLevel::Green;
            false
        }
    }
}

/// SLI monitor tracks metrics and evaluates against SLO
pub struct SliMonitor {
    definitions: HashMap<String, SliDefinition>,
    breach_states: Arc<Mutex<HashMap<String, BreachState>>>,
    metric_windows: Arc<Mutex<HashMap<String, VecDeque<(SystemTime, f64)>>>>,
}

impl SliMonitor {
    /// Creates a new SLI monitor with default SLI definitions
    pub fn new() -> Self {
        let definitions: HashMap<String, SliDefinition> = SliDefinition::all_defaults()
            .into_iter()
            .map(|def| (def.name.clone(), def))
            .collect();

        Self {
            definitions,
            breach_states: Arc::new(Mutex::new(HashMap::new())),
            metric_windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Adds a custom SLI definition
    pub fn add_sli(&mut self, sli: SliDefinition) {
        self.definitions.insert(sli.name.clone(), sli);
    }

    /// Records a metric and evaluates against SLI
    pub fn record_metric(&self, metric: &Metric) -> TelemetryResult<Option<SliViolation>> {
        // Find matching SLI definition
        let sli_def = match self.definitions.get(&metric.name) {
            Some(def) => def,
            None => return Ok(None), // Not an SLI metric
        };

        let now = SystemTime::now();

        // Add to metric window
        {
            let mut windows = self.metric_windows.lock().unwrap();
            let window = windows.entry(metric.name.clone()).or_insert_with(VecDeque::new);
            window.push_back((now, metric.value));

            // Remove old entries outside evaluation window
            let cutoff = now - sli_def.evaluation_window;
            while let Some((time, _)) = window.front() {
                if *time < cutoff {
                    window.pop_front();
                } else {
                    break;
                }
            }
        }

        // Calculate P95 (or average for rates)
        let aggregated_value = if metric.name.contains("_p95") {
            self.calculate_p95(&metric.name)?
        } else {
            self.calculate_average(&metric.name)?
        };

        // Evaluate against thresholds
        let level = sli_def.evaluate(aggregated_value);

        // Update breach state
        let mut states = self.breach_states.lock().unwrap();
        let state = states.entry(metric.name.clone()).or_insert_with(BreachState::new);

        state.record(level, now);

        // Generate violation if consecutive breaches threshold reached
        if level > ThresholdLevel::Green
            && state.consecutive_breaches >= sli_def.consecutive_breaches_required
        {
            Ok(Some(SliViolation {
                sli_name: sli_def.name.clone(),
                module: sli_def.module.clone(),
                level,
                current_value: aggregated_value,
                threshold: match level {
                    ThresholdLevel::Yellow => sli_def.yellow_threshold,
                    ThresholdLevel::Orange => sli_def.orange_threshold,
                    ThresholdLevel::Red => sli_def.red_threshold,
                    ThresholdLevel::Green => sli_def.slo_target,
                },
                consecutive_breaches: state.consecutive_breaches,
                timestamp: now,
                trace_id: metric.trace_id.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Calculates the 95th percentile of values in the window
    fn calculate_p95(&self, metric_name: &str) -> TelemetryResult<f64> {
        let windows = self.metric_windows.lock().unwrap();
        let window = windows.get(metric_name).ok_or_else(|| {
            TelemetryError::MetricRecordError(format!("No window for metric: {}", metric_name))
        })?;

        if window.is_empty() {
            return Err(TelemetryError::MetricRecordError(
                "Empty metric window".to_string(),
            ));
        }

        let mut values: Vec<f64> = window.iter().map(|(_, v)| *v).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((values.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
        Ok(values[index])
    }

    /// Calculates the average of values in the window
    fn calculate_average(&self, metric_name: &str) -> TelemetryResult<f64> {
        let windows = self.metric_windows.lock().unwrap();
        let window = windows.get(metric_name).ok_or_else(|| {
            TelemetryError::MetricRecordError(format!("No window for metric: {}", metric_name))
        })?;

        if window.is_empty() {
            return Err(TelemetryError::MetricRecordError(
                "Empty metric window".to_string(),
            ));
        }

        let sum: f64 = window.iter().map(|(_, v)| v).sum();
        Ok(sum / window.len() as f64)
    }
}

impl Default for SliMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// SLI violation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliViolation {
    pub sli_name: String,
    pub module: String,
    pub level: ThresholdLevel,
    pub current_value: f64,
    pub threshold: f64,
    pub consecutive_breaches: u8,
    pub timestamp: SystemTime,
    pub trace_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sli_definition_defaults() {
        let slis = SliDefinition::all_defaults();
        assert_eq!(slis.len(), 5);

        let session_sli = &slis[0];
        assert_eq!(session_sli.name, "session_establishment_latency_p95");
        assert_eq!(session_sli.yellow_threshold, 400.0);
        assert_eq!(session_sli.orange_threshold, 500.0);
        assert_eq!(session_sli.red_threshold, 800.0);
    }

    #[test]
    fn test_threshold_evaluation_latency() {
        let sli = SliDefinition::session_establishment_latency();

        assert_eq!(sli.evaluate(350.0), ThresholdLevel::Green);
        assert_eq!(sli.evaluate(450.0), ThresholdLevel::Yellow);
        assert_eq!(sli.evaluate(550.0), ThresholdLevel::Orange);
        assert_eq!(sli.evaluate(850.0), ThresholdLevel::Red);
    }

    #[test]
    fn test_threshold_level_ordering() {
        assert!(ThresholdLevel::Green < ThresholdLevel::Yellow);
        assert!(ThresholdLevel::Yellow < ThresholdLevel::Orange);
        assert!(ThresholdLevel::Orange < ThresholdLevel::Red);
    }

    #[test]
    fn test_breach_state_consecutive() {
        let mut state = BreachState::new();
        let now = SystemTime::now();

        state.record(ThresholdLevel::Yellow, now);
        assert_eq!(state.consecutive_breaches, 1);

        state.record(ThresholdLevel::Yellow, now);
        assert_eq!(state.consecutive_breaches, 2);

        state.record(ThresholdLevel::Yellow, now);
        assert_eq!(state.consecutive_breaches, 3);

        // Different level resets counter
        state.record(ThresholdLevel::Orange, now);
        assert_eq!(state.consecutive_breaches, 1);

        // Green resets everything
        state.record(ThresholdLevel::Green, now);
        assert_eq!(state.consecutive_breaches, 0);
    }

    #[test]
    fn test_sli_monitor_creation() {
        let monitor = SliMonitor::new();
        assert_eq!(monitor.definitions.len(), 5);
    }

    #[test]
    fn test_sli_monitor_custom_sli() {
        let mut monitor = SliMonitor::new();
        let custom_sli = SliDefinition {
            name: "custom_metric".to_string(),
            module: "test".to_string(),
            yellow_threshold: 10.0,
            orange_threshold: 20.0,
            red_threshold: 30.0,
            slo_target: 15.0,
            evaluation_window: Duration::from_secs(60),
            consecutive_breaches_required: 2,
            higher_is_better: false,
        };

        monitor.add_sli(custom_sli);
        assert_eq!(monitor.definitions.len(), 6);
    }
}
