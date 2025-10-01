# docs/architecture/overview.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> 本ドキュメントは抽象アーキテクチャのみを対象とし、実装言語・コード片・C/C++依存要素を含みません。詳細なデータフローやインタフェース仕様は関連文書を参照してください。

## 目次
- [アーキテクチャ原則](#アーキテクチャ原則)
- [コンポーネント図](#コンポーネント図)
- [コンポーネント責務](#コンポーネント責務)
- [アーキテクチャパターン](#アーキテクチャパターン)
- [境界と依存管理](#境界と依存管理)
- [可観測性とメトリクス](#可観測性とメトリクス)
- [変更容易性評価](#変更容易性評価)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## アーキテクチャ原則
1. **レイヤード + ヘキサゴナルハイブリッド:** コアドメインを境界アダプタから切り離し、外部システム変更の影響を最小化。
2. **イベント駆動補完:** QoS や状態変化はイベントで伝達し、ストリームを非同期協調させる。
3. **ゼロトラスト:** すべてのコンポーネントが相互認証し、最小権限を適用。
4. **観測可能性内蔵:** SLIs をファーストクラスオブジェクトとして扱い、[docs/testing/metrics.md](../testing/metrics.md)と同期。
5. **C/C++非依存:** 暗号・FEC等は純粋言語実装またはクラウドサービスで提供し、抽象アダプタを設計。

## コンポーネント図
```
+-------------------------------------------------------------+
|                      HoneyLink Core                          |
|                                                             |
|  +----------------------+   +-----------------------------+ |
|  |  Session Orchestrator|<->|  Policy & Profile Engine    | |
|  +----------+-----------+   +--------------+--------------+ |
|             ^                              ^                |
|             |                              |                |
|  +----------+-----------+      +-----------+------------+   |
|  | Transport Abstraction |<----| Telemetry & Insights   |   |
|  +----------+-----------+      +-----------+------------+   |
|             |                              ^                |
|             v                              |                |
|  +----------+-----------+      +-----------+------------+   |
|  | Crypto & Trust Anchor |      | Stream QoS Scheduler  |   |
|  +-----------------------+      +-----------------------+   |
+-------------------------------------------------------------+
            |                          |
            v                          v
+-----------+-----------+     +--------+---------+
| Physical Adapter Layer |     | Experience Layer |
| (Wi-Fi/5G/THz bridges) |     | (SDK, UI Shell)  |
+------------------------+     +------------------+
```

詳細なデータ経路は[docs/architecture/dataflow.md](./dataflow.md)を参照。

## コンポーネント責務
| コンポーネント | 主要責務 | 入出力 | 参照 |
|----------------|----------|--------|------|
| Session Orchestrator | ハンドシェイク、セッション状態管理、バージョン交渉 | 接続要求、ポリシー適用指示 | [docs/security/auth.md](../security/auth.md) |
| Policy & Profile Engine | ユースケース別ポリシー決定、プロファイル管理 | QoS要件、環境メタデータ | [docs/performance/scalability.md](../performance/scalability.md) |
| Transport Abstraction | 論理ストリームと物理層の橋渡し、FECアダプタ | パケット、ステータスメトリクス | [docs/architecture/interfaces.md](./interfaces.md) |
| Crypto & Trust Anchor | 鍵管理、証明書連携、秘密管理 | ハンドシェイク材料、鍵ストア | [docs/security/encryption.md](../security/encryption.md) |
| Stream QoS Scheduler | 優先度制御、帯域割当、バックプレッシャ | ストリーム要求、ネットワーク状態 | [docs/performance/benchmark.md](../performance/benchmark.md) |
| Telemetry & Insights | SLI採取、イベント分析、アラート連携 | 観測データ、レポート | [docs/testing/metrics.md](../testing/metrics.md) |
| Physical Adapter Layer | 物理層依存差分吸収、ドライバ抽象化 | 電波設定、ハードイベント | C/C++禁止のため純粋抽象API |
| Experience Layer | SDK APIと UI Shell 仕様、デバイスガイダンス | UXパターン、ユーザ入力 | [docs/ui/overview.md](../ui/overview.md) |

## アーキテクチャパターン
- **セッション制御:** マイクロカーネルとして最小コアを保持し、プロファイル/ポリシーはプラグインで拡張。
- **イベント橋:** 主要コンポーネント間はイベントバス (例: メモリ内キュー／メッセージ抽象) で通信。
- **CQRS的分離:** 構成変更 (Command) と状態参照 (Query) を分離して可観測性を高める。
- **ストラテジパターン:** プロファイルごとに QoS や暗号化モードのストラテジを差し替え。

## 境界と依存管理
- 層間の依存は上位→下位のみ。逆方向依存は禁止。
- External 連携 (例: IDプロバイダ) はポート/アダプタ模式で記述。
- 依存ルールの詳細は[docs/architecture/dependencies.md](./dependencies.md)。
- 物理層アダプタは C/C++ 実装が一般的だが、本仕様では gRPC・REST 等言語非依存 API を設計し、純粋実装または外部サービスで代替。

## 可観測性とメトリクス
- 各コンポーネントは SLIs を出力し、[docs/testing/metrics.md](../testing/metrics.md)に合致させる。
- Telemetry & Insights 層が指標を集約し、SLO逸脱時に[docs/deployment/rollback.md](../deployment/rollback.md)の手順を起動。

## 変更容易性評価
| 領域 | 変更例 | 影響範囲 | 緩和策 |
|------|--------|----------|--------|
| プロファイル追加 | 新規プロファイルテンプレ | Policy Engine、QoS Scheduler | [docs/templates/module-template.md](../templates/module-template.md)でガイド |
| 暗号方式更新 | 新しい鍵交換 | Crypto & Trust Anchor中心 | 抽象化によりその他層は鍵IDのみ更新 |
| 物理層追加 | 新規アダプタ | Physical Adapter、Transport Abstraction | API互換性を[docs/architecture/interfaces.md](./interfaces.md)で保証 |

## 受け入れ基準 (DoD)
- すべてのコンポーネントが責務・入出力・参照先を明示し、他文書とリンク済み。
- 図と表が C/C++ 非依存アプローチを反映している。
- アーキテクチャ原則が要件・性能・セキュリティ文書と矛盾しない。
- コンポーネント図が最新依存ルールと一致し、レビュー記録が[docs/notes/decision-log.md](../notes/decision-log.md)に存在。
- 観測指標が[docs/testing/metrics.md](../testing/metrics.md)と双方向参照している。
