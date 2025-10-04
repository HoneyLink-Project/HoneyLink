//! Alert system for SLI violations
//!
//! Supports PagerDuty and Slack notifications with configurable routing.

use crate::sli::{SliViolation, ThresholdLevel};
use crate::types::{TelemetryError, TelemetryResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// PagerDuty integration key
    pub pagerduty_integration_key: Option<String>,
    /// Slack webhook URL
    pub slack_webhook_url: Option<String>,
    /// Alert routing rules (SLI name -> channel)
    pub routing_rules: HashMap<String, AlertChannel>,
    /// Whether to enable test mode (no actual notifications)
    pub test_mode: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            pagerduty_integration_key: None,
            slack_webhook_url: None,
            routing_rules: HashMap::new(),
            test_mode: true, // Safe default
        }
    }
}

/// Alert channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertChannel {
    /// Send to Slack only
    Slack,
    /// Send to PagerDuty only
    PagerDuty,
    /// Send to both
    Both,
}

/// Alert event sent to external systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    /// Unique alert ID
    pub alert_id: String,
    /// Severity level
    pub severity: ThresholdLevel,
    /// SLI name
    pub sli_name: String,
    /// Current value
    pub current_value: f64,
    /// Threshold that was breached
    pub threshold: f64,
    /// Threshold type
    pub threshold_type: ThresholdLevel,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Trace ID for correlation
    pub trace_id: Option<String>,
    /// Labels (module, environment, etc.)
    pub labels: HashMap<String, String>,
}

impl AlertEvent {
    /// Creates an alert event from an SLI violation
    pub fn from_violation(violation: SliViolation) -> Self {
        let mut labels = HashMap::new();
        labels.insert("module".to_string(), violation.module.clone());
        labels.insert("environment".to_string(), "production".to_string());

        Self {
            alert_id: format!("alert_{}", uuid::Uuid::new_v4()),
            severity: violation.level,
            sli_name: violation.sli_name,
            current_value: violation.current_value,
            threshold: violation.threshold,
            threshold_type: violation.level,
            timestamp: format_system_time(violation.timestamp),
            trace_id: violation.trace_id,
            labels,
        }
    }

    /// Converts to PagerDuty event payload
    fn to_pagerduty_payload(&self) -> serde_json::Value {
        let severity = match self.severity {
            ThresholdLevel::Red => "critical",
            ThresholdLevel::Orange => "error",
            ThresholdLevel::Yellow => "warning",
            ThresholdLevel::Green => "info",
        };

        serde_json::json!({
            "routing_key": "<INTEGRATION_KEY>",
            "event_action": "trigger",
            "dedup_key": self.alert_id,
            "payload": {
                "summary": format!("SLI Violation: {} breached {} threshold", self.sli_name, self.severity),
                "severity": severity,
                "source": self.labels.get("module").unwrap_or(&"unknown".to_string()),
                "timestamp": self.timestamp,
                "custom_details": {
                    "sli_name": self.sli_name,
                    "current_value": self.current_value,
                    "threshold": self.threshold,
                    "trace_id": self.trace_id,
                }
            }
        })
    }

    /// Converts to Slack message payload
    fn to_slack_payload(&self) -> serde_json::Value {
        let color = match self.severity {
            ThresholdLevel::Red => "danger",
            ThresholdLevel::Orange => "warning",
            ThresholdLevel::Yellow => "#FFA500", // Orange color
            ThresholdLevel::Green => "good",
        };

        let emoji = match self.severity {
            ThresholdLevel::Red => ":rotating_light:",
            ThresholdLevel::Orange => ":warning:",
            ThresholdLevel::Yellow => ":large_orange_diamond:",
            ThresholdLevel::Green => ":white_check_mark:",
        };

        serde_json::json!({
            "text": format!("{} *SLI Violation Alert*", emoji),
            "attachments": [{
                "color": color,
                "fields": [
                    {
                        "title": "SLI",
                        "value": self.sli_name,
                        "short": true
                    },
                    {
                        "title": "Severity",
                        "value": format!("{}", self.severity),
                        "short": true
                    },
                    {
                        "title": "Current Value",
                        "value": format!("{:.2}", self.current_value),
                        "short": true
                    },
                    {
                        "title": "Threshold",
                        "value": format!("{:.2}", self.threshold),
                        "short": true
                    },
                    {
                        "title": "Module",
                        "value": self.labels.get("module").unwrap_or(&"unknown".to_string()),
                        "short": true
                    },
                    {
                        "title": "Timestamp",
                        "value": self.timestamp,
                        "short": true
                    }
                ],
                "footer": format!("Alert ID: {}", self.alert_id)
            }]
        })
    }
}

/// Alert manager handles routing and sending alerts
pub struct AlertManager {
    config: AlertConfig,
    http_client: reqwest::Client,
    alert_history: Arc<Mutex<Vec<AlertEvent>>>,
}

impl AlertManager {
    /// Creates a new alert manager
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            alert_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Sends an alert for an SLI violation
    pub async fn send_alert(&self, violation: SliViolation) -> TelemetryResult<()> {
        let alert = AlertEvent::from_violation(violation);

        // Store in history
        {
            let mut history = self.alert_history.lock().unwrap();
            history.push(alert.clone());

            // Keep only last 1000 alerts
            let len = history.len();
            if len > 1000 {
                history.drain(0..len - 1000);
            }
        }

        // Test mode: don't send actual alerts
        if self.config.test_mode {
            log::info!("Test mode: Alert would be sent: {:?}", alert);
            return Ok(());
        }

        // Determine routing
        let channel = self
            .config
            .routing_rules
            .get(&alert.sli_name)
            .copied()
            .unwrap_or_else(|| self.default_channel(alert.severity));

        // Send to appropriate channels
        match channel {
            AlertChannel::Slack => {
                self.send_to_slack(&alert).await?;
            }
            AlertChannel::PagerDuty => {
                self.send_to_pagerduty(&alert).await?;
            }
            AlertChannel::Both => {
                self.send_to_slack(&alert).await?;
                self.send_to_pagerduty(&alert).await?;
            }
        }

        Ok(())
    }

    /// Determines default channel based on severity
    fn default_channel(&self, severity: ThresholdLevel) -> AlertChannel {
        match severity {
            ThresholdLevel::Red => AlertChannel::Both,
            ThresholdLevel::Orange => AlertChannel::PagerDuty,
            ThresholdLevel::Yellow => AlertChannel::Slack,
            ThresholdLevel::Green => AlertChannel::Slack,
        }
    }

    /// Sends alert to PagerDuty
    async fn send_to_pagerduty(&self, alert: &AlertEvent) -> TelemetryResult<()> {
        let integration_key = self
            .config
            .pagerduty_integration_key
            .as_ref()
            .ok_or_else(|| {
                TelemetryError::AlertError("PagerDuty integration key not configured".to_string())
            })?;

        let mut payload = alert.to_pagerduty_payload();
        payload["routing_key"] = serde_json::Value::String(integration_key.clone());

        let response = self
            .http_client
            .post("https://events.pagerduty.com/v2/enqueue")
            .json(&payload)
            .send()
            .await
            .map_err(|e| TelemetryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TelemetryError::AlertError(format!(
                "PagerDuty API error: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Sends alert to Slack
    async fn send_to_slack(&self, alert: &AlertEvent) -> TelemetryResult<()> {
        let webhook_url = self.config.slack_webhook_url.as_ref().ok_or_else(|| {
            TelemetryError::AlertError("Slack webhook URL not configured".to_string())
        })?;

        let payload = alert.to_slack_payload();

        let response = self
            .http_client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| TelemetryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TelemetryError::AlertError(format!(
                "Slack API error: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Returns alert history (for testing/debugging)
    pub fn get_history(&self) -> Vec<AlertEvent> {
        self.alert_history.lock().unwrap().clone()
    }

    /// Clears alert history
    pub fn clear_history(&self) {
        self.alert_history.lock().unwrap().clear();
    }
}

/// Formats SystemTime as ISO 8601 string
fn format_system_time(time: SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time before UNIX epoch");
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    // Simple ISO 8601 format: YYYY-MM-DDTHH:MM:SS.sssZ
    // For production, use chrono crate
    format!("{}T{}.{:09}Z", secs / 86400, secs % 86400, nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_violation() -> SliViolation {
        SliViolation {
            sli_name: "test_sli".to_string(),
            module: "test_module".to_string(),
            level: ThresholdLevel::Orange,
            current_value: 550.0,
            threshold: 500.0,
            consecutive_breaches: 3,
            timestamp: SystemTime::now(),
            trace_id: Some("trace123".to_string()),
        }
    }

    #[test]
    fn test_alert_event_creation() {
        let violation = create_test_violation();
        let alert = AlertEvent::from_violation(violation);

        assert_eq!(alert.sli_name, "test_sli");
        assert_eq!(alert.severity, ThresholdLevel::Orange);
        assert_eq!(alert.current_value, 550.0);
        assert_eq!(alert.threshold, 500.0);
        assert!(alert.alert_id.starts_with("alert_"));
    }

    #[test]
    fn test_pagerduty_payload_format() {
        let violation = create_test_violation();
        let alert = AlertEvent::from_violation(violation);
        let payload = alert.to_pagerduty_payload();

        assert_eq!(payload["event_action"], "trigger");
        assert_eq!(payload["payload"]["severity"], "error"); // Orange -> error
        assert!(payload["payload"]["summary"]
            .as_str()
            .unwrap()
            .contains("test_sli"));
    }

    #[test]
    fn test_slack_payload_format() {
        let violation = create_test_violation();
        let alert = AlertEvent::from_violation(violation);
        let payload = alert.to_slack_payload();

        assert!(payload["text"].as_str().unwrap().contains("SLI Violation"));
        assert_eq!(payload["attachments"][0]["color"], "warning");
    }

    #[test]
    fn test_alert_config_default() {
        let config = AlertConfig::default();
        assert!(config.test_mode);
        assert!(config.pagerduty_integration_key.is_none());
        assert!(config.slack_webhook_url.is_none());
    }

    #[test]
    fn test_default_channel_routing() {
        let config = AlertConfig::default();
        let manager = AlertManager::new(config);

        assert_eq!(
            manager.default_channel(ThresholdLevel::Red),
            AlertChannel::Both
        );
        assert_eq!(
            manager.default_channel(ThresholdLevel::Orange),
            AlertChannel::PagerDuty
        );
        assert_eq!(
            manager.default_channel(ThresholdLevel::Yellow),
            AlertChannel::Slack
        );
    }

    #[tokio::test]
    async fn test_alert_manager_test_mode() {
        let config = AlertConfig {
            test_mode: true,
            ..Default::default()
        };
        let manager = AlertManager::new(config);
        let violation = create_test_violation();

        // Should succeed in test mode even without credentials
        let result = manager.send_alert(violation).await;
        assert!(result.is_ok());

        // Should have one entry in history
        let history = manager.get_history();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_alert_history_management() {
        let manager = AlertManager::new(AlertConfig::default());

        // Initially empty
        assert_eq!(manager.get_history().len(), 0);

        // Add alerts
        for _ in 0..5 {
            let violation = create_test_violation();
            let alert = AlertEvent::from_violation(violation);
            manager.alert_history.lock().unwrap().push(alert);
        }

        assert_eq!(manager.get_history().len(), 5);

        manager.clear_history();
        assert_eq!(manager.get_history().len(), 0);
    }
}
