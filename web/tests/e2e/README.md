# E2E Tests — Competitive Programming AST Editor

## 概要

このディレクトリには、AST Editor の E2E テストが含まれる。
`doc/view/proccess.md` の E2E 先行設計方針に基づき、**ユーザー操作順序を先に固定**し、UI 実装はテストを通すために後から追う。

> **重要**: テストを変えて実装に合わせない。実装をテストの要求に合わせる。

## テスト実行

```bash
# 全テスト実行
npm run test:e2e

# UI モード（デバッグ用）
npm run test:e2e:ui
```

前提: `npm run dev` でローカルサーバーが起動していること（playwright.config.ts の `webServer` で自動起動される）。

## ディレクトリ構成

```
tests/e2e/
├── playwright.config.ts        # Playwright 設定
├── README.md                   # この文書
├── fixtures/
│   ├── editor-page.ts          # Page Object Model
│   └── helpers.ts              # 共通ヘルパー
├── basic-array.spec.ts         # カテゴリ 1: 基本配列
├── grid.spec.ts                # カテゴリ 2: グリッド
├── tree.spec.ts                # カテゴリ 3: 木入力
├── query.spec.ts               # カテゴリ 4: クエリ列
└── multi-testcase.spec.ts      # カテゴリ 5: 複数テストケース
```

## カテゴリと対応文書

| Spec ファイル | カテゴリ | フロー文書 | 参照問題 |
|--------------|---------|-----------|---------|
| basic-array.spec.ts | 基本配列 (N + A) | §1 | ABC395-A |
| grid.spec.ts | グリッド (H W + S) | §2 | ABC390-C |
| tree.spec.ts | 木入力 (辺リスト) | §3 | 典型 ABC-D |
| query.spec.ts | クエリ列 (Choice) | §4 | ABC395-D |
| multi-testcase.spec.ts | 複数テストケース | §5 | 典型マルチケース |

フロー文書: `doc/view/problem-user-flows.md`

## data-testid 契約

E2E テストは以下の `data-testid` に依存する。UI 実装時にこれらの testid を付与すること。

### Structure ペイン

| data-testid | 説明 |
|------------|------|
| `structure-pane` | Structure ペインのルート要素 |
| `insertion-hotspot-below` | 下に追加する hotspot |
| `insertion-hotspot-right` | 右に追加する hotspot |
| `insertion-hotspot-inside` | ブロック内に追加する hotspot |
| `insertion-hotspot-variant` | variant 追加用 hotspot |
| `structure-node-{nodeId}` | 個別ノード要素 |

### ポップアップ / ウィザード

| data-testid | 説明 |
|------------|------|
| `node-popup` | ノード種別選択ポップアップ |
| `popup-option-scalar` | scalar 選択 |
| `popup-option-array` | 横配列 選択 |
| `popup-option-tuple` | tuple 選択 |
| `popup-option-grid-template` | 文字グリッドテンプレート |
| `popup-option-edge-list` | 辺リストテンプレート |
| `popup-option-query-list` | クエリ列テンプレート |
| `popup-option-multi-testcase` | 複数テストケーステンプレート |
| `popup-option-repeat` | repeat 選択 |
| `name-input` | 変数名入力フィールド |
| `type-select` | 型選択ドロップダウン |
| `length-select` | 長さ参照先選択 |
| `count-expression-input` | count 式入力（辺リスト用） |
| `variant-tag-input` | variant タグ入力 |
| `confirm-button` | 確定ボタン |

### Constraint ペイン

| data-testid | 説明 |
|------------|------|
| `constraint-pane` | Constraint ペインのルート要素 |
| `draft-constraint-{index}` | draft 制約（index は 0 始まり） |
| `completed-constraint-{index}` | 完成制約 |
| `constraint-lower-input` | Range 下限入力 |
| `constraint-upper-input` | Range 上限入力 |
| `constraint-confirm` | 制約確定ボタン |
| `property-shortcut` | Property ショートカットボタン |
| `property-option-{name}` | Property 選択肢 (tree, simple, etc.) |
| `sumbound-shortcut` | SumBound ショートカットボタン |
| `sumbound-var-select` | SumBound 対象変数選択 |
| `sumbound-upper-input` | SumBound 上界入力 |
| `charset-option-lowercase` | charset プリセット: 英小文字 |

### 右ペイン (Preview)

| data-testid | 説明 |
|------------|------|
| `preview-pane` | Preview ペインのルート要素 |
| `tex-input-format` | TeX 入力形式表示領域 |
| `tex-constraints` | TeX 制約表示領域 |
| `sample-output` | サンプルケース表示領域 |

### 数式編集

| data-testid | 説明 |
|------------|------|
| `math-editable-{id}` | クリック可能な数式要素 |
| `math-editor-input` | 数式編集入力欄 |
| `math-editor-confirm` | 数式確定ボタン |

## テストの現状

現在、全テストは **fail** が期待される。
UI に編集機能が未実装のため、各テストの hotspot クリックから先は動作しない。

Phase 3（UI 実装）でテストを通すことが目標。

## 設計原則

1. **テストを変えて実装に合わせない** — 実装をテストの要求に合わせる
2. **右ペイン三要素は必須** — TeX 入力形式 + TeX 制約 + sample は全テストで検証
3. **Structure → draft 自動生成** — ノード追加時に draft constraint が自動生成されることを検証
4. **draft → completed 昇格** — 値入力後に constraint が completed に昇格することを検証
