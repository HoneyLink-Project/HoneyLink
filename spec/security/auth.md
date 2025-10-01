# docs/security/auth.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> HoneyLink™ の認証・認可アーキテクチャを仕様化します。言語/実装に依存しない抽象記述であり、C/C++ライブラリは想定外とします。

## 目次
- [認証基盤方針](#認証基盤方針)
- [OIDC/OAuth2 フロー](#oidcoauth2-フロー)
- [セッション管理](#セッション管理)
- [認可モデル (RBAC/ABAC)](#認可モデル-rbacabac)
- [デバイストラストとコンプライアンス](#デバイストラストとコンプライアンス)
- [監査・ログ](#監査ログ)
- [脅威と対策](#脅威と対策)
- [関連文書](#関連文書)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## 認証基盤方針
- **IdP:** 外部 OIDC プロバイダ (例: Azure AD, Auth0)。C/C++ ベースの独自スタックは使用しない。
- **強制 MFA:** 管理者・SREは必須。ユーザーはリスクベースで要求。
- **デバイスバインド:** ペアリング時にデバイス証明書 (X.509) または WebAuthn を使用。
- **ゼロトラスト:** すべての API 呼び出しでユーザー/デバイス/コンテキストを評価。

## OIDC/OAuth2 フロー
| フロー | 対象 | 説明 |
|--------|------|------|
| Authorization Code with PKCE | エンドユーザー | モバイル/ウェブからポータル操作。PKCE でコード interception を防止 |
| Client Credentials | マイクロサービス間 | セッション管理とポリシーエンジン間通信 |
| Device Code | IoT | 画面を持たないデバイス用。QR/コード入力 |

### トークン構造
- **ID Token:** オーディエンス = HoneyLink ポータル。含むクレーム: `sub`, `roles`, `device_id`, `mfa_level`。
- **Access Token:** スコープ例 `stream.manage`, `policy.write`, `metrics.read`。
- **Refresh Token:** 回転式。無効化 API を提供。

## セッション管理
- セッションID: UUIDv7。サーバサイド状態は Session Orchestrator に保存。
- 有効期限: 12 時間。アクティビティで 30 分滑走更新。
- セッション鍵: [docs/security/encryption.md](./encryption.md)で定義する HKDF。
- 再接続: デバイスはキャッシュした公開鍵を使用、ユーザー確認ステップは 2 要素。
- ログアウト: 全アクセストークン・セッション鍵を即時失効。

## 認可モデル (RBAC/ABAC)
| ロール | 権限 | 属性例 |
|--------|------|--------|
| `role:user` | デバイス接続、プロファイル適用 | 企業ID、地域 |
| `role:admin` | ポリシー作成/削除、監査取得 | 監査レベル、営業時間 |
| `role:sre` | メトリクス閲覧、アラート解除 | 勤務帯、リージョン |
| `role:auditor` | リードオンリーアクセス | 契約ID |

- **ABAC:** 地理 (region=EU/US), 時間 (business_hours=true), デバイスコンプライアンス (compliant=true)。
- ポリシー言語: JSONベース DSL。C/C++ パーサは使用せず、標準 JSON 評価器で処理。

## デバイストラストとコンプライアンス
- デバイス証明書はマネージド CA で発行。失効情報は CRL/OCSP 相当のサービスで提供。
- スコアリング: OS バージョン、暗号モジュール、セキュリティパッチで評価。
- コンプライアンス判定が false の場合、アクセストークンのスコープを制限。

## 監査・ログ
- すべてのロール/トークン操作を[docs/security/encryption.md](./encryption.md)の鍵スコープと合わせて記録。
- ログ形式: Immutable JSON Lines。含むフィールド: `event_id`, `timestamp`, `actor`, `context`, `result`。
- 保持期間: 1 年、Write Once Read Many (WORM) ストレージ。

## 脅威と対策
| 脅威 | 例 | 対策 |
|------|----|------|
| フィッシング | 偽ポータル誘導 | MFA 要求、FIDO2 サポート |
| トークン窃取 | アクセストークン漏洩 | PoP トークン、短命 Access Token、DPoP | 
| リプレイ攻撃 | セッション再利用 | Nonce/リプレイ防止ストア |
| 権限昇格 | RBAC設定漏れ | 四半期レビュー、ポリシーテンプレ署名 |

## 関連文書
- [docs/security/encryption.md](./encryption.md): 鍵管理と暗号化詳細
- [docs/security/vulnerability.md](./vulnerability.md): STRIDE ベースの脆弱性対策
- [docs/architecture/interfaces.md](../architecture/interfaces.md): 認証ヘッダ仕様
- [docs/testing/e2e-tests.md](../testing/e2e-tests.md): 認証シナリオテスト

## 受け入れ基準 (DoD)
- OIDC/OAuth2 フローが全ロールに対して定義されている。
- トークン、セッション、認可の仕様が定量的に記述されている。
- C/C++ を用いない前提が明確化されている。
- 監査・脅威対策が[docs/security/vulnerability.md](./vulnerability.md)と整合。
- 関連文書リンクが有効で、ロードマップ/テストと連携している。
