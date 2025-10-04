# docs/architecture/overview.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止` `✅ P2P設計` `✅ サーバーレス`

> HoneyLinkは**Bluetoothの完全上位互換**を目指すPure P2P (Peer-to-Peer) プロトコルです。中央サーバー、データベース、クラウドサービスは一切不要で、デバイス間が直接通信します。

## 目次
- [docs/architecture/overview.md](#docsarchitectureoverviewmd)
  - [目次](#目次)
  - [P2Pアーキテクチャ原則](#p2pアーキテクチャ原則)
  - [P2Pコンポーネント図](#p2pコンポーネント図)
    - [デバイス間対称設計 (全デバイスが同じスタック)](#デバイス間対称設計-全デバイスが同じスタック)
  - [デバイス発見とペアリング](#デバイス発見とペアリング)
    - [1. デバイス発見 (Bluetoothと同UX)](#1-デバイス発見-bluetoothと同ux)
    - [2. ペアリング手順 (Bluetoothライク)](#2-ペアリング手順-bluetoothライク)
    - [3. 信頼モデル (TOFU: Trust On First Use)](#3-信頼モデル-tofu-trust-on-first-use)
  - [P2Pコンポーネント責務](#p2pコンポーネント責務)
  - [P2Pアーキテクチャパターン](#p2pアーキテクチャパターン)
    - [デザインパターン](#デザインパターン)
    - [Bluetoothとの比較](#bluetoothとの比較)
  - [境界と依存管理](#境界と依存管理)
    - [P2Pアーキテクチャの依存ルール](#p2pアーキテクチャの依存ルール)
    - [外部依存 (任意)](#外部依存-任意)
  - [ローカル可観測性](#ローカル可観測性)
    - [ローカルメトリクス収集 (サーバー送信なし)](#ローカルメトリクス収集-サーバー送信なし)
    - [主要メトリクス](#主要メトリクス)
  - [変更容易性評価 (P2P設計)](#変更容易性評価-p2p設計)
    - [P2P設計の利点](#p2p設計の利点)
  - [受け入れ基準 (DoD)](#受け入れ基準-dod)
    - [P2Pアーキテクチャ基準](#p2pアーキテクチャ基準)

## P2Pアーキテクチャ原則
1. **完全サーバーレス:** 中央サーバー、データベース、クラウドAPI一切不要。全てデバイス間直接通信。
2. **Bluetooth互換UX:** ペアリングはQRコード/PIN/NFCで完結。アカウント登録・ログイン不要。
3. **ローカルファースト:** 鍵、信頼済みピア情報、メトリクスは全てデバイスローカルに保存。
4. **NAT越え対応:** WebRTC STUN/TURNでファイアウォール・NAT越えを実現。
5. **対称設計:** 全デバイスが同じP2Pスタックを実装。クライアント/サーバーの区別なし。
6. **Pure Rust実装:** C/C++依存ゼロ。メモリ安全性とクロスプラットフォーム互換性を保証。

## P2Pコンポーネント図

### デバイス間対称設計 (全デバイスが同じスタック)
```
┌─────────────────────────────────────────────────────────────────┐
│                         Device A                                │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         Experience Layer (UI/SDK)                        │  │
│  │  ペアリングUI (QR/PIN表示/スキャン)、デバイスリスト管理  │  │
│  └────────────────────────┬─────────────────────────────────┘  │
│                           │                                     │
│  ┌────────────────────────┴─────────────────────────────────┐  │
│  │   P2P Discovery Engine (mDNS + BLE Advertising)          │  │
│  │  近くのHoneyLinkデバイスを自動検出 (Bluetoothと同じUX)   │  │
│  └────────────────────────┬─────────────────────────────────┘  │
│                           │                                     │
│  ┌────────────────────────┴─────────────────────────────────┐  │
│  │          P2P Session Orchestrator                        │  │
│  │  ペアリング → ECDH鍵交換 → セッション確立 → ストリーム  │  │
│  └─┬────────┬────────┬────────┬────────┬────────────────────┘  │
│    │        │        │        │        │                       │
│  ┌─▼──┐  ┌─▼───┐  ┌─▼────┐  ┌─▼────┐  ┌─▼──────────┐        │
│  │QoS │  │Poli-│  │Crypto│  │Telem-│  │Transport  │        │
│  │Sche│  │cy   │  │X25519│  │etry  │  │QUIC/WebRTC│        │
│  │duler│  │Eng. │  │ChaCha│  │(local)│  │(NAT越え)  │        │
│  └────┘  └─────┘  └──────┘  └──────┘  └─────┬──────┘        │
│                                              │                 │
│  ┌───────────────────────────────────────────▼──────────────┐  │
│  │      Physical Adapter (Wi-Fi/BLE/5G)                     │  │
│  │  mDNSアナウンス、BLEペリフェラル、QUIC/WebRTCエンドポイント│  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                  🔐 暗号化されたP2P直接通信
                    (中央サーバー経由なし)
                             │
┌────────────────────────────┴────────────────────────────────────┐
│                         Device B                                │
│              (同じP2Pスタック - 完全対称)                        │
│  同じコンポーネント構成でデバイスBも動作                         │
└─────────────────────────────────────────────────────────────────┘
```

詳細なP2Pデータフローは[docs/architecture/dataflow.md](./dataflow.md)を参照。

## デバイス発見とペアリング

### 1. デバイス発見 (Bluetoothと同UX)

**mDNS (Multicast DNS)**:
- サービス名: `_honeylink._tcp.local`
- TXTレコード: `device_id`, `device_name`, `device_type`, `version`
- ローカルネットワーク内で自動アナウンス (中央サーバー不要)

**BLE (Bluetooth Low Energy)**:
- HoneyLinkサービスUUIDでアドバタイジング
- アドバタイズメントデータ: `device_id` (短縮), `device_type`
- Bluetooth設定画面と同じUXで「近くのデバイス」表示

### 2. ペアリング手順 (Bluetoothライク)

**QRコード**:
- Device AがQRコード表示 (device_id + public_key含む)
- Device Bがカメラでスキャン
- ECDH鍵交換実行 → 信頼確立

**PINコード**:
- Device Aが6桁PIN表示
- Device BでPIN入力
- PIN検証後、ECDH鍵交換

**NFC (将来)**:
- NFCタップで即座にペアリング
- 近接通信で鍵交換情報伝達

### 3. 信頼モデル (TOFU: Trust On First Use)

- 初回ペアリング時に公開鍵を保存 (`~/.honeylink/trusted_peers.json`)
- 2回目以降は保存済み鍵で自動認証 (ユーザー操作不要)
- 鍵変更検出時は警告表示 + 再ペアリング要求
- 中間者攻撃 (MITM) 対策: 初回ペアリングのみ脆弱 (物理的近接性で緩和)

## P2Pコンポーネント責務
| コンポーネント | P2P責務 | 入出力 | 参照 |
|----------------|----------|--------|------|
| **P2P Discovery Engine** | mDNS/BLEアナウンス、近隣デバイススキャン | デバイスリスト、IPアドレス解決 | [docs/modules/physical-adapter.md](../modules/physical-adapter.md) |
| **Session Orchestrator (P2P)** | **デバイス間直接セッション管理**、ECDH鍵交換 | P2P接続要求、ピア公開鍵 | [docs/security/auth.md](../security/auth.md) |
| **Policy Engine (Local)** | **ローカルポリシー決定**（サーバー問い合わせなし） | デバイス設定、ユーザープリファレンス | ローカルJSON設定ファイル |
| **Transport (QUIC/WebRTC)** | **P2P直接通信**、NAT越え (STUN/TURN) | 暗号化パケット、ネットワークメトリクス | [docs/modules/transport-abstraction.md](../modules/transport-abstraction.md) |
| **Crypto (X25519+ChaCha20)** | **P2P鍵交換**、ローカル鍵保存 (`~/.honeylink/keys`) | 公開鍵、共有秘密 | [docs/security/encryption.md](../security/encryption.md) |
| **QoS Scheduler** | ストリーム優先度制御 (ローカル判断) | ストリームメタデータ、帯域状況 | [docs/modules/qos-scheduler.md](../modules/qos-scheduler.md) |
| **Telemetry (Local)** | **ローカルメトリクス収集**（サーバー送信なし） | ローカルSQLite、任意Grafanaエクスポート | [docs/modules/telemetry-insights.md](../modules/telemetry-insights.md) |
| **Physical Adapter** | Wi-Fi/BLE/5Gアダプタ、mDNSレスポンダー | 無線設定、電波イベント | Pure Rust (C/C++依存なし) |
| **Experience Layer** | **ペアリングUI** (QR/PIN表示/スキャン) | デバイスリスト、ユーザー操作 | [docs/ui/overview.md](../ui/overview.md) |

## P2Pアーキテクチャパターン

### デザインパターン
- **対称ピア設計:** 全デバイスが同じコンポーネント構成。クライアント/サーバーの区別なし。
- **マイクロカーネル:** 最小P2Pコア + プラグイン式プロファイル (例: Gaming, AR/VR, IoT)
- **イベント駆動:** デバイス発見、接続状態変化、ストリーム開始/終了をイベント通知
- **ストラテジパターン:** トランスポート層 (QUIC/WebRTC切り替え)、暗号化方式 (ChaCha20/AES)

### Bluetoothとの比較

| 項目 | Bluetooth 5.3 | HoneyLink P2P |
|------|---------------|---------------|
| **ペアリング** | QR/PIN | QR/PIN/NFC (同UX) |
| **通信範囲** | ~100m (屋内~10m) | Wi-Fi活用で最大300m |
| **レイテンシ** | ~30-50ms | ≤12ms (P99) |
| **帯域幅** | ~2Mbps (実効) | 最大1Gbps (Wi-Fi 6E) |
| **並列ストリーム** | 3-5個 | 100個 |
| **暗号化** | AES-128-CCM | ChaCha20-Poly1305 |
| **NAT越え** | 不可 | WebRTC STUN対応 |
| **サーバー** | 不要 ✅ | 不要 ✅ |

## 境界と依存管理

### P2Pアーキテクチャの依存ルール
- **サーバー依存禁止:** 中央サーバー、データベース、クラウドAPIへの依存一切禁止
- **ローカルファースト:** 全データはデバイスローカルに保存 (`~/.honeylink/`)
- **層間依存:** Experience → Session Orchestrator → Transport → Physical Adapter (上位→下位のみ)
- **Pure Rust:** C/C++ネイティブライブラリ依存禁止 (mdns-sd, btleplug等Rustクレート使用)

### 外部依存 (任意)
- **STUN/TURNサーバー:** NAT越え用 (例: `stun:stun.l.google.com:19302`) - 設定可能
- **Grafana/Prometheus:** 開発時のローカル可観測性のみ。本番不要

詳細: [docs/architecture/dependencies.md](./dependencies.md)

## ローカル可観測性

### ローカルメトリクス収集 (サーバー送信なし)
- **ローカルSQLite:** `~/.honeylink/metrics/metrics.db` にメトリクス保存
- **OpenTelemetry SDK:** ローカルOTLP Collectorへのエクスポート (開発時任意)
- **プライバシー保護:** メトリクスはデバイス外に送信しない (ユーザー許可時のみ任意Grafana連携)

### 主要メトリクス
- P2P接続数、アクティブセッション数
- ストリームレイテンシ (P50, P99, P99.9)
- パケット損失率、再送回数
- デバイス発見時間 (mDNS/BLE)
- ペアリング所要時間

詳細: [docs/testing/metrics.md](../testing/metrics.md)

## 変更容易性評価 (P2P設計)

| 領域 | 変更例 | 影響範囲 | P2Pメリット |
|------|--------|----------|--------|
| **デバイス発見** | BLE→NFC追加 | Discovery Engineのみ | サーバーAPI変更不要 |
| **ペアリング方式** | PIN→生体認証 | Experience Layer | バックエンド修正不要 |
| **暗号化更新** | ChaCha20→AES-GCM | Cryptoモジュール | 両デバイスのアップデートのみ |
| **トランスポート** | QUIC→WebRTC切り替え | Transport Layer | サーバーインフラ変更不要 |
| **プロファイル追加** | Gaming→IoT | Policy Engine | ローカルJSON追加のみ |

### P2P設計の利点
- **バックエンド不要:** サーバーコード修正・デプロイ不要
- **即座アップデート:** デバイスアプリ更新のみで機能追加
- **ダウンタイムゼロ:** サーバーメンテナンス不要

## 受け入れ基準 (DoD)

### P2Pアーキテクチャ基準
- ✅ すべてのコンポーネントが**中央サーバーに依存しない**ことを確認
- ✅ デバイス発見がmDNS/BLEで実装され、サーバー登録不要
- ✅ ペアリングがQR/PIN/NFCで完結し、アカウント作成不要
- ✅ 鍵管理がローカルファイルシステム (`~/.honeylink/`) で完結
- ✅ メトリクスがローカルに保存され、サーバー送信なし
- ✅ NAT越えがWebRTC STUN/TURNで実現 (ファイアウォール対応)
- ✅ C/C++依存ゼロ (Pure Rust実装)
- ✅ コンポーネント図がBluetooth標準との比較を含む
- ✅ 全ドキュメントがP2P設計を反映 (Control Plane削除済み)
- ✅ レビュー記録が[docs/notes/decision-log.md](../notes/decision-log.md)に存在
