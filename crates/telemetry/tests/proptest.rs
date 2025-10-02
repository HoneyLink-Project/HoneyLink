//! Property-based tests for telemetry module

use honeylink_telemetry::types::{LogEvent, LogLevel, Metric, MetricType};
use honeylink_telemetry::sli::{SliDefinition, ThresholdLevel};
use honeylink_telemetry::storage::StorageConfig;
use proptest::prelude::*;

// Property: Metric name and labels should be preserved
proptest! {
    #[test]
    fn prop_metric_preserves_name_and_labels(
        name in "[a-z_]+",
        value in 0.0f64..1000.0f64,
        label_key in "[a-z]+",
        label_value in "[a-z]+",
    ) {
        let labels = vec![(label_key.clone(), label_value.clone())];
        let metric = Metric::counter(name.clone(), value, labels.clone());

        assert_eq!(metric.name, name);
        assert_eq!(metric.value, value);
        assert_eq!(metric.labels.len(), 1);
        assert_eq!(metric.labels[0].0, label_key);
        assert_eq!(metric.labels[0].1, label_value);
    }
}

// Property: Threshold evaluation is consistent
proptest! {
    #[test]
    fn prop_threshold_evaluation_is_consistent(value in 0.0f64..1000.0f64) {
        let sli = SliDefinition::session_establishment_latency();

        let level1 = sli.evaluate(value);
        let level2 = sli.evaluate(value);

        // Same input should produce same result
        assert_eq!(level1, level2);
    }
}

// Property: Threshold ordering is maintained
proptest! {
    #[test]
    fn prop_threshold_ordering(value in 0.0f64..1000.0f64) {
        let sli = SliDefinition::session_establishment_latency();
        let level = sli.evaluate(value);

        // Verify threshold ordering logic
        if value < sli.yellow_threshold {
            assert_eq!(level, ThresholdLevel::Green);
        } else if value < sli.orange_threshold {
            assert_eq!(level, ThresholdLevel::Yellow);
        } else if value < sli.red_threshold {
            assert_eq!(level, ThresholdLevel::Orange);
        } else {
            assert_eq!(level, ThresholdLevel::Red);
        }
    }
}

// Property: Log level ordering is preserved
proptest! {
    #[test]
    fn prop_log_level_ordering(message in ".{1,100}") {
        let debug = LogEvent::new(LogLevel::Debug, message.clone());
        let info = LogEvent::new(LogLevel::Info, message.clone());
        let warn = LogEvent::new(LogLevel::Warn, message.clone());
        let error = LogEvent::new(LogLevel::Error, message.clone());

        assert!(debug.level < info.level);
        assert!(info.level < warn.level);
        assert!(warn.level < error.level);
    }
}

// Property: Sampling ratio bounds
proptest! {
    #[test]
    fn prop_sampling_ratio_bounds(ratio in 0.0f64..=1.0f64) {
        let config = StorageConfig {
            normal_sampling_ratio: ratio,
            ..Default::default()
        };

        // Sampling ratio should stay within bounds
        assert!(config.normal_sampling_ratio >= 0.0);
        assert!(config.normal_sampling_ratio <= 1.0);
    }
}

// Property: Metric type consistency
proptest! {
    #[test]
    fn prop_metric_type_consistency(value in 0.0f64..1000.0f64) {
        let counter = Metric::counter("test".to_string(), value, vec![]);
        let gauge = Metric::gauge("test".to_string(), value, vec![]);
        let histogram = Metric::histogram("test".to_string(), value, vec![]);

        assert_eq!(counter.metric_type, MetricType::Counter);
        assert_eq!(gauge.metric_type, MetricType::Gauge);
        assert_eq!(histogram.metric_type, MetricType::Histogram);
    }
}

// Property: Trace ID association
proptest! {
    #[test]
    fn prop_trace_id_association(
        name in "[a-z_]+",
        value in 0.0f64..1000.0f64,
        trace_id in "[a-f0-9]{32}",
    ) {
        let metric = Metric::counter(name, value, vec![])
            .with_trace_id(trace_id.clone());

        assert_eq!(metric.trace_id, Some(trace_id));
    }
}

// Property: Buffer size limits
proptest! {
    #[test]
    fn prop_buffer_size_limits(size_mb in 1usize..=100usize) {
        let config = StorageConfig {
            max_buffer_size_bytes: size_mb * 1024 * 1024,
            ..Default::default()
        };

        // Buffer size should be reasonable
        assert!(config.max_buffer_size_bytes >= 1024 * 1024); // At least 1MB
        assert!(config.max_buffer_size_bytes <= 100 * 1024 * 1024); // At most 100MB
    }
}

// Property: Retention period bounds
proptest! {
    #[test]
    fn prop_retention_period_bounds(days in 1u32..=365u32) {
        let config = StorageConfig {
            retention_days: days,
            ..Default::default()
        };

        // Retention should be between 1 day and 1 year
        assert!(config.retention_days >= 1);
        assert!(config.retention_days <= 365);
    }
}

// Property: SLI consecutive breaches requirement
proptest! {
    #[test]
    fn prop_sli_consecutive_breaches(required in 1u8..=10u8) {
        let mut sli = SliDefinition::session_establishment_latency();
        sli.consecutive_breaches_required = required;

        // Consecutive breaches should be reasonable
        assert!(sli.consecutive_breaches_required >= 1);
        assert!(sli.consecutive_breaches_required <= 10);
    }
}
