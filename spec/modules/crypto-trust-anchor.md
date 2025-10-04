# Module Specification: Crypto & Trust Anchor

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Crypto & Trust Anchor モジュールの実装仕様書。エンドツーエンド暗号化と鍵管理を担当します。

**トレーサビリティ ID**: `MOD-004-CRYPTO`

---

## 1. モジュール概要

- **モジュール名:** Crypto & Trust Anchor
- **担当チーム:** Security WG (ENG-SEC-01, ENG-SEC-02)
- **概要:** X25519/ChaCha20-Poly1305/HKDF-SHA512 による暗号化、鍵交換、鍵階層管理
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `crates/crypto/`

### 価値提案
- 量子コンピュータ耐性を見据えた Post-Quantum Cryptography (PQC) 準備 (Kyber統合予定)
- C/C++ 依存なし (RustCrypto suite 純実装)
- HSM 連携による Root Key 保護
- 90日自動鍵ローテーション

---

## 2. 責務と境界

### 主な責務
- **鍵交換**: X25519 ECDH (Elliptic Curve Diffie-Hellman)
- **暗号化/復号化**: ChaCha20-Poly1305 AEAD (Authenticated Encryption with Associated Data)
- **鍵導出**: HKDF-SHA512 (HMAC-based Extract-and-Expand Key Derivation Function)
- **鍵階層管理**: Root Key → Session Key → Stream Key
- **デジタル署名**: Ed25519 (プロファイル署名, Audit Log 署名)
- **鍵ローテーション**: 90日周期自動更新

### 非責務
- **鍵配布**: Session Orchestrator に委譲
- **アクセス制御**: Policy Engine に委譲
- **監査ログ保存**: Telemetry に委譲
- **HSM操作**: Infrastructure Team が管理

### 関連ドキュメント
- [spec/security/encryption.md](../security/encryption.md)
- [spec/security/key-management.md](../security/key-management.md)
- [spec/requirements.md](../requirements.md) - FR-02 (認証), NFR-02 (暗号化)

---

## 3. インターフェース

### 3.1 入力

| 名称 | プロトコル/フォーマット | 検証ルール | ソース |
|------|-------------------------|------------|--------|
| **KeyExchangeRequest** | Internal API (Rust) | public_key: 32 bytes | Session Orchestrator |
| **EncryptRequest** | Internal API (Rust) | plaintext.len() <= 1MB | Transport |
| **SignRequest** | Internal API (Rust) | data: Bytes | Policy Engine |

### 3.2 出力

| 名称 | プロトコル/フォーマット | SLA | 宛先 |
|------|-------------------------|-----|------|
| **SharedSecret** | Sync Response (32 bytes) | P95 < 10ms | Session Orchestrator |
| **EncryptedPayload** | CBOR (ChaCha20-Poly1305) | P95 < 20ms | Transport |
| **Signature** | Bytes (64) | P99 < 50ms | Policy Engine |
| **KeyRotation** | Internal API (Rust callback) | Async | Telemetry |

**EncryptedPayload スキーマ**:
```json
{
  "ciphertext": "base64...",
  "nonce": "base64(12 bytes)",
  "tag": "base64(16 bytes)",
  "key_id": "key_stream_xyz",
  "algorithm": "ChaCha20-Poly1305"
}
```

詳細: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. データモデル

### 4.1 鍵階層

```
Root Key (HSM保管)
  ↓ HKDF-SHA512
Device Master Key (90日ローテーション)
  ↓ HKDF-SHA512
Session Key (セッション確立時生成)
  ↓ HKDF-SHA512
Stream Key (ストリームID毎, 24時間ローテーション)
```

#### KeyMaterial (鍵マテリアル)
```yaml
KeyMaterial:
  key_id: String(64)  # Primary Key, prefix: key_
  key_type: Enum[Root, DeviceMaster, Session, Stream]
  key_bytes: Bytes(32)  # ChaCha20-Poly1305用
  parent_key_id: String(64) (nullable)
  created_at: Timestamp
  expires_at: Timestamp
  rotated: Boolean
  hsm_backed: Boolean  # Root Key のみ true
```

#### KeyRotationLog
```yaml
KeyRotationLog:
  rotation_id: UUIDv7
  old_key_id: String(64)
  new_key_id: String(64)
  rotation_reason: Enum[Scheduled, Compromised, Manual]
  rotated_at: Timestamp
  rotated_by: String(128)  # User ID or "system"
```

### 4.2 永続化
- **データストア**: ローカルファイル (~/.honeylink/keys/device_key.pem 0600権限) + メモリ (Session/Stream Keys - TTL付きHashMap)
- **保持期間**: Device Master (無期限, 90日推奨ローテーション), Session (24h), Stream (24h)
- **暗号/秘匿**: OS Keychain統合 (Windows DPAPI/macOS Keychain/Linux Secret Service) でDevice Master Key保護

詳細: [spec/security/key-management.md](../security/key-management.md)

---

## 5. アルゴリズム

### 5.1 X25519 鍵交換

```
Alice (Device A):
  private_key_a ← random(32 bytes)
  public_key_a ← X25519_base_point × private_key_a

Bob (Device B):
  private_key_b ← random(32 bytes)
  public_key_b ← X25519_base_point × private_key_b

Alice → Bob: public_key_a
Bob → Alice: public_key_b

Alice: shared_secret ← X25519(private_key_a, public_key_b)
Bob:   shared_secret ← X25519(private_key_b, public_key_a)

assert(Alice.shared_secret == Bob.shared_secret)
```

### 5.2 HKDF-SHA512 鍵導出

```
PRK ← HKDF-Extract(salt=random(32), IKM=shared_secret)
OKM ← HKDF-Expand(PRK, info="HoneyLink-SessionKey-v1", L=32)

Session Key ← OKM
```

### 5.3 ChaCha20-Poly1305 暗号化

```
nonce ← random(12 bytes)  # 1セッション内で再利用禁止
ciphertext, tag ← ChaCha20-Poly1305-Encrypt(key, nonce, plaintext, aad="session_id")
```

**AAD (Additional Authenticated Data)**: `session_id` を含めることで、異なるセッション間での暗号文の再利用を防止

参照: [spec/security/encryption.md](../security/encryption.md)

---

## 6. 依存関係

| 種別 | コンポーネント | インターフェース | SLA/契約 |
|------|----------------|-------------------|----------|
| **上位** | Session Orchestrator | KeyExchangeRequest | P95 < 50ms |
| **上位** | Transport | EncryptRequest | P95 < 30ms |
| **上位** | Policy Engine | SignRequest | P99 < 50ms |
| **Peer** | OS Keychain (DPAPI/Keychain/Secret Service) | System API | P99 < 50ms |

**依存ルール**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 7. 性能・スケーラビリティ

### SLO/SLI

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| 鍵交換レイテンシ (P95) | < 10ms | KeyExchangeRequest → SharedSecret |
| 暗号化レイテンシ (P95) | < 20ms | EncryptRequest → EncryptedPayload (1KB) |
| 署名生成レイテンシ (P99) | < 50ms | SignRequest → Signature |
| スループット | ≥ 10K ops/sec/instance | 暗号化操作数 |

詳細: [spec/performance/benchmark.md](../performance/benchmark.md)

---

## 8. セキュリティ & プライバシー

### 脅威対策 (STRIDE)
| 脅威 | 対策 |
|------|------|
| **Spoofing** | X25519鍵交換 + mTLS |
| **Tampering** | ChaCha20-Poly1305 AEAD tag検証 |
| **Information Disclosure** | エンドツーエンド暗号化 |
| **Denial of Service** | Rate limiting (10K ops/sec/instance) |
| **Elevation of Privilege** | HSM による Root Key 保護 |

### PQC対応ロードマップ
- **Phase 1 (2025 Q4)**: Kyber768 統合 (Post-Quantum Key Encapsulation)
- **Phase 2 (2026 Q2)**: Hybrid mode (X25519 + Kyber768)
- **Phase 3 (2027)**: X25519廃止検討

詳細: [spec/security/vulnerability.md](../security/vulnerability.md)

---

## 9. 観測性

### メトリクス

| メトリクス名 | 型 | ラベル |
|-------------|---|--------|
| `crypto_operations_total` | Counter | operation_type, result |
| `crypto_operation_duration_seconds` | Histogram | operation_type |
| `crypto_key_rotations_total` | Counter | key_type, rotation_reason |
| `crypto_active_keys_count` | Gauge | key_type |

### ログフォーマット
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "INFO",
  "event": "key.rotated",
  "key_type": "DeviceMaster",
  "old_key_id": "key_xyz",
  "new_key_id": "key_abc",
  "rotation_reason": "Scheduled",
  "trace_id": "..."
}
```

参照: [spec/testing/metrics.md](../testing/metrics.md)

---

## 10. 鍵ローテーション

### 自動ローテーションスケジュール
| 鍵タイプ | ローテーション周期 | トリガー |
|---------|------------------|---------|
| Root Key | 365日 | Manual only (高リスク) |
| Device Master Key | 90日 | Scheduled (Cron: 0 0 * * 0) |
| Session Key | セッション終了時 | Event-driven |
| Stream Key | 24時間 | Scheduled |

### ローテーション手順
```
1. 新鍵生成 (HKDF-SHA512)
2. 新鍵をローカルファイルとOS Keychainに保存
3. KeyRotationLog 記録
4. 旧鍵を `rotated=true` にマーク (即座に削除しない)
5. Grace Period (1時間) 後に旧鍵削除
6. Telemetry へイベント送信
```

詳細: [spec/security/key-management.md](../security/key-management.md)

---

## 11. 要件トレーサビリティ

### FR-02: 認証
- **関連**: デバイス認証時の X25519鍵交換
- **実装**: KeyExchangeRequest → SharedSecret

### NFR-02: 暗号化
- **関連**: 全通信の ChaCha20-Poly1305 暗号化
- **実装**: EncryptRequest → EncryptedPayload

**トレーサビリティID対応表**:
```
MOD-004-CRYPTO → FR-02 (authentication via key exchange)
MOD-004-CRYPTO → NFR-02 (end-to-end encryption)
```

---

## 12. テスト戦略

### 単体テスト
- X25519 鍵交換 (10ケース、公開鍵長さ異常系含む)
- ChaCha20-Poly1305 暗号化/復号化 (20ケース、nonce再利用検知含む)
- HKDF-SHA512 鍵導出 (15ケース)
- Ed25519 署名/検証 (10ケース)
- カバレッジ目標: 95%

### 統合テスト
- OS Keychain連携 (Device Master Key保護)
- メモリTTL管理 (Session/Stream Keys)
- 鍵ローテーション E2E (旧鍵で復号可能、Grace Period検証)

### セキュリティテスト
- Nonce再利用攻撃検知
- Tag改ざん検知
- Known-plaintext attack 耐性検証

詳細: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/security/vulnerability.md](../security/vulnerability.md)

---

## 13. デプロイ & 運用

- **デプロイ方法**: Blue/Green deployment
- **インフラ要件**: 1 vCPU, 512MB RAM/instance
- **ロールバック条件**: 暗号化エラー率 > 0.1% (1分継続)

詳細: [spec/deployment/ci-cd.md](../deployment/ci-cd.md)

---

## 14. リスク & 技術的負債

| リスク | 緩和策 |
|--------|--------|
| Nonce衝突 | Counter mode + セッションID prefix |
| 量子コンピュータ攻撃 | Kyber768統合 (2025 Q4) |
| Device Key破損 | OS Keychainから復元 or 再ペアリング |

---

## 15. 受け入れ基準 (DoD)

- [x] X25519/ChaCha20-Poly1305/HKDF-SHA512 仕様記述完了
- [x] 鍵階層図 (Root → Stream) 作成完了
- [x] encryption.md との整合性確認完了
- [x] FR-02/NFR-02 との紐付け明示
- [x] トレーサビリティID (`MOD-004-CRYPTO`) 付与
- [x] C/C++ 依存排除確認 (RustCrypto suite使用)
- [x] 90日自動ローテーション仕様完成

---

## 16. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | Security WG (ENG-SEC-01) |
