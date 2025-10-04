# HoneyLink Observability Stack

Complete observability infrastructure for HoneyLink telemetry data using OpenTelemetry, Prometheus, Jaeger, Loki, and Grafana.

## Architecture

```
┌─────────────────────┐
│ HoneyLink Services  │
│  (Rust + OTLP SDK)  │
└──────────┬──────────┘
           │ OTLP/gRPC (TLS)
           ▼
┌─────────────────────┐
│  OTEL Collector     │◄─── Receives traces, metrics, logs
│  (port 4317/4318)   │
└──────────┬──────────┘
           │
     ┌─────┼─────┬─────────┐
     ▼     ▼     ▼         ▼
┌────────┐ │ ┌────────┐ ┌──────┐
│Promethe│ │ │ Jaeger │ │ Loki │
│us      │ │ │        │ │      │
│(metrics│ │ │(traces)│ │(logs)│
└────┬───┘ │ └───┬────┘ └──┬───┘
     │     │     │         │
     └─────┴─────┴─────────┘
                 │
                 ▼
          ┌───────────┐
          │  Grafana  │◄─── Unified visualization
          │ (port 3000)│
          └───────────┘
```

## Services

| Service | Port | Purpose |
|---------|------|---------|
| **OTEL Collector** | 4317 (gRPC), 4318 (HTTP) | Receives OTLP telemetry, exports to backends |
| **Prometheus** | 9091 | Metrics storage and querying |
| **Jaeger** | 16686 (UI), 14250 (gRPC) | Distributed tracing backend |
| **Loki** | 3100 | Log aggregation and querying |
| **Grafana** | 3000 | Unified dashboards and visualization |

## Quick Start

### 1. Generate TLS Certificates (Development)

**Linux/macOS/WSL:**
```bash
cd infrastructure
chmod +x generate-certs.sh
./generate-certs.sh
```

**Windows PowerShell:**
```powershell
cd infrastructure
.\generate-certs.ps1
```

This creates self-signed certificates in `infrastructure/certs/`:
- `ca.crt` - Root CA certificate (copy to `crates/telemetry/` config)
- `server.crt`, `server.key` - OTEL Collector server TLS
- `client.crt`, `client.key` - Client authentication (optional)

### 2. Start the Observability Stack

```bash
docker-compose -f infrastructure/docker-compose.observability.yml up -d
```

### 3. Verify Services

- **OTEL Collector Health:** http://localhost:13133/health
- **Prometheus UI:** http://localhost:9091
- **Jaeger UI:** http://localhost:16686
- **Loki API:** http://localhost:3100/ready
- **Grafana UI:** http://localhost:3000 (admin/admin)

### 4. Configure HoneyLink Telemetry

In `backend/src/config.rs` or environment variables:

```rust
// OTLP exporter configuration
TelemetryConfig {
    otlp_endpoint: "https://localhost:4317".to_string(),
    tls_ca_cert: Some("/path/to/infrastructure/certs/ca.crt".to_string()),
    service_name: "honeylink".to_string(),
    environment: "development".to_string(),
}
```

Or via environment variables:
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=https://localhost:4317
export OTEL_EXPORTER_OTLP_CERTIFICATE=/path/to/infrastructure/certs/ca.crt
export OTEL_SERVICE_NAME=honeylink
export OTEL_RESOURCE_ATTRIBUTES=deployment.environment=development
```

### 5. Run HoneyLink Backend

```bash
cargo run -p honeylink-control-plane --target x86_64-unknown-linux-gnu
```

Telemetry data will flow automatically to the observability stack.

## Configuration

### OTEL Collector

Edit `otel-collector-config.yaml` to customize:
- **Receivers:** OTLP gRPC/HTTP endpoints
- **Processors:** Batching, filtering, resource enrichment
- **Exporters:** Prometheus, Jaeger, Loki, file, logging

**Environment Variables:**
- `ENVIRONMENT` - Deployment environment (development/staging/production)
- `HONEYLINK_VERSION` - Service version for resource attributes
- `JAEGER_ENDPOINT` - Jaeger collector endpoint (default: jaeger:14250)
- `LOKI_ENDPOINT` - Loki push API endpoint (default: http://loki:3100/loki/api/v1/push)

### Prometheus

Edit `prometheus.yml` to add scrape targets:
```yaml
scrape_configs:
  - job_name: 'honeylink-backend'
    static_configs:
      - targets: ['host.docker.internal:8080']  # If backend exposes metrics
```

### Loki

Edit `loki-config.yaml` to adjust:
- **Retention:** Default 7 days (`retention_period: 168h`)
- **Rate Limits:** `ingestion_rate_mb`, `per_stream_rate_limit`
- **Storage:** Filesystem (default) or S3/GCS for production

### Grafana

Default credentials: **admin / admin** (change on first login)

**Provisioned Datasources:**
- Prometheus: http://prometheus:9090
- Loki: http://loki:3100
- Jaeger: http://jaeger:16686

**Adding Dashboards:**
1. Place JSON dashboards in `grafana/dashboards/`
2. Restart Grafana: `docker-compose restart grafana`
3. Navigate to Dashboards → Browse → HoneyLink folder

## SLI Metrics (from `spec/testing/metrics.md`)

### Key Performance Indicators (SLIs)

| SLI | Metric Name | Target | Source |
|-----|-------------|--------|--------|
| **Session Establishment Latency** | `session_establishment_latency_p95` | P95 < 200ms | Session Orchestrator |
| **Policy Update Latency** | `policy_update_latency_p95` | P95 < 100ms | Policy Engine |
| **Packet Loss Rate** | `packet_loss_rate_p95` | P95 < 0.5% | Transport Layer |
| **QoS Packet Drop Rate** | `qos_packet_drop_rate_p95` | P95 < 0.1% | QoS Scheduler |

**Query in Prometheus:**
```promql
# Session establishment latency (95th percentile)
histogram_quantile(0.95, rate(honeylink_session_establishment_duration_seconds_bucket[5m]))

# Policy update latency (95th percentile)
histogram_quantile(0.95, rate(honeylink_policy_update_duration_seconds_bucket[5m]))

# Packet loss rate (P95)
histogram_quantile(0.95, rate(honeylink_packet_loss_rate_bucket[5m]))

# QoS packet drop rate (P95)
histogram_quantile(0.95, rate(honeylink_qos_packet_drop_rate_bucket[5m]))
```

## Alerting (Production)

### 1. Deploy Alertmanager

Uncomment Alertmanager section in `docker-compose.observability.yml`:
```yaml
alertmanager:
  image: prom/alertmanager:v0.27.0
  ports:
    - "9093:9093"
  volumes:
    - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
```

### 2. Configure Alert Rules

Create `prometheus-rules.yml`:
```yaml
groups:
  - name: honeylink_slis
    interval: 15s
    rules:
      - alert: HighSessionLatency
        expr: histogram_quantile(0.95, rate(honeylink_session_establishment_duration_seconds_bucket[5m])) > 0.2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Session establishment latency P95 > 200ms"
          description: "Current value: {{ $value }}s"

      - alert: HighPacketLoss
        expr: histogram_quantile(0.95, rate(honeylink_packet_loss_rate_bucket[5m])) > 0.005
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Packet loss rate P95 > 0.5%"
```

### 3. Configure Alertmanager Integrations

Edit `alertmanager.yml`:
```yaml
route:
  receiver: 'pagerduty-critical'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty-critical'
    - match:
        severity: warning
      receiver: 'slack-warnings'

receivers:
  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: '<PAGERDUTY_KEY>'

  - name: 'slack-warnings'
    slack_configs:
      - api_url: '<SLACK_WEBHOOK_URL>'
        channel: '#honeylink-alerts'
```

## Troubleshooting

### OTEL Collector not receiving data

1. **Check TLS certificates:**
   ```bash
   docker exec honeylink-otel-collector ls -la /etc/otel/certs/
   ```

2. **Verify gRPC endpoint:**
   ```bash
   grpcurl -plaintext localhost:4317 list
   ```

3. **Check OTEL Collector logs:**
   ```bash
   docker logs honeylink-otel-collector
   ```

### No data in Prometheus

1. **Check OTEL Collector Prometheus exporter:**
   ```bash
   curl http://localhost:9090/metrics
   ```

2. **Verify Prometheus scrape targets:**
   http://localhost:9091/targets

### Grafana datasources not working

1. **Test datasource connectivity:**
   - Grafana UI → Configuration → Datasources → Select datasource → "Test"

2. **Check Docker network:**
   ```bash
   docker network inspect honeylink-observability_honeylink-observability
   ```

## Production Deployment

### Security Checklist

- [ ] **TLS Certificates:** Use proper CA-signed certificates (not self-signed)
- [ ] **Authentication:** Enable OTEL Collector mTLS client authentication
- [ ] **Firewall:** Restrict ports 4317/4318 to internal network only
- [ ] **Grafana:** Change default admin password, enable LDAP/OAuth
- [ ] **Prometheus:** Enable Basic Auth or reverse proxy with authentication
- [ ] **Jaeger:** Configure storage backend (Elasticsearch/Cassandra) for persistence
- [ ] **Loki:** Use S3/GCS object storage for production-grade retention

### Scaling Considerations

- **OTEL Collector:** Deploy as DaemonSet or StatefulSet in Kubernetes
- **Prometheus:** Use federation or Thanos for multi-cluster aggregation
- **Jaeger:** Enable Kafka buffering for high-throughput environments
- **Loki:** Configure S3/GCS backend with compaction and retention policies

## Maintenance

### Data Retention

| Service | Default Retention | Configuration |
|---------|-------------------|---------------|
| Prometheus | 30 days | `--storage.tsdb.retention.time=30d` |
| Jaeger | Ephemeral (Badger) | Use Elasticsearch/Cassandra for persistence |
| Loki | 7 days | `retention_period: 168h` in loki-config.yaml |

### Backup

```bash
# Backup Prometheus data
docker exec honeylink-prometheus tar czf /tmp/prometheus-backup.tar.gz /prometheus
docker cp honeylink-prometheus:/tmp/prometheus-backup.tar.gz ./backups/

# Backup Grafana dashboards
docker exec honeylink-grafana tar czf /tmp/grafana-backup.tar.gz /var/lib/grafana
docker cp honeylink-grafana:/tmp/grafana-backup.tar.gz ./backups/
```

### Monitoring the Monitors

- **OTEL Collector Internal Metrics:** http://localhost:8888/metrics
- **Prometheus Internal Metrics:** http://localhost:9091/metrics
- **Grafana Health:** http://localhost:3000/api/health

## References

- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [OTEL Collector Configuration](https://opentelemetry.io/docs/collector/configuration/)
- [Prometheus Querying](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Jaeger Architecture](https://www.jaegertracing.io/docs/1.54/architecture/)
- [Loki Best Practices](https://grafana.com/docs/loki/latest/best-practices/)
- [Grafana Provisioning](https://grafana.com/docs/grafana/latest/administration/provisioning/)
