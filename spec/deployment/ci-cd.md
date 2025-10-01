# docs/deployment/ci-cd.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> HoneyLink™ の継続的インテグレーション／デリバリー (CI/CD) パイプライン方針を定義します。ステージ、ゲート、責務を整理し、実装コードや C/C++ 依存は含めません。

## 目次
- [パイプライン全体像](#パイプライン全体像)
- [CI ステージ](#ci-ステージ)
- [CD ステージ](#cd-ステージ)
- [品質ゲートと判定条件](#品質ゲートと判定条件)
- [ブランチ戦略とリリースチャネル](#ブランチ戦略とリリースチャネル)
- [シークレットと署名管理](#シークレットと署名管理)
- [可観測性とフィードバック](#可観測性とフィードバック)
- [エスカレーションとロールバック連携](#エスカレーションとロールバック連携)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## パイプライン全体像
```
PR → Lint/Static → Unit → Build → Integration → Security → Performance Gate → Staging Deploy → E2E → Manual Approval → Prod Deploy → Post-Deploy Checks
```
- パイプラインは GitHub Actions / Azure DevOps などのマネージド CI/CD を利用。
- すべてのランナーはコンテナ化された Rust ベース環境で動作。C/C++ ビルドチェーンは利用禁止。

## CI ステージ
1. **Lint & Static**: Rust fmt/clippy, 文書リンク検査。
2. **Unit Tests**: [docs/testing/unit-tests.md](../testing/unit-tests.md) に準拠。
3. **Build & Package**: Rust ビルド、WASM パッケージ化、SBOM 生成。
4. **Integration Tests**: [docs/testing/integration-tests.md](../testing/integration-tests.md) 参照。
5. **Security Scan**: SAST, SCA, コンテナイメージ署名検証。C/C++ 製スキャナは使用しない。

## CD ステージ
- **Staging Deploy**: IaC Plan → Apply (ステージング)。
- **E2E Tests**: [docs/testing/e2e-tests.md](../testing/e2e-tests.md) を完了するまで本番ゲートは閉鎖。
- **Manual Approval**: プロダクトオーナー + SRE。審査ログは監査用に保存。
- **Prod Deploy**: カナリア → 25% → 100% の 3 段階ローリング。自動検証完了で次段階へ。
- **Post-Deploy**: 30 分監視ウィンドウで KPI チェック。

## 品質ゲートと判定条件
| ステージ | 判定条件 | 失敗時処理 |
|----------|----------|------------|
| Lint/Static | 100% 合格 | PR ブロック |
| Unit | カバレッジ 90% 以上 | フィードバック + 再実行 |
| Integration | 成功率 98% 以上 | 自動ロールバック (ステージング) |
| Security | 新規 Critical CVE 0 | 緊急レビュー |
| Performance | P99 ≤ 120ms | ロード改善タスク生成 |
| E2E | ジャーニー完了率 ≥95% | リリース停止 |

- 判定は [docs/testing/metrics.md](../testing/metrics.md) の閾値を準拠。

## ブランチ戦略とリリースチャネル
- `main`: 常にデプロイ可能。保護ブランチ。
- `release/x.y`: 定期リリース。タグ付け後に保守フェーズへ移行。
- `feature/*`: 開発者ブランチ。PR で `main` へ統合。
- Hotfix: `hotfix/*` → 速やかに本番へリリース、後続で main/release に統合。

## シークレットと署名管理
- シークレットはクラウドマネージドシークレットストアから OIDC 連携でフェッチ。
- アーティファクト署名: Sigstore/cosign。署名鍵は KMS で保管。
- SBOM は CycloneDX 形式。発行後 1 年保管。

## 可観測性とフィードバック
- パイプラインメトリクス: 所要時間, 失敗率, 待ち時間。ダッシュボードは Looker。
- ログはクラウドログサービスへ送信し、12 ヶ月保管。
- 失敗解析は自動でチケット作成 (Jira) + Slack 通知。

## エスカレーションとロールバック連携
- 重大失敗時は自動で [docs/deployment/rollback.md](rollback.md) の手順を呼び出し。
- エスカレーション層: オンコール SRE → プラットフォーム責任者 → エグゼクティブ。
- リリース中断後は RCA を 72 時間以内に作成し、[docs/notes/decision-log.md](../notes/decision-log.md) へ登録。

## 受け入れ基準 (DoD)
- CI/CD のステージ・ゲート・責務が明確に記述されている。
- ブランチ戦略、シークレット管理、フィードバックサイクルが定義されている。
- C/C++ 依存排除が明示されている。
- ロールバック文書を含む関連ドキュメントにリンクしている。
- 判定条件がテストメトリクスと整合している。