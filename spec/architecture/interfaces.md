# docs/architecture/interfaces.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> ここでは HoneyLink™ の内部・外部インタフェースを言語非依存で定義します。APIは抽象的なパラメータと構造体で記述し、実装コードや C/C++ 依存仕様は含みません。

## 目次
- [I/F設計方針](#if設計方針)
- [外部API](#外部api)
- [内部モジュールI/F](#内部モジュールif)
- [データモデル (抽象スキーマ)](#データモデル-抽象スキーマ)
- [エラー処理と再送戦略](#エラー処理と再送戦略)
- [互換性とバージョニング](#互換性とバージョニング)
- [セキュリティと認可](#セキュリティと認可)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## I/F設計方針
1. **言語・プロトコル非依存:** JSON/YAML/CBOR等で表現可能な抽象モデルを提示。
2. **後方互換性優先:** versionフィールドを必須化し、deprecatedフラグで段階的廃止。
3. **セキュアバイデザイン:** すべての外部呼び出しは相互TLSまたはトークンにより保護。
4. **C/C++禁止:** ネイティブ拡張前提のAPI設計は不可。必要な場合はL7アダプタで解決。

## 外部API
| API名 | パス/チャネル (抽象) | メソッド | 認証 | 説明 | 応答 SLA |
|-------|----------------------|----------|------|------|-----------|
| Device Discovery Service | `/v1/discovery/devices` | QUERY | OAuth2 Client Credentials | Beacon結果取得、フィルタリング | 200ms以内 |
| Pairing Initiation | `/v1/pairing/sessions` | POST | OIDC Token + Proof of Possession | ペアリング要求作成 | 500ms以内 |
| Profile Catalog | `/v1/profiles` | GET | OAuth2 | 利用可能なプロファイル一覧 | 150ms以内 |
| Telemetry Ingest | `otlp://telemetry.honeylink/` | STREAM | mTLS | メトリクス/ログ送信 | 99.9%連続稼働 |
| Policy Management | `/v1/policies/{id}` | PUT | OAuth2 + RBAC | ポリシー定義更新 | 400ms以内 |
| Audit Export | `/v1/audit/streams` | STREAM | Signed Event Token | 不変ログ購読 | 24h遅延以内 |

## 内部モジュールI/F
| Producer | Consumer | チャネル | メッセージ構造 | 備考 |
|----------|----------|-----------|----------------|------|
| Session Orchestrator | Crypto & Trust | Async Request Queue | `HandshakeRequest` | 冪等 ID 必須 |
| Policy Engine | QoS Scheduler | Event Bus | `QoSPolicyUpdate` | バージョン付与 |
| Transport Abstraction | Telemetry & Insights | Metrics Stream | `StreamMetric` | 1秒粒度 |
| Physical Adapter | Transport Abstraction | Callback API | `LinkStateChange` | サンドボックス内通信 |

## データモデル (抽象スキーマ)
```
HandshakeRequest (object)
  - session_id: UUIDv7
  - client_public_key: Binary(32)
  - capabilities: List<CapabilityCode>
  - auth_context: map<string, string>

QoSPolicyUpdate (object)
  - policy_version: String
  - stream_id: UInt8
  - latency_budget_ms: UInt16
  - bandwidth_floor_mbps: Decimal
  - fec_mode: Enum { NONE, LIGHT, HEAVY }
  - expiration_ts: Timestamp

StreamMetric (object)
  - metric_version: String
  - stream_id: UInt8
  - latency_ms_p95: Float
  - jitter_ms_stddev: Float
  - loss_ratio: Float
  - battery_mw: Float
```

## エラー処理と再送戦略
- **共通エラーコード:** `INVALID_REQUEST`, `UNAUTHORIZED`, `CONFLICT`, `RETRYABLE`, `RATE_LIMITED`。
- **再送:** RETRYABLE 発生時は指数バックオフ (初期 200ms、最大 2s) とし、idempotency-key が必須。
- **冪等性:** 各 POST/PUT は `Idempotency-Key` ヘッダを要求。ハッシュは FNV-1a など C 依存のないハッシュ関数を採用。
- **監査:** 重大エラーは[docs/security/vulnerability.md](../security/vulnerability.md)で定義するインシデントフローへ転送。

## 互換性とバージョニング
- API バージョンは `v{major}` 形式。Major変更時は `/v2/` 等でパスを分岐。
- フィールド追加は後方互換とする。削除は `deprecated_after` メタデータを付与し、12 ヶ月の猶予。
- プロファイルテンプレの互換性マトリクスは[docs/templates/module-template.md](../templates/module-template.md)の指針に従う。

## セキュリティと認可
- 認証方式の詳細は[docs/security/auth.md](../security/auth.md)。
- 暗号パラメータ・鍵ローテーションは[docs/security/encryption.md](../security/encryption.md)。
- すべての I/F 呼び出しは RBAC/ABAC の両面で評価。スコープ/属性が不足する場合は `403` とする。

## 受け入れ基準 (DoD)
- 外部・内部 I/F の双方についてパラメータ、エラー、SLA を定義した。
- すべてのモデルが言語非依存で記述され、C/C++ 専用の概念が含まれていない。
- バージョニングと後方互換方針がロードマップ・テスト方針と整合。
- セキュリティ要件が関連文書とリンクされている。
- 冪等性・再送戦略が[docs/architecture/dataflow.md](./dataflow.md)と一致している。
