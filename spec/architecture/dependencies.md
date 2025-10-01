# docs/architecture/dependencies.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> 依存関係管理は HoneyLink™ の保守性と安全性を支えるコア要素です。本書は抽象的なモジュール依存を定義し、実装コードや C/C++ 連携を含みません。

## 目次
- [依存管理原則](#依存管理原則)
- [モジュール依存グラフ](#モジュール依存グラフ)
- [層別ルール](#層別ルール)
- [循環回避策](#循環回避策)
- [バージョン方針](#バージョン方針)
- [変更管理フロー](#変更管理フロー)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## 依存管理原則
1. **単方向:** 高層→低層のみ。逆依存は禁止。
2. **契約駆動:** インタフェース契約を[docs/architecture/interfaces.md](./interfaces.md)で定義し、直接参照せず契約経由で通信。
3. **C/C++排除:** ネイティブバイナリへのリンクは禁止。必要なら API 化したサービスを利用。
4. **プラグイン:** プロファイル/ポリシー等はプラグインスロットにホットスワップ可能。

## モジュール依存グラフ
```
      +-----------------------+
      | Experience Layer      |
      +----------+------------+
                 |
                 v
      +----------+------------+
      | Session Orchestrator  |
      +----+------------------+
           | \
           |  \
           v   v
  +--------+-----+     +-----------------------+
  | Policy Engine |--> | QoS Scheduler        |
  +--------+-----+     +-----------------------+
           |
           v
  +-------------------+
  | Crypto & Trust    |
  +--------+----------+
           |
           v
  +-------------------+
  | Transport Abstraction |
  +--------+----------+
           |
           v
  +-------------------+
  | Physical Adapter  |
  +-------------------+

Telemetry & Insights listens to events from Session Orchestrator, QoS Scheduler, and Transport Abstraction.
```

## 層別ルール
| 層 | 許容依存 | 禁止依存 |
|----|----------|----------|
| Experience | Session Orchestrator (API経由) | Policy/QoS/Cryptoへの直接依存 |
| Session | Policy, Crypto, Telemetry | Physical Adapter直接参照 |
| Policy | QoS, Telemetry | Experience, Physical |
| QoS | Transport, Telemetry | Experience |
| Transport | Physical Adapter, Telemetry | Experience, Policy |
| Telemetry | すべてからイベント購読 (一方向) | 双方向依存 |
| Physical Adapter | なし (外界のみ) | 上位層へ逆参照 |

## 循環回避策
- イベントバスを介した通知のみ許可し、同期呼び出しは一方向。
- 新機能追加時は[docs/templates/module-template.md](../templates/module-template.md)の依存チェックリストを使用。
- 依存違反検知時は[docs/notes/decision-log.md](../notes/decision-log.md)で ADR を更新。

## バージョン方針
- **SemVer:** モジュール仕様に `major.minor.patch` を付与。Major変更は互換性を破壊。
- **互換性保証期間:** Majorリリース後12ヶ月間は旧インタフェースを並行提供。
- **依存宣言:** 各モジュールはサポートする相手モジュールのバージョン範囲を宣言 (例: `PolicyEngine >=2.0 <3.0`)。
- **C/C++禁止適用:** 依存宣言にネイティブモジュールを含めることを禁止。代替は API サービスで提供。

## 変更管理フロー
1. 変更提案 (RFC) を作成し、影響モジュールとバージョン範囲を明記。
2. 依存グラフ影響を[docs/architecture/overview.md](./overview.md)と照合。
3. テスト計画を[docs/testing/integration-tests.md](../testing/integration-tests.md)に追加。
4. ロードマップ ([docs/roadmap.md](../roadmap.md)) に反映し、フェーズを調整。
5. ADR 更新 ([docs/notes/decision-log.md](../notes/decision-log.md))。

## 受け入れ基準 (DoD)
- 依存グラフがすべてのモジュールを包含し、双方向依存がない。
- 層別ルールに具体的禁止事項 (特に C/C++ 關連) を記述。
- バージョン方針がインタフェース文書と整合し、互換性期間を定義。
- 変更管理フローが他文書のプロセスとリンクしている。
- Telemetry の特例など例外が明記されている。
