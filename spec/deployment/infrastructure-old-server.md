# docs/deployment/infrastructure.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> HoneyLink™ の本番・ステージング環境のインフラ設計方針を定義します。クラウドリソース、ネットワーク、セキュリティ制御を整理し、実装コードや C/C++ 依存は含めません。

## 目次
- [環境構成](#環境構成)
- [ネットワークトポロジ](#ネットワークトポロジ)
- [コンピュートレイヤー](#コンピュートレイヤー)
- [データおよびストレージ](#データおよびストレージ)
- [秘密情報と鍵管理](#秘密情報と鍵管理)
- [監視・ロギング・トレーシング](#監視ロギングトレーシング)
- [コンプライアンスとガバナンス](#コンプライアンスとガバナンス)
- [IaC とリリース管理](#iac-とリリース管理)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## 環境構成
| 環境 | 用途 | スケール | 備考 |
|------|------|----------|------|
| 開発 (dev) | 個別検証 | 1 AZ, 25% リソース | Feature Branch 検証 |
| ステージング (stg) | 統合/E2E | 3 AZ, 50% リソース | 本番同等 | 
| 本番 (prd) | 顧客向け | 3 AZ + DR リージョン | 24/7 SRE 体制 |

- すべて Infrastructure as Code (Terraform or Bicep)。C/C++ 製 CLI/ツールは禁止。
- リージョン分散: Primary (例: East US), Secondary (例: West Europe) をホットスタンバイ。

## ネットワークトポロジ
- VNet/VPC を 3 層 (Edge, Service, Data)。サブネット間は NSG/セキュリティグループで制御。
- Southbound 通信: デバイス ↔ Edge Gateway (QUIC + TLS1.3)。Northbound: Gateway ↔ Control Plane (mTLS)。
- Zero Trust: IP allow list ではなくデバイス証明書 + ポリシー。
- プライベートエンドポイント経由でデータストアへアクセス。公開エンドポイントは API Gateway のみ。

## コンピュートレイヤー
- Edge Gateway: コンテナプラットフォーム (AKS/EKS/GKE)。Rust/WASM ランタイム、C/C++ バイナリは禁止。
- Control Plane Microservices: サーバレス Container Apps or Kubernetes。HPA/KEDA でオートスケール。
- バッチ/分析: マネージドサーバレス (Azure Functions, AWS Lambda) で夜間ジョブを実行。
- Observability エージェントは Rust/WASM ベースで提供。

## データおよびストレージ
- 時系列データ: マネージドカラムナー DB (InfluxDB Cloud, Azure Data Explorer)。保持期間: 30 日。
- メタデータ・設定: マネージド RDB (PostgreSQL マネージド)。
- キュー/ストリーム: マネージド Kafka 互換サービス or Azure Event Hubs。
- バックアップ: スナップショット + オブジェクトストレージ (冗長化: GRS/LRS)。RPO 15 分、RTO 30 分。

## 秘密情報と鍵管理
- KMS (Azure Key Vault / AWS KMS) で鍵・証明書を集中管理。
- mTLS 証明書は自動ローテ (90 日)。
- アプリケーションシークレットは HashiCorp Vault 互換 API で取得。
- C/C++ ベースの暗号ライブラリは使用せず、Rust ネイティブ or マネージドサービスを利用。

## 監視・ロギング・トレーシング
- OpenTelemetry Collector (Rust ビルド) を DaemonSet としてデプロイ。
- メトリクス → マネージドモニタリング (Datadog/NewRelic/Azure Monitor)。
- ログ → クラウドネイティブログサービス (Log Analytics/CloudWatch Logs)。
- トレース → Honeycomb or Jaeger SaaS。
- KPI/SLO は [docs/testing/metrics.md](../testing/metrics.md) と同期。SLO 違反時は PagerDuty 通知。

## コンプライアンスとガバナンス
- 標準: ISO27001, SOC2, GDPR, HIPAA (必要に応じて)。
- データレジデンシ: EU テナントは EU 内リージョンへ固定。
- 監査ログは WORM ストレージに 7 年保管。アクセスは RBAC + ABAC。
- サプライチェーン監査は [docs/security/vulnerability.md](../security/vulnerability.md) に従って四半期ごとに実施。

## IaC とリリース管理
- IaC リポジトリはアプリとは別。Pull Request → Plan → Apply の 3 ステップ。
- GitOps (ArgoCD/Flux) を使用し、本番環境への手動操作は禁止。
- コンテナイメージは Rust ベースでビルド。C/C++ ビルドチェーンは排除。
- インフラ変更の影響評価は [docs/notes/decision-log.md](../notes/decision-log.md) に記録。

## 受け入れ基準 (DoD)
- 環境構成・ネットワーク・コンピュート・ストレージが網羅的に記述されている。
- 秘密情報/鍵管理と観測性のアプローチが定義されている。
- コンプライアンスとガバナンス要件が整理されている。
- C/C++ 依存排除と IaC 方針が明文化されている。
- 他ドキュメントへのリンクが整合している。