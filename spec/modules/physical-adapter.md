# Module Specification: Physical Adapter Layer

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Physical Adapter Layer モジュ| モード | 消費電力 (目安) | スループット | レイテンシ | ユースケース | P2Pプロトコル |
|--------|-----------------|--------------|------------|------------|----------|
| **UltraLow** | ~10mA | < 1Mbps | P95 < 200ms | IoTセンサー (定期送信) | BLEのみ |
| **Low** | ~50mA | 10-50Mbps | P95 < 50ms | 音声通話 | mDNS + QUIC |
| **Normal** | ~200mA | 100-500Mbps | P95 < 20ms | HD映像 | mDNS + QUIC |
| **High** | ~500mA | 500Mbps-1Gbps | P95 < 10ms | 8K映像, AR/VR | mDNS + QUIC + WebRTC |様書。Wi-Fi/5G/THz などの物理層との統合を担当します。

**トレーサビリティ ID**: `MOD-007-PHYSICAL-ADAPTER`

---

## 1. モジュール概要

- **モジュール名:** Physical Adapter Layer
- **担当チーム:** Physical WG (ENG-PHY-01, ENG-PHY-02)
- **概要:** mDNS/BLE/QUIC/WebRTCプロトコル統合、Pure Rustクレート経由でのネットワーク制御 (no gRPC/REST servers)
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `crates/physical-adapter/`

### 価値提案
- Pure Rust実装でC/C++ドライバ依存なし (mdns-sd, btleplug, quinn, webrtc crates)
- 物理層の Hot Swap 対応 (mDNS/BLE → QUIC/WebRTC 切替を無停止で実行)
- 電力モード制御 (BLE Low Power モードで ~10mA 消費)
- NAT越え対応 (STUN/TURN, 95%成功率目標)

---

## 2. 責務と境界

### 主な責務
- **P2Pプロトコル統合**: mDNS/BLE/QUIC/WebRTC Pure Rust crates (mdns-sd, btleplug, quinn, webrtc)
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
| **mDNS Discovery** | UDP multicast (mdns-sd crate) | P95 < 100ms | Local network devices |
| **BLE Advertisement** | Bluetooth LE (btleplug crate) | P95 < 200ms | Nearby devices |
| **QUIC Connection** | UDP (quinn crate) | P95 < 50ms | Peer device (direct or STUN/TURN) |
| **WebRTC Data Channel** | UDP (webrtc crate) | P95 < 100ms | Peer device (ICE candidates) |

**mDNS Discovery 例**:
```rust
// 概念説明用 (実装コードではない)
let mdns = ServiceDaemon::new()?;
let service_type = "_honeylink._tcp.local.";
let receiver = mdns.browse(service_type)?;

for event in receiver.recv() {
    match event {
        ServiceEvent::ServiceResolved(info) => {
            println!("Found peer: {}", info.get_hostname());
        }
    }
}
```

詳細: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. データモデル

### 4.1 主要エンティティ

#### PhysicalLayerConfig (物理層設定)
```yaml
PhysicalLayerConfig:
  protocol_type: Enum[MDNS, BLE, QUIC, WebRTC]  # P2P protocols only
  mdns_service_name: String(64)  # Default: "_honeylink._tcp.local."
  ble_uuid: String(36)  # HoneyLink BLE service UUID
  quic_port: UInt16  # Default: 7843 (UDP)
  stun_server: String(256)  # Default: "stun.l.google.com:19302"
  power_mode: Enum[UltraLow, Low, Normal, High]
  max_retries: UInt8  # Default 3
  timeout_ms: UInt16  # Default 5000
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

#### P2P Native Rust Integration (Recommended)

**Direct Crate Usage** (No HTTP/gRPC servers):
- **mDNS Discovery**: `mdns-sd` crate (`_honeylink._tcp.local.` service)
- **BLE Advertising**: `btleplug` crate (HoneyLink UUID)
- **QUIC Transport**: `quinn` crate (UDP 7843, P95 < 20ms latency)
- **WebRTC Data Channels**: `webrtc` crate (STUN/TURN NAT traversal)

**Benefits**:
- Zero C/C++ dependencies (Pure Rust cryptography: `x25519-dalek`, `chacha20poly1305`)
- No vendor HTTP servers (eliminates external service dependency)
- Direct OS network stack access (better performance, lower latency)
- P2P design alignment (no backend services)

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
- **P2P Communication**: TOFU (Trust On First Use) - QR/PIN pairing, no OAuth2/mTLS
- **Transport Security**: QUIC (ChaCha20-Poly1305), WebRTC (DTLS-SRTP)
- **Device Trust**: `~/.honeylink/trusted_peers.json` based

### 脅威対策 (STRIDE)
| 脅威 | 対策 |
|------|------|
| **Spoofing** | TOFU + Physical Proximity Verification (QR/PIN/NFC) |
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
