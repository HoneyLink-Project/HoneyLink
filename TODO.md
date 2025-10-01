# HoneyL### 0.1 ワーキンググループ設立
- [x] 仕様ワーキンググループ(アーキテクチャ/プロトコル/UX/セキュリティ/オペレーション)の担当者を確定し、連絡チャネルとレビュー c- [x] VS Code / JetBrains などの IDE に Rust Analyzer・ESLint・Prettier 等のプラグインを設定し、共通設定を `.editorconfig` として共有する。
  - [x] `.vscode/settings.json` と `.vscode/extensions.json` を作成
  - [x] `.editorconfig` で統一フォーマット設定を定義
  - [x] Rust Analyzer の設定を最適化(チェックコマンド、クリッピーの有効化)
  - [x] ESLint/Prettier の設定ファイルを作成
  - ✅ `.vscode/settings.json` (rust-analyzer + clippy + TypeScript設定)、`.vscode/extensions.json` (推奨拡張機能28個)、`.editorconfig` (言語別フォーマット)、`ui/eslint.config.js` (Flat Config + TypeScript/React)、`ui/.prettierrc.json` 作成完了
- [x] 各WGのミーティング議事録テンプレートを `spec/notes/meeting-notes.md` に準拠して作成する。
- [x] ステークホルダー出席率90%以上を維持するためのリマインダー/エスカレーション体制を確立する。
  - [x] リマインダー/エスカレーションシステムを `spec/notes/attendance-system.md` として文書化完了

# HoneyLink™ 実装 TODO

> HoneyLink™ 仕様群（`spec/` 配下）を実装へ落とし込むための包括的なタスクリスト。全作業で **C/C++ 依存を排除** し、各ドキュメントの DoD と整合させること。

---

## 0. キックオフ & ガバナンス

### 0.1 ワーキンググループ設立
- [x] 仕様ワーキンググループ（アーキテクチャ/プロトコル/UX/セキュリティ/オペレーション）の担当者を確定し、連絡チャネルとレビュー cadence（週次）を設定する。
- [x] 各WGのミーティング議事録テンプレートを `spec/notes/meeting-notes.md` に準拠して作成する。
- [ ] ステークホルダー出席率90%以上を維持するためのリマインダー/エスカレーション体制を確立する。

### 0.2 ロードマップ展開
- [x] `spec/roadmap.md` に沿って P1→P4 フェーズのマイルストーンを実装ロードマップへ展開し、プロジェクト管理ツールへ同期する。
- [x] 各フェーズのエントリ/エグジット基準を確認リストとして文書化し、承認プロセスを定義する。
  - [x] P0〜P4の全フェーズエントリ/エグジット基準を `spec/notes/phase-criteria.md` として文書化完了
- [x] `spec/roadmap.md` の依存関係マップを実装タスクへマッピングし、クリティカルパスを特定する。
  - [x] クリティカルパス分析 (97日 → 67日最適化) を `spec/notes/critical-path-analysis.md` として文書化完了
  - [x] タスク依存マトリクスとボトルネック特定を完了
- [x] スケジュール遵守率95%を測定するダッシュボードを設定する。
  - [x] ダッシュボード設計仕様 (Google Sheets/Grafana オプション) を `spec/notes/schedule-dashboard.md` として文書化完了
  - [x] 測定指標定義 (遵守率/平均遅延/バッファ消費率) を完了
  - [x] アラート設定 (Slack Webhook統合) とデータ収集フローを文書化完了

### 0.3 モジュール仕様書策定
- [x] `spec/templates/module-template.md` をベースに、**Session Orchestrator** の実装仕様書ドラフトを作成する。
  - [x] 責務・入出力・API 仕様を定義
  - [x] FR-01/FR-02/FR-04 との紐付けを明示
  - [x] ステートマシン図を追加
  - [x] トレーサビリティ ID を付与 (MOD-001-SESSION-ORCH)
  - ✅ `spec/modules/session-orchestrator.md` 作成完了
- [x] **Policy & Profile Engine** の実装仕様書ドラフトを作成する。
  - [x] QoSPolicyUpdate スキーマと SemVer 対応を定義
  - [x] プロファイルテンプレート CRUD 仕様を記述
  - [x] FR-04/FR-06 との紐付けを明示
  - ✅ `spec/modules/policy-profile-engine.md` 作成完了 (MOD-002)
- [x] **Transport Abstraction** の実装仕様書ドラフトを作成する。
  - [x] 物理層アダプタ trait 定義を抽象化
  - [x] FEC 戦略の選択ロジックを記述 (Reed-Solomon, NONE/LIGHT/HEAVY)
  - [x] Weighted Fair Queuing アルゴリズムを仕様化
  - ✅ `spec/modules/transport-abstraction.md` 作成完了 (MOD-003)
- [x] **Crypto & Trust Anchor** の実装仕様書ドラフトを作成する。
  - [x] X25519/ChaCha20-Poly1305/HKDF-SHA512 の利用方針を記述
  - [x] 鍵階層 (Root → DeviceMaster → Session → Stream) を図示
  - [x] `spec/security/encryption.md` との整合性を確認
  - ✅ `spec/modules/crypto-trust-anchor.md` 作成完了 (MOD-004)
- [x] **Stream QoS Scheduler** の実装仕様書ドラフトを作成する。
  - [x] 優先度制御とバックプレッシャロジックを定義
  - [x] 帯域配分 (25/60/15) の enforcement 方法を記述
  - ✅ `spec/modules/qos-scheduler.md` 作成完了 (MOD-005)
- [x] **Telemetry & Insights** の実装仕様書ドラフトを作成する。
  - [x] OpenTelemetry メトリクス/トレース/ログの出力形式を定義
  - [x] アラート閾値 (Yellow/Orange/Red) の設定方法を記述
  - [x] `spec/testing/metrics.md` の SLI との対応を明示
  - ✅ `spec/modules/telemetry-insights.md` 作成完了 (MOD-006)
- [x] **Physical Adapter Layer** の実装仕様書ドラフトを作成する。
  - [x] Wi-Fi/5G/THz 向け抽象ドライバ API を定義
  - [x] gRPC/REST 抽象での通信プロトコルを記述
  - [x] C/C++ バインディング禁止の代替方法を明確化 (Adapter Pattern + プロセス分離)
  - ✅ `spec/modules/physical-adapter.md` 作成完了 (MOD-007)
- [x] **Experience Layer** の実装仕様書ドラフトを作成する。
  - [x] SDK API 仕様と UI Shell コンポーネントを定義
  - [x] `spec/ui/overview.md` との整合性を確認
  - [x] i18n 対応 (en/ja/es/zh) の実装指針を記述
  - ✅ `spec/modules/experience-layer.md` 作成完了 (MOD-008)

### 0.4 ガバナンス体制確立
- [x] `spec/notes/decision-log.md` の運用ルールを定義する。
  - [x] 設計変更・リスク・RCA の記録フォーマットを標準化
  - [x] 72 時間以内の記録完了を enforce する自動リマインダーを設定
  - [x] 承認フローとエスカレーションパスを明文化
  - ✅ `spec/notes/decision-log.md` に運用ルールセクション追加完了 (ADR/リスク/RCA形式、Slackbot/CI/週次リマインダー、4レベル承認フロー)
- [x] レビュー完了率90%を達成するためのチェックリストを作成する。
  - ✅ `spec/notes/review-completion-checklist.md` 作成完了 (OKR定義、8種類レビュー、自動追跡、90%閾値管理)
- [x] ドキュメント品質 DoD 満足度100%を検証するレビューテンプレートを整備する。
  - ✅ `spec/templates/document-quality-dod-review.md` 作成完了 (6次元100点評価、8文書タイプ別チェックリスト、CI統合)

## 1. 開発環境 & ツールチェーン整備

### 1.1 開発者ワークステーション
- [x] Rust stable (最新 LTS) をインストールし、`rustup` プロファイルに `clippy`・`rustfmt`・`cargo-llvm-cov`・`cargo-audit` を追加する。
  - [x] `rustup component add clippy rustfmt` を実行
  - [x] `cargo install cargo-llvm-cov cargo-audit cargo-deny` を実行
  - [x] Rust のバージョンをプロジェクトドキュメントに明記
  - ✅ `docs/RUST_SETUP.md` 作成完了 (Rust 1.89.0、clippy/rustfmt、deny.toml、CI統合、純粋Rustライブラリマトリクス)
- [x] Node.js LTS + pnpm (または npm/yarn) をインストールし、UI 実装用 TypeScript ツールチェーンを構築する（すべて純 Web 技術で C/C++ バイナリビルドを避ける）。
  - [x] Node.js LTS 最新版をインストール
  - [x] `pnpm` をグローバルインストール
  - [x] TypeScript 5.x および必要な型定義パッケージをインストール
  - [x] C/C++ ネイティブモジュールを含むパッケージの使用を禁止するポリシーを文書化
  - ✅ `docs/NODE_SETUP.md` 作成完了 (Node.js 22.15.0、pnpm 10.x、C/C++禁止ポリシー、audit-native-deps.js設計、代替ライブラリマップ)
- [x] WebAssembly ターゲット `wasm32-unknown-unknown` を追加し、必要な場合に `wasm-bindgen` など Rust 生態系のツールを導入する。
  - [x] `rustup target add wasm32-unknown-unknown` を実行
  - [x] `wasm-bindgen-cli` をインストール (CI 環境のみ)
  - [x] WASM ビルドの CI パイプラインを設定
  - ✅ `rust-toolchain.toml` に wasm32 ターゲット追加、`.github/workflows/rust-ci.yml` に test-wasm ジョブ統合完了
- [x] VS Code / JetBrains などの IDE に Rust Analyzer・ESLint・Prettier 等のプラグインを設定し、共通設定を `.editorconfig` として共有する。
  - [x] `.vscode/settings.json` と `.vscode/extensions.json` を作成
  - [x] `.editorconfig` で統一フォーマット設定を定義
  - [x] Rust Analyzer の設定を最適化（チェックコマンド、クリッピーの有効化）
  - [x] ESLint/Prettier の設定ファイルを作成
  - ✅ `.vscode/settings.json` (rust-analyzer + clippy + TypeScript設定)、`.vscode/extensions.json` (推奨拡張機能28個)、`.editorconfig` (言語別フォーマット)、`ui/eslint.config.js` (Flat Config + TypeScript/React)、`ui/.prettierrc.json` 作成完了

### 1.2 リポジトリ初期構成
- [x] ルートに `backend/`, `ui/`, `infrastructure/`, `docs/` 等のディレクトリを作成してモノレポ構成を定義する。
  - [x] ディレクトリ構造を設計し、README.md にドキュメント化
  - [x] 各ディレクトリの責務と相互依存関係を明文化
  - ✅ README.md 作成完了 (包括的なプロジェクト概要、ディレクトリ構造、依存関係マップ)
- [x] Rust ワークスペース (`Cargo.toml`) を構築し、各コアコンポーネントを `crates/<module>` として分割する（依存は一方向で `spec/architecture/dependencies.md` の層別ルールに従う）。
  - [x] ワークスペース用 `Cargo.toml` を作成
  - [x] `crates/session-orchestrator`, `crates/policy-engine`, `crates/transport`, `crates/crypto`, `crates/qos-scheduler`, `crates/telemetry`, `crates/physical-adapter`, `crates/experience` のスタブを作成
  - [x] 各クレートの `Cargo.toml` で依存関係を設定し、循環依存を排除
  - [x] 共通の trait と型定義を `crates/core` に配置
  - ✅ 全9クレート (core, crypto, session-orchestrator, policy-engine, transport, qos-scheduler, telemetry, physical-adapter, experience) 作成完了
- [x] UI は TypeScript + React/Vite など C/C++ 依存のないモダンフレームワークを採用することを決定し、`ui/package.json` を初期化する。
  - [x] Vite + React + TypeScript のプロジェクトテンプレートを作成
  - [x] `package.json` に必要な依存関係を追加（React Router, TanStack Query など）
  - [x] デザインシステム用のディレクトリ構造を設計
  - [x] ビルド設定を最適化（コード分割、ツリーシェイキング）
  - ✅ UI プロジェクト完全セットアップ (Vite + React 18 + TypeScript + TanStack Query + Zustand + Tailwind CSS + i18next)
- [x] 共通の `CONTRIBUTING.md` と `CODEOWNERS` を整備し、ドキュメント方針（実装コード非出力ポリシーとの整合）を記載する。
  - [x] `CONTRIBUTING.md` にコーディング規約、PR プロセス、レビュー基準を記載
  - [x] `CODEOWNERS` で各モジュールの責任者を設定
  - [x] コミットメッセージ規約（Conventional Commits）を定義
  - [x] ドキュメント更新の必須化ルールを明記
  - ✅ CONTRIBUTING.md (包括的開発ガイド)、CODEOWNERS (モジュール別責任者定義) 作成完了
- [x] Pre-commit フックで `cargo fmt`、`cargo clippy`、`eslint`、`stylelint`、`markdownlint` を自動実行する設定を入れる。
  - [x] `pre-commit` フレームワーク設定作成
  - [x] フックスクリプトで各リンターを実行するように設定
  - [x] フック失敗時のコミットブロックを確認
  - [x] CI でも同じチェックを実行し、一貫性を保証
  - ✅ `.pre-commit-config.yaml` (Rust/TypeScript/Markdown リンター統合)、`.markdownlint.json`、`.secrets.baseline` 作成完了

### 1.3 セキュリティ & シークレット管理
- [ ] 開発環境用に HashiCorp Vault SaaS もしくはクラウド KMS をセットアップし、`spec/security/encryption.md` のキー階層 (k_root/k_service/k_profile/k_telemetry) を模した階層構造でトークンを管理する。
  - [ ] Vault インスタンスをセットアップ（開発環境用）
  - [ ] 鍵階層に対応するシークレットパスを設計
  - [ ] アクセスポリシーを定義し、RBAC を設定
  - [ ] Vault との接続用 Rust クライアントライブラリを選定
- [x] `spec/security/key-management.md` のライフサイクル表に沿ってデモ鍵生成フローを Rust CLI で再現し、C/C++ 製ツールを利用しないことを確認する。
  - [x] 鍵生成用 Rust CLI ツールを実装
  - [x] X25519 鍵ペア生成機能を実装
  - [x] HKDF による鍵派生機能を実装
  - [x] 鍵ローテーション機能のプロトタイプを作成
  - [x] すべての暗号処理が RustCrypto クレートを使用していることを確認
  - ✅ `crates/crypto/src/bin/keygen.rs` CLI実装完了 (Root/DeviceMaster/Session/Stream 4階層、Base64エンコード、rotation.rs統合)
- [x] `.env` テンプレート (`.env.example`) を用意し、ローカルの機密情報は Vault 経由でフェッチする OIDC ワークフローを文書化する。
  - [x] `.env.example` に必要な環境変数のテンプレートを記載
  - [x] Vault から環境変数を取得するヘルパースクリプトを作成
  - [x] OIDC トークン取得フローを文書化
  - [x] 開発者オンボーディングドキュメントに Vault 認証手順を追加
  - ✅ `.env.example` 作成完了 (Vault/OIDC/OTEL/DB/Redis/QoS設定、140行、WSL統合、開発モードフラグ)

## 2. システムアーキテクチャ実装

### 2.1 Session Orchestrator ✅ **完了（100%）**
> 📋 完了レポート: `docs/TASK_2.1_SESSION_ORCHESTRATOR_COMPLETION_REPORT.md`

- [x] モジュール仕様書に基づき、ハンドシェイク・セッション状態管理・バージョンネゴシエーションを担う `crates/session-orchestrator` を作成。
  - [x] ステートマシン trait とデフォルト実装を作成
  - [x] セッション永続化インターフェースを定義（DB 抽象化）
  - [x] バージョン交渉プロトコルを実装（SemVer ベース）
  - ✅ `error.rs`, `state_machine.rs`, `session.rs`, `persistence.rs`, `versioning.rs` 実装完了
  - ✅ 9つのモジュールファイルすべて実装完了
- [x] `spec/requirements.md` の FR-01/FR-02/FR-04 をマッピングしたステートマシンを設計し、idempotency-key サポートを盛り込む。
  - [x] セッション ID を UUIDv7 形式で生成
  - [x] 5状態（Pending/Paired/Active/Suspended/Closed）の遷移ロジックを実装
  - [x] 不正な状態遷移検出とエラーハンドリングを実装
  - [x] idempotency-key のストレージと検証ロジックを実装
  - [x] セッションタイムアウト処理を実装（12h TTL、30分滑走更新）
  - ✅ `idempotency.rs` でFNV-1aハッシュベースの重複検出、24時間保持実装完了
- [x] 共有イベントバス（Rust async チャネル）を設計し、Crypto & Trust / Policy Engine と結合させる。
  - [x] 非同期チャネルの選定（tokio::sync::broadcast 採用）
  - [x] イベント型定義とシリアライゼーションを実装
  - ✅ `event_bus.rs` で SessionEstablished/StateChanged/Activity/Closed/Error イベント実装完了
- [x] セッション監査ログ出力を OpenTelemetry 形式 (JSON) で実装し、`spec/testing/metrics.md` の SLI を計測できるようにする。
  - [x] 構造化ログフォーマットを定義
  - [x] SLI メトリクス（セッション数、確立時間、失敗率）の計測を実装
  - ✅ `metrics.rs` でアトミック操作ベースのメトリクスコレクター実装完了

**次のステップ（セクション3で実装予定）:**
- [ ] トークン検証ロジック（OIDC/OAuth2 互換）の Control-Plane API 統合
- [ ] デッドレター処理の Redis Dead Letter Queue 統合
- [ ] セッションイベントの完全な OpenTelemetry SDK 統合

### 2.2 Policy & Profile Engine ✅ **完了（100%）**
> 📋 完了レポート: `docs/TASK_2.2_POLICY_ENGINE_COMPLETION_REPORT.md`

- [x] `spec/architecture/interfaces.md` の `QoSPolicyUpdate` スキーマを Rust タイプへ落とし込み、バージョニング (SemVer) と `deprecated_after` メタを処理する。
  - [x] `QoSPolicyUpdate` 構造体定義を実装
  - [x] SemVer パース・比較ロジックを実装
  - [x] `deprecated_after` メタデータの追跡システムを実装
  - [x] バージョン間互換性マトリクスを実装
  - [x] マイグレーション処理を実装
  - ✅ `crates/policy-engine/src/types.rs` 完成 (QoSPolicyUpdate, FecMode, PowerProfile, UseCase 型定義、SemVer互換性チェック、包括的バリデーション)
- [x] プロファイルテンプレートの CRUD を実装し、`spec/templates/module-template.md` のガイドに沿ってプラガブル戦略を整備する。
  - [x] プロファイルストレージインターフェースを定義
  - [x] CRUD API を実装（Create/Read/Update/Delete）
  - [x] バリデーションルールエンジンを実装
  - [x] Ed25519 署名による検証を実装
  - [x] プロファイルエクスポート機能を実装
  - ✅ `crates/policy-engine/src/profile.rs` 完成 (PolicyProfile型、ProfileStorage trait、InMemoryProfileStorage実装、Ed25519署名検証、包括的CRUD)
- [x] 省電力プロファイル (IoT) と高帯域プロファイル (AR/VR, 8K) を `spec/requirements.md` のユースケースごとにプリセットとして構築。
  - [x] IoT プロファイル（5mA 平均電流、バースト特性）を定義
  - [x] AR/VR プロファイル（P99 12ms、空間同期誤差 5cm）を定義
  - [x] 8K メディアプロファイル（1.5Gbps、フレームドロップ率 0.1%）を定義
  - [x] ゲーミング入力プロファイル（P95 6ms、音声同期 20ms）を定義
  - [x] プリセットのバリデーションテストを実装
  - ✅ `crates/policy-engine/src/presets.rs` 完成 (4プリセット実装: prof_iot_lowpower_v2, prof_arvr_spatial_v1, prof_media_8k_v1, prof_gaming_input_v1)
- [x] Policy 更新イベントを QoS Scheduler へ配信するストリーム (event bus) を設計し、失敗時に旧設定へフォールバックするロジックを実装。
  - [x] イベント発行インターフェースを実装
  - [x] 信頼性保証（最低一度配信）を実装
  - [x] フォールバック用の設定スナップショットを実装
  - [x] ロールバックトリガーとリカバリロジックを実装
  - [x] イベント配信メトリクスを実装
  - ✅ `crates/policy-engine/src/event_bus.rs` 完成 (PolicyEventBus、tokio::sync::broadcast、at-least-once配信、スナップショット管理、ロールバック機能)
  - ✅ `crates/policy-engine/src/policy.rs` 完成 (PolicyEngine統合レイヤー、プロファイルからポリシー生成、イベントバス統合)
  - ✅ 7つのモジュールファイルすべて実装完了

### 2.3 Transport Abstraction & Physical Adapter Layer 🔴 **未完了（15%）**
> ⚠️ 現状: 基本的なスタブファイルのみ存在、詳細実装は未着手
> 📂 実装場所: `crates/transport/` と `crates/physical-adapter/`

**完了済み:**
- [x] 基本モジュール構造作成（`fec.rs`, `wfq.rs`, `adapter.rs`, `traits.rs`）
- [x] `PhysicalLayer` trait の基本定義
- [x] `FecStrategy` と `WeightedFairQueuing` の型スタブ

**未実装（優先度: P1 - 次のタスク）:**
- [ ] 物理層別アダプタ API を Rust trait として定義し、Wi-Fi/5G/THz 向けに抽象ドライバを実装（外部実装とは gRPC/REST 抽象で通信、C/C++ バインディングは禁止）。
  - [ ] `PhysicalLayer` trait の完全実装（send/recv/status/configure）
  - [ ] Wi-Fi 6E/7 アダプタスタブを実装
  - [ ] 5G アダプタスタブを実装
  - [ ] THz アダプタスタブを実装
  - [ ] gRPC/REST ブリッジアダプタを実装
  - [ ] アダプタ検出・登録機能を実装
- [ ] FEC 戦略 (RaptorQ / Reed-Solomon) を Rust 純実装ライブラリで構成し、`spec/performance/benchmark.md` の計測計画で求められるパラメータを設定可能にする。
  - [ ] Reed-Solomon ライブラリ選定・統合（`reed-solomon-erasure` クレート候補）
  - [ ] FEC エンコーダを実装（NONE/LIGHT/HEAVY モード）
  - [ ] FEC デコーダを実装（損失リカバリ）
  - [ ] FEC パラメータ設定インターフェースを実装
  - [ ] FEC 効果メトリクス（リカバリ率、オーバーヘッド）を実装
  - [ ] 動的 FEC モード切替ロジックを実装（ロス率 5% 閾値）
- [ ] QoS Scheduler とのインターフェースで Weighted Fair Queuing を実装し、`spec/performance/scalability.md` の帯域配分 (25/60/15) を強制する。
  - [ ] WFQ スケジューラコアを実装
  - [ ] 重み付け設定とリアルタイム調整を実装
  - [ ] スループット測定と enforcement を実装
  - [ ] キュー溢れ時のドロップポリシーを実装（優先度ベース）
  - [ ] 帯域配分メトリクスを実装
- [ ] Telemetry 発火ポイントを Transport/Physical 層に組み込み、リンク状態変更イベントを Observability 層へ送信する。
  - [ ] `LinkStateChange` イベント定義を実装
  - [ ] イベント発火ポイントを各層に配置
  - [ ] `StreamMetric` 構造体を実装
  - [ ] 非同期メトリクス送信チャネルを実装
  - [ ] バッファリングとバッチ送信を実装

### 2.4 Crypto & Trust Anchor 🟡 **部分完了（50%）**
> 📂 実装場所: `crates/crypto/`
> ✅ keygen CLI実装済み: `crates/crypto/src/bin/keygen.rs`

**完了済み:**
- [x] 基本モジュール構造作成（`key_derivation.rs`, `key_management.rs`, `rotation.rs`, `signing.rs`）
- [x] 鍵階層の基本型定義（`KeyHierarchy`, `KeyScope`）
- [x] 鍵生成 CLI ツールを実装（Root/DeviceMaster/Session/Stream 4階層）
- [x] 鍵ローテーション基本機能（`RotationPolicy`, `VersionedKey`）
- [x] Ed25519 署名機能の基本実装

**未完了（優先度: P1-P2）:**
- [ ] `spec/security/encryption.md` に基づき、X25519 + ChaCha20-Poly1305 + HKDF-SHA512 を RustCrypto スイートで実装し、鍵ライフサイクルを `k_root → k_service → k_session` で管理する。
  - [x] 基本構造は完了
  - [ ] X25519 鍵合意モジュールの完全実装（`x25519-dalek`）
  - [ ] ChaCha20-Poly1305 AEAD の完全実装（`chacha20poly1305`）
  - [ ] HKDF-SHA512 鍵派生の完全実装（`hkdf`）
  - [ ] セキュアメモリ管理の完全実装（`zeroize`）
  - [ ] 鍵派生チェーンの完全実装
  - [ ] スコープベースアクセス制御の完全実装
- [ ] `spec/security/key-management.md` の鍵生成/ローテーション/失効フローを Rust CLI + Vault API で自動化し、エアギャップ手順は別途マニュアル化する。
  - [x] 鍵生成 CLI ツール実装済み
  - [ ] **Vault クライアントアダプタを実装（未着手）**
  - [ ] 90日自動ローテーションスケジューラを実装
  - [ ] 緊急ローテーション機能を実装（30分以内）
  - [ ] 鍵失効リスト管理を実装
  - [ ] エアギャップ環境用マニュアル手順書を作成
  - [ ] 鍵操作監査ログを実装
- [ ] セッション鍵共有の際に `spec/security/auth.md` の PoP トークン生成と連携できる API を用意する。
  - [ ] PoP トークン生成 API を実装
  - [ ] トークン検証ロジックを実装
  - [ ] セッション確立時の鍵交換フローを実装
  - [ ] Perfect Forward Secrecy 保証を実装
  - [ ] デバイス証明書検証を実装（CSR パース、チェーン検証）
  - [ ] CRL/OCSP 連携を実装

### 2.5 Telemetry & Insights 🔴 **未完了（5%）**
> ⚠️ 現状: 基本スタブのみ、実装は未着手
> 📂 実装場所: `crates/telemetry/`
> 🔗 依存: セクション6（観測性統合）と並行実装が必要

**完了済み:**
- [x] 基本モジュール構造作成（`lib.rs`のみ）

**未実装（優先度: P2）:**
- [ ] OpenTelemetry Collector (Rust 版) へのエクスポートを実装し、`spec/testing/metrics.md` のコア指標 (デプロイ成功率/変更失敗率など) をメトリクス化する。
  - [ ] OpenTelemetry SDK 統合（`opentelemetry`, `tracing-opentelemetry`）
  - [ ] メトリクスプロバイダ実装（Counter, Gauge, Histogram）
  - [ ] トレースプロバイダ実装（Span, Context propagation）
  - [ ] ログエクスポーター実装
  - [ ] OTLP エンドポイント設定と接続管理を実装
  - [ ] SLI 定義に基づくメトリクス収集を実装（セッション数、遅延分布、FEC効果、電力消費指数）
- [ ] 収集データを CockroachDB + TimescaleDB (マネージド) へ送信するパイプラインを構築し、`spec/performance/benchmark.md` のベンチ結果を保存できるようにする。
  - [ ] 時系列データストレージアダプタを実装
  - [ ] バッファリング戦略を実装（メモリ制限付き）
  - [ ] バッチ書き込み最適化を実装
  - [ ] PII 検出・除去ロジックを実装
  - [ ] データ保持ポリシー（30日）を実装
  - [ ] サンプリング戦略を実装（常時20%、障害時100%）
- [ ] アラート閾値 (Yellow/Orange/Red) を `spec/testing/metrics.md` に倣って設定し、PagerDuty/Slack 通知連携を Rust サービスで実装。
  - [ ] アラート設定パーサー（YAML/TOML）を実装
  - [ ] 閾値評価エンジンを実装（3段階：Yellow/Orange/Red）
  - [ ] PagerDuty 通知クライアントを実装
  - [ ] Slack Webhook 通知を実装
  - [ ] アラート履歴管理を実装
  - [ ] SLO 逸脱時の `spec/deployment/rollback.md` 手順トリガーを実装
  - [ ] アラート通知のテストモードを実装

## 3. コントロールプレーン API 実装 🔴 **未着手（0%）**
> ⚠️ 現状: 実装なし
> 📋 依存: セクション2.1-2.2完了により実装可能
> 🎯 優先度: P1 - 次の主要タスク

### 3.1 API フレームワーク基盤
- [ ] `spec/api/control-plane.md` のエンドポイントを Rust (Axum or Actix) で実装し、mTLS + OAuth2/OIDC(5分トークン) + OpenTelemetry Traceparent を必須化。
  - [ ] Web フレームワーク選定と初期設定（Axum 推奨）
  - [ ] mTLS ミドルウェアを実装（ed25519/secp384r1 証明書）
  - [ ] OAuth2/OIDC トークン検証ミドルウェアを実装（5分 TTL）
  - [ ] OpenTelemetry Traceparent 抽出・伝播を実装
  - [ ] CORS 設定を実装
  - [ ] レート制限ミドルウェアを実装
  - [ ] リクエスト/レスポンスロギングを実装

### 3.2 デバイス管理 API
- [ ] `POST /devices` で入力バリデーション（ID パターン、X25519 キー長、SemVer など）を行い、監査ログへ不可変記録する。
  - [ ] エンドポイントハンドラを実装
  - [ ] 入力バリデーションロジックを実装（device_id: `/^[A-Z0-9-]{4,64}$/`, public_key: 32バイト）
  - [ ] リモートアテステーション検証を実装
  - [ ] デバイストークン生成を実装（opaque）
  - [ ] ペアリングコード生成を実装（10分 TTL）
  - [ ] 不可変監査ログ記録を実装
  - [ ] エラーハンドリング（重複登録、無効な証明書）を実装
- [ ] `POST /devices/{id}/pair` で CSR 処理とポリシーバンドル署名を行い、`spec/security/encryption.md` の鍵ローテーション通知を発火する。
  - [ ] エンドポイントハンドラを実装
  - [ ] CSR パース・検証を実装
  - [ ] HSM/KMS 経由の証明書発行を実装
  - [ ] ポリシーバンドル取得・署名を実装
  - [ ] セッションエンドポイント情報生成を実装
  - [ ] ステート遷移（pending → paired）を実装
  - [ ] 鍵ローテーション通知イベント発火を実装

### 3.3 セッション管理 API
- [ ] `POST /sessions` のストリーム割当で QoS Scheduler から割当結果を取得し、セッション TTL と FEC パラメータを含める。
  - [ ] エンドポイントハンドラを実装
  - [ ] マルチストリーム要求パースを実装
  - [ ] QoS Scheduler との RPC 通信を実装
  - [ ] ストリーム割当ロジックを実装
  - [ ] セッション ID（UUIDv7）生成を実装
  - [ ] 鍵マテリアル（HKDF）生成を実装
  - [ ] TTL 管理（expires_at）を実装
  - [ ] FEC パラメータ設定を実装

### 3.4 ポリシー管理 API
- [ ] `PUT /devices/{device_id}/policy` エンドポイントを実装。
  - [ ] エンドポイントハンドラを実装
  - [ ] ポリシー更新バリデーションを実装
  - [ ] RBAC/ABAC 権限チェックを実装
  - [ ] ポリシーバージョン管理を実装
  - [ ] 既存セッションへの通知を実装
  - [ ] 監査ログ記録を実装

### 3.5 監査 API
- [ ] 監査 API (`GET /audit/events`) で WORM ストレージに保存したイベントを 24h 遅延以内で配信し、Webhook 署名 (Ed25519) を実装する。
  - [ ] エンドポイントハンドラを実装
  - [ ] WORM ストレージからの読み取りを実装
  - [ ] ページネーション・フィルタリングを実装
  - [ ] ストリーミングレスポンス（Server-Sent Events）を実装
  - [ ] Ed25519 署名生成を実装
  - [ ] 署名検証ツールを提供
  - [ ] 24時間以内配信の SLA モニタリングを実装

### 3.6 エラーハンドリング
- [ ] エラーモデル (`ERR_VALIDATION` 等) を統一したレスポンスフォーマットで返すためのミドルウェアを作成する。
  - [ ] エラー型定義を実装（8種類：VALIDATION, AUTH, AUTHZ, NOT_FOUND, CONFLICT, STATE, INTERNAL, DEPENDENCY）
  - [ ] エラーレスポンスフォーマットを実装（JSON + trace_id）
  - [ ] エラーハンドリングミドルウェアを実装
  - [ ] 適切な HTTP ステータスコードマッピングを実装
  - [ ] エラーログ記録を実装
  - [ ] クライアント向けエラードキュメントを作成

## 4. エクスペリエンスレイヤ (UI) 🔴 **基本構造のみ（15%）**
> ⚠️ 現状: Vite + React プロジェクト初期化済み、画面実装なし
> 📂 実装場所: `ui/`
> 🎯 優先度: P2（API実装後に着手）

### 4.1 UI基盤構築
**完了済み:**
- [x] Vite + React + TypeScript プロジェクトセットアップ
- [x] Tailwind CSS 設定
- [x] 基本的な依存関係インストール（TanStack Query, Zustand, i18next）
- [x] ビルド設定完了

**未完了:**
- [ ] `spec/ui/overview.md` と `spec/ui/wireframes.md` をベースに、React + TypeScript (もしくは同等の C/C++ 非依存フレームワーク) で SPA を構築する。
  - [ ] ルーティング設定（React Router）
  - [ ] 状態管理設定（Zustand or Redux Toolkit）の実装
  - [ ] API クライアント設定（TanStack Query）の実装
  - [ ] 開発サーバー設定（HMR、プロキシ）

### 4.2 デザインシステム
- [ ] デザインシステムを `spec/ui/visual-design.md` のトークンで構築し、`tailwind.config.js` または design token ファイルを生成する。
  - [ ] デザイントークン定義（色、タイポグラフィ、スペーシング、シャドウ）
  - [ ] Tailwind CSS 設定をトークンから生成
  - [ ] 基本コンポーネントライブラリを実装（Button, Card, Input, Select, Modal）
  - [ ] コンポーネントストーリーブック（Storybook）を設定
  - [ ] テーマ切替機能を実装（ライト/ダーク）
  - [ ] 「はちみつ」世界観の表現（暖色パレット、柔らかなボーダー）

### 4.3 アクセシビリティ
- [ ] `spec/ui/accessibility.md` の WCAG 2.2 AA 要件を満たすため、全コンポーネントにキーボード操作・フォーカスリング・スクリーンリーダラベルを実装する。
  - [ ] ARIA ラベル・ロール・ステートを全インタラクティブ要素に追加
  - [ ] キーボードナビゲーションを実装（Tab, Enter, Escape, Arrow keys）
  - [ ] フォーカス管理を実装（FocusTrap, フォーカスリング）
  - [ ] コントラスト比検証（4.5:1 以上）
  - [ ] axe-core による自動アクセシビリティテストを統合
  - [ ] スクリーンリーダーテスト（NVDA/JAWS）を実施
  - [ ] 状況依存ヘルプを実装（コンテキストヘルプボタン）

### 4.4 アニメーション
- [ ] `spec/ui/animations.md` のタイミング/イージングを CSS/Framer Motion 等で再現し、`prefers-reduced-motion` 対応を組み込む。
  - [ ] アニメーション設定を定義（duration, easing）
  - [ ] トランジション実装（ページ遷移、モーダル、ドロワー）
  - [ ] マイクロインタラクション実装（ボタンホバー、ローディング）
  - [ ] `prefers-reduced-motion` 検出と代替挙動を実装
  - [ ] パフォーマンス最適化（will-change, transform, opacity のみ使用）

### 4.5 主要画面実装
- [ ] `spec/ui/wireframes.md` で定義された 5 画面 (近傍デバイス一覧 / ペアリング詳細 / ストリームダッシュボード / ポリシービルダー / メトリクスハブ) をレスポンシブで実装し、モバイル/タブレット/デスクトップ差分を確認する。
  - [ ] **近傍デバイス一覧画面**を実装
    - [ ] ビーコン検出結果のカード表示
    - [ ] フィルタリング・ソート機能
    - [ ] 再スキャン機能
    - [ ] デバイス詳細モーダル
  - [ ] **ペアリング詳細画面**を実装
    - [ ] 3ステップウィザード（検出 → 認証 → 完了）
    - [ ] OOB 認証フロー
    - [ ] 進捗インジケーター
    - [ ] エラーハンドリングと再試行
  - [ ] **ストリームダッシュボード画面**を実装
    - [ ] リアルタイムストリーム状態表示
    - [ ] QoS メトリクス可視化（レイテンシ、スループット）
    - [ ] ストリーム制御（開始/停止/再設定）
    - [ ] 診断ツール
  - [ ] **ポリシービルダー画面**を実装
    - [ ] ビジュアルポリシーエディタ
    - [ ] プロファイルテンプレート選択
    - [ ] バリデーションとプレビュー
    - [ ] エクスポート機能
  - [ ] **メトリクスハブ画面**を実装
    - [ ] リアルタイムメトリクスダッシュボード
    - [ ] 時系列グラフ（Chart.js or Recharts）
    - [ ] アラート履歴表示
    - [ ] レポートエクスポート
  - [ ] レスポンシブ対応を実装（<600px, 600-1024px, >1024px）

### 4.6 国際化
- [ ] UI テキストを i18n (en/ja/es/zh) で管理し、ICU MessageFormat + 30% 拡張ルールを適用。RTL 切替時のレイアウト確認を自動テストに含める。
  - [ ] i18n ライブラリ統合（react-i18next）
  - [ ] 翻訳ファイル構造を設計（JSON）
  - [ ] ICU MessageFormat サポートを実装
  - [ ] 4言語（en/ja/es/zh）の初期翻訳を作成
  - [ ] 言語切替 UI を実装
  - [ ] RTL レイアウトサポートを実装（CSS logical properties）
  - [ ] 日付・時刻フォーマット（ISO 8601）を実装
  - [ ] 数値・通貨フォーマットを実装
  - [ ] 30%テキスト拡張を考慮したレイアウトテストを実施

### 4.7 SDK & 統合
- [ ] Rust SDK を WASM 経由で UI から利用可能にする。
  - [ ] wasm-bindgen による Rust → JS バインディング
  - [ ] SDK API ラッパーを TypeScript で実装
  - [ ] SDK 初期化・認証フローを実装
  - [ ] エラーハンドリング統合
  - [ ] SDK 使用例ドキュメントを作成

## 5. セキュリティエンジニアリング
- [ ] OIDC/OAuth2 フロー（Authorization Code with PKCE、Client Credentials、Device Code）を Rust +外部 IdP で統合し、RBAC/ABAC ポリシー評価を JSON DSL で実装する。
- [ ] セッション管理 (UUIDv7、12h 有効、30 分滑走更新) を Session Orchestrator に組み込み、ログアウト時のトークン/鍵失効処理を追加。
- [ ] `docs/security/vulnerability.md` の STRIDE マトリクスに沿って、各脅威に対するテストケースと自動防御 (Rate Limit、Nonce ストア、MFA enforcement 等) を実装する。
- [ ] 監査ログを Immutable JSON Lines + WORM ストレージに記録し、KMS 操作ログとの突合を自動化する。
- [ ] サプライチェーン保護で Sigstore/cosign によるアーティファクト署名を CI に追加し、SBOM (CycloneDX) を生成する。

## 6. 観測性 & 可観測性統合
- [ ] OpenTelemetry Collector を DaemonSet (Rust Build) でデプロイし、OTLP/gRPC over TLS1.3 の経路を構築する。
- [ ] メトリクス/ログ/トレースを CockroachDB + TimescaleDB + Honeycomb (SaaS) に送信し、`docs/testing/metrics.md` の可視化要件 (Looker/PowerBI ダッシュボード) を整備する。
- [ ] KPI/SLO (接続成功率 99.5%、P95 ペアリング 6 秒など) をダッシュボード化し、逸脱時 5 分以内に通知するアラートルールを設定する。
- [ ] SLI 計測値を `docs/testing/metrics.md` のレビューサイクル (週次/月次/四半期) に沿って自動レポート化するジョブを実装。

## 7. テスト戦略実装
## 5. セキュリティエンジニアリング

### 5.1 認証・認可基盤
- [ ] OIDC/OAuth2 フロー（Authorization Code with PKCE、Client Credentials、Device Code）を Rust +外部 IdP で統合し、RBAC/ABAC ポリシー評価を JSON DSL で実装する。
  - [ ] Authorization Code with PKCE フローを実装
  - [ ] Client Credentials フローを実装
  - [ ] Device Code フローを実装
  - [ ] IdP 連携クライアントを実装（KeyCloak/Auth0/Azure AD）
  - [ ] RBAC ポリシーエンジンを実装
  - [ ] ABAC 属性評価エンジンを実装
  - [ ] ポリシー定義 DSL（JSON ベース）を設計・実装
  - [ ] ポリシーキャッシング機構を実装

### 5.2 セッション管理
- [ ] セッション管理 (UUIDv7、12h 有効、30 分滑走更新) を Session Orchestrator に組み込み、ログアウト時のトークン/鍵失効処理を追加。
  - [ ] セッショントークン生成・検証を実装
  - [ ] セッションストレージ（Redis 互換）を実装
  - [ ] 滑走ウィンドウ更新ロジックを実装
  - [ ] ログアウト時のクリーンアップを実装
  - [ ] トークン失効リストを実装
  - [ ] 並行セッション制限を実装

### 5.3 脅威モデリングと対策
- [ ] `spec/security/vulnerability.md` の STRIDE マトリクスに沿って、各脅威に対するテストケースと自動防御 (Rate Limit、Nonce ストア、MFA enforcement 等) を実装する。
  - [ ] STRIDE 脅威分析を各コンポーネントに実施
  - [ ] レートリミッタを実装（Token Bucket アルゴリズム）
  - [ ] Nonce ストア（重複リクエスト防止）を実装
  - [ ] MFA enforcement ロジックを実装
  - [ ] CSRF トークン生成・検証を実装
  - [ ] XSS 防止策（Content Security Policy）を実装
  - [ ] SQL インジェクション防止（パラメータ化クエリ）を確認
  - [ ] 各脅威に対するテストケースを作成

### 5.4 監査ログ
- [ ] 監査ログを Immutable JSON Lines + WORM ストレージに記録し、KMS 操作ログとの突合を自動化する。
  - [ ] 監査イベント型定義を実装
  - [ ] JSON Lines フォーマットで出力
  - [ ] WORM ストレージ連携を実装
  - [ ] タイムスタンプと署名を付与
  - [ ] KMS 操作ログとの相関分析を実装
  - [ ] 90日保持ポリシーを実装
  - [ ] 監査ログクエリ API を実装

### 5.5 サプライチェーンセキュリティ
- [ ] サプライチェーン保護で Sigstore/cosign によるアーティファクト署名を CI に追加し、SBOM (CycloneDX) を生成する。
  - [ ] cosign による Docker イメージ署名を実装
  - [ ] SBOM 生成を CI に統合
  - [ ] 依存関係スキャン（cargo-audit, npm audit）を実装
  - [ ] 署名検証を展開パイプラインに統合
  - [ ] ベンダーセキュリティアドバイザリ監視を実装
  - [ ] CVE データベースとの自動照合を実装

## 6. 観測性 & 可観測性統合

### 6.1 OpenTelemetry 基盤
- [ ] OpenTelemetry Collector を DaemonSet (Rust Build) でデプロイし、OTLP/gRPC over TLS1.3 の経路を構築する。
  - [ ] Collector 設定ファイル（YAML）を作成
  - [ ] Receiver（OTLP）設定を実装
  - [ ] Processor（フィルタリング、バッチ処理）設定を実装
  - [ ] Exporter（各種バックエンド）設定を実装
  - [ ] TLS 証明書管理を実装
  - [ ] Kubernetes DaemonSet マニフェストを作成
  - [ ] サンプリング戦略を設定

### 6.2 データストレージとクエリ
- [ ] メトリクス/ログ/トレースを CockroachDB + TimescaleDB + Honeycomb (SaaS) に送信し、`spec/testing/metrics.md` の可視化要件 (Looker/PowerBI ダッシュボード) を整備する。
  - [ ] CockroachDB スキーマ設計（メタデータ）
  - [ ] TimescaleDB スキーマ設計（時系列データ）
  - [ ] データ保持ポリシー（30日）を実装
  - [ ] クエリ最適化（インデックス、パーティショニング）
  - [ ] ダッシュボード設計（Looker/PowerBI/Grafana）
  - [ ] リアルタイムクエリ API を実装

### 6.3 SLI/SLO 管理
- [ ] KPI/SLO (接続成功率 99.5%、P95 ペアリング 6 秒など) をダッシュボード化し、逸脱時 5 分以内に通知するアラートルールを設定する。
  - [ ] SLI 計算ロジックを実装
  - [ ] SLO 定義ファイル（YAML）を作成
  - [ ] エラーバジェット計算を実装
  - [ ] バーンレート分析を実装
  - [ ] アラートルール設定（Prometheus/Alertmanager 形式）
  - [ ] 通知チャネル統合（PagerDuty, Slack, Email）

### 6.4 レポーティング
- [ ] SLI 計測値を `spec/testing/metrics.md` のレビューサイクル (週次/月次/四半期) に沿って自動レポート化するジョブを実装。
  - [ ] レポート生成エンジンを実装
  - [ ] 週次レポートテンプレートを作成
  - [ ] 月次レポートテンプレートを作成
  - [ ] 四半期レポートテンプレートを作成
  - [ ] 自動配信スケジューラを実装
  - [ ] PDF/HTML エクスポート機能を実装

## 7. テスト戦略実装

### 7.1 単体テスト
- [ ] `docs/testing/unit-tests.md` の優先順位に従い、暗号鍵管理/QoS スケジューラ/設定パーサ等で命令+分岐カバレッジ 90% 以上を確保する。
- [ ] プロパティベーステスト (proptest) で鍵交換や QoS ルーティングの不変条件を検証する。
- [ ] Clippy・cargo-audit・cargo-deny をユニットテスト前に実行する CI ステップを追加する。

### 7.2 統合テスト
- [ ] `docs/testing/integration-tests.md` の 5 シナリオ (Secure Pairing, Telemetry QoS, Command Fan-out, OTA Rollout, Incident Failover) を Rust ベース Orchestrator CLI + WASM シミュレータで自動化する。
- [ ] ステージング環境との差分チェック（IaC drift）をテスト前に実行し、整合性が取れない場合は IaC を再適用する手順をスクリプト化。
- [ ] テスト成果 (JUnit XML/JSON/HTML) をアーティファクト化し、失敗時に `docs/notes/decision-log.md` へエントリ草案を生成する。

### 7.3 E2E テスト
- [ ] `docs/testing/e2e-tests.md` の主要ジャーニー (デバイス導入/アラートレスポンス/OTA/テナント拡張) を Playwright + Rust バックエンドで自動化し、週次/リリース前に実行する。
- [ ] 1/4 スケールの E2E 環境を GitOps プロファイル `env=e2e` で構築し、Telemetry/Analytics も含めた完全複製を維持する。
- [ ] 失敗時の自動ロールバックとチケット起票を実装し、RCA を 72 時間以内に記録するフローを整備。

### 7.4 性能 & セキュリティテスト
- [ ] `docs/performance/benchmark.md` の 5 ワークロード (Baseline/Burst/Command Storm/Pairing Surge/OTA Wave) を Rust ロードジェネレータで再現し、P99 ≤ 120ms・スループット達成率 ≥95% を検証する。
- [ ] `docs/performance/scalability.md` のキャパシティ計画に沿って、自動スケールルール (HPA/KEDA) の検証とキャパシティ警告テストを実施する。
- [ ] `docs/security/vulnerability.md` のセキュリティテスト対応表 (SAST/DAST/フィッシング演習/Red Team/Supply Chain) を CI/CD と運用計画に組み込む。

## 8. DevOps, CI/CD, インフラ

### 8.1 CI パイプライン
- [ ] GitHub Actions / Azure DevOps で `spec/deployment/ci-cd.md` のステージ (Lint→Unit→Build→Integration→Security→Performance→Staging→E2E→Approval→Prod) を構築する。
  - [ ] Lint ステージを実装（rustfmt, clippy, eslint, prettier）
  - [ ] Unit テストステージを実装（並列実行、カバレッジ収集）
  - [ ] Build ステージを実装（マルチアーキテクチャビルド）
  - [ ] Integration テストステージを実装
  - [ ] Security スキャンステージを実装（SAST, SCA, container scanning）
  - [ ] Performance テストステージを実装
  - [ ] Staging デプロイステージを実装
  - [ ] E2E テストステージを実装
  - [ ] Manual Approval ゲートを実装
  - [ ] Production デプロイステージを実装
- [ ] 各ステージのゲート条件 (Unit カバレッジ 90%、Integration 成功率 98%、Security Critical CVE 0 など) を実装し、失敗時にパイプラインをブロックする。
  - [ ] カバレッジ閾値チェックを実装
  - [ ] テスト成功率チェックを実装
  - [ ] CVE 重大度チェックを実装
  - [ ] パフォーマンスリグレッションチェックを実装
  - [ ] ゲート失敗時の通知を実装
  - [ ] 手動オーバーライド機能を実装（緊急時用）
- [ ] SBOM 生成・cosign 署名・artifacts の保存 (12 ヶ月) を自動化する。
  - [ ] cargo-sbom による SBOM 生成を実装
  - [ ] CycloneDX フォーマット出力を実装
  - [ ] cosign による署名を実装
  - [ ] アーティファクトストレージ（S3/Azure Blob）連携を実装
  - [ ] 保持ポリシー（12ヶ月）を設定
  - [ ] 署名検証ツールを提供

### 8.2 IaC & インフラ
- [ ] Terraform/Bicep で `spec/deployment/infrastructure.md` の環境構成 (dev/stg/prd, 3 AZ, DR リージョン) をコード化し、GitOps (ArgoCD/Flux) と連携する。
  - [ ] Terraform モジュール構成を設計（network, compute, data, security）
  - [ ] dev 環境（1 AZ, 25% リソース）を実装
  - [ ] stg 環境（3 AZ, 50% リソース）を実装
  - [ ] prd 環境（3 AZ + DR リージョン）を実装
  - [ ] リモートステート管理を設定（S3/Azure Storage）
  - [ ] ステートロック機能を実装（DynamoDB/Cosmos DB）
  - [ ] GitOps ワークフロー（ArgoCD/Flux）を設定
  - [ ] IaC drift 検出を自動化
- [ ] VNet/VPC 3 層構造、Private Endpoint、mTLS Gateways を構築し、Zero Trust 原則を実装する。
  - [ ] Edge 層ネットワークを実装
  - [ ] Service 層ネットワークを実装
  - [ ] Data 層ネットワークを実装
  - [ ] NSG/Security Group ルールを設定
  - [ ] Private Endpoint を各マネージドサービスに設定
  - [ ] mTLS Gateway（Envoy/Istio）を展開
  - [ ] Zero Trust ポリシーを実装
  - [ ] ネットワークセグメンテーションテストを実施
- [ ] CockroachDB, TimescaleDB, Managed Kafka/Event Hubs, Vault, OpenTelemetry Collector, KMS などのマネージドサービスをプロビジョニングする。
  - [ ] CockroachDB クラスタを構築（3ノード以上）
  - [ ] TimescaleDB インスタンスを構築
  - [ ] Managed Kafka/Event Hubs を構築
  - [ ] Vault クラスタを構築（HA構成）
  - [ ] OpenTelemetry Collector を DaemonSet として展開
  - [ ] KMS（AWS KMS/Azure Key Vault）を設定
  - [ ] 各サービス間の接続とアクセス制御を設定
  - [ ] サービスヘルスチェックを実装
- [ ] バックアップ (RPO 15 分, RTO 30 分) と監査ログ保存 (WORM 7 年) の設定を行う。
  - [ ] 自動バックアップスケジュールを設定
  - [ ] バックアップ検証ジョブを実装
  - [ ] ポイントインタイムリカバリを設定
  - [ ] バックアップのクロスリージョン複製を実装
  - [ ] WORM ストレージ設定（Immutable storage）
  - [ ] 7年保持ポリシーを設定
  - [ ] リストアテストを自動化

### 8.3 ロールバック & DR
- [ ] `spec/deployment/rollback.md` の自動/手動手順を GitOps パイプラインに組み込み、トリガー条件 (SLO 違反, エラー率 2% etc) を監視する。
  - [ ] 自動ロールバック判定ロジックを実装
  - [ ] トリガー条件モニタリングを実装
  - [ ] Blue/Green デプロイ戦略を実装
  - [ ] Canary デプロイ戦略を実装
  - [ ] トラフィック切り戻し機能を実装
  - [ ] ロールバック履歴管理を実装
  - [ ] 手動ロールバック承認フローを実装
- [ ] データ整合性チェックリストを自動化し、スナップショット復旧・キュー排出・整合性検証を 15 分以内で完了できるジョブを準備する。
  - [ ] データ整合性チェックスクリプトを実装
  - [ ] スナップショット復旧手順を自動化
  - [ ] メッセージキュー排出ジョブを実装
  - [ ] 整合性検証（Checksum, Row count）を実装
  - [ ] 15分以内完了の性能テストを実施
  - [ ] 復旧演習を四半期ごとに実施
- [ ] Incident Commander ワークフロー (Slack incident channel, PagerDuty escalation) を Runbook として整備。
  - [ ] Incident Runbook テンプレートを作成
  - [ ] Slack インシデントチャネル自動作成を実装
  - [ ] PagerDuty エスカレーションポリシーを設定
  - [ ] インシデントロール（Commander, Scribe, Liaison）を定義
  - [ ] ポストモーテムテンプレートを作成
  - [ ] インシデント対応訓練を実施

## 9. パフォーマンス & スケーラビリティ運用

### 9.1 レイテンシバジェット管理
- [ ] `spec/performance/scalability.md` のレイテンシバジェット (Network 40% / Crypto 25% / Logic 20% / Storage 15%) をサービスメッシュやアプリ構成で enforce する。
  - [ ] レイテンシ計測ポイントを各層に配置
  - [ ] バジェット超過検出アラートを実装
  - [ ] サービスメッシュ（Istio/Linkerd）でタイムアウト設定
  - [ ] 各層のレイテンシ最適化を実施
  - [ ] レイテンシ分布レポートを自動生成

### 9.2 DR とキャパシティ計画
- [ ] マルチリージョン/マルチクラウド DR を構築し、リージョン喪失時 60% 容量を維持できるようにキャパシティ予約を確保する。
  - [ ] セカンダリリージョンをホットスタンバイで構築
  - [ ] クロスリージョンレプリケーションを設定
  - [ ] キャパシティ予約（Reserved Instances）を確保
  - [ ] フェイルオーバーテストを自動化
  - [ ] RTO/RPO 達成を検証
  - [ ] DR 演習を半期ごとに実施

### 9.3 異常検知
- [ ] Holt-Winters + ARIMA による異常検知を Rust サービスで実装し、Queue depth・CPU・TPS 指標に適用する。
  - [ ] 時系列予測モデルを実装
  - [ ] Holt-Winters アルゴリズムを実装
  - [ ] ARIMA モデルを実装
  - [ ] 異常スコア計算を実装
  - [ ] アラート生成ロジックを実装
  - [ ] モデル再学習スケジューラを実装

### 9.4 Graceful Degradation
- [ ] Graded degradation (OTA 停止, Bulk 遅延) をフラグ制御で実装し、月次カオス検証を自動で実施する。
  - [ ] 機能フラグ管理システムを実装
  - [ ] Degradation レベル定義（Normal/Reduced/Critical）
  - [ ] 各レベルの挙動を実装
  - [ ] カオスエンジニアリングシナリオを作成
  - [ ] 月次カオステストスケジュールを設定
  - [ ] 自動復旧メカニズムを実装

## 10. コンプライアンス & 監査準備

### 10.1 データ保護規制対応
- [ ] GDPR/CCPA 応答フロー (30 日以内削除) を Rust サービスで実装し、`spec/deployment/rollback.md` の監査と連携する。
  - [ ] データサブジェクトリクエスト（DSR）ハンドリングを実装
  - [ ] Right to Access（アクセス権）実装
  - [ ] Right to Deletion（削除権）実装
  - [ ] Right to Portability（移植権）実装
  - [ ] 30日以内応答のワークフローを実装
  - [ ] 同意管理システムを実装
  - [ ] データ保護影響評価（DPIA）テンプレートを作成

### 10.2 セキュリティ認証対応
- [ ] SOC2/ISO27001 対応として、監査証跡・アクセスレビュー・鍵操作ログを自動集約し、年次監査パッケージを生成する。
  - [ ] SOC2 Type II 管理項目マッピングを作成
  - [ ] ISO27001 管理策実装状況を文書化
  - [ ] 監査証跡自動収集システムを実装
  - [ ] アクセスレビューワークフロー（四半期ごと）を実装
  - [ ] 鍵操作ログ集約を実装
  - [ ] 年次監査パッケージ生成を自動化
  - [ ] 内部監査スケジュールを設定

### 10.3 暗号輸出規制
- [ ] 暗号輸出規制管理のため、地域別鍵長・許容アルゴリズムをメタデータ化し、ポリシーエンジンで enforcement する。
  - [ ] 地域別暗号規制メタデータを作成
  - [ ] 許容アルゴリズムマッピングを実装
  - [ ] 自動コンプライアンスチェックを実装
  - [ ] 規制変更追跡システムを実装
  - [ ] 輸出管理文書を自動生成

### 10.4 サプライチェーン監査
- [ ] サプライチェーン監査 (SBOM, 署名検証, CVE モニタリング) を四半期ごとに自動レポート化する。
  - [ ] 依存関係ツリー分析を実装
  - [ ] ライセンスコンプライアンスチェックを実装
  - [ ] CVE 脆弱性スキャンを実装
  - [ ] ベンダーリスク評価を実装
  - [ ] 四半期レポート自動生成を実装
  - [ ] サプライチェーン攻撃検出を実装

## 11. ドキュメント & ナレッジ共有

### 11.1 技術ドキュメント
- [ ] `spec/templates/module-template.md` / `test-template.md` / `ui-template.md` に準拠した実装仕様・テスト計画・UI 仕様をリポジトリ内 `docs/impl/` に作成し、DoD を満たすレビューを実施する。
  - [ ] 各モジュールの実装仕様書を作成
  - [ ] テスト計画書を作成
  - [ ] UI 仕様書を作成
  - [ ] API リファレンスを生成（rustdoc, TypeDoc）
  - [ ] アーキテクチャ図を最新化
  - [ ] DoD チェックリストで全ドキュメントをレビュー

### 11.2 トレーサビリティ管理
- [ ] `spec/requirements.md` のトレーサビリティ方針に従い、要件 ↔ 実装 ↔ テスト のリンクをチケットシステムで維持する。
  - [ ] トレーサビリティマトリクスを作成
  - [ ] 要件 ID と実装 PR を紐付け
  - [ ] 実装とテストケースを紐付け
  - [ ] カバレッジギャップを可視化
  - [ ] 変更影響分析を実装
  - [ ] 双方向トレース検証を自動化

### 11.3 運用ドキュメント
- [ ] On-call Runbook、セキュリティ Runbook、性能ベンチレポート、UX リサーチ結果を Confluence or GitHub Wiki に集約し、アクセス権を RBAC で制御する。
  - [ ] On-call Runbook を作成（トラブルシューティング手順）
  - [ ] セキュリティインシデント Runbook を作成
  - [ ] 性能ベンチマークレポートテンプレートを作成
  - [ ] UX リサーチ結果テンプレートを作成
  - [ ] ドキュメントアクセス制御を設定
  - [ ] ドキュメント更新ワークフローを確立

### 11.4 KPI/OKR 管理
- [ ] `spec/README.md` の KPI/OKR を実装ロードマップと OKR ツールに同期し、達成状況を月次で更新する。
  - [ ] KPI ダッシュボードを作成
  - [ ] OKR ツール（Jira Align/Asana）と連携
  - [ ] 月次進捗レビュー会議を設定
  - [ ] 目標達成率の可視化を実装
  - [ ] 改善アクションの追跡を実装

## 12. ローンチ前後の運用

### 12.1 段階的ロールアウト
- [ ] カナリアリリース → 25% → 100% の 3 段階展開を計画し、各段階で KPI (接続成功率, Latency P95, エラー率) を監視する。
  - [ ] カナリアデプロイ設定（1-5%トラフィック）を実装
  - [ ] 25%ロールアウト設定を実装
  - [ ] 100%ロールアウト設定を実装
  - [ ] 各段階の成功基準を定義
  - [ ] 自動進行/停止判定ロジックを実装
  - [ ] トラフィック制御（Feature Flag/ルーティング）を実装

### 12.2 ポストデプロイ検証
- [ ] Post-Deploy 30 分のヘルスチェックを自動化し、逸脱時に `spec/deployment/rollback.md` の自動ロールバックをトリガーする。
  - [ ] ヘルスチェックエンドポイントを実装
  - [ ] スモークテストスイートを実装
  - [ ] 30分監視ウィンドウを設定
  - [ ] 異常検知閾値を設定
  - [ ] 自動ロールバックトリガーを実装
  - [ ] 通知とエスカレーションを実装

### 12.3 ユーザー教育
- [ ] ユーザー教育資料 (クイックスタート、API ガイド、UX ツアー) を作成し、UI 内の状況依存ヘルプとリンクさせる。
  - [ ] クイックスタートガイドを作成（3ステップ以内）
  - [ ] API ガイドを作成（全エンドポイント）
  - [ ] UX インタラクティブツアーを実装
  - [ ] ビデオチュートリアルを作成
  - [ ] FAQ を作成
  - [ ] 状況依存ヘルプシステムを実装
  - [ ] 多言語版を作成（en/ja/es/zh）

### 12.4 継続的改善
- [ ] 四半期ごとのレトロスペクティブで KPI/SLO/テスト/セキュリティイベントをレビューし、改善タスクを `spec/roadmap.md` の P4 継続改善フェーズへ反映する。
  - [ ] レトロスペクティブミーティングをスケジュール
  - [ ] レビューテンプレートを作成（What went well/What needs improvement）
  - [ ] アクションアイテム追跡システムを実装
  - [ ] 改善提案の優先順位付けを実施
  - [ ] ロードマップへのフィードバックループを確立
  - [ ] 四半期成果レポートを作成

---

## 完了基準（Definition of Done）

各セクションの完了時には以下を確認:
- [ ] すべてのサブタスクが完了している
- [ ] 関連する仕様書の DoD を満たしている
- [ ] コードレビューが完了している
- [ ] テストカバレッジ目標を達成している
- [ ] ドキュメントが更新されている
- [ ] CI/CD パイプラインがグリーンである
- [ ] セキュリティスキャンでクリティカル問題ゼロ
- [ ] パフォーマンステストが目標を達成している
- [ ] 監査ログが正しく記録されている
- [ ] `spec/notes/decision-log.md` に変更が記録されている
