# cp-ast-ecosystems: プロジェクト全体像

## ビジョン

競技プログラミング問題の入力仕様を **構造化 AST** として表現し、その単一モデルから複数の派生物を演繹的に生成するシステム。

自由テキストによる曖昧な仕様記述ではなく、意味構造を保持する AST を唯一の真実とし、そこから制約表記・入力表記・テストケース・TeX 出力などを安定的に導出する。

## アーキテクチャ

```
┌─────────────────────────────────────────────────┐
│                  GUI フロントエンド                │
│        （構造化エディタ — 不正状態を作れない）       │
└────────┬──────────────────────────┬──────────────┘
         │ Projection              │ Operation
         │ (次に何ができるか)        │ (状態遷移の実行)
         ▼                         ▼
┌─────────────────────────────────────────────────┐
│                  AST Core                        │
│  ┌───────────┐  ┌──────────────┐                │
│  │ Structure  │  │ Constraint   │                │
│  │ (入力構造) │  │ (制約群)      │                │
│  └───────────┘  └──────────────┘                │
└────────┬──────────┬──────────┬──────────────────┘
         │          │          │
    ┌────▼───┐ ┌───▼────┐ ┌──▼─────┐
    │ Render │ │ Sample │ │TeX Gen │  ← 派生物の生成
    │(text)  │ │(テスト) │ │(TeX)   │
    └────────┘ └────────┘ └────────┘
```

## コア層: AST Core

AST Core は入力仕様の **意味構造** を保持する中核。

### Structure（入力構造）

| NodeKind   | 説明                              |
|------------|----------------------------------|
| Scalar     | 単一変数（N, M, Q など）            |
| Tuple      | 同一行の複数要素（N M）             |
| Array      | 一次元配列（A_1 ... A_N）          |
| Matrix     | 二次元配列（A_{i,j}）              |
| Repeat     | 繰り返し構造（T_1 ... T_Q）        |
| Sequence   | 順序付き子ノード群                   |
| Section    | 名前付きセクション                   |
| Hole       | 未定義スロット（構造化エディタ用）     |
| Choice     | 分岐（クエリ問題等）                 |

- ノード間は `Reference` で参照（配列長 → 別スカラー等）
- Arena ベースの `StructureAst` で管理（`NodeId` で識別）

### Constraint（制約群）

| Constraint    | 例                                      |
|---------------|----------------------------------------|
| Range         | 1 ≤ N ≤ 2×10^5                        |
| TypeDecl      | N は整数                                |
| LengthRelation| \|S\| = N                               |
| Relation      | A < B                                   |
| Distinct      | A_i ≠ A_j (i ≠ j)                     |
| Sorted        | A_1 ≤ A_2 ≤ ... ≤ A_N                 |
| SumBound      | ΣN ≤ 2×10^5                           |
| Guarantee     | 「解は必ず存在する」等の自由文           |
| CharSet       | S は英小文字からなる                     |
| StringLength  | 1 ≤ \|S\| ≤ N                          |
| Property      | グラフは木である、等                     |
| RenderHint    | 表示補助情報                             |

- `Expression` で算術式を保持（Lit, Var, BinOp, Pow, FnCall）
- `ConstraintSet` で Arena 管理（`ConstraintId` で識別）

## 編集層: Projection / Operation

フロントエンドは **構造化エディタ** であり、自由テキスト入力ではない。

### Projection（読み取り）

現在の AST から「次に何ができるか」を返す。

- 存在するノード一覧
- 各スロットの状態（埋まっている / Hole）
- Hole に入れられるノード種別の候補
- 未完成箇所の検出

### Operation（状態遷移）

AST に対する合法な変更操作。

- FillHole — Hole を具体ノードで埋める
- ReplaceNode — ノードの差し替え
- AddConstraint / RemoveConstraint — 制約の追加・削除
- AddSlotElement / RemoveSlotElement — スロット要素の操作
- IntroduceMultiTestCase — マルチテストケース化

Operation は常に整合状態を保証し、GUI 上で不正状態を構築不能にする。

## 派生物層: Render / Sample / TeX

AST と制約の意味整合が保証されていれば、バックエンドで以下を **演繹的に** 生成できる。

### Render（テキスト表現）

- `render_input_format` — 入力形式のプレーンテキスト
- `render_constraints_text` — 制約のプレーンテキスト

### Render TeX（TeX 表現）

競プロ問題で一般的な TeX 断片を決定的に生成する。

- `render_input_tex` — 入力形式の TeX（`\begin{array}` レイアウト）
- `render_constraints_tex` — 制約の TeX（`\begin{itemize}` リスト）
- `render_full_tex` — 入力 + 制約をまとめた断片（Fragment / Standalone モード）

特性: 決定性・差分安定性・競プロ表記との親和性・Hole 安全

### Sample（テスト生成）

- 依存グラフ解析（どの変数が先に決まるか）
- 制約に基づくランダム値生成
- テキスト形式での出力

## 将来の拡張

この土台が固まれば、同じ AST から以下も導出可能:

- ランダムテスト生成の高度化（境界値、コーナーケース）
- Statement generator（問題文テンプレートへの接続）
- Web UI プレビュー（WASM 経由の TeX レンダリング）
- AI Agent の補助出力
- PDF ビルドパイプライン
- 出力表記・注意書きの TeX 生成

## 設計原則

1. **AST が唯一の意味モデル** — 表示と意味のズレを防ぐ
2. **バックエンド支配** — 表示規則・整形ロジックはフロントに持たせない
3. **演繹的生成** — 個別の ad-hoc ロジックではなく、単一モデルからの導出
4. **Hole は第一級市民** — 未完成状態でもシステムは壊れない
5. **決定性と差分安定** — 同じ AST なら毎回同じ出力、golden test と相性良好
