//! Storage pipeline for telemetry data
//!
//! Provides buffering, batching, PII detection, and TimescaleDB integration.

use crate::types::{Metric, TelemetryError, TelemetryResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database connection string
    pub database_url: String,
    /// Maximum buffer size in bytes (default: 10MB)
    pub max_buffer_size_bytes: usize,
    /// Batch write interval in seconds
    pub batch_interval_secs: u64,
    /// Data retention period in days
    pub retention_days: u32,
    /// Sampling ratio for normal operation (0.0-1.0)
    pub normal_sampling_ratio: f64,
    /// Sampling ratio on failure (0.0-1.0)
    pub failure_sampling_ratio: f64,
    /// Enable PII detection
    pub enable_pii_detection: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/telemetry".to_string(),
            max_buffer_size_bytes: 10 * 1024 * 1024, // 10 MB
            batch_interval_secs: 10,
            retention_days: 30,
            normal_sampling_ratio: 0.2,      // 20%
            failure_sampling_ratio: 1.0,     // 100%
            enable_pii_detection: true,
        }
    }
}

/// Metric buffer with FIFO drop policy
struct MetricBuffer {
    metrics: VecDeque<Metric>,
    current_size_bytes: usize,
    max_size_bytes: usize,
    total_dropped: u64,
}

impl MetricBuffer {
    fn new(max_size_bytes: usize) -> Self {
        Self {
            metrics: VecDeque::new(),
            current_size_bytes: 0,
            max_size_bytes,
            total_dropped: 0,
        }
    }

    /// Pushes a metric, dropping oldest if buffer is full
    fn push(&mut self, metric: Metric) {
        let metric_size = estimate_metric_size(&metric);

        // Drop oldest metrics if buffer would overflow
        while self.current_size_bytes + metric_size > self.max_size_bytes && !self.metrics.is_empty()
        {
            if let Some(dropped) = self.metrics.pop_front() {
                self.current_size_bytes -= estimate_metric_size(&dropped);
                self.total_dropped += 1;
            }
        }

        // Add new metric
        self.current_size_bytes += metric_size;
        self.metrics.push_back(metric);
    }

    /// Drains all metrics for batch write
    fn drain_all(&mut self) -> Vec<Metric> {
        self.current_size_bytes = 0;
        self.metrics.drain(..).collect()
    }

    /// Returns current buffer size
    fn len(&self) -> usize {
        self.metrics.len()
    }
}

/// Estimates metric size in bytes (approximate)
fn estimate_metric_size(metric: &Metric) -> usize {
    let mut size = 64; // Base overhead
    size += metric.name.len();
    size += metric.labels.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>();
    if let Some(ref trace_id) = metric.trace_id {
        size += trace_id.len();
    }
    size
}

/// Storage pipeline manages buffering and batch writes
pub struct StoragePipeline {
    config: StorageConfig,
    buffer: Arc<Mutex<MetricBuffer>>,
    is_failure_mode: Arc<Mutex<bool>>,
}

impl StoragePipeline {
    /// Creates a new storage pipeline
    pub fn new(config: StorageConfig) -> Self {
        let buffer = Arc::new(Mutex::new(MetricBuffer::new(config.max_buffer_size_bytes)));

        Self {
            config,
            buffer,
            is_failure_mode: Arc::new(Mutex::new(false)),
        }
    }

    /// Adds a metric to the buffer with sampling
    pub async fn add_metric(&self, mut metric: Metric) -> TelemetryResult<()> {
        // Apply PII detection if enabled
        if self.config.enable_pii_detection {
            self.remove_pii(&mut metric)?;
        }

        // Apply sampling
        let is_failure = *self.is_failure_mode.lock().await;
        let sampling_ratio = if is_failure {
            self.config.failure_sampling_ratio
        } else {
            self.config.normal_sampling_ratio
        };

        if !should_sample(sampling_ratio) {
            return Ok(()); // Skip this metric
        }

        // Add to buffer
        let mut buffer = self.buffer.lock().await;
        buffer.push(metric);

        Ok(())
    }

    /// Sets failure mode (enables 100% sampling)
    pub async fn set_failure_mode(&self, enabled: bool) {
        *self.is_failure_mode.lock().await = enabled;
    }

    /// Starts background batch writer task
    pub fn start_batch_writer(&self) -> tokio::task::JoinHandle<()> {
        let buffer = self.buffer.clone();
        let interval = Duration::from_secs(self.config.batch_interval_secs);
        let database_url = self.config.database_url.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

                // Drain buffer
                let metrics = {
                    let mut buf = buffer.lock().await;
                    if buf.len() == 0 {
                        continue;
                    }
                    buf.drain_all()
                };

                // Write batch to database
                if let Err(e) = Self::write_batch(&database_url, metrics).await {
                    log::error!("Failed to write batch to database: {}", e);
                }
            }
        })
    }

    /// Writes a batch of metrics to TimescaleDB
    async fn write_batch(database_url: &str, metrics: Vec<Metric>) -> TelemetryResult<()> {
        // In a real implementation, use sqlx to insert metrics
        // For now, just log
        log::info!("Would write {} metrics to {}", metrics.len(), database_url);

        // Pseudo-code for sqlx:
        // let pool = sqlx::PgPool::connect(database_url).await?;
        // for metric in metrics {
        //     sqlx::query("INSERT INTO metrics (...) VALUES (...)")
        //         .execute(&pool)
        //         .await?;
        // }

        Ok(())
    }

    /// Removes PII from metric labels
    fn remove_pii(&self, metric: &mut Metric) -> TelemetryResult<()> {
        let pii_patterns = vec![
            "user_id",
            "email",
            "phone",
            "ssn",
            "credit_card",
            "password",
            "token",
        ];

        metric.labels.retain(|(key, _)| {
            !pii_patterns
                .iter()
                .any(|pattern| key.to_lowercase().contains(pattern))
        });

        Ok(())
    }

    /// Returns buffer statistics
    pub async fn get_stats(&self) -> BufferStats {
        let buffer = self.buffer.lock().await;
        BufferStats {
            buffered_metrics: buffer.len(),
            buffer_size_bytes: buffer.current_size_bytes,
            max_buffer_size_bytes: buffer.max_size_bytes,
            total_dropped: buffer.total_dropped,
        }
    }
}

/// Buffer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferStats {
    pub buffered_metrics: usize,
    pub buffer_size_bytes: usize,
    pub max_buffer_size_bytes: usize,
    pub total_dropped: u64,
}

/// Sampling decision based on ratio
fn should_sample(ratio: f64) -> bool {
    if ratio >= 1.0 {
        true
    } else if ratio <= 0.0 {
        false
    } else {
        use rand::Rng;
        rand::thread_rng().gen::<f64>() < ratio
    }
}

/// Data retention manager (cleanup old data)
pub struct RetentionManager {
    database_url: String,
    retention_days: u32,
}

impl RetentionManager {
    /// Creates a new retention manager
    pub fn new(database_url: String, retention_days: u32) -> Self {
        Self {
            database_url,
            retention_days,
        }
    }

    /// Starts background cleanup task
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let database_url = self.database_url.clone();
        let retention_days = self.retention_days;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(86400)); // Daily

            loop {
                ticker.tick().await;

                if let Err(e) = Self::cleanup_old_data(&database_url, retention_days).await {
                    log::error!("Failed to cleanup old data: {}", e);
                }
            }
        })
    }

    /// Deletes data older than retention period
    async fn cleanup_old_data(database_url: &str, retention_days: u32) -> TelemetryResult<()> {
        let cutoff = SystemTime::now() - Duration::from_secs(retention_days as u64 * 86400);

        log::info!(
            "Would delete metrics older than {} days from {}",
            retention_days,
            database_url
        );

        // Pseudo-code for sqlx:
        // let pool = sqlx::PgPool::connect(database_url).await?;
        // sqlx::query("DELETE FROM metrics WHERE timestamp < $1")
        //     .bind(cutoff)
        //     .execute(&pool)
        //     .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MetricType;

    fn create_test_metric(name: &str, value: f64) -> Metric {
        Metric::new(
            name.to_string(),
            MetricType::Counter,
            value,
            vec![("label1".to_string(), "value1".to_string())],
        )
    }

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.max_buffer_size_bytes, 10 * 1024 * 1024);
        assert_eq!(config.batch_interval_secs, 10);
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.normal_sampling_ratio, 0.2);
        assert_eq!(config.failure_sampling_ratio, 1.0);
    }

    #[test]
    fn test_metric_buffer_push() {
        let mut buffer = MetricBuffer::new(1024);
        let metric = create_test_metric("test", 42.0);

        buffer.push(metric);
        assert_eq!(buffer.len(), 1);
        assert!(buffer.current_size_bytes > 0);
    }

    #[test]
    fn test_metric_buffer_fifo_drop() {
        let mut buffer = MetricBuffer::new(200); // Small buffer

        // Push metrics until buffer is full
        for i in 0..100 {
            buffer.push(create_test_metric(&format!("metric{}", i), i as f64));
        }

        // Buffer should have dropped some metrics
        assert!(buffer.total_dropped > 0);
        assert!(buffer.current_size_bytes <= buffer.max_size_bytes);
    }

    #[test]
    fn test_metric_buffer_drain() {
        let mut buffer = MetricBuffer::new(10240);

        for i in 0..10 {
            buffer.push(create_test_metric(&format!("metric{}", i), i as f64));
        }

        let drained = buffer.drain_all();
        assert_eq!(drained.len(), 10);
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.current_size_bytes, 0);
    }

    #[tokio::test]
    async fn test_storage_pipeline_creation() {
        let config = StorageConfig::default();
        let pipeline = StoragePipeline::new(config);

        let stats = pipeline.get_stats().await;
        assert_eq!(stats.buffered_metrics, 0);
    }

    #[tokio::test]
    async fn test_storage_pipeline_add_metric() {
        let config = StorageConfig {
            normal_sampling_ratio: 1.0, // Always sample
            ..Default::default()
        };
        let pipeline = StoragePipeline::new(config);
        let metric = create_test_metric("test", 42.0);

        let result = pipeline.add_metric(metric).await;
        assert!(result.is_ok());

        let stats = pipeline.get_stats().await;
        assert_eq!(stats.buffered_metrics, 1);
    }

    #[tokio::test]
    async fn test_storage_pipeline_failure_mode() {
        let pipeline = StoragePipeline::new(StorageConfig::default());

        pipeline.set_failure_mode(true).await;
        assert_eq!(*pipeline.is_failure_mode.lock().await, true);

        pipeline.set_failure_mode(false).await;
        assert_eq!(*pipeline.is_failure_mode.lock().await, false);
    }

    #[test]
    fn test_pii_removal() {
        let config = StorageConfig::default();
        let pipeline = StoragePipeline::new(config);

        let mut metric = Metric::new(
            "test".to_string(),
            MetricType::Counter,
            42.0,
            vec![
                ("user_id".to_string(), "12345".to_string()),
                ("safe_label".to_string(), "safe_value".to_string()),
            ],
        );

        pipeline.remove_pii(&mut metric).unwrap();

        assert_eq!(metric.labels.len(), 1);
        assert_eq!(metric.labels[0].0, "safe_label");
    }

    #[test]
    fn test_sampling_logic() {
        // Always sample
        assert!(should_sample(1.0));

        // Never sample
        assert!(!should_sample(0.0));

        // Probabilistic (test multiple times)
        let mut sampled_count = 0;
        for _ in 0..1000 {
            if should_sample(0.5) {
                sampled_count += 1;
            }
        }

        // Should be roughly 50% (allow 40-60% range)
        assert!(sampled_count >= 400 && sampled_count <= 600);
    }

    #[test]
    fn test_retention_manager_creation() {
        let manager = RetentionManager::new("test_url".to_string(), 30);
        assert_eq!(manager.retention_days, 30);
    }
}
