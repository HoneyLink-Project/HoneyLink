# Module Specification: Experience Layer (SDK & UI Shell)

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> Experience Layer モジュールの実装仕様書。SDK API と UI Shell の提供を担当します。

**トレーサビリティ ID**: `MOD-008-EXPERIENCE-LAYER`

---

## 1. モジュール概要

- **モジュール名:** Experience Layer (SDK & UI Shell)
- **担当チーム:** UX WG (ENG-UX-01, ENG-UX-02), API WG (ENG-API-01)
- **概要:** アプリケーション開発者向け SDK API と、エンドユーザー向け UI Shell の提供
- **ステータス:** 実装中 (P1フェーズ)
- **リポジトリパス:** `sdk/`, `ui-shell/`

### 価値提案
- 10行以下のコードでペアリング・セッション確立可能
- React/TypeScript 製 UI Shell (Material-UI ベース)
- 多言語対応 (en/ja/es/zh)
- アクセシビリティ WCAG 2.1 AA準拠

---

## 2. 責務と境界

### 主な責務
- **SDK API**: Rust/TypeScript/Python 向けクライアントライブラリ
- **UI Shell**: ペアリング/セッション管理/QoS設定画面
- **多言語化**: i18n (react-i18next)
- **アクセシビリティ**: ARIA属性、キーボードナビゲーション
- **アニメーション**: ペアリング進捗、セッション状態変化の視覚化

### 非責務
- **セッション管理**: Session Orchestrator に委譲
- **QoSポリシー決定**: Policy Engine に委譲
- **認証**: Crypto & Trust Anchor に委譲
- **ビジネスロジック**: 各モジュールに委譲

### 関連ドキュメント
- [spec/ui/overview.md](../ui/overview.md)
- [spec/ui/visual-design.md](../ui/visual-design.md)
- [spec/ui/accessibility.md](../ui/accessibility.md)
- [spec/requirements.md](../requirements.md) - FR-05 (UI), FR-06 (プロファイル共有)

---

## 3. インターフェース

### 3.1 SDK API (TypeScript)

```typescript
// 概念説明用 (実装コードではない)
class HoneyLinkClient {
  async connect(deviceId: string, options?: ConnectOptions): Promise<Session>;
  async disconnect(sessionId: string): Promise<void>;
  async updateQoS(sessionId: string, policy: QoSPolicy): Promise<void>;
  async exportProfile(profileId: string): Promise<ProfileTemplate>;
  onSessionStateChanged(callback: (state: SessionState) => void): void;
}

interface ConnectOptions {
  profile?: string;  // e.g., "prof_arvr_spatial_v1"
  timeout?: number;  // デフォルト 30秒
  physicalLayer?: "WiFi" | "5G" | "THz" | "Ethernet";
}
```

### 3.2 UI Shell 画面構成

| 画面名 | ルート | 主要コンポーネント |
|--------|--------|-------------------|
| **ペアリング** | `/pairing` | QRコード表示、手動入力フォーム |
| **セッション一覧** | `/sessions` | アクティブセッションカード、状態インジケータ |
| **QoS設定** | `/sessions/:id/qos` | プロファイル選択、スライダー (latency/bandwidth) |
| **プロファイル管理** | `/profiles` | CRUD, エクスポート/インポート |
| **設定** | `/settings` | 言語選択、テーマ (Light/Dark) |

詳細: [spec/ui/wireframes.md](../ui/wireframes.md)

### 3.3 Control-Plane API 呼び出し

| SDK メソッド | HTTP メソッド | エンドポイント | 説明 |
|-------------|--------------|----------------|------|
| `connect()` | POST | `/api/v1/sessions` | セッション確立 |
| `disconnect()` | DELETE | `/api/v1/sessions/:id` | セッション切断 |
| `updateQoS()` | PATCH | `/api/v1/sessions/:id/qos` | QoSポリシー更新 |
| `exportProfile()` | GET | `/api/v1/profiles/:id/export` | プロファイル エクスポート |

詳細: [spec/modules/session-orchestrator.md](./session-orchestrator.md) (P2Pセッション管理)

---

## 4. データモデル

### 4.1 SDK データモデル

#### Session (セッション)
```typescript
interface Session {
  sessionId: string;
  deviceId: string;
  state: "Pending" | "Paired" | "Active" | "Suspended" | "Closed";
  profile: ProfileTemplate;
  metrics: SessionMetrics;
  createdAt: Date;
  updatedAt: Date;
}

interface SessionMetrics {
  latencyP95Ms: number;
  throughputMbps: number;
  packetLossRate: number;
  uptime: number;  // 秒
}
```

#### QoSPolicy
```typescript
interface QoSPolicy {
  profileId: string;
  latencyBudgetMs: number;
  bandwidthFloorMbps: number;
  bandwidthCeilingMbps: number;
  fecMode: "NONE" | "LIGHT" | "HEAVY";
  priority: number;  // 0-7
}
```

### 4.2 UI Shell データフロー

```
User Action (ペアリングボタンクリック)
  ↓
React Component (PairingView.tsx)
  ↓
SDK API (client.connect())
  ↓
HTTP POST /api/v1/sessions
  ↓
Control-Plane API
  ↓
Session Orchestrator
  ↓
WebSocket/SSE で状態更新
  ↓
React Component (リアルタイム更新)
```

詳細: [spec/architecture/dataflow.md](../architecture/dataflow.md)

---

## 5. 多言語化 (i18n)

### 対応言語
- **en**: English (デフォルト)
- **ja**: 日本語
- **es**: Español (スペイン語)
- **zh**: 中文 (中国語簡体字)

### 翻訳キー例
```json
{
  "pairing.title": {
    "en": "Pair Device",
    "ja": "デバイスをペアリング",
    "es": "Emparejar dispositivo",
    "zh": "配对设备"
  },
  "session.state.active": {
    "en": "Active",
    "ja": "アクティブ",
    "es": "Activo",
    "zh": "活跃"
  },
  "qos.latency_budget": {
    "en": "Latency Budget (ms)",
    "ja": "レイテンシ予算 (ms)",
    "es": "Presupuesto de latencia (ms)",
    "zh": "延迟预算 (ms)"
  }
}
```

**ライブラリ**: `react-i18next` v12+

詳細: [spec/ui/overview.md](../ui/overview.md) - 多言語化

---

## 6. アクセシビリティ

### WCAG 2.1 AA準拠項目
- **1.1.1 非テキストコンテンツ**: 全画像に `alt` 属性
- **1.4.3 コントラスト**: 最低4.5:1 (通常テキスト), 3:1 (大きいテキスト)
- **2.1.1 キーボード**: 全操作をキーボードのみで実行可能
- **2.4.7 フォーカスの可視化**: フォーカスリング表示
- **3.3.2 ラベルまたは説明**: 全フォームフィールドに `<label>`

### ARIA属性例
```html
<!-- ペアリング進捗 -->
<div role="progressbar" aria-valuenow="60" aria-valuemin="0" aria-valuemax="100">
  60% Complete
</div>

<!-- セッション状態 -->
<span role="status" aria-live="polite">
  Session Active
</span>
```

詳細: [spec/ui/accessibility.md](../ui/accessibility.md)

---

## 7. アニメーション

### ペアリング進捗アニメーション
- **種類**: ローディングスピナー + プログレスバー
- **Duration**: 0-100% (推定30秒)
- **Easing**: ease-in-out
- **アクセシビリティ**: `prefers-reduced-motion` 対応

### セッション状態変化
| 遷移 | アニメーション | Duration |
|------|---------------|----------|
| Pending → Paired | フェードイン + 緑色パルス | 500ms |
| Active → Suspended | 黄色点滅 | 1000ms (repeat) |
| Any → Closed | フェードアウト | 300ms |

詳細: [spec/ui/animations.md](../ui/animations.md)

---

## 8. 依存関係

| 種別 | コンポーネント | インターフェース | SLA/契約 |
|------|----------------|-------------------|----------|
| **下位** | Control-Plane API | REST/WebSocket | P95 < 500ms |
| **下位** | Session Orchestrator | Event Bus (SSE) | リアルタイム |
| **Peer** | Policy Engine | REST (プロファイル取得) | P95 < 300ms |

**依存ルール**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 9. 性能・スケーラビリティ

### SLO/SLI

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| SDK API レイテンシ (P95) | < 500ms | connect() → Session返却 |
| UI 初期ロード時間 (P95) | < 2秒 | First Contentful Paint |
| WebSocket 再接続時間 (P95) | < 3秒 | 切断 → 再接続完了 |
| アニメーション FPS | ≥ 60fps | Chrome DevTools Performance |

詳細: [spec/performance/benchmark.md](../performance/benchmark.md)

---

## 10. セキュリティ & プライバシー

### 認証/認可
- **SDK**: OAuth2 + mTLS
- **UI Shell**: OAuth2 + PKCE (Authorization Code Flow)

### 脅威対策 (STRIDE)
| 脅威 | 対策 |
|------|------|
| **Spoofing** | OAuth2 + mTLS |
| **Cross-Site Scripting (XSS)** | React自動エスケープ + Content Security Policy |
| **Cross-Site Request Forgery (CSRF)** | SameSite Cookie + CSRF Token |

詳細: [spec/security/auth.md](../security/auth.md)

---

## 11. 観測性

### メトリクス

| メトリクス名 | 型 | ラベル |
|-------------|---|--------|
| `sdk_api_calls_total` | Counter | method, result |
| `sdk_api_duration_seconds` | Histogram | method |
| `ui_page_views_total` | Counter | route |
| `ui_load_time_seconds` | Histogram | route |

### ログフォーマット
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "INFO",
  "event": "session.connected",
  "session_id": "sess_xyz",
  "device_id": "DEV-***",
  "profile_id": "prof_iot_lowpower_v2",
  "trace_id": "..."
}
```

参照: [spec/testing/metrics.md](../testing/metrics.md)

---

## 12. SDK サンプルコード

### TypeScript
```typescript
import { HoneyLinkClient } from '@honeylink/sdk';

const client = new HoneyLinkClient({
  apiEndpoint: 'https://api.honeylink.example.com',
  authToken: 'your-oauth2-token'
});

// ペアリング & セッション確立
const session = await client.connect('DEVICE-ABC-123', {
  profile: 'prof_arvr_spatial_v1',
  timeout: 30000
});

console.log(`Session ID: ${session.sessionId}`);
console.log(`State: ${session.state}`);

// 状態変化の監視
client.onSessionStateChanged((state) => {
  console.log(`New state: ${state}`);
});

// QoS更新
await client.updateQoS(session.sessionId, {
  profileId: 'prof_gaming_input_v1',
  latencyBudgetMs: 6,
  bandwidthFloorMbps: 5,
  bandwidthCeilingMbps: 50,
  fecMode: 'LIGHT',
  priority: 7
});

// 切断
await client.disconnect(session.sessionId);
```

### Python
```python
from honeylink_sdk import HoneyLinkClient

client = HoneyLinkClient(
    api_endpoint='https://api.honeylink.example.com',
    auth_token='your-oauth2-token'
)

# ペアリング & セッション確立
session = client.connect('DEVICE-ABC-123', profile='prof_iot_lowpower_v2')
print(f"Session ID: {session.session_id}")

# 切断
client.disconnect(session.session_id)
```

---

## 13. 要件トレーサビリティ

### FR-05: UI
- **関連**: ペアリング/QoS設定のGUI提供
- **実装**: React UI Shell

### FR-06: プロファイルテンプレ共有
- **関連**: プロファイルエクスポート/インポート機能
- **実装**: SDK API `exportProfile()` / UI Shell プロファイル管理画面

**トレーサビリティID対応表**:
```
MOD-008-EXPERIENCE-LAYER → FR-05 (UI provision)
MOD-008-EXPERIENCE-LAYER → FR-06 (profile sharing)
```

---

## 14. テスト戦略

### 単体テスト (SDK)
- API メソッド (connect/disconnect/updateQoS) - 各10ケース
- エラー処理 (timeout, 401 Unauthorized) - 15ケース
- カバレッジ目標: 90%

### 単体テスト (UI Shell)
- React Component (Jest + React Testing Library) - 各画面5ケース
- 多言語化 (4言語 × 主要文言10個)
- アクセシビリティ (jest-axe) - 各画面

### E2E テスト
- Playwright: ペアリング → セッション確立 → QoS更新 → 切断
- 多言語切替
- キーボードナビゲーション

詳細: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/testing/e2e-tests.md](../testing/e2e-tests.md)

---

## 15. デプロイ & 運用

### SDK
- **配布**: npm (TypeScript), PyPI (Python), crates.io (Rust)
- **バージョニング**: SemVer

### UI Shell
- **デプロイ方法**: Static hosting (Vite build → S3/CloudFront)
- **CI/CD**: GitHub Actions (lint → test → build → deploy)

詳細: [spec/deployment/ci-cd.md](../deployment/ci-cd.md)

---

## 16. リスク & 技術的負債

| リスク | 緩和策 |
|--------|--------|
| WebSocket切断頻発 | 自動再接続 (Exponential backoff) |
| 多言語翻訳品質 | ネイティブスピーカーレビュー |
| アクセシビリティ未達 | CI で jest-axe 自動チェック |

---

## 17. 受け入れ基準 (DoD)

- [x] SDK API (TypeScript/Python/Rust) 仕様完成
- [x] UI Shell 画面構成定義完了
- [x] ui/overview.md との整合性確認完了
- [x] 多言語化 (en/ja/es/zh) 仕様記述
- [x] アクセシビリティ WCAG 2.1 AA 準拠項目明示
- [x] FR-05/FR-06 との紐付け明示
- [x] トレーサビリティID (`MOD-008-EXPERIENCE-LAYER`) 付与
- [x] C/C++ 依存排除確認 (TypeScript/React純実装)

---

## 18. 変更履歴

| バージョン | 日付 | 変更内容 | 承認者 |
|-----------|------|---------|--------|
| 1.0 | 2025-10-01 | 初版作成 | UX WG (ENG-UX-01) |
