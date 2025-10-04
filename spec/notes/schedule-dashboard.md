# Schedule Adherence Dashboard Setup Guide

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> スケジュール遵守率95%を測定するダッシュボードの設計仕様と設定手順を定義します。

---

## 目次
1. [概要](#1-概要)
2. [測定指標定義](#2-測定指標定義)
3. [ダッシュボード設計](#3-ダッシュボード設計)
4. [実装オプション](#4-実装オプション)
5. [データ収集フロー](#5-データ収集フロー)
6. [アラート設定](#6-アラート設定)
7. [運用手順](#7-運用手順)
8. [変更履歴](#8-変更履歴)

---

## 1. 概要

### 1.1 目的
- `spec/roadmap.md` で定義されたスケジュール遵守率95%を客観的に測定
- フェーズごとのマイルストーン達成状況をリアルタイム可視化
- 遅延の早期発見と是正アクションの促進

### 1.2 スコープ
- **測定対象**: P0〜P4の全フェーズマイルストーン、`TODO.md` の主要タスク
- **更新頻度**: 日次自動更新 (タスク完了時) + 週次手動レビュー
- **アクセス権限**: 全WG Chairs + Operations WG メンバー (編集権限)、その他ステークホルダー (閲覧権限)

### 1.3 関連ドキュメント
- [spec/roadmap.md](../roadmap.md) - 測定指標 (SLI/SLO)
- [spec/testing/metrics.md](../testing/metrics.md) - KPI測定方法
- [spec/notes/critical-path-analysis.md](critical-path-analysis.md) - クリティカルパス分析
- [TODO.md](../../TODO.md) - 実装タスクリスト

---

## 2. 測定指標定義

### 2.1 スケジュール遵守率 (Schedule Adherence Rate)

**定義**:
```
スケジュール遵守率 = (計画通り完了したマイルストーン数 / 全マイルストーン数) × 100
```

**計画通り完了の判定基準**:
- マイルストーンの予定完了日 ≥ 実際の完了日
- **許容遅延**: 2営業日以内の遅延は「計画通り」とみなす（緩衝措置）

**測定単位**:
- **フェーズ単位**: P0〜P4の各フェーズごとに集計
- **月次単位**: 各月ごとに集計（四半期レビュー用）
- **タスクカテゴリ単位**: 環境整備/アーキテクチャ/API/UI/テストなど

### 2.2 補助指標

| 指標名 | 計算式 | 目標値 | 用途 |
|--------|--------|--------|------|
| **平均遅延日数** | Σ(実完了日 - 予定完了日) / 全タスク数 | ≤ 1.5日 | 遅延の深刻度評価 |
| **クリティカルパス遵守率** | クリティカルパス上の遵守タスク数 / CP上の全タスク数 × 100 | ≥ 98% | ボトルネック管理 |
| **バッファ消費率** | 累積遅延日数 / 設定バッファ日数 × 100 | ≤ 70% | リスク早期警告 |
| **タスク完了速度** | 直近7日間の完了タスク数 / 7 | (計画ベロシティ±10%) | 進捗ペース監視 |

### 2.3 データソース

| データ項目 | 取得元 | 更新頻度 | 形式 |
|-----------|--------|---------|------|
| タスクID、タスク名 | `TODO.md` | 手動更新 | Markdown チェックボックス |
| 予定完了日 | `spec/notes/critical-path-analysis.md` の CSV | 手動更新 | YYYY-MM-DD |
| 実完了日 | GitHub Issues/Projects | 自動更新 | YYYY-MM-DD |
| 担当者 | `TODO.md` または GitHub Assignee | 手動/自動 | Role ID |
| フェーズ | タスクに付与されたラベル (P0/P1/P2/P3/P4) | 手動更新 | ラベル |

---

## 3. ダッシュボード設計

### 3.1 レイアウト構成

ダッシュボードは以下の4セクションで構成します。

```
+--------------------------------------------------------+
| 🎯 全体サマリー (KPI Cards)                              |
| - スケジュール遵守率: [95.2%] ✅                          |
| - 平均遅延日数: [1.2日] ✅                                |
| - バッファ消費率: [45%] ⚠️                                |
| - 今月完了タスク: [12/15] ⚠️                              |
+--------------------------------------------------------+
| 📊 フェーズ別遵守率 (Bar Chart)                           |
| P0: ████████████████████ 100%                          |
| P1: ███████████████      75% ⚠️                         |
| P2: ██████               30% (進行中)                   |
| P3: ░░░░░░░░░░░░░░░░     0% (未開始)                    |
| P4: ░░░░░░░░░░░░░░░░     0% (未開始)                    |
+--------------------------------------------------------+
| 📈 遅延トレンド (Time Series Chart)                      |
| (月次の平均遅延日数を折れ線グラフで表示)                   |
| 8月: 0.5日 → 9月: 1.8日 → 10月: 1.2日                   |
+--------------------------------------------------------+
| ⚠️ 遅延タスク一覧 (Table)                                |
| TaskID | タスク名 | 予定 | 実績 | 遅延 | 担当 | アクション |
| T2.1   | Session  | 10/15| 10/18| 3日  | ENG-02| レビュー中 |
| ...    | ...      | ...  | ...  | ...  | ...   | ...       |
+--------------------------------------------------------+
```

### 3.2 視覚デザイン

- **色分け基準**:
  - 🟢 緑: 遵守率 ≥ 95% (目標達成)
  - 🟡 黄: 遵守率 90〜94% (注意)
  - 🔴 赤: 遵守率 < 90% (アクション必要)
  
- **アイコン**:
  - ✅ 目標達成
  - ⚠️ 注意が必要
  - 🚨 緊急対応必要

- **フォント**: Roboto (数値), Noto Sans JP (日本語)
- **アクセシビリティ**: WCAG 2.1 AA準拠 (色覚多様性対応、コントラスト比4.5:1以上)

---

## 4. 実装オプション

### 4.1 オプションA: Google Sheets (推奨: 初期導入時)

**メリット**:
- 設定が簡単、コスト不要
- 全ステークホルダーがアクセス可能
- Google Apps Script で自動化可能

**デメリット**:
- リアルタイム性が低い (手動更新が必要な場合あり)
- スケーラビリティに限界

**設定手順**:
1. Google Sheets テンプレートを作成 (後述 Section 4.4)
2. `spec/notes/critical-path.csv` をインポート
3. 数式で遵守率を自動計算
4. 条件付き書式で色分け設定
5. Google Data Studio (Looker Studio) でビジュアライゼーション作成

### 4.2 オプションB: Grafana + TimescaleDB (推奨: 本格運用時)

**メリット**:
- リアルタイム更新
- 高度な可視化 (ヒストグラム、ヒートマップ等)
- アラート機能が強力

**デメリット**:
- インフラ構築コスト (月額$50〜)
- 学習コスト

**設定手順 (P2Pローカル設計)**:
1. ローカルSQLiteデータベース作成 (`~/.honeylink/metrics/metrics.db`, 500MB max)
2. タスクデータを日次でSQLiteへ保存 (ローカルスクリプト)
3. Grafanaをローカルインストールし、SQLiteデータソースを追加 (開発環境のみ)
4. ダッシュボード JSON テンプレートをインポート (後述 Section 4.5)
5. ローカルOS通知を設定 (Windows Toast/macOS Notification Center/Linux libnotify)

**注意**: P2P設計ではクラウドインフラ不要 (月額$0, プライバシー保護)

### 4.3 オプションC: GitHub Projects (カンバンボード連携)

**メリット**:
- タスク管理と統合可能
- GitHub Actions でデータ自動収集

**デメリット**:
- 集計機能が弱い (外部ツールと連携必要)

**設定手順**:
1. GitHub Projects でカンバンボード作成
2. タスクに `phase:P1`, `milestone:2025-Q4` などのラベルを付与
3. GitHub Actions でタスク完了イベントを検知し、Sheets/DBへ送信
4. Sheets/Grafana でダッシュボード表示

### 4.4 Google Sheets テンプレート設計

#### シート構成

| シート名 | 用途 | 主要カラム |
|---------|------|-----------|
| **RawData** | タスク一覧と実績 | TaskID, TaskName, PlannedDate, ActualDate, Phase, Owner |
| **Metrics** | 集計結果 | Phase, ScheduleAdherence(%), AvgDelayDays, BufferUsage(%) |
| **Dashboard** | 可視化用 | KPI Cards, Charts |

#### 主要数式

**スケジュール遵守率 (Metrics!B2)**:
```excel
=COUNTIFS(RawData!$F:$F, Metrics!A2, RawData!$G:$G, "<=2") / COUNTIF(RawData!$F:$F, Metrics!A2) * 100
```
- `RawData!$F:$F`: Phase カラム
- `RawData!$G:$G`: 遅延日数カラム (`=ActualDate - PlannedDate`)
- `Metrics!A2`: 対象フェーズ (例: P1)
- `"<=2"`: 許容遅延2日以内

**平均遅延日数 (Metrics!C2)**:
```excel
=AVERAGEIF(RawData!$F:$F, Metrics!A2, RawData!$G:$G)
```

#### 条件付き書式

- **遵守率カラム** (Metrics!B:B):
  - `>= 95`: 緑背景 `#D4EDDA`
  - `90-94`: 黄背景 `#FFF3CD`
  - `< 90`: 赤背景 `#F8D7DA`

### 4.5 Grafana ダッシュボード JSON (サンプル)

以下のJSONを Grafana へインポート (詳細は省略):

```json
{
  "dashboard": {
    "title": "HoneyLink Schedule Adherence Dashboard",
    "panels": [
      {
        "type": "stat",
        "title": "スケジュール遵守率",
        "targets": [
          {
            "query": "SELECT AVG(adherence_rate) FROM metrics WHERE time > now() - 30d"
          }
        ],
        "thresholds": [
          {"value": 90, "color": "red"},
          {"value": 95, "color": "green"}
        ]
      },
      {
        "type": "bargauge",
        "title": "フェーズ別遵守率",
        "targets": [
          {
            "query": "SELECT phase, adherence_rate FROM metrics WHERE time > now() - 7d GROUP BY phase"
          }
        ]
      }
    ]
  }
}
```

---

## 5. データ収集フロー

### 5.1 手動データ入力 (初期運用)

```
週次レビューミーティング (毎週金曜 10:00)
  ↓
Operations WG が完了タスクを確認
  ↓
Google Sheets の RawData シートへ実績日を入力
  ↓
数式が自動的に遅延日数・遵守率を計算
  ↓
Dashboard シートで可視化確認
```

### 5.2 自動データ収集 (本格運用)

#### GitHub Actions ワークフロー例

`.github/workflows/sync-task-metrics.yml`:
```yaml
name: Sync Task Metrics to Dashboard

on:
  issues:
    types: [closed]
  schedule:
    - cron: '0 0 * * *'  # 毎日0時 (UTC) に実行

jobs:
  sync-metrics:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v3

      - name: Extract task completion data
        id: extract
        run: |
          # GitHub API でクローズされた Issue を取得
          curl -H "Authorization: token ${{ secrets.GITHUB_TOKEN }}" \
            "https://api.github.com/repos/${{ github.repository }}/issues?state=closed&since=$(date -d '1 day ago' --iso-8601)" \
            > closed_issues.json
          
          # TaskID, 完了日を抽出 (jq でパース)
          cat closed_issues.json | jq -r '.[] | "\(.number),\(.closed_at)"' > task_data.csv

      - name: Upload to Google Sheets
        env:
          GOOGLE_SHEETS_API_KEY: ${{ secrets.GOOGLE_SHEETS_API_KEY }}
        run: |
          # Google Sheets API でデータ追加
          python scripts/upload_to_sheets.py --file task_data.csv --sheet "RawData"
```

**注意**: `secrets.GOOGLE_SHEETS_API_KEY` は GitHub Secrets で管理すること。

---

## 6. アラート設定

### 6.1 アラート条件

| アラートレベル | トリガー条件 | 通知先 | アクション |
|--------------|------------|--------|-----------|
| 🟡 **Warning** | 遵守率が90〜94%に低下 | Slack `#hl-ops-wg` | 週次レビューで原因分析 |
| 🔴 **Critical** | 遵守率が90%未満 | Slack `#hl-wg-leads` + PagerDuty | 緊急ミーティング招集 (24h以内) |
| ⚠️ **Buffer Alert** | バッファ消費率が70%超 | Governance Council (Matrix) | リスク緩和計画の見直し |
| 📉 **Velocity Drop** | 完了速度が計画比80%未満 | Operations WG Chair | リソース追加投入検討 |

### 6.2 Slack Webhook 設定例

Google Apps Script (Sheets → Slack):

```javascript
function checkScheduleAdherence() {
  const sheet = SpreadsheetApp.getActiveSpreadsheet().getSheetByName('Metrics');
  const adherenceRate = sheet.getRange('B2').getValue(); // P1フェーズの遵守率
  
  if (adherenceRate < 90) {
    sendSlackAlert('🔴 Critical: スケジュール遵守率が90%未満です (' + adherenceRate + '%)');
  } else if (adherenceRate < 95) {
    sendSlackAlert('🟡 Warning: スケジュール遵守率が目標を下回っています (' + adherenceRate + '%)');
  }
}

function sendSlackAlert(message) {
  const webhookUrl = PropertiesService.getScriptProperties().getProperty('SLACK_WEBHOOK_URL');
  const payload = JSON.stringify({ text: message });
  
  UrlFetchApp.fetch(webhookUrl, {
    method: 'post',
    contentType: 'application/json',
    payload: payload
  });
}

// トリガー設定: 毎日9:00に実行
```

**設定手順**:
1. Google Apps Script エディタで上記コードを貼り付け
2. `スクリプトのプロパティ` に `SLACK_WEBHOOK_URL` を追加
3. トリガーを設定 (時間主導型, 毎日 9:00-10:00)

---

## 7. 運用手順

### 7.1 日次タスク (Operations WG)

- [ ] **08:30**: 前日完了タスクを GitHub Projects で確認
- [ ] **09:00**: Google Sheets RawData シートへ実績日を入力 (手動運用時)
- [ ] **09:15**: Dashboard シートでKPI確認、異常値があれば Slack へ報告
- [ ] **09:30**: 遅延タスクの担当者へ Slack DM でフォローアップ

### 7.2 週次タスク (Operations WG)

- [ ] **金曜 10:00**: 週次進捗レビューミーティング
  - 前週の遵守率を報告
  - 遅延タスクの原因分析
  - 翌週のリスク予測
- [ ] **金曜 11:00**: レビュー結果を `spec/notes/meeting-notes.md` へ記録
- [ ] **金曜 15:00**: ダッシュボードのスクリーンショットを Slack `#hl-ops-wg` へ共有

### 7.3 月次タスク (Governance Council)

- [ ] **月初5営業日**: 前月の月次レポート作成
  - フェーズ別遵守率サマリー
  - 遅延原因トップ3
  - 改善アクションプラン
- [ ] **月初10営業日**: ガバナンス委員会レビュー
  - 遵守率95%未達の場合、改善計画承認
  - 次月の目標設定

### 7.4 四半期タスク (All WGs)

- [ ] **四半期末**: 四半期レトロスペクティブ
  - 遵守率トレンド分析
  - プロセス改善提案
  - ダッシュボード仕様のアップデート (必要に応じて)

---

## 8. 初期セットアップチェックリスト

- [ ] **ツール選定**: Google Sheets / Grafana / GitHub Projects のいずれかを決定 (期限: 2025-10-05)
- [ ] **ダッシュボードテンプレート作成**: Section 4.4 の仕様に従い作成 (期限: 2025-10-10)
- [ ] **初期データ投入**: `critical-path.csv` から予定完了日をインポート (期限: 2025-10-12)
- [ ] **アクセス権限設定**: 全WG Chairs に編集権限、その他に閲覧権限を付与 (期限: 2025-10-12)
- [ ] **アラート設定**: Slack Webhook + Google Apps Script トリガー設定 (期限: 2025-10-15)
- [ ] **運用マニュアル配布**: 本ドキュメントを全WGへ共有し、質疑応答セッション実施 (期限: 2025-10-15)
- [ ] **初回レビュー実施**: 2025-10-18 (金) の週次レビューでダッシュボードを初使用
- [ ] **フィードバック収集**: 初回使用後、改善点を `spec/notes/decision-log.md` へ記録

---

## 9. トラブルシューティング

### 9.1 よくある問題

| 問題 | 原因 | 解決策 |
|------|------|--------|
| 数式エラー `#DIV/0!` | 該当フェーズのタスクが0件 | IF文でゼロ除算を回避: `=IF(COUNTIF(...) = 0, "N/A", ...)` |
| 実績日が空白のままアラート発火 | タスク完了報告の遅延 | 週次レビューで未報告タスクを確認し、強制入力 |
| Slack通知が届かない | Webhook URL の期限切れ | Slack App 設定で新しいWebhook URLを再生成 |
| グラフが表示されない | データ範囲の設定ミス | グラフのデータソース範囲を `RawData!A:G` に修正 |

---

## 10. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 (設計仕様とGoogle Sheets実装手順) | Governance Council |

---

**テンプレートファイル**:
- Google Sheets テンプレート: `https://docs.google.com/spreadsheets/d/<template_id>` (作成後リンク追加)
- Grafana JSON: `spec/notes/grafana-dashboard-template.json` (作成後配置)
- GitHub Actions ワークフロー: `.github/workflows/sync-task-metrics.yml`

---

**関連ドキュメント**:
- [spec/roadmap.md](../roadmap.md) - 測定指標 (SLI/SLO) の定義
- [spec/testing/metrics.md](../testing/metrics.md) - KPI測定方法
- [spec/notes/critical-path-analysis.md](critical-path-analysis.md) - タスク依存関係とバッファ
- [spec/notes/governance.md](governance.md) - Operations WG の責務
