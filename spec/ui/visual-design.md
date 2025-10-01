# docs/ui/visual-design.md

**バッジ:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> 本書は HoneyLink™ UI の視覚言語 (カラー、タイポグラフィ、間隔、グリッド、テーマ) を定義します。実装コード・CSS・C/C++依存ツールは扱いません。

## 目次
- [ブランドコンセプト](#ブランドコンセプト)
- [カラーパレット](#カラーパレット)
- [タイポグラフィ](#タイポグラフィ)
- [スペーシングとグリッド](#スペーシングとグリッド)
- [コンポーネントスタイル](#コンポーネントスタイル)
- [テーマとダークモード](#テーマとダークモード)
- [デザイントークン](#デザイントークン)
- [アクセシビリティ整合](#アクセシビリティ整合)
- [受け入れ基準 (DoD)](#受け入れ基準-dod)

## ブランドコンセプト
- **キーワード:** 「蜜」「連携」「光」「信頼」。
- **体験:** 温かみと未来感を併せ持つビジュアル。柔らかな曲線と蜂蜜滴を想起させるハイライト。

## カラーパレット
| トークン | HEX | 使用箇所 | コントラスト比 (対背景) |
|----------|-----|----------|--------------------------|
| `color.primary` | `#F4B400` | CTAボタン、強調テキスト | 4.6:1 (対 #1C1B29) |
| `color.primary-dark` | `#D99800` | ホバー | 7.2:1 |
| `color.secondary` | `#7F5AF0` | サブCTA、リンク | 5.0:1 |
| `color.success` | `#2EC4B6` | 成功トースト | 4.5:1 |
| `color.warning` | `#FF9F1C` | 注意バナー | 4.7:1 |
| `color.error` | `#EF476F` | エラー表示 | 5.4:1 |
| `color.surface` | `#FFFFFF` | カード背景 | - |
| `color.surface-alt` | `#F7F7FB` | サブ背景 | - |
| `color.text-primary` | `#1C1B29` | 本文 | 14.2:1 |
| `color.text-inverse` | `#FFFFFF` | ダーク背景 | 12.6:1 |

## タイポグラフィ
| トークン | フォント分類 | サイズ | 行間 | 用途 |
|----------|---------------|--------|------|------|
| `font.display` | Humanist Sans (例: Inter, Noto Sans) | 32px | 120% | ヒーロータイトル |
| `font.heading` | 同上 | 24px | 125% | セクションタイトル |
| `font.subheading` | 同上 | 20px | 130% | カードタイトル |
| `font.body` | Sans-serif | 16px | 150% | 本文 |
| `font.mono` | Monospace | 14px | 140% | ログ・コード名 |

- フォントは Web セーフ + オープンライセンスを前提。C/C++依存フォントレンダラは使用しない。

## スペーシングとグリッド
- **ベースユニット:** 8px。`space.1 = 8px`, `space.2 = 16px`, ...
- **グリッド:** 12 カラム、ガター 24px。モバイル 4 カラム、タブレット 8 カラム。
- **カード:** 外側余白 `space.3`、内側 `space.2`。
- **行間:** タイポグラフィ表参照。

## コンポーネントスタイル
| コンポーネント | スタイル要素 | 状態変化 |
|----------------|---------------|----------|
| ボタン (Primary) | 背景: `color.primary`, 角丸 12px, シャドウ軽度 | Hover: primary-dark, Focus: 2px outline `color.secondary`, Disabled: `color.surface-alt` |
| カード | 背景 `color.surface`, 角丸 16px, 影: 0 12 24 rgba(0,0,0,0.08) | Hover: 4px上昇、影強調 |
| バナー | 背景 `color.secondary`, テキスト inverse, アイコン | 消灯: フェード 250ms |
| チップ | 背景 `color.surface-alt`, 境界 `color.secondary` | 選択時: 塗り `color.secondary`, テキスト inverse |

## テーマとダークモード
- **ライトテーマ:** 明るい背景、ハイコントラスト文字。
- **ダークテーマ:** 背景 `#12121C`, テキスト `#E4E4F4`, primary: `#F4B400` (不変)。
- **自動切替:** システム設定 + ユーザー設定を尊重。
- **トークン差分:** `color.surface` ↔ `color.surface-dark`, `shadow` 弱体化。

## デザイントークン
```
color.primary = #F4B400
color.primary-dark = #D99800
color.secondary = #7F5AF0
font.display.size = 32px
space.3 = 24px
radius.large = 16px
shadow.elevated = 0 12px 24px rgba(0,0,0,0.08)
```
- トークン管理は JSON/YAML 等のデータ形式を想定。C/C++依存ツールは禁止。
- トークンは[docs/templates/ui-template.md](../templates/ui-template.md)で参照可能形式にする。

## アクセシビリティ整合
- コントラスト比 4.5:1 以上を保証。詳細は[docs/ui/accessibility.md](./accessibility.md)。
- フォーカスリングは 2px `color.secondary`、内側 1px `#FFFFFF`。
- モーションは[docs/ui/animations.md](./animations.md)の遅延設定を使用。

## 受け入れ基準 (DoD)
- カラー・タイポ・スペーシング・グリッドが定量的に定義されている。
- テーマ差分・ダークモード対応が説明されている。
- デザイントークンをテキスト形式で示し、C/C++依存を排除している。
- アクセシビリティとの整合が確認できるリンクが存在する。
- コンポーネントスタイルがワイヤーフレーム・アニメーション仕様と矛盾しない。
