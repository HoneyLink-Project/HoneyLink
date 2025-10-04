# Stakeholder Attendance & Reminder System

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> ステークホルダー出席率90%以上を維持するためのリマインダー・エスカレーション体制を定義します。

---

## 1. 目的とスコープ

### 1.1 目的
- 各ワーキンググループ(WG)の定例会議における**コアメンバー出席率90%以上**を維持する
- 欠席時の迅速な補完レビュープロセスを確立し、意思決定の遅延を防止する
- エスカレーションパスを明確化し、ブロッカーを早期解決する

### 1.2 スコープ
- 対象: Architecture/Protocol/UX/Security/Operations の全5ワーキンググループ
- 測定期間: 月次集計（毎月1日に前月実績をレビュー）
- 適用会議: `spec/notes/governance.md` で定義された週次定例会議

---

## 2. 出席率測定方法

### 2.1 定義
- **出席率** = (実出席者数 / コアメンバー総数) × 100
- **コアメンバー**: `spec/notes/governance.md` の "Core Members (Role IDs)" カラムに記載されたメンバー
- **実出席**: 会議開始から終了まで80%以上の時間参加した場合を「出席」とカウント
- **事前承認欠席**: 24時間前までに Chair に通知し、補完レビュー計画を提出した場合は出席率計算から除外

### 2.2 記録方法
- 各WGのChairは `spec/notes/meeting-notes.md` テンプレートに従い、参加者リストを記録
- 欠席者は「Absent:」欄に記載し、理由（事前承認/無連絡）を明記
- 月末に Chair が出席率を算出し、`spec/notes/governance.md` の Metrics セクションにレポート

---

## 3. リマインダーシステム

### 3.1 自動リマインダー設定

#### 3.1.1 会議前リマインダー
| タイミング | 送信先 | チャネル | 内容 |
|-----------|--------|---------|------|
| 会議48時間前 | 全コアメンバー | Slack + Email | 会議日時・アジェンダ・事前資料リンク |
| 会議24時間前 | 未応答者のみ | Slack DM | 出席確認・欠席時の補完レビュー手順案内 |
| 会議2時間前 | 全コアメンバー | Slack `#<wg-channel>` | 最終リマインダー・Zoom/Meetリンク |

#### 3.1.2 リマインダー実装方針
- **ツール**: Slack Workflow Builder または外部スケジューラー（GitHub Actions + Slack API）
- **設定責任者**: 各WG Chair（初回設定後は Operations WG がメンテナンス）
- **テンプレート**:
  ```
  📅 [HoneyLink WG] <WG名> 定例会議リマインダー
  
  日時: <YYYY-MM-DD HH:MM JST>
  場所: <Zoom/Meet Link>
  アジェンダ: <spec/notes/meeting-notes.mdへのリンク>
  
  欠席の場合は24時間前までに @<Chair Role ID> へ連絡してください。
  補完レビューフォーム: <リンク>
  ```

### 3.2 欠席時の対応フロー

#### 3.2.1 事前承認欠席
1. メンバーは欠席24時間前までに Chair へ Slack DM で通知
2. 補完レビュー計画を提出（下記 Section 4.1 参照）
3. Chair が承認し、会議議事録に「事前承認欠席」と記録

#### 3.2.2 無連絡欠席
1. 会議終了後1時間以内に Chair が欠席者へ Slack DM で連絡
2. 欠席理由を確認し、48時間以内の補完レビュー実施を要請
3. 48時間経過しても応答がない場合、Section 5 のエスカレーションへ移行

---

## 4. 補完レビュープロセス

### 4.1 補完レビュー計画書テンプレート
欠席予定者は以下の情報を Chair へ提出:

```markdown
## 補完レビュー計画書

- **氏名/Role ID**: <記入>
- **欠席対象会議**: <YYYY-MM-DD WG名>
- **欠席理由**: <簡潔に>
- **補完レビュー方法**: 
  - [ ] 議事録を会議後6時間以内に確認し、コメントを Slack スレッドへ投稿
  - [ ] 決定事項に対する賛否を Chair へ個別報告（48時間以内）
  - [ ] 必要に応じて Chair または代理メンバーと1on1実施（30分以内）
- **完了予定日時**: <YYYY-MM-DD HH:MM>
```

### 4.2 補完レビュー完了基準
- 議事録へのコメント投稿 or Chair への個別報告完了
- 決定事項(Decision Log)へのフィードバック提出
- アクションアイテムのうち自分が担当する項目の accept/reject を明示

### 4.3 記録
- 補完レビュー完了後、Chair が `spec/notes/meeting-notes.md` の該当エントリに「補完レビュー完了」と追記
- 出席率計算時に「事前承認欠席+補完レビュー完了」は除外対象とする

---

## 5. エスカレーションパス

### 5.1 エスカレーショントリガー

| 状況 | エスカレーション先 | タイムライン | アクション |
|------|-------------------|-------------|-----------|
| 月次出席率が90%未満 | WG Chair → Governance Council | 月初5営業日以内 | 改善計画書提出、次月で回復目標設定 |
| 同一メンバーが連続2回無連絡欠席 | WG Chair → 当人の上長 | 2回目欠席後24時間以内 | 1on1実施、参加継続可否確認 |
| 補完レビュー未完了(48時間超過) | WG Chair → Governance Council | 超過後即座 | 代理メンバー指名、決定事項の再承認プロセス |
| クリティカルな決定事項で過半数未達 | WG Chair → 全WG Chairs Sync | 会議終了後2時間以内 | 緊急Slack huddle、48時間以内の臨時会議開催 |

### 5.2 Governance Council の対応
- **構成**: Architecture + Security + Operations WG Chairs
- **連絡手段**: Matrix secure room `!govcouncil:honeylink.local`
- **決定権限**: メンバー交代勧告、補完レビュープロセスの一時緩和、会議cadence調整
- **記録**: `spec/notes/decision-log.md` へエスカレーション内容と対応策を記録（ステータス: `エスカレーション`）

### 5.3 改善計画書テンプレート
出席率90%未満のWGは以下を提出:

```markdown
## 出席率改善計画書

- **対象WG**: <WG名>
- **対象月**: <YYYY-MM>
- **実績出席率**: <XX.X%>
- **根本原因分析**:
  - <箇条書き: 会議時間帯の不適合、アジェンダの不明確さ、等>
- **改善施策**:
  1. <具体的施策: 会議時間変更、事前資料配布の徹底、等>
  2. <施策2>
- **目標**: 次月出席率 95% 以上達成
- **モニタリング**: 週次で Chair が出席状況を Slack `#hl-wg-leads` へ報告
- **承認者**: Governance Council
- **提出日**: <YYYY-MM-DD>
```

---

## 6. ツール設定ガイド

### 6.1 Slack Workflow Builder 設定例

#### ステップ1: Workflow作成
1. Slack ワークスペースで「Tools」→「Workflow Builder」を開く
2. 「Create」→「Scheduled Date & Time」を選択
3. トリガー: 毎週 `<会議曜日>` の `<会議時刻-48h>` に実行

#### ステップ2: メッセージ設定
1. 「Add Step」→「Send a message」
2. 送信先: `#<wg-channel>` と各メンバーへのDM
3. メッセージ: Section 3.1.1 のテンプレートを使用
4. Variables: `{{meeting_date}}`, `{{agenda_link}}` を動的挿入

#### ステップ3: 条件分岐（未応答者フォロー）
1. 「Add Step」→「Wait for a response」(24時間)
2. 応答がない場合 → Chair へ通知 + 該当メンバーへリマインダーDM再送

### 6.2 GitHub Actions による自動化 ✅ **実装完了**

**実装ファイル**:
- `.github/workflows/attendance-reminder.yml` - 6時間毎に実行される自動リマインダー
- `scripts/attendance-reminder.ts` - TypeScriptベースの統合システム (420行)
- `scripts/package.json` - Node.js依存関係定義
- `scripts/.env.attendance.example` - 環境変数テンプレート
- `.github/ISSUE_TEMPLATE/補完レビュー計画書.md` - 補完レビュー用GitHubイシューテンプレート

**セットアップ手順**:
1. GitHubリポジトリのSecretsに以下を登録:
   - `SLACK_BOT_TOKEN` (xoxb-... 形式のBot Token)
   - `SLACK_WEBHOOK_URL` (失敗通知用)
   - `GITHUB_TOKEN` (エスカレーションイシュー作成用、自動設定済み)

2. Slackワークスペースでアプリを作成:
   - Bot Token Scopes: `chat:write`, `chat:write.public`, `users:read`
   - 各WGチャンネルにBotを招待

3. 初回実行テスト:
   ```powershell
   cd scripts
   npm install
   copy .env.attendance.example .env
   # .envに実際のトークンを設定
   npm run reminder
   ```

4. GitHub Actionsが自動的に6時間毎に実行を開始

**機能**:
- ✅ 48時間前リマインダー (チャンネル全体へ通知)
- ✅ 24時間前DM (未応答者へ個別通知)
- ✅ 2時間前最終リマインダー
- ✅ 出席率90%未達時の自動エスカレーションイシュー作成
- ✅ 失敗時のSlack通知

**技術スタック**:
- Pure Node.js/TypeScript実装 (C/C++依存なし)
- @octokit/rest 20.0.2 (GitHub API, Pure JS)
- @slack/web-api 7.0.2 (Slack API, Pure JS)
- Idempotent設計 (複数回実行しても安全)
- JST (Asia/Tokyo) タイムゾーン対応

---

## 7. モニタリングとレポート

### 7.1 月次レポート項目
Operations WG が毎月5日までに以下を集計し、`spec/notes/governance.md` へ追記:

| WG | 対象月 | コアメンバー数 | 実出席延べ人数 | 出席率 | 事前承認欠席数 | 無連絡欠席数 | 補完レビュー完了率 |
|----|--------|----------------|----------------|--------|----------------|-------------|-------------------|
| Architecture | 2025-09 | 4 | 15/16 | 93.8% | 1 | 0 | 100% |
| Protocol | 2025-09 | 4 | 14/16 | 87.5% | 1 | 1 | 50% |
| ... | ... | ... | ... | ... | ... | ... | ... |

### 7.2 ダッシュボード（オプション）
- **ツール**: Grafana + TimescaleDB または Google Sheets
- **データソース**: 各WGの議事録から自動集計（Operations WG が月次スクリプト実行）
- **可視化項目**: 
  - 月次出席率トレンド（折れ線グラフ）
  - WG別出席率比較（棒グラフ）
  - 無連絡欠席者ランキング（改善施策の優先度付け）

---

## 8. 成功指標 (DoD)

- [ ] 全5WGに対して Slack Workflow または同等の自動リマインダーが設定済み
- [ ] 補完レビュープロセステンプレートが `spec/notes/` に配置され、全メンバーが参照可能
- [ ] エスカレーションパスが `spec/notes/governance.md` に明記され、Governance Council の連絡先が最新
- [ ] 初回月次レポート（2025年10月分）が11月5日までに生成され、出席率90%以上を達成
- [ ] Chair 向けトレーニング資料（本文書 + FAQ）が完成し、全WG Chairsが内容を確認済み

---

## 9. FAQ

**Q1: 病欠など緊急の欠席時はどうするか？**  
A1: 可能な限り会議開始前に Chair へ Slack DM で連絡。会議後48時間以内に補完レビューを実施すれば「事前承認欠席」と同等扱い。

**Q2: タイムゾーンの異なるメンバーへの配慮は？**  
A2: 会議時間は `spec/notes/governance.md` のレビューcadence策定時に全メンバーの可用性を考慮済み。どうしても参加困難な場合は Chair へ相談し、録画視聴+非同期レビューを承認。

**Q3: 出席率90%を下回った場合のペナルティは？**  
A3: ペナルティではなく、改善計画書の提出と Governance Council のサポートを受けることで次月回復を目指す。連続3ヶ月未達の場合はメンバー構成の見直しを検討。

**Q4: ゲスト参加者は出席率に含まれるか？**  
A4: 含まれない。コアメンバーのみが対象。ただしゲストの継続参加が有益な場合、Chair はコアメンバーへの昇格を Governance Council へ提案可能。

---

## 10. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | Governance Council |

---

**関連ドキュメント**:
- [spec/notes/governance.md](governance.md) - WG構成と週次cadence
- [spec/notes/meeting-notes.md](meeting-notes.md) - 会議議事録テンプレート
- [spec/notes/decision-log.md](decision-log.md) - 決定事項記録先
- [spec/roadmap.md](../roadmap.md) - 測定指標(SLI/SLO)との整合性

