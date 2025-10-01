# Module Specification: Policy & Profile Engine

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Policy & Profile Engine モジュールの実装仕様書。QoSポリシー管理とプロファイルテンプレートの CRUD を統括します。

**トレーサビリティ ID**: `MOD-002-POLICY-ENGINE`

---

## 1. モジュール概要

- **モジュール名:** Policy & Profile Engine
- **担当チーム:** Protocol WG (ENG-PROTO-01, ENG-PROTO-03)
- **概要:** QoSポリシーとプロファイルテンプレートの定義、検証、配信、バージョン管理を担当
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `crates/policy-engine/`

### 価値提案
- ユースケース別（IoT/AR/VR/8K/ゲーミング）のプリセットプロファイル提供
- SemVerによるポリシーバージョン管理と後方互換性保証
- Ed25519署名によるプロファイル改ざん防止
- リアルタイムポリシー更新とフォールバック機構

---

## 2. 責務と境界

### 主な責務
- **プロファイル管理**: CRUD操作、テンプレート検証、署名・検証
- **ポリシー配信**: QoS Scheduler へのイベント配信とバージョン管理
- **バージョン管理**: SemVer準拠、deprecated_after メタデータ処理
- **プリセット管理**: IoT/AR/VR/8K/ゲーミング向けデフォルトプロファイル
- **バリデーション**: JSON Schema/YAML による入力検証
- **フォールバック**: ポリシー適用失敗時の旧設定復元

### 非責務
- **QoS実行**: QoS Scheduler に委譲
- **ネットワーク制御**: Transport Abstraction に委譲
- **UI表示**: Experience Layer に委譲
- **鍵管理**: Crypto & Trust Anchor に委譲

### 関連ドキュメント
- [spec/architecture/overview.md](../architecture/overview.md)
- [spec/requirements.md](../requirements.md) - FR-04 (QoS調整), FR-06 (プロファイルテンプレ共有)

---

## 3. インターフェース

### 3.1 入力

| 名称 | プロトコル/フォーマット | 検証ルール | ソース |
|------|-------------------------|------------|--------|
| **CreateProfile** | REST API (JSON) | JSON Schema, Ed25519 signature | Control-Plane API |
| **UpdatePolicy** | gRPC (Protobuf) | SemVer, latency_budget > 0 | Session Orchestrator |
| **QueryProfile** | gRPC (Protobuf) | profile_id: String(64) | Experience Layer |

### 3.2 出力

| 名称 | プロトコル/フォーマット | SLA | 宛先 |
|------|-------------------------|-----|------|
| **QoSPolicyUpdate** | Event Bus (JSON) | P95 < 300ms | QoS Scheduler |
| **PolicyValidation** | Sync Response (JSON) | P99 < 150ms | Control-Plane API |
| **ProfileExport** | REST API (JSON) | P95 < 500ms | Experience Layer |

**QoSPolicyUpdate スキーマ**:
```json
{
  "schema_version": "1.2.0",
  "policy_id": "pol_xyz123",
  "profile_id": "prof_iot_lowpower_v2",
  "stream_id": 3,
  "latency_budget_ms": 50,
  "bandwidth_floor_mbps": 0.5,
  "fec_mode": "LIGHT",
  "priority": 2,
  "deprecated_after": "2026-01-01T00:00:00Z",
  "expiration_ts": "2025-10-02T10:30:00Z",
  "signature": "ed25519:base64..."
}
```

詳細: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. データモデル

### 4.1 主要エンティティ

#### PolicyProfile (プロファイル)
```yaml
PolicyProfile:
  profile_id: String(64)  # Primary Key, prefix: prof_
  profile_name: String(128)
  profile_version: SemVer
  use_case: Enum[IoT, AR_VR, Media8K, Gaming, Custom]
  latency_budget_ms: UInt16
  bandwidth_floor_mbps: Decimal(10,2)
  bandwidth_ceiling_mbps: Decimal(10,2)
  fec_mode: Enum[NONE, LIGHT, HEAVY]
  priority: UInt8  # 0 (lowest) - 7 (highest)
  power_profile: Enum[Ultra_Low, Low, Normal, High]
  deprecated_after: Timestamp (nullable)
  signature: Bytes(64)  # Ed25519
  created_at: Timestamp
  updated_at: Timestamp
```

#### QoSPolicy (ポリシーインスタンス)
```yaml
QoSPolicy:
  policy_id: String(64)  # Primary Key
  profile_id: String(64)  # Foreign Key
  device_id: String(64)
  stream_id: UInt8
  active: Boolean
  applied_at: Timestamp
  expires_at: Timestamp
```

### 4.2 プリセットプロファイル

| プロファイル名 | use_case | latency_ms | bandwidth_mbps | fec_mode | priority | power_profile |
|---------------|----------|------------|----------------|----------|----------|---------------|
| `prof_iot_lowpower_v2` | IoT | 200 | 0.1-1.0 | NONE | 1 | Ultra_Low |
| `prof_arvr_spatial_v1` | AR_VR | 12 | 50-200 | HEAVY | 7 | High |
| `prof_media_8k_v1` | Media8K | 50 | 1000-1500 | HEAVY | 6 | High |
| `prof_gaming_input_v1` | Gaming | 6 | 5-50 | LIGHT | 7 | Normal |

詳細: [spec/requirements.md](../requirements.md) - ユースケース

### 4.3 永続化
- **データストア**: CockroachDB (JSON カラムで拡張属性対応)
- **保持期間**: Active プロファイル: 無期限、Deprecated: 12ヶ月
- **暗号/秘匿**: signature フィールドで改ざん防止、Ed25519検証必須

---

## 5. 依存関係

| 種別 | コンポーネント | インターフェース | SLA/契約 |
|------|----------------|-------------------|----------|
| **上位** | Control-Plane API | REST/gRPC | P95 < 400ms |
| **上位** | Session Orchestrator | Event Bus | Best-effort |
| **下位** | QoS Scheduler | Event Bus (QoSPolicyUpdate) | At-least-once delivery |
| **下位** | Crypto & Trust (署名検証) | Sync API | P99 < 50ms |
| **Peer** | CockroachDB | SQL | P99 < 100ms |

**依存ルール**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 6. 性能・スケーラビリティ

### SLO/SLI

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| ポリシー配信レイテンシ (P95) | < 300ms | UpdatePolicy → QoSPolicyUpdate 発火 |
| プロファイル作成レイテンシ (P95) | < 500ms | CreateProfile → DB保存完了 |
| バリデーション成功率 | ≥ 99% | 検証成功数 / 総リクエスト数 |
| イベント配信信頼性 | ≥ 99.9% (at-least-once) | 配信確認数 / 発行数 |

詳細: [spec/performance/scalability.md](../performance/scalability.md)

---

## 7. セキュリティ & プライバシー

### 認証/認可
- **認証**: OAuth2 + mTLS
- **認可**: RBAC (Roles: `policy:create`, `policy:update`, `policy:read`, `policy:delete`)
- 詳細: [spec/security/auth.md](../security/auth.md)

### 脅威対策 (STRIDE)
| 脅威 | 対策 |
|------|------|
| **Tampering** | Ed25519署名検証 |
| **Repudiation** | 監査ログ (全変更記録) |
| **Information Disclosure** | プロファイルは非機密データだがアクセス制御 |

詳細: [spec/security/vulnerability.md](../security/vulnerability.md)

---

## 8. 観測性

### メトリクス

| メトリクス名 | 型 | ラベル |
|-------------|---|--------|
| `policy_updates_total` | Counter | profile_id, result |
| `policy_validation_duration_seconds` | Histogram | validation_type |
| `profile_active_count` | Gauge | use_case |

### ログフォーマット
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "INFO",
  "event": "policy.updated",
  "policy_id": "pol_xyz",
  "profile_id": "prof_iot_lowpower_v2",
  "device_id": "DEV-***",
  "trace_id": "..."
}
```

参照: [spec/testing/metrics.md](../testing/metrics.md)

---

## 9. SemVer 対応とバージョン管理

### バージョニング戦略
- **Major**: 後方非互換変更 (例: latency_budget_ms → latency_budget_us)
- **Minor**: 後方互換の機能追加 (例: 新フィールド追加)
- **Patch**: バグ修正

### deprecated_after メタデータ
```json
{
  "profile_version": "2.1.0",
  "deprecated_after": "2026-01-01T00:00:00Z",
  "migration_guide_url": "https://docs.honeylink/migration/v2-to-v3"
}
```

### 互換性マトリクス
| Client Version | Server Version | 互換性 |
|----------------|----------------|--------|
| 1.x | 1.x | ✅ Full |
| 1.x | 2.x | ✅ Read-only (deprecated警告) |
| 2.x | 1.x | ❌ Not supported |

---

## 10. フォールバック機構

### ポリシー適用失敗時の動作
```
1. QoS Scheduler へポリシー配信
2. Scheduler から ACK 待機 (タイムアウト 5秒)
3. NACK or タイムアウト発生
   ↓
4. 設定スナップショット から旧ポリシー取得
5. ロールバックイベント発行
6. 監査ログへ記録
```

### スナップショット管理
- **保存タイミング**: ポリシー適用成功時
- **保持期間**: 直近3世代
- **ストレージ**: Redis (TTL 24h)

---

## 11. 要件トレーサビリティ

### FR-04: QoS調整
- **関連**: ネットワーク状態に応じた動的ポリシー更新
- **実装**: Session Orchestrator からのトリガーでプロファイル再選択

### FR-06: プロファイルテンプレ共有
- **関連**: ベンダ固有設定のパッケージ化とエクスポート
- **実装**: ProfileExport API + Ed25519署名

**トレーサビリティID対応表**:
```
MOD-002-POLICY-ENGINE → FR-04 (policy management)
MOD-002-POLICY-ENGINE → FR-06 (profile sharing)
```

---

## 12. テスト戦略

### 単体テスト
- SemVer パース・比較ロジック (20ケース)
- JSON Schema バリデーション (正常/異常系 各15ケース)
- Ed25519 署名検証 (10ケース)
- カバレッジ目標: 90%

### 統合テスト
- ポリシー配信 → QoS Scheduler受信 (E2E)
- フォールバック動作検証
- プロファイルエクスポート/インポート

詳細: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/testing/integration-tests.md](../testing/integration-tests.md)

---

## 13. デプロイ & 運用

- **デプロイ方法**: Blue/Green deployment
- **インフラ要件**: 1 vCPU, 1GB RAM/instance
- **ロールバック条件**: バリデーション成功率 < 90% (5分継続)

詳細: [spec/deployment/ci-cd.md](../deployment/ci-cd.md)

---

## 14. リスク & 技術的負債

| リスク | 緩和策 |
|--------|--------|
| プロファイル署名鍵の漏洩 | 90日自動ローテーション + HSM保管 |
| バージョン互換性バグ | 統合テスト自動化 |

---

## 15. 受け入れ基準 (DoD)

- [x] QoSPolicyUpdate スキーマ定義完了
- [x] SemVer対応とバージョン管理仕様完成
- [x] プロファイルCRUD仕様記述完了
- [x] FR-04/FR-06 との紐付け明示
- [x] トレーサビリティID (`MOD-002-POLICY-ENGINE`) 付与
- [x] C/C++ 依存排除確認 (Rust純実装)
- [x] フォールバック機構仕様化完了

---

## 16. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | Protocol WG (ENG-PROTO-01) |

