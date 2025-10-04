# Telemetry & Insights

OpenTelemetry integration for metrics, traces, and logs with **local-first storage**.

## ⚠️ P2P Design

This crate is designed for **local-first telemetry** with optional server export:

- ✅ **Default:** Local SQLite storage (`~/.honeylink/metrics/metrics.db`, 500MB max, 13-month retention)
- ✅ **Optional:** OTLP Collector export (requires user consent, feature-gated)
- ❌ **Removed:** Automatic server upload (PagerDuty, Slack, TimescaleDB)

## Features

- **Metrics:** Counter, Gauge, Histogram via OpenTelemetry
- **Tracing:** Distributed tracing with W3C Trace Context
- **Logging:** Structured logging to `~/.honeylink/logs/honeylink.log` (JSON Lines, 50MB max)
- **SLI/SLO:** 5 predefined SLIs with Yellow/Orange/Red local alerting (OS toast notifications)
- **Storage:** Local SQLite with automatic VACUUM and compression
- **Privacy:** No automatic data collection, OTLP export requires explicit user opt-in

## Usage

```rust
use honeylink_telemetry::{TelemetryCollector, TelemetryConfig};
use honeylink_telemetry::types::Metric;

#[tokio::main]
async fn main() {
    let mut collector = TelemetryCollector::new();
    
    // Local-only configuration (default)
    let config = TelemetryConfig {
        export_enabled: false,  // No server upload
        local_storage_path: "~/.honeylink/metrics/metrics.db".into(),
        ..Default::default()
    };
    
    collector.initialize(config).await.unwrap();
    
    let metric = Metric::counter("requests_total".to_string(), 1.0, vec![]);
    collector.record_metric(metric).await.unwrap();
}
```

## Optional OTLP Export

**Requires user consent and explicit configuration:**

```toml
[dependencies]
honeylink-telemetry = { version = "0.1", features = ["otlp-export"] }
```

```rust
let config = TelemetryConfig {
    export_enabled: true,  // User must explicitly enable
    otlp_endpoint: "http://localhost:4317".to_string(),  // User-configured
    ..Default::default()
};
```

## Architecture Consistency

- **No automatic server uploads:** All metrics stored locally by default
- **User privacy:** OTLP export requires explicit configuration in `~/.honeylink/config.toml`
- **No PagerDuty/Slack:** Alerts shown as OS notifications (Windows: toast, macOS: Notification Center, Linux: libnotify)
- **Pure Rust:** No C/C++ dependencies

## Related Documentation

- [spec/modules/telemetry-insights.md](../../spec/modules/telemetry-insights.md) - P2P telemetry specification
- [spec/testing/metrics.md](../../spec/testing/metrics.md) - SLI/SLO definitions
