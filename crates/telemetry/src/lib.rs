//! # Telemetry & Insights
//!
//! OpenTelemetry integration for metrics, traces, and logs with SLI/SLO monitoring.
//!
//! ## Features
//! - **Metrics**: Counter, Gauge, Histogram via OpenTelemetry
//! - **Tracing**: Distributed tracing with W3C Trace Context
//! - **Logging**: Structured logging with JSON export
//! - **SLI/SLO**: 5 predefined SLIs with Yellow/Orange/Red alerting
//! - **Alerting**: PagerDuty and Slack integration
//! - **Storage**: TimescaleDB buffering with PII detection
//!
//! ## Usage
//! ```rust,no_run
//! use honeylink_telemetry::{TelemetryCollector, TelemetryConfig};
//! use honeylink_telemetry::types::Metric;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut collector = TelemetryCollector::new();
//!     let config = TelemetryConfig::default();
//!
//!     collector.initialize(config).await.unwrap();
//!
//!     let metric = Metric::counter("requests_total".to_string(), 1.0, vec![]);
//!     collector.record_metric(metric).await.unwrap();
//! }
//! ```

pub mod alert;
pub mod collector;
pub mod crypto_metrics;
pub mod otel;
pub mod sli;
pub mod storage;
pub mod transport_events;
pub mod types;

// Re-exports
pub use collector::{TelemetryCollector, TelemetryConfig};
pub use transport_events::{
    FecStrategyChangeEvent, LinkStateChangeEvent, PacketReceiveFailedEvent, PacketSentEvent,
    PowerModeChangeEvent, QueueDepthWarningEvent, TransportEvent, TransportMetrics,
};
pub use types::{LogEvent, LogLevel, Metric, MetricType, TelemetryError, TelemetryResult};
