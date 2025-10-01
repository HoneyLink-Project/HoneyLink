# docs/architecture/tech-stack.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> 技術選定は純仕様であり、いかなる実装コードや C/C++ 依存コンポーネントを含みません。比較は抽象能力と提供形態に基づきます。

## 目次
- [選定方針](#選定方針)
- [技術候補比較表](#技術候補比較表)
- [採用技術と理由](#採用技術と理由)
- [リスクと緩和策](#リスクと緩和策)
- [撤退戦略](#撤退戦略)
- [運用・セキュリティ評価](#運用セキュリティ評価)
- [性能評価](#性能評価)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## 選定方針
1. **C/C++非依存:** 言語ランタイムはガーベジコレクションまたはメモリ安全機構を持ち、ネイティブ連携が必要な場合はサンドボックス化。
2. **プラガブル:** 代替実装容易化のため、ポート/アダプタ抽象を定義。
3. **標準準拠:** 暗号・認証は国際標準に準拠し、第三者監査可能。
4. **運用容易性:** 観測性スタックと統合しやすいメトリクス/トレース機構。
5. **長期サポート:** 5年以上のコミュニティ/企業サポートが見込める。

## 技術候補比較表
| レイヤ | 候補 | 長所 | 短所 | C/C++依存回避策 | 判定 |
|--------|------|------|------|----------------|------|
| 言語ランタイム | Rust (純粋)、Kotlin/JVM、Go | メモリ安全、非同期性能 | 学習コスト | 標準ライブラリのみ、FFI禁止 | **採用: Rust** |
| 暗号ライブラリ | RustCrypto Suite、WebCryptoサービス | 安全監査済、最新アルゴリズム | RSA互換が限定 | SaaS併用でC依存排除 | **採用: RustCrypto + SaaS fallback** |
| メッセージング | NATS JetStream、Kafka (マネージド) | スケール容易 | Kafka OSSはJava依存 | マネージドKafka使用、内部は抽象 | 条件付き採用 |
| 設定管理 | HashiCorp Consul (API利用)、自前YAML | API柔軟 | OSSバイナリはGo製 | サービス版かクラウドマネージド利用 | **採用: マネージドConsul API** |
| 観測性 | OpenTelemetry (OTLP)、Prometheus SaaS | 標準化 | 自前構築でC++ exporterあり | SaaSまたはRust Exporter利用 | **採用: OTLP + SaaS** |
| データストア | CockroachDB (SQL)、ScyllaDB (NoSQL) | 分散耐障害、低遅延 | 運用複雑 | フルマネージド版、Rustクライアント使用 | **採用: CockroachDB (Managed)** |

## 採用技術と理由
- **言語ランタイム:** Rust (標準ツールチェーンのみ) を想定。理由: メモリ安全、非同期性能、C/C++依存が最小。
- **暗号:** RustCrypto + 外部KMSサービス。X25519/ChaCha20-Poly1305 を純粋実装で提供し、ハードウェアアクセラレーションはクラウドKMS経由で実現。
- **FEC:** Rust実装の RaptorQ / Reed-Solomon を仕様基準として採用。ライブラリは純粋実装に限定。
- **可観測性:** OpenTelemetry OTLP 出力を標準化し、ダッシュボードは SaaS (例: Grafana Cloud) を利用。
- **データストア:** 手続き管理が必要な設定データは CockroachDB、時系列メトリクスはマネージドTimescaleDB。

## リスクと緩和策
| リスク | 内容 | 緩和策 |
|--------|------|--------|
| Rustエコシステムの成熟度 | 一部プロトコルライブラリが未成熟 | コア部を内製仕様化、相互運用テスト ([docs/testing/integration-tests.md](../testing/integration-tests.md)) |
| マネージドサービス依存 | 地域規制による利用不可 | 代替として OSS + Rust製エージェントを準備、運用SOPで切替手順定義 |
| 高性能要件 | Rust実装でも最適化が必要 | 仕様レベルでプロファイリング計画を[docs/performance/benchmark.md](../performance/benchmark.md)に記載 |

## 撤退戦略
- 技術が KPI を満たさない場合、代替スタックを以下の手順で検討：
  1. 失敗原因を[docs/notes/decision-log.md](../notes/decision-log.md)へ記録。
  2. 候補技術 (例: Erlang/Elixir, Java) の仕様評価を追加し、C/C++依存が発生しないか確認。
  3. 互換レイヤ (ポート/アダプタ) をアップデートし、インターフェース後方互換を維持。

## 運用・セキュリティ評価
- **キー管理:** 暗号鍵はクラウドKMS (例: Hashicorp Vault SaaS) で保管、ローテーションポリシーは[docs/security/encryption.md](../security/encryption.md)参照。
- **監査:** SaaS活用時でも監査ログが取得可能か確認し、[docs/security/vulnerability.md](../security/vulnerability.md)に連携。
- **運用負荷:** マネージド採用により C/C++ ビルド環境を不要化。

## 性能評価
- Rust + async ランタイム (Tokio 相当) で遅延目標を満たせる見込み。詳細なスループットモデルは[docs/performance/scalability.md](../performance/scalability.md)参照。
- FEC 実装の計算負荷は[docs/performance/benchmark.md](../performance/benchmark.md)でベンチマーク。

## 受け入れ基準 (DoD)
- 各レイヤで少なくとも 2 案を比較し、採用理由と撤退戦略を明示している。
- C/C++依存回避策が全候補で記述されている。
- セキュリティ・運用・性能文書と双方向参照が存在する。
- リスク緩和がロードマップ (マイルストーン) と整合している。
- 仕様更新時のチェックリストが[docs/templates/module-template.md](../templates/module-template.md)に反映可能である。
