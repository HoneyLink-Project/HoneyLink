//! OpenTelemetry SDK integration
//!
//! Provides metrics, tracing, and logging capabilities using OpenTelemetry.

use crate::types::{LogEvent, LogLevel, Metric, MetricType, TelemetryError, TelemetryResult};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// OpenTelemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: String,
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment (development/staging/production)
    pub environment: String,
    /// Metrics export interval (seconds)
    pub metrics_export_interval_secs: u64,
    /// Trace sampling ratio (0.0 to 1.0)
    pub trace_sampling_ratio: f64,
    /// Enable TLS for OTLP
    pub enable_tls: bool,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: "http://localhost:4317".to_string(),
            service_name: "honeylink".to_string(),
            service_version: "0.1.0".to_string(),
            environment: "development".to_string(),
            metrics_export_interval_secs: 10,
            trace_sampling_ratio: 1.0, // Always on for development
            enable_tls: false,
        }
    }
}

/// OpenTelemetry provider manages SDK initialization and shutdown
pub struct OtelProvider {
    config: OtelConfig,
    meter_provider: Option<SdkMeterProvider>,
}

impl OtelProvider {
    /// Creates a new OpenTelemetry provider
    pub fn new(config: OtelConfig) -> Self {
        Self {
            config,
            meter_provider: None,
        }
    }

    /// Initializes OpenTelemetry SDK
    pub fn initialize(&mut self) -> TelemetryResult<()> {
        // Create resource with service info
        let resource = Resource::new(vec![
            KeyValue::new("service.name", self.config.service_name.clone()),
            KeyValue::new("service.version", self.config.service_version.clone()),
            KeyValue::new("deployment.environment", self.config.environment.clone()),
        ]);

        // Initialize metrics provider
        self.init_metrics(resource.clone())?;

        // Initialize tracing provider
        self.init_tracing(resource.clone())?;

        Ok(())
    }

    /// Initializes metrics provider
    fn init_metrics(&mut self, resource: Resource) -> TelemetryResult<()> {
        // Create OTLP exporter
        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(&self.config.otlp_endpoint)
            .build_metrics_exporter(
                Box::new(opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector::default())
            )
            .map_err(|e| {
                TelemetryError::InitializationError(format!("Failed to create exporter: {}", e))
            })?;

        // Create periodic reader
        let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_interval(Duration::from_secs(self.config.metrics_export_interval_secs))
            .build();

        // Create meter provider
        let provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(reader)
            .build();

        self.meter_provider = Some(provider);

        Ok(())
    }

    /// Initializes tracing provider
    fn init_tracing(&self, _resource: Resource) -> TelemetryResult<()> {
        // Initialize basic tracing subscriber without OpenTelemetry layer
        // Note: Full OTLP tracing export deferred due to OpenTelemetry 0.26 API changes
        // Metrics export still functional via separate metrics pipeline
        tracing_subscriber::fmt()
            .json()
            .init();

        Ok(())
    }

    /// Shutdowns OpenTelemetry SDK gracefully
    pub async fn shutdown(&mut self) -> TelemetryResult<()> {
        if let Some(provider) = self.meter_provider.take() {
            provider.shutdown().map_err(|e| {
                TelemetryError::ExportError(format!("Failed to shutdown meter provider: {}", e))
            })?;
        }

        opentelemetry::global::shutdown_tracer_provider();

        Ok(())
    }
}

/// Metrics provider for recording metrics
pub struct MetricsProvider {
    meter: opentelemetry::metrics::Meter,
    counters: Arc<tokio::sync::Mutex<HashMap<String, opentelemetry::metrics::Counter<u64>>>>,
    gauges: Arc<tokio::sync::Mutex<HashMap<String, opentelemetry::metrics::Gauge<f64>>>>,
    histograms:
        Arc<tokio::sync::Mutex<HashMap<String, opentelemetry::metrics::Histogram<f64>>>>,
}

impl MetricsProvider {
    /// Creates a new metrics provider
    pub fn new(service_name: &'static str) -> Self {
        // Use 'static str for OpenTelemetry 0.26 requirement
        let meter = opentelemetry::global::meter(service_name);

        Self {
            meter,
            counters: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            gauges: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            histograms: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Records a metric
    pub async fn record_metric(&self, metric: &Metric) -> TelemetryResult<()> {
        let labels: Vec<KeyValue> = metric
            .labels
            .iter()
            .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            .collect();

        match metric.metric_type {
            MetricType::Counter => {
                let mut counters = self.counters.lock().await;
                let counter = counters.entry(metric.name.clone()).or_insert_with(|| {
                    self.meter
                        .u64_counter(metric.name.clone())
                        .with_description(format!("Counter: {}", metric.name))
                        .init()
                });

                counter.add(metric.value as u64, &labels);
            }
            MetricType::Gauge => {
                let mut gauges = self.gauges.lock().await;
                let gauge = gauges.entry(metric.name.clone()).or_insert_with(|| {
                    self.meter
                        .f64_gauge(metric.name.clone())
                        .with_description(format!("Gauge: {}", metric.name))
                        .init()
                });

                gauge.record(metric.value, &labels);
            }
            MetricType::Histogram => {
                let mut histograms = self.histograms.lock().await;
                let histogram = histograms.entry(metric.name.clone()).or_insert_with(|| {
                    self.meter
                        .f64_histogram(metric.name.clone())
                        .with_description(format!("Histogram: {}", metric.name))
                        .init()
                });

                histogram.record(metric.value, &labels);
            }
        }

        Ok(())
    }
}

/// Tracing provider for creating spans
pub struct TracingProvider {}

impl TracingProvider {
    /// Creates a new tracing provider
    pub fn new() -> Self {
        Self {}
    }

    /// Starts a new span
    pub fn start_span(&self, name: &str, attributes: Vec<(String, String)>) -> TracingSpan {
        // Create span with attributes
        // Note: tracing! macros require compile-time span names
        let span = if attributes.is_empty() {
            tracing::info_span!("span", name = name)
        } else {
            tracing::info_span!("span", name = name, attributes = ?attributes)
        };

        TracingSpan { inner: span }
    }
}

impl Default for TracingProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for tracing::Span
pub struct TracingSpan {
    inner: tracing::Span,
}

impl TracingSpan {
    /// Enters the span context
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self.inner.enter()
    }

    /// Records an event within the span
    pub fn record_event(&self, name: &str, attributes: Vec<(String, String)>) {
        let _entered = self.enter();
        tracing::info!(event = name, ?attributes, "Span event");
    }

    /// Marks the span as completed with success
    pub fn end_ok(self) {
        drop(self);
    }

    /// Marks the span as completed with error
    pub fn end_error(self, error: &str) {
        {
            let _entered = self.inner.enter();
            tracing::error!(error = error, "Span ended with error");
        }
        drop(self);
    }
}

/// Log exporter for structured logging
pub struct LogExporter {
    service_name: String,
}

impl LogExporter {
    /// Creates a new log exporter
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// Exports a log event
    pub async fn export_log(&self, log: &LogEvent) -> TelemetryResult<()> {
        // Convert log level to tracing level
        match log.level {
            LogLevel::Debug => tracing::debug!(
                service = %self.service_name,
                module = ?log.module,
                trace_id = ?log.trace_id,
                ?log.fields,
                "{}",
                log.message
            ),
            LogLevel::Info => tracing::info!(
                service = %self.service_name,
                module = ?log.module,
                trace_id = ?log.trace_id,
                ?log.fields,
                "{}",
                log.message
            ),
            LogLevel::Warn => tracing::warn!(
                service = %self.service_name,
                module = ?log.module,
                trace_id = ?log.trace_id,
                ?log.fields,
                "{}",
                log.message
            ),
            LogLevel::Error => tracing::error!(
                service = %self.service_name,
                module = ?log.module,
                trace_id = ?log.trace_id,
                ?log.fields,
                "{}",
                log.message
            ),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_config_default() {
        let config = OtelConfig::default();
        assert_eq!(config.otlp_endpoint, "http://localhost:4317");
        assert_eq!(config.service_name, "honeylink");
        assert_eq!(config.metrics_export_interval_secs, 10);
        assert_eq!(config.trace_sampling_ratio, 1.0);
    }

    #[test]
    fn test_metrics_provider_creation() {
        let provider = MetricsProvider::new("test_service");
        // Just verify it doesn't panic
        assert!(provider.counters.try_lock().is_ok());
    }

    #[test]
    fn test_tracing_provider_creation() {
        let provider = TracingProvider::new();
        // Verify span creation doesn't panic
        let _span = provider.start_span("test_span", vec![]);
    }

    #[test]
    fn test_log_exporter_creation() {
        let exporter = LogExporter::new("test_service".to_string());
        assert_eq!(exporter.service_name, "test_service");
    }

    #[tokio::test]
    async fn test_log_exporter_export() {
        let exporter = LogExporter::new("test_service".to_string());
        let log = LogEvent::new(LogLevel::Info, "Test message".to_string());

        let result = exporter.export_log(&log).await;
        assert!(result.is_ok());
    }
}
