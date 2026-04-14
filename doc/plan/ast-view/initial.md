# cp-ast-ecosystems Frontend Phase 1 Viewer - Design Brief for AI Agent

## 1. 目的

本タスクの目的は、`cp-ast-core` をブラウザ上で **可視化する最初のフロントエンド** を作ることである。

この段階では、まだ AST を編集できなくてよい。まずは以下を満たすことを目標にする。

- Rust -> wasm -> JS の接続が成立している
- AST が lossless に JS 側へ渡せている
- HTML 上に正しく表示できている
- Structure AST / Constraint / TeX / Sample Case を 1 画面で確認できる
- sample case は seed を変えて shuffle 的に再生成できる
- 人間が目で確認するための preview ページがある

このフェーズの本質は、**編集機能ではなく viewer の成立確認** である。

---

## 2. 前提

- `cp-ast-core` はすでに存在する
- `cp-ast-core` には `constraint`, `operation`, `projection`, `render`, `render_tex`, `sample`, `structure` がある
- `AstEngine` は `StructureAst` と `ConstraintSet` を所有する
- 既存で TeX 生成と sample 生成は進んでいる
- AST tree viewer の既存設計がある
- 今回は projection / action による編集 UI にはまだ踏み込まない
- 今回は「AST を受け取って表示すること」が最優先である

---

## 3. スコープ

### 3.1 含むもの

- Preact による viewer UI
- wasm 経由で Rust の AST を JS 側へ渡す
- Structure AST の tree 表示
- Constraint の一覧表示
- TeX preview 表示
- Sample Case preview 表示
- seed 指定 or shuffle による sample 再生成
- あらかたの制約パターンを含む 1 ページ preview

### 3.2 含まないもの

- AST 編集
- action 送信 UI
- projection ベースの guided editor
- undo / redo
- 差分同期
- 複数ドキュメント管理
- 永続化
- 認証
- サーバー API

---

## 4. このフェーズの成功条件

このフェーズは、以下が満たされれば成功とみなす。

### 4.1 wasm 接続成功

- Rust 側で保持している AST document を wasm 経由で JS に渡せる
- JS 側から Rust の関数を呼べる
- Rust -> JS -> Rust の基本往復が成立している

### 4.2 viewer 表示成功

- Structure AST が tree として見える
- Constraint が node ごと or grouped list として見える
- TeX が preview として見える
- Sample Case が表示される

### 4.3 sample shuffle 成功

- seed を変えると sample が再生成される
- 同じ seed なら同じ sample が出る
- UI から shuffle 的に再試行できる

### 4.4 preview ページ成功

- 人が一通り眺めるための sample page がある
- 代表的な `NodeKind` / `Constraint` の例が入っている
- AST / Constraint / TeX / Sample が 1 画面で確認できる

---

## 5. UI の基本方針

このフェーズの UI は **editor ではなく viewer** である。

したがって、操作の中心は次だけに絞る。

- ノード選択
- preview タブ切り替え
- sample seed 変更
- shuffle ボタン

編集フォームや action ボタンはまだ不要である。

---

## 6. 推奨スタック

### フロントエンド

- Preact
- TypeScript
- Vite

### Rust 連携

- wasm-bindgen ベース
- JSON string API でも JS object API でもよいが、初期は単純さ優先でよい

### スタイリング

- まずは素の CSS または CSS Modules
- UI ライブラリ依存は最小限にする

### 状態管理

- Preact Signals もしくは `useState`
- Redux などの大規模状態管理は不要

---

## 7. 画面構成

推奨する初期レイアウトは 4 ペイン構成。

```text
+----------------------+----------------------+----------------------+
| Structure AST        | Constraints          | Preview              |
|                      |                      | [TeX] [Sample]       |
| tree                 | grouped list         |                      |
|                      |                      | preview content      |
+----------------------+----------------------+----------------------+
| Toolbar / Status / Seed / Shuffle                                  |
+--------------------------------------------------------------------+
```

### ペイン 1: Structure AST

- tree 表示
- ノード選択可能
- hole を強調表示
- `NodeId` 表示は debug option として持ってよい

### ペイン 2: Constraints

- node 単位で grouped list
- global constraint も別セクション表示
- Structure AST で選択中ノードに対応する constraint を強調できるとよい

### ペイン 3: Preview

タブで切り替える。

- TeX preview
- Sample Case preview

### 下部バー

- 現在 seed
- shuffle button
- 必要なら status / diagnostics

## 8. 表示方針

### 8.1 Structure AST

- tree として表示
- 既存の tree viewer 設計を踏襲してよい
- `NodeKind` に応じた label は Rust 側で生成してもよい
- フロントで `NodeKind` を過剰に解釈しない

例:

```text
Scalar(N)
Array(A, len=N)
Repeat(count=M)
Choice(tag=T)
Hole
```

### 8.2 Constraint

- tree でなく grouped list でよい
- top level は target node 単位
- second level は個別 constraint

例:

```text
N
Range: 1 <= N <= 2e5
TypeDecl: Int

A
Range: 0 <= A_i <= 1e9

(global)
Property: Simple graph
```

### 8.3 TeX preview

- そのまま text / code block として表示でもよい
- 初期段階では live rendering までは不要
- まずは Rust の tex 出力が HTML 上で見えることを確認する

### 8.4 Sample preview

- 現在 seed に対する sample を text で表示
- shuffle で seed を更新して再生成
- 同一 seed の deterministic 性が確認できるようにする

## 9. 連動仕様

最初の段階で必要な連動は最小限でよい。

### 必須

- Structure AST で node を選ぶ
- Constraints ペインで対応箇所が分かる

### あると嬉しい

- 選択ノードに関係する constraint を絞る
- 選択ノードが TeX のどこに効いているかを将来的に見せる余地を残す

### まだ不要

- TeX 文字列内の位置ハイライト
- Sample 内の対応箇所ハイライト
- 双方向選択

## 10. wasm 境界の方針

このフェーズでは projection / action ではなく、まず **AST を渡して描画に必要な派生物を取る** ことを目的にする。

最低限必要な wasm API 候補は以下。

### 10.1 document 読み込み

- `load_document(json)` または `from_json(json)`

### 10.2 tree / listing 生成

- `render_structure_tree(document_json) -> string`
- `render_constraint_tree(document_json) -> string`
- 必要なら `render_combined_tree(document_json) -> string`

### 10.3 tex 生成

- `render_input_tex(document_json) -> string`
- `render_constraints_tex(document_json) -> string`

### 10.4 sample 生成

- `generate_sample(document_json, seed) -> string`

ここでは、JS 側で AST を深く解釈させるより、**Rust 側で文字列表現や DTO を返す** 方がよい。

## 11. データの持ち方

初期段階では、フロント state は最小でよい。

```ts
type ViewerState = {
  documentJson: string;
  selectedNodeId?: string;
  activePreviewTab: "tex" | "sample";
  sampleSeed: number;
  structureTreeText: string;
  constraintTreeText: string;
  texText: string;
  sampleText: string;
};
```

重要なのは、まず wasm 接続と表示の成立であり、洗練された state model ではない。

## 12. preview ページ

このフェーズでは、実用 UI だけでなく **確認用 preview page** が重要である。

目的:

- 人が AST / constraint / tex / sample の整合を目視確認する
- 代表的パターンを 1 ページで確認できる
- wasm 接続成功を可視化する

preview に含めるべき例:

少なくとも以下の代表ケースを入れる。

- scalar only
- scalar + array
- tuple + repeat
- choice
- hole を含む構造
- matrix
- sorted / distinct / property など代表 constraint を含む例

preview page の形:

- 1 ページに複数ケースを cards で並べる
- 各 case ごとに以下を表示する
- Structure AST
- Constraints
- TeX
- Sample

## 13. 実装優先順位

### Phase 1

- Preact + Vite の土台作成
- wasm 初期化確認
- 固定 AST を Rust から受けて画面表示

### Phase 2

- Structure AST tree 表示
- Constraint list 表示

### Phase 3

- TeX preview 表示
- Sample preview 表示

### Phase 4

- seed 入力
- shuffle button
- deterministic sample 再生成

### Phase 5

- preview page 作成
- 代表ケースを複数並べる

## 14. テスト方針

### 技術確認

- wasm 初期化できるか
- Rust 関数が JS から呼べるか
- AST document を渡しても壊れないか

### UI 確認

- Structure AST が表示される
- Constraint が表示される
- TeX が表示される
- Sample が表示される

### sample 確認

- 同じ seed で同じ output
- seed を変えると output が変わる

### preview 確認

- 代表ケースが 1 ページにまとまっている
- 人間が比較しやすい

## 15. 非目標

このフェーズでは次をやらない。

- AST 編集
- Action UI
- projection-driven editor
- 保存機能
- ネットワーク同期
- 差分更新
- 複雑な state 管理
- 最適化
- デザインの作り込み

## 16. AI Agent への依頼

以下を設計・実装してください。

- Preact + Vite + wasm の最小 viewer を構築する
- Rust 側の AST / tex / sample を JS から呼び出せるようにする
- Structure AST / Constraints / TeX / Sample を 1 画面で確認できるようにする
- shuffle button と seed による sample 再生成を入れる
- representative preview page を作る
- まだ編集機能は入れない
- フロント側で AST の意味論を再実装しない
- まずは viewer と wasm 成功確認を優先する

## 17. 出力してほしい成果物

AI Agent には少なくとも次を出してほしい。

- ディレクトリ構成案
- コンポーネント構成案
- wasm bridge の API 案
- preview page の構成案
- representative sample document 一覧
- 実装順序
- 想定リスク
- 後回し項目

## 18. 最終判断

このフェーズは editor を作る段階ではない。最初に作るべきなのは、以下である。

- Rust -> wasm -> JS の接続確認
- AST / Constraint / TeX / Sample の 4 系統表示
- 人間が整合を確認できる preview page

つまり、最初のフロントエンドは viewer であり、editor ではない。

この方針で、UI の初期段階を進めること。
