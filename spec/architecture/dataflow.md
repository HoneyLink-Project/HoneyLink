# docs/architecture/dataflow.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> 本文は HoneyLink™ のデータフローと処理フローを抽象的に定義します。実装コード・CLI コマンド・C/C++依存プロトコルスタックは一切含みません。

## 目次
- [全体フロー概要](#全体フロー概要)
- [シーケンス: 初期ペアリング](#シーケンス-初期ペアリング)
- [シーケンス: マルチストリーム制御](#シーケンス-マルチストリーム制御)
- [シーケンス: IoT省電力モード](#シーケンス-iot省電力モード)
- [データ整合性と冪等性](#データ整合性と冪等性)
- [トランザクションとリトライ戦略](#トランザクションとリトライ戦略)
- [監視ポイントとテレメトリ](#監視ポイントとテレメトリ)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## 全体フロー概要
```
[Beacon Broadcast]
        |
        v
[Session Orchestrator] ---(Handshake Events)---> [Crypto & Trust]
        |
        v
[Policy Engine] ---(Profile Decision)---> [QoS Scheduler]
        |
        +---> [Telemetry & Insights]
        |
        v
[Transport Abstraction] ---(Packets)---> [Physical Adapter]
```

## シーケンス: 初期ペアリング
1. **Beacon検出:** デバイスは 100ms 間隔で Beacon メッセージ (公開鍵ハッシュ + Capability) を発信。
2. **接続要求:** クライアントが Session Orchestrator へ接続要求イベントを送信。
3. **鍵交換:** Crypto & Trust Anchor が X25519 による共有秘密を生成。
4. **プロファイル交渉:** Policy Engine がクライアント要求と環境条件からプロファイル候補を算出。
5. **確定通知:** Session Orchestrator が確定プロファイルとセッションIDを通知。
6. **テレメトリ登録:** Telemetry & Insights がセッション初期メトリクスを生成。

```
Client        Orchestrator     Crypto       Policy       Telemetry
  |                |             |             |             |
  |---Beacon------>|             |             |             |
  |---Connect Req--------------->|             |             |
  |                |---Key Req-->|             |             |
  |                |<--Key Ack---|             |             |
  |                |---Profile Req----------->|             |
  |                |<--Profile Resp-----------|             |
  |<--Session Ack--|             |             |---Init----->|
```

## シーケンス: マルチストリーム制御
1. Stream QoS Scheduler がメインストリームを優先度順にキューイング。
2. Transport Abstraction がストリームごとに FEC 係数とバッチサイズを決定。
3. ネットワーク状態 (RTT/ロス率) を Telemetry & Insights から Pull。
4. ポリシー変更が必要な場合、Policy Engine へイベント通知。
5. Session Orchestrator が更新情報を各クライアントに配信。

## シーケンス: IoT省電力モード
1. IoTデバイスが「省電力ハートビート」を 10 分間隔で送信。
2. Policy Engine が電力予算を再評価し、送信頻度・バッチ転送時間帯を調整。
3. Telemetry & Insights が省電力効果を指標化し、SLI (平均電流) を更新。
4. 必要に応じて[docs/performance/scalability.md](../performance/scalability.md)で定義されたバッチモードへ切替。

## データ整合性と冪等性
- **セッション管理:** セッションIDは UUIDv7 等の時間順序を持つ形式。再送時には idempotency-key を付与し、重複処理を防止。
- **ストリーム制御:** QoS 更新はバージョン番号付き差分イベント。最新バージョンのみ適用。
- **テレメトリ:** 時系列データは append-only。集約処理は idempotent aggregator により実行。

## トランザクションとリトライ戦略
| 操作 | トランザクション境界 | リトライ戦略 | バックプレッシャ |
|------|----------------------|--------------|------------------|
| ハンドシェイク | Session Orchestrator + Crypto | 指数バックオフ (最大 3 回) | 再試行間隔中は Beacon 監視継続 |
| プロファイル更新 | Policy Engine | 逐次再評価、最大 2 回 | QoS Scheduler が旧設定を維持 |
| Telemetry Export | Telemetry & Insights | 恒久キュー、デッドレタ | 取得遅延時はサマリ粒度を粗く |

## 監視ポイントとテレメトリ
- **Handshake Latency:** 主要 SLI。閾値超過時は[docs/deployment/rollback.md](../deployment/rollback.md)を参照。
- **Loss Recovery Rate:** FEC 効率の測定。データは[docs/performance/benchmark.md](../performance/benchmark.md)で分析。
- **Power Budget Utilization:** IoT シナリオの主要指標。結果を[docs/testing/metrics.md](../testing/metrics.md)でダッシュボード化。
- **Security Event Stream:** 異常検知を[docs/security/vulnerability.md](../security/vulnerability.md)のマトリクスへ連携。

## 受け入れ基準 (DoD)
- 3 つ以上の代表的シーケンス図を提示し、重要な分岐・例外を説明している。
- 冪等性とリトライ戦略が機能要件 FR-02, FR-03 と整合。
- 監視ポイントが測定可能な SLI と紐付いている。
- C/C++依存を伴う処理が存在せず、必要に応じ代替手段が明記されている。
- 他のアーキテクチャ文書 (overview, interfaces, dependencies) への参照が正確である。
