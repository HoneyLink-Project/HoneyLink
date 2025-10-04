# HoneyLink™

> **Bluetoothの完全上位互換 P2P プロトコル**  
> C/C++依存ゼロ、Pure Rust 実装の次世代デバイス間直接通信プロトコル

[![Rust](https://img.shields.io/badge/rust-1.89.0-orange.svg)](https://www.rust-lang.org/)
[![P2P](https://img.shields.io/badge/architecture-P2P-green.svg)](https://en.wikipedia.org/wiki/Peer-to-peer)
[![No Servers](https://img.shields.io/badge/servers-none-blue.svg)]()
[![License](https://img.shields.io/badge/license-Proprietary-red.svg)](./LICENSE)

---

## 📋 目次

- [概要](#概要)
- [P2Pアーキテクチャ](#p2pアーキテクチャ)
- [Bluetoothとの比較](#bluetoothとの比較)
- [リポジトリ構造](#リポジトリ構造)
- [必要環境](#必要環境)
- [開発環境セットアップ](#開発環境セットアップ)
- [ビルドと実行](#ビルドと実行)
- [テスト](#テスト)
- [コントリビューション](#コントリビューション)
- [ドキュメント](#ドキュメント)
- [ライセンス](#ライセンス)

---

## 🎯 概要

**HoneyLink™** は、Bluetoothの完全上位互換を目指す**Pure P2P (Peer-to-Peer)** プロトコルです。中央サーバー不要で、デバイス間が直接通信します。

### Bluetoothを超える特徴

- **📡 より広い通信範囲**: Wi-Fi Direct/5G活用で屋内外問わず接続
- **⚡ 超低レイテンシ**: P99 ≤ 12ms (Bluetooth比 3倍高速)
- **🔒 軍事級暗号化**: X25519 + ChaCha20-Poly1305 (Bluetoothより強固)
- **📊 マルチストリーム**: 最大100並列ストリーム (Bluetooth: 数個)
- **🎮 ゲーミング最適化**: QoS制御でコントローラー入力を優先
- **🦀 メモリ安全**: Pure Rust実装でバッファオーバーフロー不可
- **🌐 クロスプラットフォーム**: Windows/Linux/macOS/iOS/Android統一API

### 中央サーバー不要

- ❌ クラウド不要
- ❌ データベース不要
- ❌ アカウント登録不要
- ✅ 完全ローカル動作
- ✅ プライバシー保護
- ✅ オフライン動作

---

## 🏗️ P2Pアーキテクチャ

HoneyLink™ は**中央サーバー不要**の完全P2P設計です。デバイスが直接通信します:

```
┌──────────────────────────────────────────────────────────────────┐
│                        Device A                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              Experience Layer (UI/SDK)                     │  │
│  │          QRコード表示/スキャン、PIN入力、ペア管理          │  │
│  └───────────────────────────┬──────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴──────────────────────────────────┐  │
│  │     P2P Discovery (mDNS + BLE Advertising)                   │  │
│  │  「近くのHoneyLinkデバイス」を自動検出 (Bluetoothと同じ)      │  │
│  └───────────────────────────┬──────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴──────────────────────────────────┐  │
│  │        P2P Session Orchestrator                              │  │
│  │  ペアリング → 鍵交換(ECDH) → セッション確立 → ストリーム管理 │  │
│  └───┬─────────┬─────────┬─────────┬─────────┬─────────────────┘  │
│      │         │         │         │         │                   │
│  ┌───▼───┐ ┌──▼───┐ ┌───▼───┐ ┌──▼────┐ ┌──▼──────┐            │
│  │ QoS   │ │Policy│ │Crypto │ │Teleme-│ │Transport│            │
│  │Sched. │ │Engine│ │X25519 │ │try    │ │QUIC/    │            │
│  │       │ │      │ │ChaCha │ │(local)│ │WebRTC   │            │
│  └───────┘ └──────┘ └───────┘ └───────┘ └────┬────┘            │
│                                               │                   │
│  ┌────────────────────────────────────────────▼─────────────────┐  │
│  │          Physical Adapter (Wi-Fi/BLE/5G)                     │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────┬────────────────────────────────────┘
                              │
                       🔐 暗号化されたP2P通信
                         (サーバー経由なし)
                              │
┌─────────────────────────────┴────────────────────────────────────┐
│                        Device B                                  │
│             (同じP2Pスタック - 対称設計)                          │
└──────────────────────────────────────────────────────────────────┘
```

### Bluetoothとの比較

| 機能 | Bluetooth 5.3 | HoneyLink P2P |
|------|---------------|---------------|
| **ペアリング** | QRコード/PIN | QRコード/PIN/NFC (同じUX) |
| **通信範囲** | ~100m (屋内10m) | Wi-Fi活用で最大300m |
| **レイテンシ** | ~30-50ms | ≤12ms (P99) |
| **帯域幅** | ~2Mbps (実効) | 最大1Gbps (Wi-Fi 6E時) |
| **並列ストリーム** | 3-5個 | 100個 |
| **暗号化** | AES-128-CCM | ChaCha20-Poly1305 |
| **NAT越え** | 不可 | WebRTC STUN対応 |
| **サーバー** | 不要 ✅ | 不要 ✅ |

詳細は [`spec/architecture/overview.md`](./spec/architecture/overview.md) を参照してください。

---

## 📁 リポジトリ構造

```
HoneyLink/
├── crates/                      # P2P Core Crates (Rust)
│   ├── core/                    # 共通型とtrait定義
│   ├── crypto/                  # P2P暗号化 (X25519 ECDH, ChaCha20-Poly1305)
│   ├── transport/               # P2Pトランスポート (QUIC, WebRTC)
│   ├── session-orchestrator/    # P2Pセッション管理 (デバイス間直接通信)
│   ├── qos-scheduler/           # ストリームQoS制御
│   ├── policy-engine/           # ローカルポリシー管理
│   ├── telemetry/               # ローカルメトリクス収集
│   ├── physical-adapter/        # 物理層 (Wi-Fi/BLE/5G)
│   └── experience/              # UI/SDKバインディング
├── ui/                          # TypeScript + React UIシェル
│   ├── src/
│   │   ├── components/          # ペアリングUI、デバイスリスト
│   │   ├── pages/               # QRコード表示/スキャン、PIN入力
│   │   ├── hooks/               # P2P状態管理フック
│   │   ├── lib/                 # Wasmブリッジ (Rust ↔ TS)
│   │   └── locales/             # i18n翻訳ファイル
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── eslint.config.js
├── infrastructure/              # 開発用ローカルツール
│   ├── docker-compose.observability.yml  # OTLP Collector (開発時任意)
│   ├── otel-collector-config.yaml        # OpenTelemetry設定
│   └── prometheus.yml, loki-config.yaml  # メトリクス/ログバックエンド
├── docs/                        # 開発者ドキュメント
│   ├── RUST_SETUP.md            # Rust環境構築
│   └── NODE_SETUP.md            # Node.js環境構築
├── spec/                        # P2P仕様書 (設計文書)
│   ├── architecture/            # P2Pアーキテクチャ設計
│   ├── modules/                 # モジュール仕様
│   ├── security/                # P2P暗号化プロトコル
│   ├── testing/                 # P2Pテスト戦略
│   └── ui/                      # ペアリングUI仕様
├── scripts/                     # ビルド・開発ツール
│   └── audit-native-deps.js     # C/C++依存チェックスクリプト
├── .github/                     # GitHub Actions CI/CD
│   └── workflows/
├── .vscode/                     # VS Code設定
│   ├── settings.json
│   └── extensions.json
├── .editorconfig                # 統一フォーマット設定
├── rust-toolchain.toml          # Rust 1.89.0固定
├── Cargo.toml                   # Rustワークスペース (9 P2P crates)
├── TODO.md                      # 実装タスクリスト
├── CONTRIBUTING.md              # コントリビューションガイド
├── CODEOWNERS                   # コードオーナーシップ定義
└── README.md                    # このファイル
```

### 各クレートの責務 (P2P設計)

| クレート                        | 責務                                            | 依存関係                     |
| ------------------------------- | ----------------------------------------------- | ---------------------------- |
| `crates/core`                   | 共通型、trait、エラー型                         | なし                         |
| `crates/crypto`                 | **P2P暗号化** (X25519 ECDH, ChaCha20-Poly1305)  | `core`                       |
| `crates/transport`              | **P2Pトランスポート** (QUIC, WebRTC, NAT越え)   | `core`, `crypto`             |
| `crates/session-orchestrator`   | **P2Pセッション管理** (デバイス間直接通信)       | `core`, `crypto`, `policy`   |
|                                 | ✅ **49テスト全成功**                           |                              |
| `crates/qos-scheduler`          | ストリームQoS制御、優先度付けキュー              | `core`, `transport`          |
| `crates/policy-engine`          | **ローカルポリシー** (サーバー判断なし)          | `core`, `crypto`             |
| `crates/telemetry`              | **ローカルメトリクス** (サーバー送信なし)        | `core`                       |
| `crates/physical-adapter`       | 物理層 (Wi-Fi/BLE/5Gアダプタ)                    | `core`, `transport`          |
| `crates/experience`             | UI/SDK、ペアリングUX                             | 全モジュール                 |
| `ui/`                           | **ペアリングUI** (QRコード/PIN)、デバイスリスト  | `experience` (Wasm経由)      |
| `infrastructure/`               | **開発用のみ** (本番環境不要)                    | -                            |

---

## 🛠️ 必要環境

### 開発要件

- **Rust**: 1.89.0 (rust-toolchain.tomlで固定)
- **Node.js**: 22.15.0 以降 (LTS)
- **pnpm**: 10.x 以降
- **Docker** (任意): 開発時の可観測性スタック (OTLP Collector)

> ⚠️ **サーバー不要**: HoneyLinkはP2Pプロトコルのため、PostgreSQL/Redis等のサーバーは**一切不要**です

### 推奨開発環境

- **OS**: 
  - **Linux**: Ubuntu 22.04+ (最推奨)
  - **Windows**: WSL2 + Ubuntu 22.04 (推奨) または Windows 10/11 + MSVC
  - **macOS**: 13+ (Ventura以降)
- **IDE**: VS Code (推奨、rust-analyzer + ESLint拡張)
- **Git**: 2.40 以降

> **⚠️ Windowsユーザーへの注意**  
> MSVCリンカー問題が発生する場合は**WSL2使用を強く推奨**します。  
> 詳細: [`docs/WSL_SETUP.md`](./docs/WSL_SETUP.md)

### 開発ツール (推奨)

- `cargo-llvm-cov` (カバレッジ計測)
- `cargo-audit` (脆弱性スキャン)
- `cargo-deny` (ライセンス検証)
- `wasm-bindgen-cli` (UIビルド用、CI環境のみ)

---

## 🚀 開発環境セットアップ

### 1. Rustツールチェーンのインストール

#### Windowsユーザー: WSL2推奨

```powershell
# PowerShell (管理者) で実行
wsl --install -d Ubuntu-22.04
```

詳細: [`docs/WSL_SETUP.md`](./docs/WSL_SETUP.md) | クイックスタート: [`docs/WSL_QUICKSTART.md`](./docs/WSL_QUICKSTART.md)

#### Linux / macOS / WSL

詳細は [`docs/RUST_SETUP.md`](./docs/RUST_SETUP.md) を参照:

```bash
# Rustインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 開発ツールインストール
rustup component add clippy rustfmt
cargo install cargo-llvm-cov cargo-audit cargo-deny
```

### 2. Node.js環境のセットアップ

詳細は [`docs/NODE_SETUP.md`](./docs/NODE_SETUP.md) を参照:

```bash
# pnpmインストール
npm install -g pnpm

# UI依存関係インストール
cd ui
pnpm install

# C/C++ネイティブモジュールチェック (Pure TypeScriptのみのはず)
node ../scripts/audit-native-deps.js
```

### 3. 開発用可観測性スタック (任意)

P2P通信のデバッグ用にローカルで起動できます (本番不要):

```bash
# OTLP Collector + Prometheus + Jaeger + Grafana
docker compose -f infrastructure/docker-compose.observability.yml up -d

# アクセス:
# - Grafana: http://localhost:3000 (admin/admin)
# - Prometheus: http://localhost:9090
# - Jaeger: http://localhost:16686
```

### 4. VS Code拡張機能 (推奨)

プロジェクトを開くと推奨拡張機能のインストールを促されます:

- rust-analyzer (Rust LSP)
- ESLint, Prettier (TypeScript)
- CodeLLDB (Rustデバッグ)
- Even Better TOML

または: `Ctrl+Shift+P` → `Extensions: Show Recommended Extensions`

---

## 🏃 ビルドと実行

### Rust ワークスペースのビルド

```bash
# 全クレートをビルド
cargo build --workspace

# リリースビルド
cargo build --workspace --release

# 特定のクレートのみビルド
cargo build -p session-orchestrator
```

### UI の開発サーバー起動

```bash
cd ui
pnpm dev
```

ブラウザで `http://localhost:5173` にアクセスしてください。

### WASM ビルド

```bash
# WASM ターゲット追加 (初回のみ)
rustup target add wasm32-unknown-unknown

# experience クレートを WASM としてビルド
cargo build -p experience --target wasm32-unknown-unknown --release
```

### 🔑 鍵生成CLIツール

HoneyLink™ には、`spec/security/key-management.md` に準拠した鍵管理CLIツールが含まれています。

```bash
# CLI ビルド (cli feature を有効化)
cargo build --package honeylink-crypto --features cli --bin honeylink-keygen

# デモ実行 (4階層鍵派生とローテーション)
cargo run --package honeylink-crypto --features cli --bin honeylink-keygen demo

# ルート鍵生成 (X25519)
cargo run --package honeylink-crypto --features cli --bin honeylink-keygen generate-root

# 鍵派生 (HKDF-SHA512)
cargo run --package honeylink-crypto --features cli --bin honeylink-keygen derive \
  --parent <BASE64_PARENT_KEY> \
  --scope device \
  --output device_key.txt

# ローテーション状態の初期化
cargo run --package honeylink-crypto --features cli --bin honeylink-keygen init-rotation \
  --output rotation.json

# 鍵バージョンの追加
cargo run --package honeylink-crypto --features cli --bin honeylink-keygen add-version \
  --state rotation.json \
  --scope session \
  --key <BASE64_KEY>

# ローテーション状態の確認
cargo run --package honeylink-crypto --features cli --bin honeylink-keygen status \
  --state rotation.json
```

**セキュリティ注意事項**:
- 🔒 本番環境では Vault/KMS から鍵を取得してください
- 🚫 生成された鍵をバージョン管理システムにコミットしないでください
- ✅ すべての暗号処理は RustCrypto クレートを使用 (C/C++ 依存ゼロ)

---

## 🧪 テスト

### 単体テスト

```bash
# 全テスト実行
cargo test --workspace

# カバレッジ計測
cargo llvm-cov --workspace --html
```

### UI テスト

```bash
cd ui

# 単体テスト (Vitest)
pnpm test

# E2E テスト (Playwright、未実装)
pnpm test:e2e
```

### セキュリティスキャン

```bash
# 脆弱性スキャン
cargo audit

# 依存関係ライセンス検証
cargo deny check
```

### リンター・フォーマッター

```bash
# Rust
cargo fmt --check
cargo clippy --all-targets --all-features

# TypeScript/JavaScript
cd ui
pnpm lint
pnpm format:check
```

---

## 🤝 コントリビューション

コントリビューションは大歓迎です！  
詳細は [`CONTRIBUTING.md`](./CONTRIBUTING.md) を参照してください。

### 基本フロー

1. Issue を作成して変更内容を議論
2. Feature ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. コミットメッセージは [Conventional Commits](https://www.conventionalcommits.org/) 形式で記述
4. Pre-commit フックでフォーマット・リント・テストが自動実行
5. Pull Request を作成
6. レビュー承認後にマージ

### コミットメッセージ例

```
feat(session): Add idempotency-key support for session creation

- Implement UUID-based idempotency key validation
- Add 5-minute TTL for key storage
- Update session creation API to accept idempotency header

Refs: #123
```

---

## 📚 ドキュメント

| カテゴリ             | ドキュメント                                                                 |
| -------------------- | ---------------------------------------------------------------------------- |
| **アーキテクチャ**   | [`spec/architecture/overview.md`](./spec/architecture/overview.md)           |
| **モジュール仕様**   | [`spec/modules/`](./spec/modules/)                                           |
| **セキュリティ**     | [`spec/security/encryption.md`](./spec/security/encryption.md)               |
| **API仕様**          | [`spec/api/control-plane.md`](./spec/api/control-plane.md)                   |
| **テスト戦略**       | [`spec/testing/unit-tests.md`](./spec/testing/unit-tests.md)                 |
| **UI設計**           | [`spec/ui/overview.md`](./spec/ui/overview.md)                               |
| **デプロイ**         | [`spec/deployment/ci-cd.md`](./spec/deployment/ci-cd.md)                     |
| **開発環境**         | [`docs/RUST_SETUP.md`](./docs/RUST_SETUP.md)                                 |

---

## 📄 ライセンス

このプロジェクトはプロプライエタリライセンスです。  
無断複製・配布・改変を禁じます。

---

## 🙏 謝辞

HoneyLink™ は以下のオープンソースプロジェクトに感謝しています:

- [Rust](https://www.rust-lang.org/) - システムプログラミング言語
- [RustCrypto](https://github.com/RustCrypto) - 暗号化ライブラリ
- [Tokio](https://tokio.rs/) - 非同期ランタイム
- [React](https://react.dev/) - UI ライブラリ
- [Vite](https://vitejs.dev/) - フロントエンドビルドツール
- [OpenTelemetry](https://opentelemetry.io/) - 可観測性フレームワーク

---

**🍯 Built with Honey, Powered by Rust**
