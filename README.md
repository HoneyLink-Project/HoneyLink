# HoneyLink™

> **次世代マルチデバイスセッションプラットフォーム**  
> C/C++依存ゼロ、Rust + TypeScript で構築された高品質・高信頼性のデバイス間通信基盤

[![Rust](https://img.shields.io/badge/rust-1.89.0-orange.svg)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.x-blue.svg)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/license-Proprietary-red.svg)](./LICENSE)

---

## 📋 目次

- [概要](#概要)
- [アーキテクチャ](#アーキテクチャ)
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

**HoneyLink™** は、IoT・AR/VR・8Kメディア・ゲーミングなど多様なユースケースに対応する、セキュアでスケーラブルなマルチデバイスセッションプラットフォームです。

### 主な特徴

- **🦀 純粋Rust実装**: C/C++依存を完全に排除し、メモリ安全性と高性能を両立
- **🔒 ゼロトラスト設計**: X25519/ChaCha20-Poly1305/HKDF-SHA512 による強固な暗号化
- **⚡ 低レイテンシ**: P99 ≤ 12ms の超低遅延セッション確立
- **📊 完全な可観測性**: OpenTelemetry による包括的なメトリクス・トレース・ログ
- **🌐 国際化対応**: en/ja/es/zh 4言語サポート、RTL レイアウト対応
- **♿ アクセシビリティ**: WCAG 2.2 AA 準拠

---

## 🏗️ アーキテクチャ

HoneyLink™ は8つのコアモジュールで構成されています:

```
┌─────────────────────────────────────────────────────────────┐
│                     Experience Layer                        │
│              (SDK API + UI Shell + i18n)                    │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────────┐
│                   Control Plane API                         │
│     (Device Mgmt / Session / Policy / Audit / Telemetry)    │
└────┬──────────┬──────────┬──────────┬──────────┬───────────┘
     │          │          │          │          │
     ▼          ▼          ▼          ▼          ▼
┌─────────┐ ┌─────────┐ ┌──────┐ ┌─────────┐ ┌──────────┐
│ Session │ │ Policy  │ │ QoS  │ │ Crypto  │ │Telemetry │
│ Orch.   │ │ Engine  │ │Sched.│ │ & Trust │ │& Insights│
└────┬────┘ └────┬────┘ └──┬───┘ └────┬────┘ └────┬─────┘
     │           │          │          │           │
     └───────────┴──────────┴──────────┴───────────┘
                     │
┌────────────────────┴────────────────────────────────────────┐
│              Transport Abstraction Layer                    │
│          (FEC / WFQ / Multi-Path Routing)                   │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────────┐
│             Physical Adapter Layer                          │
│        (Wi-Fi 6E/7 / 5G / THz / Bluetooth)                  │
└─────────────────────────────────────────────────────────────┘
```

詳細は [`spec/architecture/overview.md`](./spec/architecture/overview.md) を参照してください。

---

## 📁 リポジトリ構造

```
HoneyLink/
├── backend/                     # バックエンドサービス (今後実装)
├── crates/                      # Rust ワークスペース
│   ├── core/                    # 共通型と trait 定義
│   ├── session-orchestrator/    # セッション管理
│   ├── policy-engine/           # ポリシー・プロファイル管理
│   ├── transport/               # トランスポート抽象化
│   ├── crypto/                  # 暗号化・信頼アンカー
│   ├── qos-scheduler/           # QoS スケジューラ
│   ├── telemetry/               # テレメトリ・可観測性
│   ├── physical-adapter/        # 物理層アダプタ
│   └── experience/              # SDK・UI バインディング
├── ui/                          # TypeScript + React UI
│   ├── src/
│   │   ├── components/          # 再利用可能コンポーネント
│   │   ├── pages/               # ページコンポーネント
│   │   ├── hooks/               # カスタムフック
│   │   ├── lib/                 # ユーティリティ
│   │   └── locales/             # i18n 翻訳ファイル
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── eslint.config.js
│   └── .prettierrc.json
├── infrastructure/              # IaC (Terraform/Bicep)
│   ├── terraform/               # Terraform モジュール
│   ├── kubernetes/              # K8s マニフェスト
│   └── scripts/                 # デプロイスクリプト
├── docs/                        # 開発者ドキュメント
│   ├── RUST_SETUP.md            # Rust 環境構築
│   └── NODE_SETUP.md            # Node.js 環境構築
├── spec/                        # 仕様書 (ADR, 設計文書)
│   ├── architecture/
│   ├── modules/
│   ├── security/
│   ├── testing/
│   └── ui/
├── scripts/                     # ビルド・開発ツール
│   └── audit-native-deps.js     # C/C++依存チェックスクリプト
├── .github/                     # GitHub Actions CI/CD
│   └── workflows/
├── .vscode/                     # VS Code 設定
│   ├── settings.json
│   └── extensions.json
├── .editorconfig                # 統一フォーマット設定
├── rust-toolchain.toml          # Rust ツールチェーン定義
├── Cargo.toml                   # Rust ワークスペース設定
├── TODO.md                      # 実装タスクリスト
├── CONTRIBUTING.md              # コントリビューションガイド
├── CODEOWNERS                   # コードオーナーシップ定義
└── README.md                    # このファイル
```

### 各ディレクトリの責務

| ディレクトリ         | 責務                                                         | 依存関係                              |
| -------------------- | ------------------------------------------------------------ | ------------------------------------- |
| `crates/core`        | 共通型、trait、エラー型の定義                                | なし（全モジュールの基盤）            |
| `crates/crypto`      | 暗号化、鍵管理、署名検証                                     | `core`                                |
| `crates/transport`   | トランスポート抽象化、FEC、WFQ                               | `core`, `crypto`                      |
| `crates/qos-scheduler` | ストリーム優先度制御、帯域配分                             | `core`, `transport`                   |
| `crates/policy-engine` | ポリシー・プロファイル管理                                 | `core`, `crypto`                      |
| `crates/session-orchestrator` | セッション状態管理、ハンドシェイク                 | `core`, `crypto`, `policy-engine`     |
|                              | ✅ **実装完了** (49テスト、100%成功)                |                                       |
| `crates/telemetry`   | OpenTelemetry メトリクス・トレース                           | `core`                                |
| `crates/physical-adapter` | 物理層ドライバ抽象化                                    | `core`, `transport`                   |
| `crates/experience`  | SDK API、WASM バインディング                                 | 全モジュール                          |
| `ui/`                | TypeScript + React フロントエンド                            | `crates/experience` (WASM 経由)       |
| `backend/`           | 将来のバックエンドサービス (Control Plane API)               | 全 crates                             |
| `infrastructure/`    | IaC、K8s マニフェスト、デプロイスクリプト                    | デプロイ対象                          |

---

## 🛠️ 必要環境

### 基本要件

- **Rust**: 1.89.0 以降 (LTS)
- **Node.js**: 22.15.0 以降 (LTS)
- **pnpm**: 10.x 以降
- **Docker & Docker Compose**: Database development (TimescaleDB)

### 推奨開発環境

- **OS**: 
  - **Linux**: Ubuntu 22.04+ (推奨)
  - **Windows**: WSL2 + Ubuntu 22.04 (推奨) または Windows 10/11 + Visual Studio Build Tools
  - **macOS**: 13+
- **IDE**: VS Code (推奨) または JetBrains IDEs
- **Git**: 2.40 以降

> **⚠️ Windows ユーザーへの重要な注意**  
> Windows環境でMSVCリンカーの問題が発生する場合は、**WSL2を使用することを強く推奨**します。  
> 詳細: [`docs/WSL_SETUP.md`](./docs/WSL_SETUP.md)

### 追加ツール

- `cargo-llvm-cov` (カバレッジ計測)
- `cargo-audit` (脆弱性スキャン)
- `cargo-deny` (依存関係ライセンス検証)
- `wasm-bindgen-cli` (WASM ビルド、CI のみ)

---

## 🚀 開発環境セットアップ

### Windows ユーザー: WSL2を使用（推奨）

Windows環境でのMSVCリンカー問題を回避するため、WSL2の使用を推奨します:

```powershell
# PowerShell (管理者権限) で実行
wsl --install -d Ubuntu-22.04
```

詳細なセットアップ手順:
- **クイックスタート**: [`docs/WSL_QUICKSTART.md`](./docs/WSL_QUICKSTART.md)
- **詳細ガイド**: [`docs/WSL_SETUP.md`](./docs/WSL_SETUP.md)

自動セットアップスクリプト（WSL内で実行）:
```bash
curl -sSL https://raw.githubusercontent.com/HoneyLink-Project/HoneyLink/master/scripts/setup-wsl.sh | bash
```

### Linux / macOS / WSL: Rust ツールチェーンのインストール

詳細は [`docs/RUST_SETUP.md`](./docs/RUST_SETUP.md) を参照してください。

```bash
# Rust インストール (rustup 経由)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# コンポーネント追加
rustup component add clippy rustfmt

# 開発ツールインストール
cargo install cargo-llvm-cov cargo-audit cargo-deny
```

### 2. Database Setup (TimescaleDB)

詳細は [`docs/DATABASE_SETUP.md`](./docs/DATABASE_SETUP.md) を参照してください。

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Start TimescaleDB
docker compose -f infrastructure/docker-compose.db.yml up -d

# Run migrations
cd backend
sqlx migrate run
cargo sqlx prepare
```

### 3. Node.js 環境のセットアップ

詳細は [`docs/NODE_SETUP.md`](./docs/NODE_SETUP.md) を参照してください。

```bash
# pnpm インストール (npm 経由)
npm install -g pnpm

# UI 依存関係インストール
cd ui
pnpm install

# C/C++ ネイティブモジュールチェック
node ../scripts/audit-native-deps.js
```

### 4. VS Code 拡張機能のインストール

VS Code で開いた際、推奨拡張機能のインストールを促すプロンプトが表示されます。  
または、コマンドパレット (`Ctrl+Shift+P`) から:

```
Extensions: Show Recommended Extensions
```

推奨拡張機能一覧は [`.vscode/extensions.json`](./.vscode/extensions.json) を参照してください。

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
