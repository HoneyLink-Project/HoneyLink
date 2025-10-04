# Module Specification: Telemetry & Insights

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Telemetry & Insights モジュールの実装仕様書。OpenTelemetry統合とSLI/SLO監視を担当します。

**トレーサビリティ ID**: `MOD-006-TELEMETRY`

---

## 1. モジュール概要

- **モジュール名:** Telemetry & Insights
- **担当チーム:** Observability WG (ENG-OBS-01, ENG-OBS-02)
- **概要:** OpenTelemetry (Traces/Metrics/Logs) 統合、SLI/SLO監視、アラート生成
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `crates/telemetry/`

### 価値提案
- 全モジュール横断の分散トレーシング (Trace ID伝播)
- リアルタイムSLI可視化とローカルアラート (Yellow/Orange/Red閾値)
- Local SQLite によるメトリクス長期保存 (13ヶ月, 自動圧縮)
- オプションでOTLP Collectorへのエクスポート (ユーザー同意必須)

---

## 2. 責務と境界

### 主な責務
- **メトリクス収集**: Counter/Gauge/Histogram の集約とローカル保存
- **トレーシング**: Span生成、Trace ID伝播、親子関係管理
- **ログ集約**: 構造化ログ (JSON Lines) のローカルファイル保存
- **SLI/SLO監視**: Yellow/Orange/Red閾値でのローカルアラート生成
- **長期保存**: Local SQLite へのメトリクス保存 (13ヶ月)

### 非責務
- **ビジネスロジック**: 各モジュールに委譲
- **アラート送信**: PagerDuty/Slack 統合は Infrastructure Team が管理
- **ダッシュボード作成**: Grafana は SRE が管理
- **ログ検索**: Loki は SRE が管理

### 関連ドキュメント
- [spec/architecture/overview.md](../architecture/overview.md)
- [spec/testing/metrics.md](../testing/metrics.md)
- [spec/requirements.md](../requirements.md) - NFR-03 (可観測性)

---

## 3. インターフェース

### 3.1 入力

| 名称 | プロトコル/フォーマット | 検証ルール | ソース |
|------|-------------------------|------------|--------|
| **RecordMetric** | Internal API (Rust) | metric_name: valid identifier | 全モジュール |
| **StartSpan** | Internal API (Rust) | span_name: String(128) | 全モジュール |
| **LogEvent** | Internal API (Rust) | level ∈ {DEBUG, INFO, WARN, ERROR} | 全モジュール |

### 3.2 出力

| 名称 | プロトコル/フォーマット | SLA | 宛先 |
|------|-------------------------|-----|------|
| **LocalMetrics** | SQLite insert | 10秒バッチ | ~/.honeylink/metrics/metrics.db |
| **LocalTraces** | SQLite insert | リアルタイム | ~/.honeylink/metrics/traces.db |
| **LocalLogs** | JSON Lines append | 5秒バッチ | ~/.honeylink/logs/honeylink.log |
| **AlertEvent** | Local notification (OS toast) | P95 < 500ms | OS Notification System |
| **OTLP Export** (optional) | gRPC/Protobuf (OTLP) | Best-effort | User-configured OTLP Collector (opt-in) |

**AlertEvent スキーマ**:
```json
{
  "alert_id": "alert_xyz",
  "severity": "Orange",
  "sli_name": "session_establishment_latency_p95",
  "current_value": 650.5,
  "threshold": 500,
  "threshold_type": "Orange",
  "timestamp": "2025-10-01T10:30:00Z",
  "trace_id": "...",
  "labels": {
    "module": "session-orchestrator",
    "environment": "production"
  }
}
```

詳細: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. データモデル

### 4.1 主要エンティティ

#### Metric (メトリクス)
```yaml
Metric:
  metric_name: String(128)  # e.g., "session_establishment_duration_seconds"
  metric_type: Enum[Counter, Gauge, Histogram]
  value: Float64
  labels: Map<String, String>
  timestamp: UnixNano
  trace_id: String(32) (nullable)
```

#### Span (トレース)
```yaml
Span:
  span_id: String(16)  # Hex encoded
  trace_id: String(32)  # Hex encoded
  parent_span_id: String(16) (nullable)
  span_name: String(128)
  start_time: UnixNano
  end_time: UnixNano
  attributes: Map<String, String>
  events: Vec<SpanEvent>
  status: Enum[Ok, Error]
```

#### SLIThreshold (SLI閾値設定)
```yaml
SLIThreshold:
  sli_name: String(128)
  yellow_threshold: Float64
  orange_threshold: Float64
  red_threshold: Float64
  evaluation_window: Duration  # e.g., 5m
  consecutive_breaches_required: UInt8  # デフォルト 3
```

### 4.2 SLI/SLO定義

| SLI名 | 対象モジュール | Yellow | Orange | Red | SLO |
|-------|---------------|--------|--------|-----|-----|
| `session_establishment_latency_p95` | Session Orchestrator | > 400ms | > 500ms | > 800ms | < 500ms |
| `policy_update_latency_p95` | Policy Engine | > 250ms | > 300ms | > 500ms | < 300ms |
| `encryption_latency_p95` | Crypto | > 15ms | > 20ms | > 50ms | < 20ms |
| `packet_loss_rate` | Transport | > 0.05 | > 0.10 | > 0.20 | < 0.01 |
| `qos_packet_drop_rate` | QoS Scheduler | > 0.005 | > 0.01 | > 0.05 | < 0.01 |

**アラート生成ルール**:
- **Yellow**: 5分間で3回連続閾値超過 → Slackへ通知
- **Orange**: 5分間で3回連続閾値超過 → PagerDuty (Low priority)
- **Red**: 1回でも閾値超過 → PagerDuty (High priority)

詳細: [spec/testing/metrics.md](../testing/metrics.md)

### 4.3 永続化
- **データストア**: Local SQLite (~/.honeylink/metrics/metrics.db, 自動VACUUM)
- **保持期間**: 13ヶ月 (高解像度 7日、1分集約 90日、1時間集約 13ヶ月)
- **ファイルサイズ管理**: 最大500MB、超過時に古いデータから自動削除

---

## 5. アルゴリズム: Trace ID伝播

### 概要
W3C Trace Context 標準準拠。HTTP ヘッダー (`traceparent`) または gRPC メタデータで伝播。

### Trace Context フォーマット
```
traceparent: 00-<trace-id>-<parent-span-id>-<trace-flags>

例:
00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
```

### Span階層例
```
Root Span: session.establish (Session Orchestrator)
  ├─ Child Span: policy.fetch (Policy Engine)
  ├─ Child Span: key.exchange (Crypto)
  └─ Child Span: packet.send (Transport)
      └─ Child Span: physical.transmit (Physical Adapter)
```

**実装ガイド**:
- Rust: `tracing` crate + `opentelemetry-rust` SDK
- Trace ID は UUIDv7 ではなく OpenTelemetry 標準 (128bit HEX)

参照: [spec/architecture/dataflow.md](../architecture/dataflow.md)

---

## 6. 依存関係

| 種別 | コンポーネント | インターフェース | SLA/契約 |
|------|----------------|-------------------|----------|
| **上位** | 全モジュール | RecordMetric/StartSpan/LogEvent | Best-effort (非同期) |
| **下位** | OpenTelemetry Collector | OTLP/gRPC | P99 < 200ms |
| **下位** | Local SQLite | rusqlite | P99 < 10ms |
| **Peer** | Alertmanager | HTTP POST | P95 < 500ms |

**依存ルール**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 7. 性能・スケーラビリティ

### SLO/SLI

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| メトリクス送信レイテンシ (P95) | < 100ms | RecordMetric → OTLP送出 |
| トレース送信レイテンシ (P95) | < 50ms | EndSpan → OTLP送出 |
| メトリクス取りこぼし率 | < 0.1% | 送信失敗数 / 総記録数 |
| スループット | ≥ 100K events/sec/instance | Metrics + Traces + Logs合計 |

**バックプレッシャー対策**:
- **バッファリング**: 10秒分のメトリクスをメモリバッファ (最大 10MB)
- **Drop Policy**: バッファ満杯時は古いメトリクスから破棄 (FIFO)

詳細: [spec/performance/scalability.md](../performance/scalability.md)

---

## 8. セキュリティ & プライバシー

### 認証/認可
- **OTLP送信** (オプション): QUIC (ChaCha20-Poly1305) - ユーザー同意必須
- **Local SQLite**: ファイルパーミッション 0644 (読み取り専用), ネットワークアップロードなし

### 機密データ取り扱い
- **ペイロード**: メトリクス/ログにペイロード含めない
- **PII**: デバイスID/ユーザーIDはハッシュ化 (SHA256) してラベル化
- **監査ログ**: 改ざん防止のため Ed25519署名付き

詳細: [spec/security/auth.md](../security/auth.md)

---

## 9. 観測性 (Self-Monitoring)

### メトリクス

| メトリクス名 | 型 | ラベル |
|-------------|---|--------|
| `telemetry_events_recorded_total` | Counter | event_type, result |
| `telemetry_export_duration_seconds` | Histogram | exporter_type |
| `telemetry_buffer_size_bytes` | Gauge | buffer_type |
| `telemetry_alerts_triggered_total` | Counter | severity, sli_name |

### ログフォーマット
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "ERROR",
  "event": "export.failed",
  "exporter_type": "OTLP",
  "error": "connection timeout",
  "retry_count": 3,
  "trace_id": "..."
}
```

参照: [spec/testing/metrics.md](../testing/metrics.md)

---

## 10. OpenTelemetry統合

### SDK構成
- **言語SDK**: `opentelemetry-rust` v0.20+
- **Exporter**: `opentelemetry-otlp` (gRPC over TLS 1.3)
- **Propagator**: W3C Trace Context
- **Sampler**: AlwaysOn (開発環境), TraceIdRatioBased(0.1) (本番環境)

### Collector構成
```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4317
        # P2P: No TLS server certificates (local-only export)
        # Use QUIC with ChaCha20-Poly1305 for remote export if needed

exporters:
  prometheusremotewrite:
    endpoint: http://prometheus:9090/api/v1/write
  jaeger:
    endpoint: jaeger:14250
  loki:
    endpoint: http://loki:3100/loki/api/v1/push

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024
```

詳細: [spec/deployment/infrastructure.md](../deployment/infrastructure.md)

---

## 11. 要件トレーサビリティ

### NFR-03: 可観測性
- **関連**: 全トランザクションの追跡可能性とSLI/SLO監視
- **実装**: OpenTelemetry統合 + Local SQLite長期保存

**トレーサビリティID対応表**:
```
MOD-006-TELEMETRY → NFR-03 (observability and monitoring)
```

---

## 12. テスト戦略

### 単体テスト
- Trace ID伝播ロジック (10ケース)
- SLI閾値評価 (Yellow/Orange/Red判定、15ケース)
- バッファ満杯時のDrop Policy (10ケース)
- カバレッジ目標: 85%

### 統合テスト
- OpenTelemetry Collector (Mock) 連携
- Local SQLite書き込み/読み取り
- Alertmanager POST検証

### E2E テスト
- Session Orchestrator → Transport の full trace 取得
- 13ヶ月データ保持検証 (自動圧縮確認)

詳細: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/testing/e2e-tests.md](../testing/e2e-tests.md)

---

## 13. デプロイ & 運用

- **デプロイ方法**: Blue/Green deployment
- **インフラ要件**: 1 vCPU, 1GB RAM/instance
- **ロールバック条件**: メトリクス取りこぼし率 > 1% (5分継続)

詳細: [spec/deployment/ci-cd.md](../deployment/ci-cd.md)

---

## 14. リスク & 技術的負債

| リスク | 緩和策 |
|--------|--------|
| Collector障害によるメトリクス損失 | 10秒バッファ + リトライ (Exponential backoff) |
| SQLite容量不足 | 自動VACUUM + 13ヶ月自動削除 (最大500MB) |
| Trace ID衝突 | 128bit HEX (衝突確率 < 10^-30) |

---

## 15. 受け入れ基準 (DoD)

- [x] OpenTelemetry統合仕様完成
- [x] Yellow/Orange/Red閾値定義完了
- [x] SLI対応表作成完了
- [x] NFR-03 との紐付け明示
- [x] トレーサビリティID (`MOD-006-TELEMETRY`) 付与
- [x] C/C++ 依存排除確認 (Rust OpenTelemetry SDK使用)
- [x] Local SQLite長期保存仕様完成

---

## 16. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | Observability WG (ENG-OBS-01) |
