//! Telemetry collector
//!
//! Unified API for recording metrics, traces, and logs.

use crate::alert::{AlertConfig, AlertManager};
use crate::otel::{LogExporter, MetricsProvider, OtelConfig, OtelProvider, TracingProvider};
use crate::sli::SliMonitor;
use crate::storage::{StorageConfig, StoragePipeline};
use crate::types::{LogEvent, Metric, TelemetryError, TelemetryResult};
use std::sync::Arc;

/// Telemetry collector configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub otel: OtelConfig,
    pub storage: StorageConfig,
    pub alert: AlertConfig,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            otel: OtelConfig::default(),
            storage: StorageConfig::default(),
            alert: AlertConfig::default(),
        }
    }
}

/// Telemetry collector manages all telemetry operations
pub struct TelemetryCollector {
    enabled: bool,
    otel_provider: Option<OtelProvider>,
    metrics_provider: Option<MetricsProvider>,
    tracing_provider: Option<TracingProvider>,
    log_exporter: Option<LogExporter>,
    sli_monitor: Arc<SliMonitor>,
    storage_pipeline: Arc<StoragePipeline>,
    alert_manager: Arc<AlertManager>,
}

impl TelemetryCollector {
    /// Creates a new telemetry collector
    pub fn new() -> Self {
        Self {
            enabled: true,
            otel_provider: None,
            metrics_provider: None,
            tracing_provider: None,
            log_exporter: None,
            sli_monitor: Arc::new(SliMonitor::new()),
            storage_pipeline: Arc::new(StoragePipeline::new(StorageConfig::default())),
            alert_manager: Arc::new(AlertManager::new(AlertConfig::default())),
        }
    }

    /// Creates a telemetry collector with custom configuration
    pub fn with_config(config: TelemetryConfig) -> Self {
        Self {
            enabled: true,
            otel_provider: None,
            metrics_provider: None,
            tracing_provider: None,
            log_exporter: None,
            sli_monitor: Arc::new(SliMonitor::new()),
            storage_pipeline: Arc::new(StoragePipeline::new(config.storage)),
            alert_manager: Arc::new(AlertManager::new(config.alert)),
        }
    }

    /// Initializes the telemetry collector
    pub async fn initialize(&mut self, config: TelemetryConfig) -> TelemetryResult<()> {
        // Initialize OpenTelemetry
        let mut otel_provider = OtelProvider::new(config.otel.clone());
        otel_provider.initialize()?;
        self.otel_provider = Some(otel_provider);

        // Leak service_name to get 'static lifetime required by OpenTelemetry 0.26 APIs
        let service_name: &'static str = Box::leak(config.otel.service_name.clone().into_boxed_str());

        // Initialize providers
        self.metrics_provider = Some(MetricsProvider::new(service_name));
        self.tracing_provider = Some(TracingProvider::new());
        self.log_exporter = Some(LogExporter::new(service_name.to_string()));

        // Start storage batch writer
        self.storage_pipeline.start_batch_writer();

        log::info!("Telemetry collector initialized");

        Ok(())
    }

    /// Records a metric
    pub async fn record_metric(&self, metric: Metric) -> TelemetryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Record to OpenTelemetry
        if let Some(ref provider) = self.metrics_provider {
            provider.record_metric(&metric).await?;
        }

        // Check against SLI thresholds
        if let Some(violation) = self.sli_monitor.record_metric(&metric)? {
            // Send alert
            self.alert_manager.send_alert(violation).await?;
        }

        // Add to storage pipeline
        self.storage_pipeline.add_metric(metric).await?;

        Ok(())
    }

    /// Starts a new tracing span
    pub fn start_span(
        &self,
        name: &str,
        attributes: Vec<(String, String)>,
    ) -> TelemetryResult<crate::otel::TracingSpan> {
        if !self.enabled {
            return Err(TelemetryError::MetricRecordError(
                "Telemetry disabled".to_string(),
            ));
        }

        let provider = self.tracing_provider.as_ref().ok_or_else(|| {
            TelemetryError::MetricRecordError("Tracing provider not initialized".to_string())
        })?;

        Ok(provider.start_span(name, attributes))
    }

    /// Exports a log event
    pub async fn log(&self, log: LogEvent) -> TelemetryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let exporter = self.log_exporter.as_ref().ok_or_else(|| {
            TelemetryError::MetricRecordError("Log exporter not initialized".to_string())
        })?;

        exporter.export_log(&log).await?;

        Ok(())
    }

    /// Enables telemetry collection
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables telemetry collection
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Returns whether telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns storage pipeline statistics
    pub async fn get_stats(&self) -> crate::storage::BufferStats {
        self.storage_pipeline.get_stats().await
    }

    /// Shutdowns the telemetry collector gracefully
    pub async fn shutdown(&mut self) -> TelemetryResult<()> {
        if let Some(ref mut provider) = self.otel_provider {
            provider.shutdown().await?;
        }

        log::info!("Telemetry collector shutdown");

        Ok(())
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LogLevel, MetricType};

    #[test]
    fn test_collector_creation() {
        let collector = TelemetryCollector::new();
        assert!(collector.is_enabled());
    }

    #[test]
    fn test_collector_enable_disable() {
        let mut collector = TelemetryCollector::new();

        collector.disable();
        assert!(!collector.is_enabled());

        collector.enable();
        assert!(collector.is_enabled());
    }

    #[test]
    fn test_collector_with_config() {
        let config = TelemetryConfig::default();
        let collector = TelemetryCollector::with_config(config);
        assert!(collector.is_enabled());
    }

    #[tokio::test]
    async fn test_record_metric_when_disabled() {
        let mut collector = TelemetryCollector::new();
        collector.disable();

        let metric = Metric::counter("test".to_string(), 1.0, vec![]);
        let result = collector.record_metric(metric).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_when_disabled() {
        let mut collector = TelemetryCollector::new();
        collector.disable();

        let log = LogEvent::new(LogLevel::Info, "Test".to_string());
        let result = collector.log(log).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let collector = TelemetryCollector::new();
        let stats = collector.get_stats().await;

        assert_eq!(stats.buffered_metrics, 0);
        assert_eq!(stats.total_dropped, 0);
    }
}
