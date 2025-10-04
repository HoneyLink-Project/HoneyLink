# docs/requirements.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> ここで定義する要件はすべて言語非依存・実装非記述です。C/C++由来のスタックは採用禁止とし、純粋な代替手段やサービス連携のみを扱います。

## 目次
- [docs/requirements.md](#docsrequirementsmd)
  - [目次](#目次)
  - [前提と文書間トレーサビリティ](#前提と文書間トレーサビリティ)
  - [ペルソナ](#ペルソナ)
  - [ユースケース](#ユースケース)
  - [機能要件](#機能要件)
  - [非機能要件](#非機能要件)
    - [性能](#性能)
    - [可用性・信頼性](#可用性信頼性)
    - [セキュリティ・プライバシー](#セキュリティプライバシー)
    - [運用・可観測性](#運用可観測性)
    - [拡張性・保守性](#拡張性保守性)
    - [アクセシビリティ・国際化](#アクセシビリティ国際化)
    - [コンプライアンス](#コンプライアンス)
  - [環境要件](#環境要件)
  - [制約と禁止事項](#制約と禁止事項)
  - [スコープ外項目](#スコープ外項目)
  - [成功指標](#成功指標)
  - [トレーサビリティ方針](#トレーサビリティ方針)
  - [用語集](#用語集)
  - [受け入れ基準 (DoD)](#受け入れ基準-dod)

## 前提と文書間トレーサビリティ
- 本書の要求は[docs/architecture/overview.md](./architecture/overview.md)、[docs/ui/overview.md](./ui/overview.md)、[docs/security/auth.md](./security/auth.md)等で設計化される。
- テスト方針との紐付けは[docs/testing/unit-tests.md](./testing/unit-tests.md)ほかを参照。
- KPI/SLI は[docs/README.md](./README.md)および[docs/testing/metrics.md](./testing/metrics.md)に反映。

## ペルソナ
| 名称 | 役割 | ゴール | 痛点 |
|------|------|--------|------|
| **Lia** (UX主導の製品マネージャ) | 接続体験の策定 | アプリから 3 ステップ以内で接続フローを提供 | 複数プロトコル統合の複雑さ |
| **Noah** (組み込みエンジニア) | プロトコルを製品へ組み込む | 省電力とリアルタイム要件の両立 | 物理層ごとの実装差異 |
| **Mika** (セキュリティアナリスト) | 監査とコンプライアンス | 暗号キーマネジメントと証跡確保 | デバイスによるセキュリティ水準のばらつき |
| **Aria** (運用SRE) | 稼働監視と障害対応 | SLA維持、障害時ロールバック手順 | 可視化不足、プロトコル固有の指標欠如 |

## ユースケース
1. **ゲーミング入力 + 音声同時配信**
   - トリガ: ユーザーがゲームパッドとヘッドセットを同時接続
   - 成功条件: 入力遅延 P95 6ms以下、音声同期ズレ < 20ms
   - 参照: [docs/performance/scalability.md](./performance/scalability.md)
2. **IoT 群制御とセンサーデータ転送**
   - トリガ: 100台のセンサーを一括登録
   - 成功条件: 登録成功率 99.9%、1台あたり平均電流 5mA 以下
   - 参照: [docs/architecture/dataflow.md](./architecture/dataflow.md)
3. **高解像度メディア転送 (8K)**
   - トリガ: モバイルから TV へ映像ストリーミング
   - 成功条件: 平均スループット 1.5Gbps、フレームドロップ率 < 0.1%
   - 参照: [docs/performance/benchmark.md](./performance/benchmark.md)
4. **企業ネットワーク統合**
   - トリガ: 管理者が RBAC で権限を割当
   - 成功条件: 役割登録後 5 分以内にポリシー反映、全操作が監査ログに記録
   - 参照: [docs/security/auth.md](./security/auth.md)
5. **AR/VR マルチユーザセッション**
   - トリガ: 同一空間で 10 名が同期体験
   - 成功条件: 空間同期誤差 < 5cm、P99 遅延 12ms 以下
   - 参照: [docs/ui/animations.md](./ui/animations.md)

## 機能要件
| ID | 分類 | 概要 | 入力 | 処理 | 出力 | エラー/例外 |
|----|------|------|------|------|------|--------------|
| FR-01 | 接続 | ビーコン検出と一覧提示 | 近傍デバイス情報 | フィルタリング・表示ソート | UXコンポーネントでカード表示 | 重複検出失敗時: エラー表示と再スキャン案内 |
| FR-02 | ペアリング | OOBを含む多要素ペアリング | ペアリング要求、認証情報 | X25519鍵合意、プロファイル交渉 | セッション共有秘密 | 認証失敗時: リトライ回数制限と警告 |
| FR-03 | ストリーム管理 | 最大8ストリームの同時管理 | ストリーム作成要求 | QoS分類とリソース割当 | ストリームハンドル | リソース枯渇時: 優先度に応じた拒否 |
| FR-04 | QoS調整 | ネットワーク状態に応じた再設定 | RTT, ロス率, バッテリー残量 | プロファイル再選択、FEC率変更 | 更新ポリシー通知 | 計測不可時: デフォルトプロファイル継続 |
| FR-05 | セキュリティ監査 | 監査証跡生成と保存 | 制御プレーンイベント | 署名付きログ生成 | 連続監査ログストリーム | ストレージ異常時: アラート発報とバックアップ経路 |
| FR-06 | プロファイルテンプレ共有 | ベンダ固有設定のパッケージ化 | プロファイル定義入力 | バリデーションと署名 | エクスポートファイル(抽象) | 検証失敗時: 不備内容と改善ガイド |
| FR-07 | 観測性標準化 | OpenTelemetry 互換のメトリクス/トレース生成 | 収集対象イベント、属性 | SDKレス計装による整形・匿名化・キューイング | OTLP エンドポイント向けバンドル | 出力停止時: バッファ退避と運用アラート |

## 非機能要件
### 性能
- **遅延:** LL入力ストリーム P95 8ms 以下、RT音声 P95 15ms 以下、Bulk転送平均 1Gbps 以上。
- **ジッタ:** 低遅延ストリーム標準偏差 3ms 以下。
- **復旧:** ロス率 5% までなら FEC で 99.9% データ再構築。
- **測定:** [docs/performance/benchmark.md](./performance/benchmark.md)の計測計画を利用。

### 可用性・信頼性
- **SLO:** サービス可用性 99.95%、障害検知 30 秒以内、復旧 MTTR 15 分以内。
- **冗長:** 制御チャネル二重化、フェールオーバー設計は[docs/performance/scalability.md](./performance/scalability.md)参照。

### セキュリティ・プライバシー
- **暗号:** 鍵合意 X25519、対称暗号 ChaCha20-Poly1305、HKDF による鍵派生。
- **認証:** OIDC/OAuth2 互換、RBAC+ABAC ハイブリッド (詳細は[docs/security/auth.md](./security/auth.md))。
- **監査:** すべての重要操作は不可変ログへ記録、90日保持。

### 運用・可観測性
- **メトリクス:** セッション数、遅延分布、FEC効果、電力消費指数。
- **アラート:** KPI 逸脱時 5 分以内に通知。閾値は[docs/deployment/ci-cd.md](./deployment/ci-cd.md)で定義。
- **可観測性:** OpenTelemetry互換の抽象エクスポートを規定 (C/C++エージェント禁止)。

### 拡張性・保守性
- **モジュール化:** 各プロファイルは疎結合な仕様モジュールで定義。新規モジュール追加手順は[docs/templates/module-template.md](./templates/module-template.md)。
- **バージョニング:** SemVer準拠の仕様番号、互換性ポリシーは[docs/architecture/interfaces.md](./architecture/interfaces.md)に準拠。
- **ドキュメントテンプレ:** 仕様更新時は[docs/templates/test-template.md](./templates/test-template.md)等を使用して整合性チェックリストを更新。

### アクセシビリティ・国際化
- **基準:** すべての管理 UI は WCAG 2.2 AA 達成。詳細は[docs/ui/accessibility.md](./ui/accessibility.md)。
- **多言語:** コントロールプレーン出力と通知は少なくとも EN/JA/ES をサポート。翻訳フローは[docs/ui/overview.md](./ui/overview.md)に準拠。
- **身体的制約:** Reduced Motion 設定やハイコントラスト対応を必須化し、[docs/ui/animations.md](./ui/animations.md)の代替挙動を実装要件化。

### コンプライアンス
- **データ保護:** GDPR/CCPA に基づき、利用者削除リクエストは 30 日以内に完了。手順は[docs/deployment/rollback.md](./deployment/rollback.md)と連携して監査。
- **暗号輸出規制:** 地域別鍵長・アルゴリズム許容範囲をメタデータで管理し、[docs/security/encryption.md](./security/encryption.md)を参照。
- **監査対応:** SOC2/ISO27001 の監査証跡を保管し、評価結果を[docs/notes/decision-log.md](./notes/decision-log.md)へ登録。

## 環境要件
- **対象デバイス:** モバイル、PC、ゲーミング周辺機器、IoTゲートウェイ。
- **物理層:** Wi-Fi 6E/7、5G/6G、ミリ波、将来のTHz帯。各物理層の差異はアダプタ仕様で吸収。
- **管理ツール:** Webポータル (ブラウザベース) とモバイルアプリ。UI仕様は[docs/ui/wireframes.md](./ui/wireframes.md)参照。
- **地域規制:** 各国電波法に準拠し、地域別設定をメタデータで通知。

## 制約と禁止事項
- C/C++言語・ライブラリの使用禁止。暗号・FEC なども純粋言語実装またはマネージドサービスで提供。
- 実装コード、CLIコマンド、Dockerfileなどの実行可能記述は禁止。
- 標準化団体との合意がない暗号化方式の使用不可。
- デバイスにハードウェア改造を強要しないこと。

## スコープ外項目
- 物理層ハードウェア設計やアンテナ最適化。
- ファームウェアのチューニング具体手順。
- 外部サービスへの課金設計。
- 端末製造プロセス。

## 成功指標
- KPI 達成率 95%以上 (四半期評価)。
- ユーザビリティスコア (SUS) 85 点以上。
- セキュリティ監査指摘件数ゼロ。
- サポート問い合わせ削減率 40% (既存比較)。

## トレーサビリティ方針
1. 要件 (本書) にユニークIDを付与。
2. 設計文書は対応IDを本文・表に表示。
3. テスト計画 ([docs/testing/*](./testing)) は同じ ID をシナリオに記載。
4. デプロイ文書は変更管理 ID を索引に含める。
5. 変更履歴は[docs/notes/decision-log.md](./notes/decision-log.md)で記録。

## 用語集
| 用語 | 定義 | 参照 |
|------|------|------|
| セッション | ハンドシェイク済み通信単位 | [docs/architecture/dataflow.md](./architecture/dataflow.md) |
| プロファイル | ストリーム設定テンプレ | [docs/architecture/interfaces.md](./architecture/interfaces.md) |
| QoS グレード | 遅延/帯域/信頼性レベルの集合 | [docs/performance/scalability.md](./performance/scalability.md) |
| FEC | Forward Error Correction を仕様化した抽象機構 | [docs/performance/benchmark.md](./performance/benchmark.md) |
| RBAC | 役割ベースアクセス制御 | [docs/security/auth.md](./security/auth.md) |
| ABAC | 属性ベースアクセス制御 | 同上 |
| SLI/SLO | サービス品質指標・目標 | [docs/testing/metrics.md](./testing/metrics.md) |

## 受け入れ基準 (DoD)
- 全機能要件にユニーク ID が付与され、対応する設計・テスト文書へのリンクが存在する。
- 非機能要件が測定基準とセットで定義され、性能・セキュリティ・運用の各文書と整合している。
- 制約と禁止事項に C/C++ 回避方針、実装コード非出力方針が明記されている。
- ペルソナ・ユースケースが主要シナリオを網羅し、成功指標と矛盾しない。
- トレーサビリティ方針がすべてのワーキンググループで適用可能であるとレビュー済み。
