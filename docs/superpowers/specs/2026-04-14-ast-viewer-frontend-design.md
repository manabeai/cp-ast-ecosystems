# AST Viewer Frontend — Phase 1 Design Spec

> Phase 1 は **viewer の成立確認** である。AST を受け取り、入力形式・制約・TeX・サンプルの 4 系統を 1 画面で確認できるようにする。編集機能は含まない。

## 1. Architecture Overview

### Crate & Directory Layout

```
cp-ast-ecosystems/
├── crates/
│   ├── cp-ast-core/       # 既存: AST 型 + render + tex + sample
│   ├── cp-ast-tree/       # 既存: ASCII tree renderer
│   ├── cp-ast-json/       # 既存: lossless JSON 境界層
│   └── cp-ast-wasm/       # 新規: wasm-bindgen バインディング
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs       # #[wasm_bindgen] 関数群
│           └── presets.rs   # 代表例 AST をプリセットとして定義
└── web/                   # 新規: Preact + Vite + TypeScript
    ├── package.json
    ├── vite.config.ts
    ├── tsconfig.json
    ├── index.html
    └── src/
        ├── main.tsx
        ├── app.tsx
        ├── wasm.ts           # wasm モジュール初期化
        ├── types.ts          # TypeScript 型定義
        └── components/
            ├── viewer/
            │   ├── StructurePane.tsx
            │   ├── ConstraintPane.tsx
            │   ├── PreviewPane.tsx
            │   └── Toolbar.tsx
            └── preview/
                ├── PreviewPage.tsx
                └── PreviewCard.tsx
```

### Data Flow

```
  ┌──────────────────┐     String     ┌──────────────────┐
  │  cp-ast-wasm     │ ─────────────→ │  web/ (Preact)   │
  │  (Rust, wasm)    │                │  (表示のみ)       │
  │                  │ ←───────────── │                  │
  │  全処理を実行     │  JSON string   │  AST を解釈しない │
  └──────────────────┘  (document)    └──────────────────┘
```

**原則**: フロントエンド側で AST の内部構造を再実装しない。全ての処理（レンダリング、TeX 生成、サンプル生成）は Rust 側で行い、結果の文字列だけを JS に渡す。

### Technology Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Frontend | Preact + TypeScript | 軽量、React 互換 |
| Build | Vite | 高速 HMR、wasm プラグインあり |
| Wasm | wasm-bindgen + wasm-pack | Rust → wasm の標準ツールチェイン |
| TeX rendering | KaTeX | 軽量、高速、ブラウザレンダリング |
| State management | Preact Signals or useState | 最小限の状態管理 |
| Styling | CSS Modules | スコープ付き CSS、ライブラリ依存なし |
| wasm API pattern | JSON String API | シンプル、JS 側が内部構造を知らない |

## 2. wasm API (cp-ast-wasm)

新規クレート `crates/cp-ast-wasm/` を作成。`wasm-bindgen` で JS から呼べる関数を公開する。

### Dependencies

```toml
[dependencies]
cp-ast-core = { path = "../cp-ast-core" }
cp-ast-tree = { path = "../cp-ast-tree" }
cp-ast-json = { path = "../cp-ast-json" }
wasm-bindgen = "0.2"
```

### Public Functions

全関数は `#[wasm_bindgen]` でエクスポートする。document は cp-ast-json の JSON string として受け渡す。

```rust
/// 入力形式テキストを返す (人間向けメイン表示)
/// 例: "N\nA_1 A_2 ... A_N"
#[wasm_bindgen]
pub fn render_input_format(document_json: &str) -> Result<String, JsError>

/// Structure AST tree を返す (AST トグル用)
/// 例: "Sequence (#0)\n├── Scalar(N) (#1)\n└── Array(A) (#2)"
#[wasm_bindgen]
pub fn render_structure_tree(document_json: &str) -> Result<String, JsError>

/// 制約テキストを返す (人間向けメイン表示)
/// 例: "1 ≤ N ≤ 2×10^5\n0 ≤ A_i ≤ 10^9"
#[wasm_bindgen]
pub fn render_constraints_text(document_json: &str) -> Result<String, JsError>

/// Constraint AST tree を返す (AST トグル用)
#[wasm_bindgen]
pub fn render_constraint_tree(document_json: &str) -> Result<String, JsError>

/// TeX を返す (入力形式 + 制約)
/// KaTeX でレンダリングする前の TeX 文字列
#[wasm_bindgen]
pub fn render_full_tex(document_json: &str) -> Result<String, JsError>

/// サンプルケースを返す (seed 指定)
/// 例: "5\n3 1 4 1 5"
/// seed は u32 範囲 (JS Number 互換、wasm-bindgen で BigInt 不要)
#[wasm_bindgen]
pub fn generate_sample(document_json: &str, seed: u32) -> Result<String, JsError>

/// プリセット一覧を返す (JSON array of {name: string, description: string})
#[wasm_bindgen]
pub fn list_presets() -> String

/// 指定プリセットの document JSON を返す
#[wasm_bindgen]
pub fn get_preset(name: &str) -> Result<String, JsError>
```

### Internal Implementation

各関数の内部処理:

1. `document_json` を `cp_ast_json::deserialize_ast()` で `AstEngine` に復元
2. `cp-ast-core` / `cp-ast-tree` の既存関数を呼び出し
3. 結果の `String` を返す

```rust
pub fn render_input_format(document_json: &str) -> Result<String, JsError> {
    let engine = cp_ast_json::deserialize_ast(document_json)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(cp_ast_core::render::render_input(&engine))
}
```

### Presets

`presets.rs` に代表的な AST をハードコードする。Rust コードで `AstEngine` を構築し、`cp_ast_json::serialize_ast()` で JSON 化して返す。

代表プリセット:
- `scalar_only` — スカラー値のみ (N)
- `scalar_array` — N + A_1..A_N
- `tuple_repeat` — N + (A_i, B_i) × N
- `matrix` — H×W 行列
- `choice` — タグによる分岐
- `graph_simple` — N 頂点 M 辺の単純グラフ
- `sorted_distinct` — ソート済み + 相異なる制約
- `string_problem` — 文字列制約 (CharSet, StringLength)
- `hole_structure` — Hole を含む不完全な AST

## 3. Frontend Components

### Page Structure

2 ページ構成。URL ルーティングは hash-based (`#/viewer`, `#/preview`) で十分。

#### Viewer Page (`/viewer`)

3 カラム + 下部ツールバー:

```
+-------------------------+-------------------------+-------------------------+
| 📝 入力形式       [AST] | 📋 制約           [AST] | Preview                 |
|                         |                         | [TeX] [Sample]          |
| render_input_format()   | render_constraints_text()| render_full_tex()      |
| or                      | or                      | or                      |
| render_structure_tree() | render_constraint_tree() | generate_sample(seed)  |
+-------------------------+-------------------------+-------------------------+
| Preset: [▼ scalar_array]  Seed: [____42____] [🔀 Shuffle]    5 nodes · 4 constraints |
+-----------------------------------------------------------------------------------+
```

**ペイン共通仕様:**
- 右上に `[AST]` トグルボタン
- デフォルト = 人間向け表示 (入力形式 / 制約テキスト)
- トグル ON = raw AST tree 表示
- 表示内容はプレーンテキストを `<pre>` で描画

**Preview ペイン:**
- TeX タブ: `render_full_tex()` の結果を KaTeX でレンダリング
- Sample タブ: `generate_sample(json, seed)` の結果を `<pre>` で表示

**ツールバー:**
- プリセットセレクタ (ドロップダウン)
- Seed 入力フィールド (直接入力可能)
- Shuffle ボタン (seed をランダムに変更)
- ステータス表示 (ノード数・制約数)

#### Preview Page (`/preview`)

カードギャラリー。全プリセットを一覧表示。

```
+----------------------------------+----------------------------------+
| 📝 Scalar + Array               | 📝 Tuple + Repeat               |
| ┌─Structure──┬──TeX───┐         | ┌─Structure──┬──TeX───┐         |
| │ N          │ N      │         | │ N          │ N      │         |
| │ A_1...A_N  │ A₁…Aₙ  │         | │ A_i B_i    │ A₁ B₁  │         |
| ├─Constraints─────────┤         | ├─Constraints─────────┤         |
| │ 1≤N≤2×10⁵           │         | │ 1≤N≤2×10⁵           │         |
| ├─Sample (seed=0)─────┤         | ├─Sample (seed=0)─────┤         |
| │ 5                   │         | │ 3                   │         |
| │ 3 1 4 1 5           │         | │ 1 5                 │         |
| └──────────────────────┘         | └──────────────────────┘         |
+----------------------------------+----------------------------------+
```

各カードに 4 系統 (Structure / Constraints / TeX / Sample) を表示。カードクリックで Viewer Page に遷移（当該プリセットをロード）。

### Component Hierarchy

```
App
├── Header (ページ切替: Viewer / Preview)
├── ViewerPage
│   ├── StructurePane
│   │   └── AST toggle, <pre> block
│   ├── ConstraintPane
│   │   └── AST toggle, <pre> block
│   ├── PreviewPane
│   │   ├── TexTab (KaTeX rendering)
│   │   └── SampleTab (<pre> block)
│   └── Toolbar
│       ├── PresetSelector
│       ├── SeedInput
│       ├── ShuffleButton
│       └── StatusBar
└── PreviewPage
    └── PreviewCard[] (grid of cards)
```

## 4. State Management

Preact Signals を使用。最小限の状態:

```typescript
// Viewer state
const documentJson = signal<string>("");       // 現在のドキュメント JSON
const activePreset = signal<string>("scalar_array");
const sampleSeed = signal<number>(0);
const activePreviewTab = signal<"tex" | "sample">("tex");
const structureAstMode = signal<boolean>(false); // AST toggle
const constraintAstMode = signal<boolean>(false); // AST toggle

// Derived (computed from wasm calls)
const structureText = computed(() => {
  if (!documentJson.value) return "";
  return structureAstMode.value
    ? wasm.render_structure_tree(documentJson.value)
    : wasm.render_input_format(documentJson.value);
});
// ... same pattern for constraints, tex, sample
```

## 5. TeX Rendering

- Rust 側 `render_full_tex()` が TeX 文字列を返す
- JS 側で KaTeX を使いレンダリング
- KaTeX は CDN から読み込み (`katex.min.js` + `katex.min.css`)
- `katex.renderToString(texString, { displayMode: true })` で HTML 化

```typescript
// TexTab.tsx
import katex from "katex";

function TexTab({ texString }: { texString: string }) {
  const html = useMemo(() => {
    try {
      return katex.renderToString(texString, {
        displayMode: true,
        throwOnError: false,
      });
    } catch {
      return `<pre>${texString}</pre>`; // fallback
    }
  }, [texString]);

  return <div dangerouslySetInnerHTML={{ __html: html }} />;
}
```

## 6. Sample Generation

- 初期 seed = 0
- `generate_sample(json, seed)` で生成
- Shuffle ボタン: `Math.floor(Math.random() * 2**32)` で新 seed 生成 (u32 範囲)
- Seed 入力フィールド: 直接数値入力 (0〜4294967295)、Enter で再生成
- 同じ seed → 同じ出力 (deterministic)

## 7. Build & Dev Workflow

### wasm ビルド

```bash
cd crates/cp-ast-wasm
wasm-pack build --target web --out-dir ../../web/wasm
```

`--out-dir` で `web/wasm/` に出力。Vite がこれを import する。

### Frontend 開発

```bash
cd web
npm install
npm run dev    # Vite dev server
npm run build  # production build
```

### Full build

```bash
# wasm ビルド → フロントエンドビルド
cd crates/cp-ast-wasm && wasm-pack build --target web --out-dir ../../web/wasm
cd ../../web && npm run build
```

### .gitignore

`web/wasm/` は wasm-pack の出力先のため `.gitignore` に追加する。`web/node_modules/` も同様。

```
web/wasm/
web/node_modules/
web/dist/
.superpowers/
```

## 8. Preview Page Presets

代表的な問題パターンをカバー:

| Name | Structure | Key Constraints |
|------|-----------|-----------------|
| `scalar_only` | N のみ | `1 ≤ N ≤ 10^9`, Int |
| `scalar_array` | N + A[N] | Range, TypeDecl |
| `tuple_repeat` | N + (A,B)×N | Repeat with Tuple |
| `matrix` | H×W grid | Matrix node |
| `choice` | T + 分岐 | Choice node |
| `graph_simple` | N,M + edges | Property(Simple), Distinct |
| `sorted_distinct` | N + sorted A[N] | Sorted, Distinct |
| `string_problem` | S (文字列) | CharSet, StringLength |
| `hole_structure` | 不完全 AST | Hole node |

## 9. Testing Strategy

### wasm レベル

- `wasm-pack test --headless --chrome` で wasm 関数の単体テスト
- 各 render 関数が空でない文字列を返すことを確認
- `generate_sample` の deterministic 性テスト

### フロントエンド レベル

- 手動目視確認を優先 (Phase 1 の本質)
- Preview Page が全プリセットを正しく表示するか
- Shuffle で seed が変わり出力が変わるか

### 統合確認

- wasm init 成功
- 全プリセットで 4 系統表示 (入力形式、制約、TeX、Sample) が動作
- KaTeX レンダリングが壊れていないか

## 10. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| wasm バイナリサイズが大きい | wasm-opt で最適化、必要なら lto = true |
| KaTeX が Rust TeX 出力を parse できない | fallback で生 TeX 表示。Rust TeX は `\paragraph{}`, `\begin{itemize}` を含むため KaTeX 非互換の可能性あり。該当箇所は生テキスト表示にフォールバック |
| rand (getrandom) が wasm で動かない | 既に cp-ast-core で `getrandom/js` feature 対応済み |
| wasm-pack のバージョン互換 | CI で wasm-pack バージョン固定 |

## 11. Out of Scope (Phase 1)

- AST 編集
- Action 送信 UI
- Projection-based guided editor
- Undo / Redo
- 差分同期
- 複数ドキュメント管理
- 永続化・保存機能
- 認証・サーバー API
- ノード選択による連動ハイライト
- 複雑な state 管理・最適化
- デザインの作り込み

## 12. Implementation Phases

### Phase 1: 土台

- cp-ast-wasm クレート作成 + wasm-pack ビルド確認
- web/ Preact + Vite 初期化
- wasm 読み込み + 1 関数呼び出し確認

### Phase 2: 3カラム Viewer

- StructurePane (入力形式 + AST toggle)
- ConstraintPane (制約テキスト + AST toggle)
- PreviewPane (TeX + Sample タブ)
- Toolbar (プリセット選択、Seed、Shuffle)

### Phase 3: Preview Page

- PreviewCard コンポーネント
- 全プリセットをカードギャラリーで表示
- カードから Viewer への遷移

### Phase 4: 仕上げ

- KaTeX レンダリング統合
- エラーハンドリング (wasm 関数失敗時の fallback)
- 目視確認による品質チェック
