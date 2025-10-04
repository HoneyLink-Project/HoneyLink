# docs/security/auth.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> HoneyLink™ のP2P認証・信頼モデルを仕様化します。TOFU (Trust On First Use) ベースのデバイス間直接ペアリングアーキテクチャであり、中央サーバーやOAuth2を使用しません。

## 目次
- [P2P認証方針](#p2p認証方針)
- [TOFU信頼モデル](#tofu信頼モデル)
- [ペアリングプロトコル](#ペアリングプロトコル)
- [鍵交換とセッション確立](#鍵交換とセッション確立)
- [信頼リスト管理](#信頼リスト管理)
- [鍵変更検出と警告](#鍵変更検出と警告)
- [監査・ログ](#監査ログ)
- [脅威と対策](#脅威と対策)
- [関連文書](#関連文書)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## P2P認証方針
- **信頼モデル:** TOFU (Trust On First Use) - 初回ペアリング時に信頼関係を確立、以降は公開鍵で認証
- **中央サーバーなし:** IdP、OAuth2、JWT不要。全ての認証はデバイス間直接通信
- **ペアリング方法:** QRコード、6桁PIN、NFC (将来)
- **暗号化:** X25519 ECDH鍵交換 + ChaCha20-Poly1305 AEAD
- **Pure Rust:** RustCrypto suite使用、C/C++依存なし

## TOFU信頼モデル

### 基本原則
1. **初回ペアリング時に信頼確立**
   - デバイスAがデバイスBを初めて発見 → ユーザーがQR/PINで認証 → 公開鍵を相互交換
   - 公開鍵をローカルの信頼リスト (`~/.honeylink/trusted_peers.json`) に保存
   - 以降のセッション確立時は公開鍵で自動認証

2. **Bluetooth互換の体験**
   - 初回: 「ペアリングしますか？」→ YES (Bluetoothと同じ)
   - 2回目以降: 自動接続 (Bluetoothと同じ)
   - 公開鍵変更時: 「デバイス鍵が変更されました。再ペアリングしますか？」警告

3. **信頼スコープ**
   - 信頼関係はデバイス単位 (ユーザー単位ではない)
   - 各デバイスは独自の `device_key.pem` (X25519秘密鍵) を持つ
   - OS Keychainで秘密鍵を保護 (Windows DPAPI/macOS Keychain/Linux Secret Service)

## ペアリングプロトコル

### QRコードペアリング
```
1. デバイスA: mDNS/BLEでデバイスBを発見
2. デバイスB: ペアリング待機画面でQRコード表示
   QRコード内容: {"device_id": "DEV-B-UUID", "public_key": "base64(32 bytes)"}
3. デバイスA: カメラでQRコード読み取り
4. デバイスA: ユーザーに確認 "デバイス 'Bob's iPhone' とペアリングしますか？"
5. ユーザーがYES → ECDH鍵交換開始
6. 相互に公開鍵をtrusted_peers.jsonに保存
```

### 6桁PINペアリング
```
1. デバイスA: mDNS/BLEでデバイスBを発見
2. デバイスB: 6桁PIN表示 (例: 123456, 30秒有効)
3. デバイスA: PIN入力画面
4. ユーザーがPIN入力 → デバイスAがBLEでPIN送信
5. デバイスB: PIN検証成功 → ECDH鍵交換開始
6. 相互に公開鍵をtrusted_peers.jsonに保存
```

### NFCペアリング (Phase 2以降)
```
1. デバイスA: NFC検出待機
2. ユーザーがデバイスAとBをタップ
3. NFC経由で公開鍵交換
4. 自動的にtrusted_peers.jsonに保存
```

## 鍵交換とセッション確立

### 初回ペアリング時
```
Alice (Device A):
  1. private_key_a ← ~/.honeylink/keys/device_key.pem
  2. public_key_a ← X25519_base_point × private_key_a
  3. QR/PIN認証成功
  4. public_key_b ← ピアから受信
  5. shared_secret ← X25519(private_key_a, public_key_b)
  6. session_key ← HKDF-SHA512(shared_secret, "HoneyLink-Session-v1", 32 bytes)
  7. trusted_peers.json に追加:
     {
       "device_id": "DEV-B-UUID",
       "device_name": "Bob's iPhone",
       "public_key": "base64(public_key_b)",
       "first_seen": "2025-10-01T10:30:00Z",
       "last_seen": "2025-10-01T10:30:00Z",
       "pairing_method": "qr_code"
     }
```

### 2回目以降のセッション確立
```
Alice (Device A):
  1. mDNS/BLEでDevice Bを発見
  2. trusted_peers.jsonから public_key_b を検索
  3. 見つかった → 自動的にECDH鍵交換 (ユーザー確認不要)
  4. 見つからない → 初回ペアリングフローへ
```

## 信頼リスト管理

### trusted_peers.json構造
```json
{
  "version": "1.0.0",
  "device_id": "DEV-A-UUID",
  "peers": [
    {
      "device_id": "DEV-B-UUID",
      "device_name": "Bob's iPhone",
      "public_key": "base64(32 bytes)",
      "first_seen": "2025-10-01T10:30:00Z",
      "last_seen": "2025-10-04T15:22:00Z",
      "pairing_method": "qr_code",
      "key_fingerprint": "SHA256:abc123...",
      "trusted": true,
      "notes": "Work device"
    }
  ]
}
```

### ファイル保存
- **パス:** `~/.honeylink/trusted_peers.json`
- **パーミッション:** 0600 (所有者のみ読み書き)
- **バックアップ:** OS Keychainに暗号化バックアップ (オプション)

### UI操作
- **信頼デバイス一覧:** Experience Layer で表示
- **デバイス削除:** ユーザーが手動削除可能 (次回は再ペアリング必要)
- **デバイス名変更:** ユーザーが任意に変更可能

## 鍵変更検出と警告

### 鍵変更シナリオ
```
1. Device Bの秘密鍵が変更された (OSクリーンインストール、デバイス交換など)
2. Device AがDevice Bと接続試行
3. 受信した public_key_b が trusted_peers.json と一致しない
4. Device A: ⚠️ 警告表示
   "デバイス 'Bob's iPhone' の鍵が変更されました。
    以前: SHA256:abc123...
    現在: SHA256:def456...
    これは正当な変更ですか？"
5. ユーザー選択:
   - 「信頼して続行」→ trusted_peers.json更新
   - 「接続拒否」→ セッション確立中止
```

### 中間者攻撃対策
- 初回ペアリング時の物理的な確認 (QR/PIN) でMITM防止
- 鍵変更時の警告でMITM検出
- 公開鍵フィンガープリント表示 (SHA256ハッシュ)

## 監査・ログ

### ローカル監査ログ
- **パス:** `~/.honeylink/logs/audit.log`
- **形式:** JSON Lines (append-only)
- **含むフィールド:**
  ```json
  {
    "timestamp": "2025-10-01T10:30:00Z",
    "event_type": "pairing.success",
    "device_id": "DEV-A-UUID",
    "peer_device_id": "DEV-B-UUID",
    "pairing_method": "qr_code",
    "key_fingerprint": "SHA256:abc123...",
    "user_action": "approved",
    "context": {
      "discovery_protocol": "mdns",
      "ip_address": "192.168.1.100"
    }
  }
  ```
- **保持期間:** 90日 (自動削除)
- **最大サイズ:** 50MB (ローテーション)

### 監査イベント
| イベント | event_type | 説明 |
|---------|------------|------|
| 初回ペアリング | `pairing.success` | QR/PINペアリング成功 |
| ペアリング失敗 | `pairing.failed` | PIN誤り、タイムアウト |
| 鍵変更検出 | `key.changed` | 公開鍵不一致検出 |
| デバイス削除 | `peer.removed` | ユーザーが信頼リストから削除 |
| セッション確立 | `session.established` | 自動セッション確立 |
| セッション拒否 | `session.rejected` | 信頼なし、鍵不一致 |

## 脅威と対策

| 脅威 | シナリオ | 対策 |
|------|---------|------|
| 中間者攻撃 (MITM) | 初回ペアリング時の盗聴 | QR/PIN物理的確認、鍵変更警告 |
| 公開鍵なりすまし | 攻撃者がBLEでなりすまし | QR/PINでデバイス識別、フィンガープリント表示 |
| リプレイ攻撃 | 古いセッションパケット再送 | Nonceベースのリプレイ防止 (QUIC内蔵) |
| 秘密鍵漏洩 | device_key.pem盗難 | OS Keychain保護 (DPAPI/Keychain/Secret Service) |
| デバイス紛失 | 物理デバイス紛失 | リモート削除不可 (P2P設計)、OS暗号化必須 |
| フィッシング | 偽ペアリング画面 | 不可能 (中央サーバーなし) |

### MITM攻撃詳細
```
攻撃シナリオ:
1. Device AがDevice Bとペアリング試行
2. 攻撃者がBLEで割り込み、自身の公開鍵を送信
3. Device Aは攻撃者と鍵交換

対策:
- QR/PINで物理的にデバイス識別 (攻撃者は画面を偽造できない)
- 公開鍵フィンガープリント表示 (ユーザーが確認可能)
- 2回目以降の接続で鍵変更を検出
```

## 関連文書
- [spec/security/encryption.md](./encryption.md): X25519/ChaCha20-Poly1305詳細
- [spec/security/key-management.md](./key-management.md): デバイス鍵管理
- [spec/modules/crypto-trust-anchor.md](../modules/crypto-trust-anchor.md): 暗号実装仕様
- [spec/modules/physical-adapter.md](../modules/physical-adapter.md): mDNS/BLE Discovery
- [spec/ui/pairing-screens.md](../ui/pairing-screens.md): ペアリングUI仕様

## 受け入れ基準 (DoD)
- [x] TOFU信頼モデルが定義されている
- [x] QR/PINペアリングプロトコルが定量的に記述されている
- [x] trusted_peers.json構造が仕様化されている
- [x] 鍵変更検出フローが定義されている
- [x] MITM攻撃対策が具体的に記述されている
- [x] ローカル監査ログ仕様が定義されている
- [x] C/C++依存なし (Pure Rust, RustCrypto suite)
- [x] Bluetooth互換の体験 (初回ペアリング、2回目以降自動接続)
- [x] 中央サーバー不要 (完全P2P)
