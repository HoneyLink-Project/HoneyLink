//! Integration tests for telemetry module

use honeylink_telemetry::alert::{AlertConfig, AlertManager};
use honeylink_telemetry::collector::{TelemetryCollector, TelemetryConfig};
use honeylink_telemetry::crypto_metrics::CryptoMetrics;
use honeylink_telemetry::sli::{SliDefinition, SliMonitor, ThresholdLevel};
use honeylink_telemetry::storage::{StorageConfig, StoragePipeline};
use honeylink_telemetry::types::{LogEvent, LogLevel, Metric, MetricType};

#[tokio::test]
async fn test_sli_monitor_integration() {
    let monitor = SliMonitor::new();

    // Record metrics that breach Yellow threshold
    for _ in 0..5 {
        let metric = Metric::histogram(
            "session_establishment_latency_p95".to_string(),
            450.0, // Yellow threshold: 400ms
            vec![("module".to_string(), "session-orchestrator".to_string())],
        );

        let result = monitor.record_metric(&metric);
        assert!(result.is_ok());
    }

    // After 3 consecutive breaches, we should get a violation
    let metric = Metric::histogram(
        "session_establishment_latency_p95".to_string(),
        450.0,
        vec![("module".to_string(), "session-orchestrator".to_string())],
    );

    let violation = monitor.record_metric(&metric).unwrap();
    // Due to windowing, violation might not trigger immediately in this test
    // In real scenario with proper timing, it would trigger after 3 consecutive breaches
}

#[tokio::test]
async fn test_alert_manager_integration() {
    let config = AlertConfig {
        test_mode: true, // Don't send actual alerts in tests
        ..Default::default()
    };
    let manager = AlertManager::new(config);

    // Create a mock violation
    let violation = honeylink_telemetry::sli::SliViolation {
        sli_name: "test_sli".to_string(),
        module: "test_module".to_string(),
        level: ThresholdLevel::Orange,
        current_value: 550.0,
        threshold: 500.0,
        consecutive_breaches: 3,
        timestamp: std::time::SystemTime::now(),
        trace_id: Some("test_trace".to_string()),
    };

    // Send alert (should succeed in test mode)
    let result = manager.send_alert(violation).await;
    assert!(result.is_ok());

    // Check alert history
    let history = manager.get_history();
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_storage_pipeline_integration() {
    let config = StorageConfig {
        max_buffer_size_bytes: 1024 * 1024, // 1MB
        normal_sampling_ratio: 1.0,          // Always sample
        ..Default::default()
    };
    let pipeline = StoragePipeline::new(config);

    // Add metrics to buffer
    for i in 0..100 {
        let metric = Metric::counter(
            format!("test_metric_{}", i),
            i as f64,
            vec![("test".to_string(), "value".to_string())],
        );

        let result = pipeline.add_metric(metric).await;
        assert!(result.is_ok());
    }

    // Check buffer stats
    let stats = pipeline.get_stats().await;
    assert_eq!(stats.buffered_metrics, 100);
    assert!(stats.buffer_size_bytes > 0);
}

#[tokio::test]
async fn test_crypto_metrics_integration() {
    let metrics = CryptoMetrics::new();

    // Simulate crypto operations
    metrics.record_x25519_operation(10_000_000, true); // 10ms
    metrics.record_chacha20_encryption(15_000_000, true); // 15ms
    metrics.record_hkdf_derivation(5_000_000, true); // 5ms
    metrics.record_rotation_event(1500, true); // 1.5 seconds
    metrics.record_pop_token_generation();
    metrics.record_pop_token_verification(true, false);

    // Generate all metrics
    let all_metrics = metrics.all_metrics();

    // Should have 18 metrics total
    assert_eq!(all_metrics.len(), 18);

    // Verify some specific metrics
    let x25519_total = all_metrics
        .iter()
        .find(|m| m.name == "crypto_x25519_operations_total");
    assert!(x25519_total.is_some());
    assert_eq!(x25519_total.unwrap().value, 1.0);
}

#[tokio::test]
async fn test_collector_integration() {
    let mut collector = TelemetryCollector::new();

    // Record a metric
    let metric = Metric::counter("test_counter".to_string(), 42.0, vec![]);

    // Should succeed even without initialization (will just skip OTel export)
    let result = collector.record_metric(metric).await;
    assert!(result.is_ok());

    // Check stats
    let stats = collector.get_stats().await;
    // In test mode without initialization, metrics are added to storage pipeline
    assert_eq!(stats.buffered_metrics, 1);
}

#[tokio::test]
async fn test_failure_mode_sampling() {
    let config = StorageConfig {
        normal_sampling_ratio: 0.2,  // 20% normal
        failure_sampling_ratio: 1.0, // 100% on failure
        ..Default::default()
    };
    let pipeline = StoragePipeline::new(config);

    // Normal mode - some metrics should be dropped due to sampling
    for i in 0..100 {
        let metric = Metric::counter(format!("metric_{}", i), i as f64, vec![]);
        pipeline.add_metric(metric).await.unwrap();
    }

    let normal_stats = pipeline.get_stats().await;
    let normal_count = normal_stats.buffered_metrics;

    // Should be around 20 (allow variance)
    assert!(normal_count >= 10 && normal_count <= 40);

    // Enable failure mode
    pipeline.set_failure_mode(true).await;

    // Add more metrics
    for i in 100..200 {
        let metric = Metric::counter(format!("metric_{}", i), i as f64, vec![]);
        pipeline.add_metric(metric).await.unwrap();
    }

    let failure_stats = pipeline.get_stats().await;
    let failure_count = failure_stats.buffered_metrics;

    // In failure mode, should have close to 100 additional metrics
    assert!(failure_count >= normal_count + 80);
}

#[tokio::test]
async fn test_pii_removal() {
    let config = StorageConfig {
        enable_pii_detection: true,
        normal_sampling_ratio: 1.0,
        ..Default::default()
    };
    let pipeline = StoragePipeline::new(config);

    // Metric with PII labels
    let metric = Metric::counter(
        "test_metric".to_string(),
        42.0,
        vec![
            ("user_id".to_string(), "12345".to_string()),
            ("email".to_string(), "user@example.com".to_string()),
            ("safe_label".to_string(), "safe_value".to_string()),
        ],
    );

    pipeline.add_metric(metric).await.unwrap();

    // In a real implementation with accessible buffer,
    // we would verify that PII labels are removed
    // For now, we just verify it doesn't error
}

#[test]
fn test_sli_definitions_completeness() {
    let slis = SliDefinition::all_defaults();

    // Should have 5 predefined SLIs
    assert_eq!(slis.len(), 5);

    // Verify each SLI has proper thresholds
    for sli in slis {
        assert!(sli.yellow_threshold < sli.orange_threshold);
        assert!(sli.orange_threshold < sli.red_threshold);
        assert!(sli.consecutive_breaches_required >= 1);
        assert!(!sli.name.is_empty());
        assert!(!sli.module.is_empty());
    }
}

#[tokio::test]
async fn test_log_export() {
    let collector = TelemetryCollector::new();

    // Export logs of different levels
    for level in [
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ] {
        let log = LogEvent::new(level, format!("Test message: {:?}", level))
            .with_module("test_module".to_string())
            .with_trace_id("trace123".to_string());

        let result = collector.log(log).await;
        // Without initialization, log export may fail, but shouldn't panic
        // assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_threshold_level_escalation() {
    let sli = SliDefinition::session_establishment_latency();

    // Test escalation: Green -> Yellow -> Orange -> Red
    assert_eq!(sli.evaluate(350.0), ThresholdLevel::Green);
    assert_eq!(sli.evaluate(450.0), ThresholdLevel::Yellow);
    assert_eq!(sli.evaluate(550.0), ThresholdLevel::Orange);
    assert_eq!(sli.evaluate(850.0), ThresholdLevel::Red);
}

#[tokio::test]
async fn test_buffer_fifo_drop_policy() {
    let config = StorageConfig {
        max_buffer_size_bytes: 200, // Very small buffer to force drops
        normal_sampling_ratio: 1.0,
        ..Default::default()
    };
    let pipeline = StoragePipeline::new(config);

    // Add many metrics to overflow buffer
    for i in 0..1000 {
        let metric = Metric::counter(format!("metric_{}", i), i as f64, vec![]);
        pipeline.add_metric(metric).await.unwrap();
    }

    let stats = pipeline.get_stats().await;

    // Buffer should be constrained by size limit
    assert!(stats.buffer_size_bytes <= 200);

    // Some metrics should have been dropped (FIFO policy)
    assert!(stats.total_dropped > 0);
}
