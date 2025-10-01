# Phase Entry/Exit Criteria Checklist

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> HoneyLink™ ロードマップの各フェーズ (P0–P4) におけるエントリ基準・エグジット基準を確認リスト形式で文書化し、承認プロセスを明確化します。

---

## 目次
1. [概要](#1-概要)
2. [承認プロセス](#2-承認プロセス)
3. [P0: コンセプト確定](#3-p0-コンセプト確定)
4. [P1: 仕様詳細化](#4-p1-仕様詳細化)
5. [P2: プロトタイプ設計](#5-p2-プロトタイプ設計)
6. [P3: 標準化準備](#6-p3-標準化準備)
7. [P4: 継続改善](#7-p4-継続改善)
8. [変更履歴](#8-変更履歴)

---

## 1. 概要

### 1.1 目的
- 各フェーズの開始条件と完了条件を客観的に評価可能にする
- フェーズ間の移行判断を透明化し、ステークホルダー間の合意形成を促進する
- リスクの早期発見と緩和策の実行を確実にする

### 1.2 使用方法
1. フェーズ開始前に **エントリ基準チェックリスト** を実施
2. すべての項目が✅であることを確認後、Governance Council が承認
3. フェーズ終了時に **エグジット基準チェックリスト** を実施
4. 未達項目がある場合はアクションプランを策定し、達成後に次フェーズへ移行

### 1.3 関連ドキュメント
- [spec/roadmap.md](../roadmap.md) - フェーズ概要とタイムライン
- [spec/requirements.md](../requirements.md) - 機能要件・非機能要件
- [spec/notes/decision-log.md](decision-log.md) - 決定事項の記録先
- [spec/testing/metrics.md](../testing/metrics.md) - 測定指標(SLI/SLO)

---

## 2. 承認プロセス

### 2.1 エントリ基準の承認フロー

```
1. Architecture WG Chair が基準チェックリストを作成
   ↓
2. 各関連WG (Protocol/Security/UX/Operations) がレビュー (72時間以内)
   ↓
3. 未達項目がある場合
   ├─ クリティカル項目 → ブロッカーとして即座にエスカレーション
   └─ 非クリティカル項目 → 改善計画書を添付し、条件付き承認
   ↓
4. Governance Council が最終承認 (会議またはMatrix投票)
   ↓
5. 承認結果を `spec/notes/decision-log.md` へ記録 (ADR形式)
```

### 2.2 エグジット基準の承認フロー

```
1. フェーズ終了1週間前に Operations WG が自己評価を実施
   ↓
2. 各WGが成果物をレビュー (5営業日以内)
   ├─ DoD満足度100%を確認
   ├─ トレーサビリティリンクの整合性チェック
   └─ リスク解消率の検証
   ↓
3. 未達項目がある場合
   ├─ P0/P1級 → フェーズ延長 (最大2週間) + 改善計画書
   └─ P2/P3級 → 次フェーズで並行対応 (技術的負債として記録)
   ↓
4. Governance Council が承認 + 次フェーズキックオフ日を決定
   ↓
5. 承認結果と成果物リストを `spec/roadmap.md` のタイムラインセクションへ反映
```

### 2.3 承認者の役割分担

| 役割 | 責任 | 承認権限 |
|------|------|---------|
| Architecture WG Chair | チェックリスト作成・更新、技術的実現可能性評価 | エントリ基準の技術要素に対する拒否権 |
| Security WG Chair | セキュリティ要件・脅威モデルの妥当性確認 | セキュリティ関連項目に対する拒否権 |
| Operations WG Chair | 運用実現性・計測可能性の検証 | SLI/SLO未定義項目に対する拒否権 |
| UX WG Lead | ユーザビリティ・アクセシビリティ要件の確認 | UX関連DoD未達に対する拒否権 |
| Governance Council (3 Chairs) | 最終承認・フェーズ移行判断 | フェーズ移行の承認/延期決定 |

---

## 3. P0: コンセプト確定

### 3.1 エントリ基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| E0-01 | 主要ペルソナ (Lia/Noah/Mika/Aria) が承認済み | `spec/requirements.md` のペルソナセクションに記載完了 | Architecture WG | ⬜ |
| E0-02 | 5大ユースケース (ゲーミング/IoT/8K/企業/AR/VR) が文書化済み | `spec/requirements.md` に5件以上のユースケース記載 | Protocol WG | ⬜ |
| E0-03 | ステークホルダー (WG Chairs) の役割が定義済み | `spec/notes/governance.md` に5WGのChairとメンバーリスト記載 | Operations WG | ⬜ |
| E0-04 | エレベーターピッチが30秒以内で説明可能 | `spec/README.md` にピッチ文を記載し、社内レビュー済み | UX WG | ⬜ |
| E0-05 | 初期リスク仮説が10件以上リストアップ済み | `spec/roadmap.md` のリスクセクションに記載 | Security WG | ⬜ |

**承認条件**: すべての項目が✅ + Governance Council の承認決議

---

### 3.2 エグジット基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| X0-01 | エレベーターピッチとKPIに合意済み | `spec/README.md` にKPI数値目標を記載 | Architecture WG | ⬜ |
| X0-02 | リスク仮説が整理され、Medium以上に緩和策が定義済み | `spec/roadmap.md` のリスクテーブルに緩和策カラム記入完了 | Security WG | ⬜ |
| X0-03 | ペルソナごとの痛点が3件以上文書化済み | `spec/requirements.md` のペルソナテーブルに「痛点」カラム記入 | UX WG | ⬜ |
| X0-04 | 要件抽出が完了し、FR/NFRの初版がレビュー済み | `spec/requirements.md` に10件以上のFR、5カテゴリ以上のNFRを記載 | Protocol WG | ⬜ |
| X0-05 | P1フェーズの成果物リスト (30文書) が策定済み | `spec/roadmap.md` のタイムラインに30文書リストを追記 | Operations WG | ⬜ |
| X0-06 | 「実装コード非出力」「C/C++依存禁止」方針に全WG合意 | 各WG Chairが `spec/notes/decision-log.md` に承認署名 | Governance Council | ⬜ |

**承認条件**: すべての項目が✅ + 次フェーズキックオフ日の決定

---

## 4. P1: 仕様詳細化

### 4.1 エントリ基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| E1-01 | P0フェーズのエグジット基準がすべて✅ | P0エグジットチェックリスト参照 | Governance Council | ⬜ |
| E1-02 | WG構成が完了し、週次レビューcadenceが設定済み | `spec/notes/governance.md` に全5WGのスケジュール記載 | Operations WG | ⬜ |
| E1-03 | 30文書の執筆担当者が割り当て済み | 各文書の先頭にOwner (Role ID) を記載 | Architecture WG | ⬜ |
| E1-04 | ドキュメントテンプレート (module/test/ui) が準備済み | `spec/templates/` に3種類のテンプレート配置 | Operations WG | ⬜ |
| E1-05 | トレーサビリティIDのナンバリングルールが定義済み | `spec/requirements.md` のトレーサビリティ方針セクションに記載 | Architecture WG | ⬜ |

**承認条件**: すべての項目が✅ + P1キックオフミーティング完了

---

### 4.2 エグジット基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| X1-01 | 30文書が策定完了し、すべてDoD満足度100% | 各文書末尾の「受け入れ基準(DoD)」チェックリストが全✅ | Operations WG | ⬜ |
| X1-02 | プロトコル仕様 (制御プレーンAPI) が詳細化済み | `spec/api/control-plane.md` に8エンドポイント仕様記載 | Protocol WG | ⬜ |
| X1-03 | UI仕様 (ワイヤーフレーム/デザイントークン) が完成 | `spec/ui/wireframes.md` + `spec/ui/visual-design.md` が承認済み | UX WG | ⬜ |
| X1-04 | セキュリティ仕様 (暗号/鍵管理/脅威モデル) が承認済み | `spec/security/` 配下の4文書がレビュー完了 | Security WG | ⬜ |
| X1-05 | トレーサビリティマトリクスが確立済み | FR-XX → 設計文書 → テスト仕様のリンクが全件存在 | Architecture WG | ⬜ |
| X1-06 | 依存関係マップが更新済み | `spec/architecture/dependencies.md` にモジュール依存図を記載 | Architecture WG | ⬜ |
| X1-07 | レビュー完了率90%以上を達成 | `spec/notes/governance.md` の月次レポートで確認 | Operations WG | ⬜ |
| X1-08 | 「C/C++依存禁止」が全文書で守られていることを査読済み | Security WGが全文書をスキャン、違反ゼロ確認 | Security WG | ⬜ |

**承認条件**: すべての項目が✅ + P2フェーズのPoC資源確保完了

---

## 5. P2: プロトタイプ設計

### 5.1 エントリ基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| E2-01 | P1フェーズのエグジット基準がすべて✅ | P1エグジットチェックリスト参照 | Governance Council | ⬜ |
| E2-02 | PoCに必要なリソース (人員/予算/環境) が確保済み | Operations WG が予算承認書と環境構築計画を提出 | Operations WG | ⬜ |
| E2-03 | PoC目標 (KPI数値) が定義済み | `spec/performance/benchmark.md` にベンチマーク目標値を記載 | Protocol WG | ⬜ |
| E2-04 | テスト設計テンプレート (unit/integration/e2e) が準備済み | `spec/templates/test-template.md` が完成 | Operations WG | ⬜ |
| E2-05 | セキュリティ監査の外部パートナーが選定済み | Security WG が監査ベンダー契約を締結 | Security WG | ⬜ |

**承認条件**: すべての項目が✅ + P2キックオフ&PoCデザインレビュー完了

---

### 5.2 エグジット基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| X2-01 | PoC設計書が完成 (実装記述なし、抽象設計のみ) | `spec/architecture/overview.md` にPoC設計セクション追加 | Architecture WG | ⬜ |
| X2-02 | 試験計画 (ベンチマーク/スケーラビリティ) が確定 | `spec/performance/benchmark.md` + `scalability.md` が承認済み | Protocol WG | ⬜ |
| X2-03 | テストテンプレート (unit/integration/e2e) が3種類完成 | `spec/testing/` 配下に3文書が配置済み | Operations WG | ⬜ |
| X2-04 | 脅威モデル (STRIDE) が更新済み | `spec/security/vulnerability.md` にSTRIDE分析結果を記載 | Security WG | ⬜ |
| X2-05 | UI実装指針 (アクセシビリティ/i18n) が文書化済み | `spec/ui/accessibility.md` が承認済み | UX WG | ⬜ |
| X2-06 | ロールバック戦略が定義済み | `spec/deployment/rollback.md` にロールバック手順を記載 | Operations WG | ⬜ |
| X2-07 | 外部パートナー (監査ベンダー/テストラボ) が合意済み | Security WG が契約書締結完了 | Security WG | ⬜ |

**承認条件**: すべての項目が✅ + 外部評価の準備完了

---

## 6. P3: 標準化準備

### 6.1 エントリ基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| E3-01 | P2フェーズのエグジット基準がすべて✅ | P2エグジットチェックリスト参照 | Governance Council | ⬜ |
| E3-02 | 外部パートナー (認証機関/技術評価者) との合意済み | Security WG が評価契約を締結 | Security WG | ⬜ |
| E3-03 | ガバナンス文書 (Charter/Decision Log) が運用中 | `spec/notes/governance.md` + `decision-log.md` が最新 | Operations WG | ⬜ |
| E3-04 | CI/CD方針が策定済み | `spec/deployment/ci-cd.md` が承認済み | Operations WG | ⬜ |
| E3-05 | インフラ構成図 (抽象) が完成 | `spec/deployment/infrastructure.md` に記載 | Architecture WG | ⬜ |

**承認条件**: すべての項目が✅ + 外部評価キックオフミーティング完了

---

### 6.2 エグジット基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| X3-01 | ガバナンス文書が承認済み | `spec/notes/governance.md` が Governance Council 承認 | Operations WG | ⬜ |
| X3-02 | CI/CD + ローリングリリース方針が文書化済み | `spec/deployment/ci-cd.md` にリリースプロセス記載 | Operations WG | ⬜ |
| X3-03 | 外部監査レポートが提出済み (指摘事項対応中) | Security WG が監査結果を `spec/security/vulnerability.md` に反映 | Security WG | ⬜ |
| X3-04 | KPI測定ダッシュボードが稼働開始 | Operations WG が Grafana/Sheets でダッシュボード公開 | Operations WG | ⬜ |
| X3-05 | リスク解消率100% (Medium以上) を達成 | `spec/roadmap.md` のリスクテーブルで確認 | Security WG | ⬜ |
| X3-06 | ドキュメント相互参照の整合性が検証済み | Architecture WG が全リンク自動チェック実施 | Architecture WG | ⬜ |
| X3-07 | 標準化提案書 (オプション) が準備完了 | Protocol WG が標準化団体への提出パッケージを作成 | Protocol WG | ⬜ |

**承認条件**: すべての項目が✅ + P4継続改善フェーズへの移行承認

---

## 7. P4: 継続改善

### 7.1 エントリ基準チェックリスト

| # | 項目 | 検証方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| E4-01 | P3フェーズのエグジット基準がすべて✅ | P3エグジットチェックリスト参照 | Governance Council | ⬜ |
| E4-02 | KPIダッシュボードが稼働中で四半期レビュー日程確定 | Operations WG が次4四半期のレビュー日をカレンダー登録 | Operations WG | ⬜ |
| E4-03 | 仕様改訂プロセス (SemVer) が文書化済み | `spec/architecture/interfaces.md` にバージョニング規則記載 | Architecture WG | ⬜ |
| E4-04 | フィードバック収集チャネルが設置済み | UX WG がユーザーフィードバックフォームを公開 | UX WG | ⬜ |

**承認条件**: すべての項目が✅ + P4フェーズ開始宣言

---

### 7.2 継続的評価基準 (四半期ごと)

| # | 項目 | 評価方法 | 責任者 | ステータス |
|---|------|---------|--------|-----------|
| C4-01 | KPI達成状況が四半期レビューで報告済み | Operations WG が四半期レポートを作成 | Operations WG | ⬜ |
| C4-02 | 改訂プロセスが稼働中 (仕様変更が30日以内に文書反映) | Architecture WG が改訂ログを `spec/notes/decision-log.md` に記録 | Architecture WG | ⬜ |
| C4-03 | ユーザーフィードバックが月次で集計され、改善案に反映 | UX WG がフィードバックサマリーを公開 | UX WG | ⬜ |
| C4-04 | セキュリティ脆弱性が報告された場合、48時間以内に対応開始 | Security WG がインシデントレスポンスログを記録 | Security WG | ⬜ |
| C4-05 | 撤退戦略の評価 (KPI未達2期連続の場合) | Governance Council が代替方式検討を開始 | Governance Council | ⬜ |

**評価サイクル**: 四半期末 (Q1/Q2/Q3/Q4の最終週) に実施

---

## 8. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 (P0-P4の全チェックリスト定義) | Governance Council |

---

**テンプレート使用例**:
```markdown
# P1エントリ基準評価 (2025-10-15実施)

| 項目 | ステータス | 備考 |
|------|-----------|------|
| E1-01 | ✅ | P0完了確認済み |
| E1-02 | ✅ | governance.mdに記載済み |
| E1-03 | ⚠️ | 3文書の担当者未定 → 2025-10-17までにアサイン |
| E1-04 | ✅ | templates/配下に配置済み |
| E1-05 | ✅ | requirements.mdに記載済み |

**総合判定**: 条件付き承認 (E1-03を2025-10-17までに解消)
**承認者**: Governance Council (Architecture + Security + Operations Chairs)
**記録先**: `spec/notes/decision-log.md` (ADR-042)
```

---

**関連ドキュメント**:
- [spec/roadmap.md](../roadmap.md) - フェーズ概要とマイルストーン
- [spec/notes/governance.md](governance.md) - WG構成と承認権限
- [spec/notes/decision-log.md](decision-log.md) - 承認結果の記録先
- [spec/requirements.md](../requirements.md) - トレーサビリティの起点
- [spec/testing/metrics.md](../testing/metrics.md) - KPI/SLI測定方法

