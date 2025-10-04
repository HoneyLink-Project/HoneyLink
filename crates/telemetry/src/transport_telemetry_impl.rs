//! Transport telemetry implementation
//!
//! Implements TransportTelemetry trait defined in honeylink-transport.

use async_trait::async_trait;
use honeylink_transport::{LinkQualityMetrics, TransportTelemetry as TransportTelemetryTrait};
use std::sync::Arc;
use crate::{Metric, MetricType, TelemetryCollector};

/// Transport telemetry recorder implementation
///
/// Wraps TelemetryCollector to provide transport-specific metrics.
/// Thread-safe via Arc (TelemetryCollector is already thread-safe).
#[derive(Clone)]
pub struct TransportTelemetryImpl {
    collector: Arc<TelemetryCollector>,
}

impl TransportTelemetryImpl {
    /// Create new transport telemetry recorder
    ///
    /// # Arguments
    /// * `collector` - Shared telemetry collector instance
    pub fn new(collector: Arc<TelemetryCollector>) -> Self {
        Self { collector }
    }
}

#[async_trait]
impl TransportTelemetryTrait for TransportTelemetryImpl {
    fn new_transport_telemetry() -> Self {
        // Default implementation - users should use new() with actual collector
        panic!("Use TransportTelemetryImpl::new(collector) instead of trait default")
    }

    async fn record_packet_loss_rate(
        &self,
        loss_rate: f32,
        physical_layer: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let labels = vec![
            ("physical_layer".to_string(), physical_layer.to_string()),
        ];

        let metric = Metric::new(
            "packet_loss_rate".to_string(),
            MetricType::Gauge,
            (loss_rate * 100.0) as f64, // Convert to percentage
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    async fn record_qos_packet_drop_rate(
        &self,
        drop_rate: f32,
        priority: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let labels = vec![
            ("priority".to_string(), priority.to_string()),
        ];

        let metric = Metric::new(
            "qos_packet_drop_rate".to_string(),
            MetricType::Gauge,
            (drop_rate * 100.0) as f64, // Convert to percentage
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    async fn record_link_quality(
        &self,
        metrics: &LinkQualityMetrics,
        physical_layer: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let labels = vec![
            ("physical_layer".to_string(), physical_layer.to_string()),
        ];

        // Record RSSI
        let rssi_metric = Metric::new(
            "link_rssi_dbm".to_string(),
            MetricType::Gauge,
            metrics.rssi_dbm as f64,
            labels.clone(),
        );
        self.collector.record_metric(rssi_metric).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Record SNR
        let snr_metric = Metric::new(
            "link_snr_db".to_string(),
            MetricType::Gauge,
            metrics.snr_db as f64,
            labels.clone(),
        );
        self.collector.record_metric(snr_metric).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Record RTT
        let rtt_metric = Metric::new(
            "link_rtt_ms".to_string(),
            MetricType::Gauge,
            metrics.rtt_ms as f64,
            labels.clone(),
        );
        self.collector.record_metric(rtt_metric).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Record bandwidth
        let bw_metric = Metric::new(
            "link_bandwidth_mbps".to_string(),
            MetricType::Gauge,
            metrics.bandwidth_mbps as f64,
            labels,
        );
        self.collector.record_metric(bw_metric).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    async fn record_fec_strategy_change(
        &self,
        from_strategy: &str,
        to_strategy: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let labels = vec![
            ("from_strategy".to_string(), from_strategy.to_string()),
            ("to_strategy".to_string(), to_strategy.to_string()),
            ("reason".to_string(), reason.to_string()),
        ];

        let metric = Metric::new(
            "fec_strategy_changes_total".to_string(),
            MetricType::Counter,
            1.0,
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    async fn record_wfq_queue_depth(
        &self,
        queue_depth: usize,
        priority: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let labels = vec![
            ("priority".to_string(), priority.to_string()),
        ];

        let metric = Metric::new(
            "wfq_queue_depth".to_string(),
            MetricType::Gauge,
            queue_depth as f64,
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    async fn record_throughput(
        &self,
        bytes_per_sec: u64,
        physical_layer: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let labels = vec![
            ("physical_layer".to_string(), physical_layer.to_string()),
        ];

        let metric = Metric::new(
            "transport_throughput_bytes_per_sec".to_string(),
            MetricType::Gauge,
            bytes_per_sec as f64,
            labels,
        );

        self.collector
            .record_metric(metric)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TelemetryCollector, TelemetryConfig};

    #[tokio::test]
    async fn test_record_packet_loss_rate() {
        let config = TelemetryConfig::default();
        let collector = Arc::new(TelemetryCollector::new(config).unwrap());
        let telemetry = TransportTelemetryImpl::new(collector.clone());

        let result = telemetry.record_packet_loss_rate(0.005, "lora").await;
        assert!(result.is_ok(), "Failed to record packet loss rate: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_record_link_quality() {
        let config = TelemetryConfig::default();
        let collector = Arc::new(TelemetryCollector::new(config).unwrap());
        let telemetry = TransportTelemetryImpl::new(collector.clone());

        let metrics = LinkQualityMetrics {
            rssi_dbm: -75,
            snr_db: 25.5,
            loss_rate: 0.01,
            bandwidth_mbps: 2.5,
            rtt_ms: 150,
        };

        let result = telemetry.record_link_quality(&metrics, "lora").await;
        assert!(result.is_ok(), "Failed to record link quality: {:?}", result.err());
    }
}
