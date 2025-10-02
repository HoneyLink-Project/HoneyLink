//! Common types for telemetry module

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use thiserror::Error;

/// Telemetry errors
#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("Failed to initialize OpenTelemetry: {0}")]
    InitializationError(String),

    #[error("Failed to export telemetry data: {0}")]
    ExportError(String),

    #[error("Failed to record metric: {0}")]
    MetricRecordError(String),

    #[error("Failed to create span: {0}")]
    SpanCreationError(String),

    #[error("Failed to store telemetry data: {0}")]
    StorageError(String),

    #[error("Failed to send alert: {0}")]
    AlertError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("PII detection error: {0}")]
    PiiDetectionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Result type for telemetry operations
pub type TelemetryResult<T> = Result<T, TelemetryError>;

/// Metric types according to OpenTelemetry specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,
    /// Value that can go up and down
    Gauge,
    /// Statistical distribution of values
    Histogram,
}

/// Metric value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name (e.g., "session_establishment_duration_seconds")
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Numeric value
    pub value: f64,
    /// Key-value labels
    pub labels: Vec<(String, String)>,
    /// Timestamp (Unix nanoseconds)
    pub timestamp_ns: u64,
    /// Optional trace ID for correlation
    pub trace_id: Option<String>,
}

impl Metric {
    /// Creates a new metric with current timestamp
    pub fn new(
        name: String,
        metric_type: MetricType,
        value: f64,
        labels: Vec<(String, String)>,
    ) -> Self {
        Self {
            name,
            metric_type,
            value,
            labels,
            timestamp_ns: current_time_ns(),
            trace_id: None,
        }
    }

    /// Creates a counter metric
    pub fn counter(name: String, value: f64, labels: Vec<(String, String)>) -> Self {
        Self::new(name, MetricType::Counter, value, labels)
    }

    /// Creates a gauge metric
    pub fn gauge(name: String, value: f64, labels: Vec<(String, String)>) -> Self {
        Self::new(name, MetricType::Gauge, value, labels)
    }

    /// Creates a histogram metric
    pub fn histogram(name: String, value: f64, labels: Vec<(String, String)>) -> Self {
        Self::new(name, MetricType::Histogram, value, labels)
    }

    /// Adds a trace ID for correlation
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

/// Span for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Span ID (16-byte hex string)
    pub span_id: String,
    /// Trace ID (32-byte hex string)
    pub trace_id: String,
    /// Parent span ID (optional)
    pub parent_span_id: Option<String>,
    /// Span name (e.g., "session.establish")
    pub span_name: String,
    /// Start time (Unix nanoseconds)
    pub start_time_ns: u64,
    /// End time (Unix nanoseconds, optional for in-progress spans)
    pub end_time_ns: Option<u64>,
    /// Key-value attributes
    pub attributes: Vec<(String, String)>,
    /// Span events
    pub events: Vec<SpanEvent>,
    /// Span status
    pub status: SpanStatus,
}

/// Span event (log-like entry within a span)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp_ns: u64,
    pub attributes: Vec<(String, String)>,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpanStatus {
    Ok,
    Error,
}

/// Log event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Timestamp (Unix nanoseconds)
    pub timestamp_ns: u64,
    /// Module path
    pub module: Option<String>,
    /// Key-value fields
    pub fields: Vec<(String, String)>,
    /// Trace ID for correlation
    pub trace_id: Option<String>,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogEvent {
    /// Creates a new log event with current timestamp
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            level,
            message,
            timestamp_ns: current_time_ns(),
            module: None,
            fields: Vec::new(),
            trace_id: None,
        }
    }

    /// Adds a module path
    pub fn with_module(mut self, module: String) -> Self {
        self.module = Some(module);
        self
    }

    /// Adds fields
    pub fn with_fields(mut self, fields: Vec<(String, String)>) -> Self {
        self.fields = fields;
        self
    }

    /// Adds a trace ID
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

/// Returns current Unix time in nanoseconds
pub fn current_time_ns() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_nanos() as u64
}

/// Returns current Unix time in milliseconds
pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::counter("test_counter".to_string(), 42.0, vec![]);
        assert_eq!(metric.name, "test_counter");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.value, 42.0);
        assert!(metric.trace_id.is_none());
    }

    #[test]
    fn test_metric_with_trace_id() {
        let metric = Metric::gauge("test_gauge".to_string(), 3.14, vec![])
            .with_trace_id("abc123".to_string());
        assert_eq!(metric.trace_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_log_event_creation() {
        let log = LogEvent::new(LogLevel::Info, "Test message".to_string())
            .with_module("test_module".to_string())
            .with_fields(vec![("key".to_string(), "value".to_string())]);

        assert_eq!(log.level, LogLevel::Info);
        assert_eq!(log.message, "Test message");
        assert_eq!(log.module, Some("test_module".to_string()));
        assert_eq!(log.fields.len(), 1);
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_current_time_functions() {
        let time_ns = current_time_ns();
        let time_ms = current_time_ms();

        // Nanoseconds should be roughly 1,000,000 times milliseconds
        let ratio = time_ns / (time_ms * 1_000_000);
        assert!(ratio >= 1 && ratio <= 2); // Allow some execution time variance
    }
}
