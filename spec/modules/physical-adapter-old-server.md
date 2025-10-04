# Module Specification: Physical Adapter Layer

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Physical Adapter Layer モジュールの実装仕様書。Wi-Fi/5G/THz などの物理層との統合を担当します。

**トレーサビリティ ID**: `MOD-007-PHYSICAL-ADAPTER`

---

## 1. モジュール概要

- **モジュール名:** Physical Adapter Layer
- **担当チーム:** Physical WG (ENG-PHY-01, ENG-PHY-02)
- **概要:** Wi-Fi/5G/THz などの物理層との統合、gRPC/REST API経由でのドライバ制御
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `crates/physical-adapter/`

### 価値提案
- C/C++ ドライバ依存の完全排除 (gRPC/REST経由での間接制御)
- 物理層の Hot Swap 対応 (Wi-Fi → 5G 切替を無停止で実行)
- 電力モード制御 (Ultra Low Power モードで 5mA 消費)
- THz帯域の実験的サポート

---

## 2. 責務と境界

### 主な責務
- **物理層統合**: Wi-Fi/5G/THz/Ethernet との gRPC/REST API連携
- **電力モード制御**: Ultra Low / Low / Normal / High の4段階制御
- **リンク品質監視**: RSSI/SNR/パケロス率の定期測定
- **Hot Swap**: 物理層切替時のセッション維持
- **プロトコル変換**: Transport Layer の統一 API → 物理層固有 API

### 非責務
- **物理層ドライバ実装**: Vendor提供のドライバ/サービスに委譲
- **QoS制御**: QoS Scheduler に委譲
- **暗号化**: Crypto & Trust Anchor に委譲
- **ルーティング**: Session Orchestrator に委譲

### 関連ドキュメント
- [spec/architecture/overview.md](../architecture/overview.md)
- [spec/requirements.md](../requirements.md) - FR-01 (ペアリング), FR-03 (消費電力最適化)

---

## 3. インターフェース

### 3.1 入力 (Transport Layer から)

| 名称 | プロトコル/フォーマット | 検証ルール | ソース |
|------|-------------------------|------------|--------|
| **PhysicalSend** | Internal API (Rust) | data.len() <= 64KB | Transport |
| **SetPowerMode** | Internal API (Rust) | mode ∈ {UltraLow, Low, Normal, High} | Policy Engine |
| **SwitchPhysicalLayer** | Internal API (Rust) | target_type ∈ {WiFi, 5G, THz, Ethernet} | Session Orchestrator |

### 3.2 出力 (物理層へ)

| 名称 | プロトコル/フォーマット | SLA | 宛先 |
|------|-------------------------|-----|------|
| **WiFi gRPC API** | gRPC/Protobuf | P95 < 20ms | Wi-Fi Controller Service |
| **5G REST API** | HTTP/JSON | P95 < 50ms | 5G Modem HTTP Server |
| **THz gRPC API** | gRPC/Protobuf | P95 < 30ms | THz Experimental Service |

**WiFi gRPC API スキーマ例**:
```protobuf
service WiFiController {
  rpc Send(SendRequest) returns (SendResponse);
  rpc GetLinkQuality(Empty) returns (LinkQualityResponse);
  rpc SetPowerMode(PowerModeRequest) returns (Empty);
}

message SendRequest {
  bytes payload = 1;
  uint32 priority = 2;
}

message LinkQualityResponse {
  int32 rssi_dbm = 1;
  float snr_db = 2;
  float packet_loss_rate = 3;
}
```

詳細: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. データモデル

### 4.1 主要エンティティ

#### PhysicalLayerConfig (物理層設定)
```yaml
PhysicalLayerConfig:
  physical_type: Enum[WiFi, FiveG, THz, Ethernet]
  endpoint: String(256)  # gRPC: "http://localhost:50051", REST: "http://10.0.0.1:8080"
  protocol: Enum[gRPC, REST]
  power_mode: Enum[UltraLow, Low, Normal, High]
  max_retries: UInt8  # デフォルト 3
  timeout_ms: UInt16  # デフォルト 5000
  tls_enabled: Boolean
```

#### PhysicalLayerMetrics (物理層メトリクス)
```yaml
PhysicalLayerMetrics:
  physical_type: Enum[WiFi, FiveG, THz, Ethernet]
  rssi_dbm: Int16 (WiFi/5G)
  snr_db: Decimal(5,2)
  packet_loss_rate: Decimal(5,4)
  throughput_mbps: Decimal(10,2)
  power_consumption_mw: Decimal(10,2)
  measured_at: Timestamp
```

### 4.2 電力モード定義

| モード | 消費電力 (目安) | スループット | レイテンシ | ユースケース |
|--------|-----------------|--------------|------------|--------------|
| **UltraLow** | 5mA | < 1Mbps | P95 < 200ms | IoTセンサー (定期送信) |
| **Low** | 50mA | 10-50Mbps | P95 < 50ms | 音声通話 |
| **Normal** | 200mA | 100-500Mbps | P95 < 20ms | HD映像 |
| **High** | 500mA+ | 500Mbps-1.5Gbps | P95 < 10ms | 8K映像, AR/VR |

**電力モード切替ロジック**:
```
if (stream_priority >= 6 && latency_budget_ms < 20) {
  power_mode = High
} else if (stream_priority <= 2 && latency_budget_ms > 100) {
  power_mode = UltraLow
} else {
  power_mode = Normal
}
```

詳細: [spec/requirements.md](../requirements.md) - FR-03

---

## 5. アルゴリズム: Hot Swap (物理層切替)

### 概要
セッション維持しながら Wi-Fi → 5G など物理層を切替。Transport Layer からは透過的。

### 切替手順
```
1. Session Orchestrator から SwitchPhysicalLayer(target=5G) 受信
2. 新物理層 (5G) の接続確立
   - gRPC/REST エンドポイント接続
   - リンク品質測定 (RSSI/SNR)
3. 新物理層が Ready 状態になったら、Transport Layer へ通知
4. Transport Layer が新物理層経由でパケット送信開始
5. 旧物理層 (WiFi) 経由のパケット送信完了を待機 (最大 5秒)
6. 旧物理層接続切断
7. Telemetry へ PhysicalLayerSwitched イベント送信
```

**ロールバック条件**:
- 新物理層接続失敗 (3回リトライ後)
- リンク品質が基準以下 (packet_loss_rate > 0.2)

詳細: [spec/architecture/dataflow.md](../architecture/dataflow.md)

---

## 6. C/C++ 依存の排除戦略

### 問題
従来の物理層ドライバは C/C++ で実装され、Rust から直接呼び出すと依存が発生。

### 解決策
**Adapter Pattern + プロセス分離**:
```
Rust (Physical Adapter Layer)
  ↓ gRPC/REST
Vendor Driver Service (C/C++/Python/etc.)
  ↓ Native API
Physical Layer Hardware (Wi-Fi/5G/THz)
```

### 実装例

#### Wi-Fi Controller Service (Go/Python/Node.js で実装)
```
- ポート: 50051 (gRPC)
- プロトコル: gRPC/Protobuf
- API:
  - Send(payload, priority) → ACK
  - GetLinkQuality() → {rssi, snr, loss_rate}
  - SetPowerMode(mode) → ACK
```

#### 5G Modem HTTP Server (Vendor提供)
```
- ポート: 8080 (HTTP)
- プロトコル: REST/JSON
- API:
  - POST /send → {"status": "ok"}
  - GET /metrics → {"rssi": -70, "snr": 25.5, "loss_rate": 0.01}
  - POST /power_mode → {"mode": "low"}
```

**利点**:
- Rust コードに C/C++ 依存なし
- Vendor Driver Service のクラッシュが Physical Adapter Layer に影響しない
- 物理層の実装言語を自由に選択可能 (Go/Python/Node.js/etc.)

参照: [spec/architecture/tech-stack.md](../architecture/tech-stack.md)

---

## 7. 依存関係

| 種別 | コンポーネント | インターフェース | SLA/契約 |
|------|----------------|-------------------|----------|
| **上位** | Transport | PhysicalSend | P95 < 30ms |
| **上位** | Session Orchestrator | SwitchPhysicalLayer | Best-effort |
| **下位** | WiFi Controller Service | gRPC | P95 < 20ms |
| **下位** | 5G Modem HTTP Server | REST | P95 < 50ms |
| **下位** | THz Experimental Service | gRPC | P95 < 30ms |

**依存ルール**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 8. 性能・スケーラビリティ

### SLO/SLI

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| 物理層送信レイテンシ (P95) | < 30ms | PhysicalSend → 物理層ACK |
| Hot Swap レイテンシ (P95) | < 2秒 | SwitchPhysicalLayer → Ready |
| 電力モード切替レイテンシ (P95) | < 500ms | SetPowerMode → ACK |
| リンク品質測定周期 | 5秒 | 定期ポーリング |

詳細: [spec/performance/benchmark.md](../performance/benchmark.md)

---

## 9. セキュリティ & プライバシー

### 認証/認可
- **gRPC通信**: mTLS (client certificate)
- **REST通信**: Bearer Token (OAuth2)

### 脅威対策 (STRIDE)
| 脅威 | 対策 |
|------|------|
| **Spoofing** | mTLS/OAuth2 |
| **Tampering** | TLS 1.3 encryption |
| **Denial of Service** | Rate limiting (per physical layer) |

詳細: [spec/security/auth.md](../security/auth.md)

---

## 10. 観測性

### メトリクス

| メトリクス名 | 型 | ラベル |
|-------------|---|--------|
| `physical_packets_sent_total` | Counter | physical_type, result |
| `physical_link_quality_rssi_dbm` | Gauge | physical_type |
| `physical_power_consumption_mw` | Gauge | physical_type, power_mode |
| `physical_layer_switches_total` | Counter | from_type, to_type, result |

### ログフォーマット
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "INFO",
  "event": "physical.switched",
  "from_type": "WiFi",
  "to_type": "5G",
  "duration_ms": 1800,
  "trace_id": "..."
}
```

参照: [spec/testing/metrics.md](../testing/metrics.md)

---

## 11. 要件トレーサビリティ

### FR-01: ペアリング
- **関連**: ペアリング中の物理層選択 (Wi-Fi優先)
- **実装**: PhysicalLayerConfig による初期選択

### FR-03: 消費電力最適化
- **関連**: UltraLow モードで 5mA 消費
- **実装**: SetPowerMode(UltraLow) 呼び出し

**トレーサビリティID対応表**:
```
MOD-007-PHYSICAL-ADAPTER → FR-01 (physical layer selection)
MOD-007-PHYSICAL-ADAPTER → FR-03 (power consumption optimization)
```

---

## 12. テスト戦略

### 単体テスト
- Hot Swap ロジック (10ケース、ロールバック含む)
- 電力モード切替 (8ケース)
- gRPC/REST エラー処理 (15ケース)
- カバレッジ目標: 85%

### 統合テスト
- Mock WiFi Controller Service 連携
- Mock 5G Modem HTTP Server 連携
- Hot Swap E2E (Wi-Fi → 5G → Ethernet)

### E2E テスト
- Transport → Physical Adapter → 実機 Wi-Fi モジュール
- 電力消費実測 (UltraLow モードで 5mA 以下)

詳細: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/testing/e2e-tests.md](../testing/e2e-tests.md)

---

## 13. デプロイ & 運用

- **デプロイ方法**: Rolling update
- **インフラ要件**: 1 vCPU, 512MB RAM/instance
- **ロールバック条件**: 物理層送信エラー率 > 5% (3分継続)

詳細: [spec/deployment/rollback.md](../deployment/rollback.md)

---

## 14. リスク & 技術的負債

| リスク | 緩和策 |
|--------|--------|
| Vendor Driver Service クラッシュ | プロセス分離 + 自動再起動 (systemd) |
| THz帯域の不安定性 | 実験的機能としてフラグ管理 |
| gRPC/REST API のバージョン不一致 | API バージョニング (v1/v2) |

---

## 15. 受け入れ基準 (DoD)

- [x] gRPC/REST統合仕様完成
- [x] C/C++ 依存排除戦略記述完了
- [x] Hot Swap アルゴリズム仕様化完了
- [x] FR-01/FR-03 との紐付け明示
- [x] トレーサビリティID (`MOD-007-PHYSICAL-ADAPTER`) 付与
- [x] C/C++ 依存排除確認 (Adapter Pattern + プロセス分離)
- [x] 電力モード制御仕様完成

---

## 16. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | Physical WG (ENG-PHY-01) |

