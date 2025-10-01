# HoneyLink™ コントロールプレーン API 仕様書

## 概要
HoneyLink™ コントロールプレーン API は、デバイス登録、セッション制御、ポリシー配布を統合的に管理するための HTTPS ベース REST/JSON API である。全通信は HoneyLink Secure Transport (HLST) 上の mTLS を必須とし、任意の環境 (オンプレミス、エアギャップ、クラウド) で同一仕様を適用できるよう設計する。

- **ベース URL:** `https://<control-plane-host>/api/v1`
- **メディアタイプ:** `application/json; charset=utf-8`
- **認可:** mTLS + OAuth2 相当の短命トークン (5 分)
- **トレーサビリティ:** OpenTelemetry Traceparent ヘッダ必須

## セキュリティ
- クライアント証明書は `ed25519` または `secp384r1` を許可。RSA は禁止。
- トークンは OIDC 互換 IdP から発行するか、オフライン環境ではハードウェアトークンを利用して同等の署名付きアクセストークンを生成。
- 監査ログはすべて `docs/security/key-management.md` の監査要件に準拠。

## エラーモデル
| HTTP | エラーコード | 説明 |
|------|--------------|------|
| 400 | `ERR_VALIDATION` | 入力検証エラー |
| 401 | `ERR_AUTH` | 認証失敗 / 証明書不正 |
| 403 | `ERR_AUTHZ` | RBAC/ABAC ポリシー違反 |
| 404 | `ERR_NOT_FOUND` | リソース未存在 |
| 409 | `ERR_CONFLICT` | バージョン衝突、重複登録 |
| 422 | `ERR_STATE` | 状態遷移違反 |
| 500 | `ERR_INTERNAL` | 想定外エラー |
| 503 | `ERR_DEPENDENCY` | 下位サービス不可 |

```json
{
  "error_code": "ERR_VALIDATION",
  "message": "pairing_code is invalid",
  "trace_id": "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
}
```

## 主要リソース

### 1. デバイス登録
- **Endpoint:** `POST /devices`
- **目的:** 製造ラインまたはフィールドでのデバイス初回登録。
- **リクエスト**
```json
{
  "device_id": "HL-EDGE-0001",
  "public_key": "base64(x25519 pub)",
  "firmware_version": "1.2.0",
  "capabilities": ["telemetry", "control"],
  "attestation": {
    "format": "remote-attestation-v1",
    "evidence": "base64(blob)",
    "nonce": "hex"
  },
  "metadata": {
    "location": "factory-A",
    "serial": "SN12345678"
  }
}
```
- **バリデーション**
  - `device_id`: `/^[A-Z0-9-]{4,64}$/`
  - `public_key`: X25519 32 バイト
  - `firmware_version`: SemVer 互換
- **レスポンス (201)**
```json
{
  "device_token": "opaque",
  "pairing_code": "N3Q6-9P4L-7XZC",
  "registered_at": "2025-03-01T09:00:00Z",
  "expires_at": "2025-03-01T09:10:00Z"
}
```

### 2. ペアリング確立
- **Endpoint:** `POST /devices/{device_id}/pair`
- **目的:** デバイスがコントロールプレーンと相互認証する。
- **リクエスト**
```json
{
  "pairing_code": "N3Q6-9P4L-7XZC",
  "device_cert_csr": "base64(pem)",
  "telemetry_topics": ["default", "latency"],
  "policy_version": "2025.03.01"
}
```
- **レスポンス (200)**
```json
{
  "device_certificate": "base64(pem)",
  "policy_bundle": {
    "version": "2025.03.01",
    "sha512": "hex",
    "signed_payload": "base64"
  },
  "session_endpoint": "quic://core.honeylink.local:7843"
}
```
- **状態遷移:** `pending` → `paired`

### 3. セッション制御
- **Endpoint:** `POST /sessions`
- **目的:** マルチストリームセッション確立要求。
- **リクエスト**
```json
{
  "device_id": "HL-EDGE-0001",
  "requested_streams": [
    { "name": "telemetry", "mode": "unreliable", "qos": "burst" },
    { "name": "control", "mode": "reliable", "qos": "latency" }
  ],
  "traceparent": "00-..."
}
```
- **レスポンス (201)**
```json
{
  "session_id": "sess_8f1c0d3a",
  "expires_at": "2025-03-01T09:30:00Z",
  "stream_allocations": [
    { "name": "telemetry", "cid": 4, "key_material": "base64" },
    { "name": "control", "cid": 1, "key_material": "base64" }
  ]
}
```
- **備考:** `key_material` は HKDF でデバイス側と共有し、揮発メモリにのみ保存。

### 4. ポリシー配布
- **Endpoint:** `PUT /devices/{device_id}/policy`
- **目的:** QoS、暗号スイート、機能フラグの更新。
- **リクエスト**
```json
{
  "policy_version": "2025.03.10",
  "qos": {
    "telemetry": { "priority": 3, "latency_budget_ms": 150 },
    "control": { "priority": 1, "latency_budget_ms": 30 }
  },
  "encryption": {
    "ciphers": ["chacha20-poly1305"],
    "fallback": null
  },
  "features": {
    "ota_update": false,
    "diagnostics": true
  }
}
```
- **レスポンス (202)**
```json
{
  "policy_version": "2025.03.10",
  "applied": true,
  "applied_at": "2025-03-10T02:00:00Z"
}
```

### 5. 鍵ローテーション
- **Endpoint:** `POST /devices/{device_id}/keys/rotate`
- **目的:** アドホックな鍵更新を要求。
- **リクエスト**
```json
{
  "reason": "suspected-compromise",
  "requested_by": "secops",
  "traceparent": "00-..."
}
```
- **レスポンス (200)**
```json
{
  "rotation_id": "rot_45ab",
  "status": "initiated",
  "expected_completion": "2025-03-01T09:05:00Z"
}
```
- **後続:** デバイスはローテーション完了後に `POST /devices/{device_id}/keys/ack` を送信。

### 6. 監査イベント取得
- **Endpoint:** `GET /audit/events`
- **クエリ:** `?device_id=HL-EDGE-0001&since=2025-03-01T00:00:00Z`
- **レスポンス (200)**
```json
{
  "events": [
    {
      "id": "evt_c4a2",
      "timestamp": "2025-03-01T08:59:00Z",
      "category": "key-rotation",
      "actor": "secops",
      "outcome": "success",
      "details": {
        "rotation_id": "rot_45ab"
      }
    }
  ],
  "next": null
}
```

## Webhook (オプション)
- **Endpoint:** `POST https://<customer-endpoint>/honeylink/events`
- **イベント種別:** `policy-applied`, `key-rotation`, `device-alert`
- **署名:** HTTP ヘッダ `X-HoneyLink-Signature: ed25519(base64)`
- **リトライ:** 指数バックオフ (最大 24 時間)。
- **オンプレ要件:** オフライン環境では Webhook を利用せず、監査 API でのポーリングのみを許可。

## バージョン管理
- 破壊的変更は `/api/v{n}` のメジャーバージョンで区切る。
- スキーマ差分は `docs/templates/api-change-log.md` に追記。

## SLA 指標
| 指標 | 目標 | 備考 |
|------|------|------|
| API 成功率 | 99.95% | 集計期間: 月次 |
| 平均応答時間 (p95) | 180ms | セッション制御エンドポイント |
| 監査配信レイテンシ | 60 秒以内 | Webhook またはポーリング |

## 関連ドキュメント
- `docs/security/key-management.md`: 鍵ローテーションと発行ポリシー
- `docs/requirements.md`: FR-03, FR-05, NFR-02, NFR-07
- `docs/architecture/module-control-plane.md`: コントロールプレーン設計
- `docs/testing/api.md`: API テストケース
