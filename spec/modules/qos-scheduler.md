# Module Specification: Stream QoS Scheduler

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Stream QoS Scheduler モジュールの実装仕様書。ストリーム優先度制御と帯域割り当てを担当します。

**トレーサビリティ ID**: `MOD-005-QOS-SCHEDULER`

---

## 1. モジュール概要

- **モジュール名:** Stream QoS Scheduler
- **担当チーム:** Protocol WG (ENG-PROTO-02), QoS WG (ENG-QOS-01)
- **概要:** ストリーム優先度制御、帯域割り当て (25%/60%/15%)、バックプレッシャー処理
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `crates/qos-scheduler/`

### 価値提案
- リアルタイムストリーム (AR/VR, ゲーミング) の低レイテンシ保証
- 公平性と優先度のバランス (Weighted Fair Queuing ベース)
- バックプレッシャー機構による過負荷保護
- 動的帯域再配分 (アイドルストリームからアクティブストリームへ)

---

## 2. 責務と境界

### 主な責務
- **優先度制御**: Priority 0-7 に基づく送信順序決定
- **帯域割り当て**: High(25%) / Medium(60%) / Low(15%) の3層割り当て
- **バックプレッシャー**: キュー満杯時の送信元へのフィードバック
- **動的再配分**: アイドルストリームの帯域を他ストリームへ再配分
- **公平性保証**: 同一優先度内での Round Robin

### 非責務
- **ポリシー決定**: Policy Engine に委譲
- **パケット送信**: Transport Abstraction に委譲
- **セッション管理**: Session Orchestrator に委譲
- **暗号化**: Crypto & Trust Anchor に委譲

### 関連ドキュメント
- [spec/architecture/overview.md](../architecture/overview.md)
- [spec/requirements.md](../requirements.md) - FR-04 (QoS調整), NFR-01 (レイテンシ)

---

## 3. インターフェース

### 3.1 入力

| 名称 | プロトコル/フォーマット | 検証ルール | ソース |
|------|-------------------------|------------|--------|
| **EnqueueRequest** | Internal API (Rust) | stream_id: UInt8, priority ∈ [0,7] | Session Orchestrator |
| **QoSPolicyUpdate** | Internal API (Rust) | latency_budget_ms > 0 | Policy Engine |
| **BackpressureSignal** | Internal API (Rust) | queue_depth: UInt32 | Transport |

### 3.2 出力

| 名称 | プロトコル/フォーマット | SLA | 宛先 |
|------|-------------------------|-----|------|
| **ScheduledPacket** | Internal API (Rust) | P95 < 5ms | Transport |
| **BackpressureAck** | Internal API (Rust callback) | P95 < 100ms | Session Orchestrator |
| **QoSMetrics** | Local SQLite insert | 10秒バッチ | Telemetry (local metrics.db) |

**QoSMetrics スキーマ**:
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "stream_id": 3,
  "priority": 7,
  "queue_depth": 120,
  "packets_dropped": 5,
  "bandwidth_allocated_mbps": 150.5,
  "latency_p95_ms": 8.2
}
```

詳細: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. データモデル

### 4.1 主要エンティティ

#### StreamQueue (ストリームキュー)
```yaml
StreamQueue:
  stream_id: UInt8  # Primary Key
  priority: UInt8  # 0-7
  queue_depth: UInt32
  max_queue_depth: UInt32  # デフォルト 10000
  packets: Vec<Packet>
  bandwidth_allocated_mbps: Decimal(10,2)
  latency_budget_ms: UInt16
  last_scheduled_at: Timestamp
```

#### BandwidthAllocation (帯域割り当て)
```yaml
BandwidthAllocation:
  priority_tier: Enum[High, Medium, Low]
  base_allocation_percent: UInt8  # High=25, Medium=60, Low=15
  current_allocation_mbps: Decimal(10,2)
  idle_streams_count: UInt8
  reallocation_enabled: Boolean
```

### 4.2 帯域割り当てルール

| Priority | Tier | Base Allocation | Latency Budget | ユースケース |
|----------|------|-----------------|----------------|--------------|
| **7** | High | 25% | P95 < 10ms | AR/VR, ゲーミング |
| **6** | High | 25% | P95 < 20ms | 8K映像 |
| **4-5** | Medium | 60% | P95 < 50ms | HD映像, 音声 |
| **2-3** | Medium | 60% | P95 < 100ms | センサーデータ |
| **0-1** | Low | 15% | Best-effort | バックグラウンド同期 |

**動的再配分ルール**:
```
if (idle_streams_in_tier > 0) {
  unused_bandwidth = base_allocation * (idle_streams / total_streams)
  redistribute_to_active_streams(unused_bandwidth)
}
```

詳細: [spec/performance/scalability.md](../performance/scalability.md)

---

## 5. アルゴリズム: 優先度付きWFQ

### 概要
Weighted Fair Queuing (WFQ) に優先度層を追加したハイブリッド方式。

### スケジューリングロジック
```
1. High Tier (Priority 6-7) から選択
   - WFQ: virtual_time が最小のストリームを選択
   - 選択確率: 25%

2. High Tier が空 → Medium Tier (Priority 2-5) から選択
   - WFQ: virtual_time が最小のストリームを選択
   - 選択確率: 60%

3. Medium Tier が空 → Low Tier (Priority 0-1) から選択
   - Round Robin
   - 選択確率: 15%

4. パケット送信後、virtual_time更新
   virtual_time += packet_size / weight
   weight = 2^priority
```

### バックプレッシャー処理
```
if (queue_depth > max_queue_depth * 0.9) {
  send_backpressure_signal(stream_id, severity: WARNING)
}

if (queue_depth == max_queue_depth) {
  drop_packet(stream_id, drop_policy: TAIL_DROP)
  send_backpressure_signal(stream_id, severity: CRITICAL)
}
```

**TAIL_DROP**: キューの末尾パケットを破棄 (RED/WRED は将来実装検討)

参照: [spec/architecture/dataflow.md](../architecture/dataflow.md)

---

## 6. 依存関係

| 種別 | コンポーネント | インターフェース | SLA/契約 |
|------|----------------|-------------------|----------|
| **上位** | Session Orchestrator | EnqueueRequest | P95 < 10ms |
| **上位** | Policy Engine | QoSPolicyUpdate (Internal API) | Best-effort |
| **下位** | Transport | ScheduledPacket | P95 < 5ms |
| **Peer** | Telemetry | OTLP/gRPC | Best-effort |

**依存ルール**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 7. 性能・スケーラビリティ

### SLO/SLI

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| スケジューリングレイテンシ (P95) | < 5ms | EnqueueRequest → ScheduledPacket |
| Priority 7 レイテンシ (P95) | < 10ms | Enqueue → Transport送出 |
| パケットドロップ率 | < 0.01% | dropped / (enqueued + dropped) |
| スループット | ≥ 50K packets/sec/instance | 全ストリーム合計 |

詳細: [spec/performance/benchmark.md](../performance/benchmark.md)

---

## 8. セキュリティ & プライバシー

### 脅威対策 (STRIDE)
| 脅威 | 対策 |
|------|------|
| **Denial of Service** | Per-stream max_queue_depth制限 |
| **Elevation of Privilege** | Priority改ざん防止 (Policy Engineが署名付き設定配信) |
| **Information Disclosure** | メトリクスにペイロード含めない |

詳細: [spec/security/vulnerability.md](../security/vulnerability.md)

---

## 9. 観測性

### メトリクス

| メトリクス名 | 型 | ラベル |
|-------------|---|--------|
| `qos_packets_enqueued_total` | Counter | stream_id, priority |
| `qos_packets_dropped_total` | Counter | stream_id, drop_reason |
| `qos_queue_depth` | Gauge | stream_id |
| `qos_scheduling_duration_seconds` | Histogram | priority |
| `qos_bandwidth_allocated_mbps` | Gauge | priority_tier |

### ログフォーマット
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "WARN",
  "event": "backpressure.triggered",
  "stream_id": 3,
  "priority": 7,
  "queue_depth": 9500,
  "severity": "WARNING",
  "trace_id": "..."
}
```

参照: [spec/testing/metrics.md](../testing/metrics.md)

---

## 10. 公平性保証

### 同一優先度内の公平性
- **方式**: Round Robin (同一 priority 内)
- **保証**: 各ストリームが順番に少なくとも1パケット送信可能

### Starvation 防止
- **Low Tier保証帯域**: 総帯域の最低 5% (15%のうち、アイドル時の再配分後も保証)
- **タイムアウト**: Priority 0-1 のパケットが 10秒以上待機 → 強制送信 (Priority 4 相当に昇格)

---

## 11. 要件トレーサビリティ

### FR-04: QoS調整
- **関連**: ネットワーク状態変化時のポリシー動的更新
- **実装**: QoSPolicyUpdate イベント受信 → bandwidth_allocated_mbps / latency_budget_ms 更新

### NFR-01: レイテンシ
- **関連**: Priority 7 で P95 < 10ms 保証
- **実装**: High Tier 優先スケジューリング + WFQ

**トレーサビリティID対応表**:
```
MOD-005-QOS-SCHEDULER → FR-04 (dynamic QoS adjustment)
MOD-005-QOS-SCHEDULER → NFR-01 (latency guarantee)
```

---

## 12. テスト戦略

### 単体テスト
- WFQ virtual_time 計算ロジック (20ケース)
- 帯域割り当て (25%/60%/15%) 検証 (10ケース)
- バックプレッシャートリガー条件 (15ケース)
- カバレッジ目標: 90%

### 統合テスト
- Policy Engine → QoS Scheduler → Transport の E2E
- 動的再配分検証 (アイドルストリーム発生時)
- Starvation 防止検証 (Priority 0 を10秒待機させる)

### 負荷テスト
- 100K packets/sec 投入時のレイテンシ測定
- パケットドロップ率測定 (target < 0.01%)

詳細: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/testing/integration-tests.md](../testing/integration-tests.md)

---

## 13. デプロイ & 運用

- **デプロイ方法**: Rolling update
- **インフラ要件**: 2 vCPU, 2GB RAM/instance
- **ロールバック条件**: パケットドロップ率 > 0.1% (3分継続)

詳細: [spec/deployment/rollback.md](../deployment/rollback.md)

---

## 14. リスク & 技術的負債

| リスク | 緩和策 |
|--------|--------|
| Priority 0-1 の完全 starvation | 10秒タイムアウト後、強制昇格 |
| バックプレッシャー無視 | 3回無視後、セッション強制切断 |
| 帯域計算誤差蓄積 | 10秒毎にリセット |

---

## 15. 受け入れ基準 (DoD)

- [x] 優先度付きWFQアルゴリズム記述完了
- [x] バックプレッシャー処理仕様完成
- [x] 25%/60%/15% 帯域割り当てルール明示
- [x] FR-04/NFR-01 との紐付け明示
- [x] トレーサビリティID (`MOD-005-QOS-SCHEDULER`) 付与
- [x] C/C++ 依存排除確認
- [x] Starvation 防止機構仕様化完了

---

## 16. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | QoS WG (ENG-QOS-01) |
