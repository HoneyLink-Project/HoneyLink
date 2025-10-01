# Module Specification: Transport Abstraction Layer

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Transport Abstraction Layer モジュールの実装仕様書。物理層の抽象化と統一APIの提供を担当します。

**トレーサビリティ ID**: `MOD-003-TRANSPORT`

---

## 1. モジュール概要

- **モジュール名:** Transport Abstraction Layer
- **担当チーム:** Protocol WG (ENG-PROTO-02), Physical WG (ENG-PHY-01)
- **概要:** Wi-Fi/5G/THz などの物理層を統一インターフェースで抽象化し、FEC (Forward Error Correction) を提供
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `crates/transport/`

### 価値提案
- 物理層切替時のアプリケーション層コード変更なし
- Reed-Solomon FEC による劣悪環境でのパケット回復率向上 (目標 95%)
- WFQ (Weighted Fair Queuing) による優先度制御
- C/C++ ドライバ依存の完全排除

---

## 2. 責務と境界

### 主な責務
- **物理層抽象化**: PhysicalLayer trait の定義と統一API提供
- **FEC管理**: Reed-Solomon符号化/復号化 (Rust純実装)
- **キューイング**: WFQ (Weighted Fair Queuing) による優先度制御
- **エラー処理**: タイムアウト、再送、劣化検知
- **メトリクス収集**: パケロス率、スループット、レイテンシ

### 非責務
- **物理層実装**: Physical Adapter Layer に委譲
- **QoSポリシー決定**: Policy Engine に委譲
- **暗号化**: Crypto & Trust Anchor に委譲
- **ルーティング**: Session Orchestrator に委譲

### 関連ドキュメント
- [spec/architecture/overview.md](../architecture/overview.md)
- [spec/requirements.md](../requirements.md) - FR-01 (ペアリング), FR-03 (消費電力最適化)

---

## 3. インターフェース

### 3.1 PhysicalLayer Trait (統一API)

```rust
// 概念説明用 (実装コードではない)
trait PhysicalLayer {
    fn send_packet(data: Bytes, priority: u8) -> Result<(), TransportError>;
    fn recv_packet(timeout: Duration) -> Result<Bytes, TransportError>;
    fn get_link_quality() -> LinkQualityMetrics;
    fn set_power_mode(mode: PowerMode) -> Result<(), TransportError>;
}
```

### 3.2 入力

| 名称 | プロトコル/フォーマット | 検証ルール | ソース |
|------|-------------------------|------------|--------|
| **SendRequest** | Internal API (Rust) | data.len() <= 64KB | Session Orchestrator |
| **FECConfig** | Event Bus (JSON) | mode ∈ {NONE, LIGHT, HEAVY} | Policy Engine |
| **PhysicalMetrics** | Internal API | valid_enum(physical_type) | Physical Adapter |

### 3.3 出力

| 名称 | プロトコル/フォーマット | SLA | 宛先 |
|------|-------------------------|-----|------|
| **EncodedPacket** | Binary (Reed-Solomon) | P95 < 5ms encoding | Physical Adapter |
| **TransportMetrics** | OTLP/gRPC | 10秒バッチ | Telemetry |
| **LinkDegradation** | Event Bus (JSON) | P95 < 100ms | Session Orchestrator |

詳細: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. データモデル

### 4.1 主要エンティティ

#### Packet (パケット構造)
```yaml
Packet:
  packet_id: UInt64  # シーケンス番号
  stream_id: UInt8
  priority: UInt8  # 0-7 (WFQウェイト計算に使用)
  payload: Bytes(up to 64KB)
  fec_redundancy: Bytes (nullable)
  timestamp: UnixNano
  checksum: UInt32  # CRC32
```

#### LinkQualityMetrics
```yaml
LinkQualityMetrics:
  physical_type: Enum[WiFi, FiveG, THz, Ethernet]
  rssi_dbm: Int16 (WiFi/5G)
  snr_db: Decimal(5,2)
  packet_loss_rate: Decimal(5,4)  # 0.0000-1.0000
  latency_p50_ms: UInt16
  latency_p95_ms: UInt16
  throughput_mbps: Decimal(10,2)
  measured_at: Timestamp
```

### 4.2 FEC戦略

| モード | 冗長度 | 回復能力 | オーバーヘッド | ユースケース |
|--------|--------|----------|----------------|--------------|
| **NONE** | 0% | なし | 0% | 有線LAN (高品質) |
| **LIGHT** | 20% | 10%パケロス回復 | +25% bandwidth | 通常Wi-Fi |
| **HEAVY** | 50% | 25%パケロス回復 | +100% bandwidth | THz (実験的), 移動中5G |

**Reed-Solomon実装**:
- ライブラリ: `reed-solomon-erasure` (Rust純実装, C/C++依存なし)
- パラメータ: (n, k) - データシンボル k、冗長シンボル (n-k)
  - LIGHT: (10, 8) → 20%冗長度
  - HEAVY: (10, 5) → 100%冗長度

詳細: [spec/architecture/tech-stack.md](../architecture/tech-stack.md) - FEC

---

## 5. アルゴリズム: WFQ (Weighted Fair Queuing)

### 概要
優先度 (0-7) に基づいてパケット送信順序を決定。低優先度のパケットが完全に starve しないよう保証。

### 仮想時間進行ルール
```
virtual_time(packet) = arrival_time + (packet_size / weight)
weight = 2^priority  # priority=7 → weight=128
```

### 送信順序
1. 各パケットの virtual_time を計算
2. virtual_time が最小のパケットを選択
3. 送信後、キューから削除

### 例
```
Packet A: priority=7, size=1KB, arrival=0ms  → vtime=0 + 1/128=0.0078
Packet B: priority=2, size=1KB, arrival=1ms  → vtime=1 + 1/4=1.25
Packet C: priority=7, size=4KB, arrival=2ms  → vtime=2 + 4/128=2.03125

送信順序: A → C → B
```

参照: [spec/performance/scalability.md](../performance/scalability.md)

---

## 6. 依存関係

| 種別 | コンポーネント | インターフェース | SLA/契約 |
|------|----------------|-------------------|----------|
| **上位** | Session Orchestrator | SendRequest API | P95 < 50ms |
| **上位** | Policy Engine | FECConfig (Event Bus) | Best-effort |
| **下位** | Physical Adapter | PhysicalLayer trait | P95 < 10ms |
| **Peer** | Telemetry | OTLP/gRPC | Best-effort |

**依存ルール**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 7. 性能・スケーラビリティ

### SLO/SLI

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| パケット送信レイテンシ (P95) | < 10ms | SendRequest → EncodedPacket 送出 |
| FEC符号化レイテンシ (P95) | < 5ms | Payload → Reed-Solomon 完了 |
| パケット回復率 (LIGHT) | ≥ 95% @ 10% パケロス | 受信側での復号成功率 |
| スループット | ≥ 1Gbps @ priority=7 | WFQ経由での実効転送速度 |

詳細: [spec/performance/benchmark.md](../performance/benchmark.md)

---

## 8. セキュリティ & プライバシー

### 脅威対策 (STRIDE)
| 脅威 | 対策 |
|------|------|
| **Tampering** | Packet checksum (CRC32), 上位層で暗号化 |
| **Denial of Service** | WFQによる帯域保護 (各優先度に最低保証) |
| **Elevation of Privilege** | Priority改ざん防止 (Policy Engine が署名付き設定配信) |

詳細: [spec/security/vulnerability.md](../security/vulnerability.md)

---

## 9. 観測性

### メトリクス

| メトリクス名 | 型 | ラベル |
|-------------|---|--------|
| `transport_packets_sent_total` | Counter | physical_type, priority |
| `transport_fec_encoding_duration_seconds` | Histogram | fec_mode |
| `transport_packet_loss_rate` | Gauge | physical_type |
| `transport_wfq_queue_depth` | Gauge | priority |

### ログフォーマット
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "WARN",
  "event": "link.degradation",
  "physical_type": "WiFi",
  "packet_loss_rate": 0.12,
  "rssi_dbm": -78,
  "trace_id": "..."
}
```

参照: [spec/testing/metrics.md](../testing/metrics.md)

---

## 10. エラー処理

### TransportError 列挙型
```rust
enum TransportError {
    Timeout,             // 送信タイムアウト (5秒)
    LinkDown,            // 物理層切断
    BufferOverflow,      // WFQキュー満杯 (10000パケット)
    FECDecodingFailed,   // Reed-Solomon復号失敗
    InvalidPriority,     // priority > 7
}
```

### 再送ポリシー
- **Max Retry**: 3回
- **Backoff**: Exponential (100ms, 200ms, 400ms)
- **Dead Letter Queue**: 3回失敗後は DLQ へ転送 (Telemetry で監視)

---

## 11. 要件トレーサビリティ

### FR-01: ペアリング
- **関連**: ペアリング中の物理層選択 (Wi-Fi優先 → 5G fallback)
- **実装**: PhysicalLayer trait による切替

### FR-03: 消費電力最適化
- **関連**: FEC NONE モード選択で符号化コスト削減
- **実装**: set_power_mode(PowerMode::UltraLow) 呼び出し

**トレーサビリティID対応表**:
```
MOD-003-TRANSPORT → FR-01 (physical layer abstraction)
MOD-003-TRANSPORT → FR-03 (power optimization via FEC mode)
```

---

## 12. テスト戦略

### 単体テスト
- WFQ virtual_time 計算ロジック (20ケース)
- Reed-Solomon 符号化/復号化 (10ケース、パケロス0-30%)
- CRC32 checksum 検証 (10ケース)
- カバレッジ目標: 90%

### 統合テスト
- Physical Adapter (Mock) 経由でのパケット送受信
- FEC モード切替時の回復率測定
- WFQ 動作検証 (優先度7と優先度0の送信比率)

### E2E テスト
- Session Orchestrator → Transport → Physical Adapter の full stack
- 10%パケロス環境でのFEC LIGHT有効性検証

詳細: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/testing/e2e-tests.md](../testing/e2e-tests.md)

---

## 13. デプロイ & 運用

- **デプロイ方法**: Rolling update (1インスタンスずつ)
- **インフラ要件**: 2 vCPU, 2GB RAM/instance
- **ロールバック条件**: パケロス率 > 20% (3分継続)

詳細: [spec/deployment/rollback.md](../deployment/rollback.md)

---

## 14. リスク & 技術的負債

| リスク | 緩和策 |
|--------|--------|
| Reed-Solomon実装のバグ | fuzzing (cargo-fuzz) |
| WFQのstarvation | 最低帯域保証 (priority=0でも5%保証) |
| Physical Adapter の C/C++ 依存混入 | CI で依存チェック |

---

## 15. 受け入れ基準 (DoD)

- [x] PhysicalLayer trait 定義完了
- [x] FEC戦略 (NONE/LIGHT/HEAVY) 仕様化完了
- [x] WFQアルゴリズム記述完了
- [x] FR-01/FR-03 との紐付け明示
- [x] トレーサビリティID (`MOD-003-TRANSPORT`) 付与
- [x] C/C++ 依存排除確認 (reed-solomon-erasure crate使用)
- [x] エラー処理とリトライ仕様完成

---

## 16. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | Protocol WG (ENG-PROTO-02) |

