# docs/templates/module-template.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> HoneyLink™ モジュール仕様書のテンプレートです。機能定義、契約、品質基準を統一的に記述します。

---

## 1. モジュール概要
- **モジュール名:** <!-- 例: Telemetry Normalizer -->
- **担当チーム:** <!-- Team name -->
- **概要:** <!-- モジュールの目的と価値提案 -->
- **ステータス:** 提案中 / 実装中 / 本番 / 廃止予定

## 2. 責務と境界
- **主な責務:**
  - <!-- 箇条書き -->
- **非責務:**
  - <!-- 箇条書き -->
- **関連ドメイン:** [docs/architecture/overview.md](../architecture/overview.md)

## 3. インターフェース
### 入力
| 名称 | プロトコル/フォーマット | 検証ルール | ソース |
|------|-------------------------|------------|--------|
| | | | |

### 出力
| 名称 | プロトコル/フォーマット | SLA | 宛先 |
|------|-------------------------|-----|------|
| | | | |

- 詳細仕様は [docs/architecture/interfaces.md](../architecture/interfaces.md) を参照。

## 4. データモデル
- **主要エンティティ:** <!-- 概要、スキーマ参照 -->
- **永続化:** <!-- 使用するデータストア、保持期間 -->
- **暗号/秘匿:** [docs/security/encryption.md](../security/encryption.md) に準拠。

## 5. 依存関係
| 種別 | コンポーネント | インターフェース | SLA/契約 | 備考 |
|------|----------------|-------------------|----------|------|
| 上位 | | | | |
| 下位 | | | | |

- 依存ルールは [docs/architecture/dependencies.md](../architecture/dependencies.md) に従う。

## 6. 性能・スケーラビリティ
- **SLO/SLI:** <!-- レイテンシ, スループット, 可用性 -->
- **キャパシティ計画:** [docs/performance/scalability.md](../performance/scalability.md)
- **性能テスト:** [docs/performance/benchmark.md](../performance/benchmark.md)

## 7. セキュリティ & プライバシー
- **認証/認可:** [docs/security/auth.md](../security/auth.md)
- **脅威モデル対策:** [docs/security/vulnerability.md](../security/vulnerability.md)
- **データ分類:** <!-- PII/Confidential/Public など -->

## 8. 観測性
- **メトリクス:** <!-- 収集する指標と理由 -->
- **ログ:** <!-- フォーマット、保持期間 -->
- **トレース:** <-- サンプリング戦略 -->
- 参照: [docs/testing/metrics.md](../testing/metrics.md)

## 9. テスト戦略
| レイヤー | 方針 | 成功基準 | 参照 |
|----------|------|----------|------|
| Unit | | | [docs/testing/unit-tests.md](../testing/unit-tests.md) |
| Integration | | | [docs/testing/integration-tests.md](../testing/integration-tests.md) |
| E2E | | | [docs/testing/e2e-tests.md](../testing/e2e-tests.md) |
| セキュリティ | | | [docs/security/vulnerability.md](../security/vulnerability.md) |

## 10. デプロイ & 運用
- **デプロイ方法:** [docs/deployment/ci-cd.md](../deployment/ci-cd.md)
- **インフラ要件:** [docs/deployment/infrastructure.md](../deployment/infrastructure.md)
- **ロールバック:** [docs/deployment/rollback.md](../deployment/rollback.md)

## 11. リスク & 技術的負債
- **既知のリスク:** <!-- 箇条書き -->
- **軽減策:** <!-- 箇条書き -->
- **テクデット:** <!-- 優先度と解消計画 -->

## 12. 受け入れ基準 (DoD)
- 責務と境界が定義されている。
- インターフェースとデータモデルが明記されている。
- 性能・セキュリティ・テスト・デプロイ基準がリンクされている。
- C/C++ 依存を排除する方針が反映されている。
- リスクと改善計画が整理されている。