# 競技プログラミング問題記述 DSL — 最終設計文書

> Phase 1 (Sprint 1–3) 全出力の統合
>
> 入力: site-survey-report.md, research-alignment-report.md, domain-model.md [Rev.1], projection-operation.md [Rev.1], critical-review.md, coverage-validation.md, sample-generation.md, initial-prompt.md

---

## 第 1 章　Executive Summary

本設計は、**競技プログラミング問題の形式仕様を構造的に表現・編集・検証するための DSL（Domain-Specific Language）** を定義する。コアは 4 モジュール ── StructureAST, ConstraintAST, ProjectionAPI, Operation ── から構成され、Rust で実装する。

**設計の特徴:**

- **Structure-first**: テキストではなく AST（抽象構文木）を正本とする。Projectional editing のアーキテクチャに従い、テキストや GUI は投影（projection）に過ぎない。
- **構造と制約の明示的分離**: 「何がどこにあるか」（StructureAST）と「何が許されるか」（ConstraintAST）を独立した層として管理する。
- **三方向投影**: 単一の AST から (1) canonical な仕様テキスト、(2) 制約充足するランダムサンプルケース、(3) 編集可能な GUI/CLI を導出する。
- **Hole 第一級**: Hazelnut (Omar et al. 2017) に倣い、未完成状態を第一級のノードとして扱い、すべての編集状態が構造的に有意味であることを保証する。

**実証状況:**

- AtCoder を中心に 18 問を検証。StructureAST のカバー率は 72.2%（完全対応）/ 94.4%（部分対応以上）。唯一の非対応はインタラクティブ問題（Phase 2 に延期）。
- Rev.1 改訂で重大欠陥 3 件（S-1: 型情報二重管理, S-2: 制約 ID 欠如, S-3: FillContent 肥大化）を修正済み。
- 先行研究との関係を整理し、Hazelnut / MPS / QuickCheck 等との差異と、「競プロ仕様記述への構造エディタ概念の適用」という新規性の範囲を明確化済み。

**今すぐ実装すべき最小コア:** 9 NodeKind + 12 Constraint + Arena ベース AST + 基本 Operation（FillHole, AddConstraint, RemoveConstraint）+ 依存グラフ駆動の Range/Int 限定サンプル生成器。

---

## 第 2 章　観察した競プロサイトの構造的特徴

> 出典: doc/survey/site-survey-report.md — AtCoder 14 問のサーベイ

### 2.1 入力構造の共通パターン

サイト調査で確認された 7 つの入力パターン:

| ID | パターン | 概要 | 出現頻度 |
|----|----------|------|----------|
| I-1 | ヘッダ行 + データ行 | `N [M] …` + 変数依存行数のデータ | **最頻出** |
| I-2 | グリッド入力 | `H W` + H 行の文字/数値グリッド | 高 |
| I-3 | グラフ入力（辺リスト） | `N M` + M 行の辺 `u_i v_i [w_i]` | 高 |
| I-4 | クエリ形式 | `Q` + Q 行のクエリ（型コード分岐あり） | 中 |
| I-5 | 複数テストケース | `T` + T 回のケース（総和制約付き） | 中 |
| I-6 | 複合セクション | グラフ + 条件リスト等の独立セクション | 中 |
| I-7 | インタラクティブ | 双方向 I/O（`?` で質問、`!` で回答） | 低 |

**この設計で表現できる問題の例:** ABC284-C（I-3 グラフ入力）は Sequence > Tuple(N,M) + Repeat(M, Tuple(u,v)) として自然に表現される。

### 2.2 制約の共通パターン

| ID | パターン | 具体例 |
|----|----------|--------|
| C-1 | 値域制約 | `1 ≤ N ≤ 300` |
| C-2 | 型宣言 | 「入力は全て整数」 |
| C-3 | 導出制約 | `0 ≤ M ≤ N(N-1)/2` |
| C-4 | 変数間関係 | `1 ≤ u_i < v_i ≤ N` |
| C-5 | 構造的性質 | 「グラフは単純かつ連結」「A は順列」 |
| C-6 | 総和制約 | 「N の総和は 3×10^5 以下」 |
| C-7 | 存在保証 | 「A+B=C_i なる i が丁度 1 つ存在する」 |

### 2.3 出力形式の多様性

単一整数、Yes/No、空白区切り列、クエリ応答、条件分岐出力、可変行数操作列、mod 出力、インタラクティブの 8 パターンを確認。

### 2.4 表記揺れ・例外ケース

- 添字表記の揺れ: `C_i` vs `C[i][j]` vs `A_{i,j}`（同一コンテスト内でも不統一）
- 三角行列入力（abc371_c）: 行ごとに列数が減少する特殊構造
- 複数グリッドの区切りなし連結（abc300_b）: パーサ依存の暗黙分割
- 文字グリッドは無区切り、数値行列は空白区切りという暗黙規則

### 2.5 DSL で押さえるべき最小共通核

1. **Header + RepeatLines + Line** — 全問題の 90%以上をカバー
2. **Grid + CharSeq** — 2D グリッド問題に必須
3. **Range + TypeDecl + Relation** — 制約の 80% をカバー
4. **Section 分割** — 複合入力問題に必要
5. **multi_testcase / Choice** — 複数テストケース・クエリ分岐

---

## 第 3 章　設計目標と非目標

### 3.1 設計目標（Goals）

| # | 目標 | 根拠 |
|---|------|------|
| G-1 | 4 モジュールの責務分離が明確であること | initial-prompt.md 中核要件 |
| G-2 | 競プロサイトの現実の問題記述に対して適用可能であること | 18 問のカバレッジ検証で実証 |
| G-3 | AST から canonical な制約・入力形式テキストを再構成可能 | structure-first 原則 |
| G-4 | AST から制約充足するランダムサンプルケースを生成可能 | sample generation 要件 |
| G-5 | GUI/CLI/AI Agent が同一の API で操作可能 | ProjectionAPI + Operation の統一 |
| G-6 | 未完成状態が第一級であり、段階的構築が可能 | Hole 設計方針（Hazelnut に倣う） |
| G-7 | Rust で実装し、型安全性を活用 | 実装言語要件 |

### 3.2 非目標（Non-Goals）

| # | 非目標 | 理由 |
|---|--------|------|
| NG-1 | 問題文全文の自然言語 DSL | 本 DSL は形式仕様 DSL であり、自然言語本文は対象外 |
| NG-2 | パーサの実装（Phase 1） | structure-first; テキスト→AST 変換は将来拡張 |
| NG-3 | インタラクティブ問題の完全対応 | 双方向 I/O は静的構造木モデルと本質的に異なる（Phase 2） |
| NG-4 | 全制約の充足保証 | Guarantee 等の一般述語は決定不能; best-effort + 警告 |
| NG-5 | GUI レイアウト・描画の設計 | GUI は Projection の薄い投影; UI の関心は View 層 |
| NG-6 | Hazelnut 級の形式的証明 | 形式化は Phase 2 以降の課題; Phase 1 はテストベースで保証 |
| NG-7 | 全サイト横断の統一対応 | まず AtCoder の ABC 問題を最優先ターゲットとする |

### 3.3 先行研究との関係

本設計は新しい理論を提案するものではなく、以下の既存概念の**ドメイン適用**である:

- **Projectional editing** (Voelter 2015; JetBrains MPS): AST を正本とし、テキストを投影とするアーキテクチャ
- **Hazelnut** (Omar et al. 2017): Hole を第一級に扱う構造エディタ理論
- **QuickCheck** (Claessen & Hughes 2000): 制約からのランダムテスト生成

**新規性が主張できる範囲:**
- 競プロ仕様記述への構造エディタ概念の適用（ドメイン適用としての新規性）
- 単一 AST からの三方向投影（rendering + generation + editing の統合）
- StructureAST / ConstraintAST の二層分離によるドメイン固有の柔軟性

**言いすぎになること（research-alignment-report より）:**
- 「AST を正本にする新しいアーキテクチャ」→ projectional editing の定義そのもの
- 「Hole を第一級に扱う新しいアプローチ」→ Hazelnut の応用
- 「不正状態を事前排除する新しい原則」→ Hazelnut の Sensibility 定理

---

## 第 4 章　4 モジュールの責務分離

### 4.1 モジュール概要

```
┌─────────────────────────────────────────────────┐
│                   View / Client                  │
│            (GUI / CLI / AI Agent / Test)          │
└─────────────┬───────────────┬────────────────────┘
              │ read-only     │ Action
              ▼               ▼
┌─────────────────┐   ┌─────────────────┐
│  ProjectionAPI  │   │    Operation    │
│   （問い合わせ）  │   │  （状態遷移）   │
└────────┬────────┘   └───────┬─────────┘
         │                    │
         ▼                    ▼
┌─────────────────────────────────────────┐
│         StructureAST + ConstraintAST     │
│               （正本データ）               │
└─────────────────────────────────────────┘
```

### 4.2 各モジュールの責務

| モジュール | 役割 | 責務 | してはならないこと |
|------------|------|------|-------------------|
| **StructureAST** | 形 | 問題仕様の構造的正本を保持 | 型情報・制約情報を持つこと（S-1 修正済み） |
| **ConstraintAST** | 許容条件 | 構造上の各位置に対する制約を保持 | 構造の変更; 制約の整合性検証（検証は Operation） |
| **ProjectionAPI** | 読み取り像 | AST から read-only な像を導出 | 状態変更; Undo/Redo; UI レイアウト |
| **Operation** | 状態遷移 | 編集操作の検証・適用・新状態返却 | UI 固有のロジック; AI の判断 |

### 4.3 追加層: Builder 層

Critical Review S-3 の指摘を受け、FillContent（EdgeList, QueryList 等の高レベル操作）は **Builder 層**に分離する:

```
[ユーザー / AI] → 高レベル意図（EdgeList を追加）
[Builder 層]    → Action 列に展開（FillHole + AddSlotElement + AddConstraint の列）
[Operation 層]  → 各 Action を NodeKind レベルで検証・適用
```

Builder 層はドメイン知識（グラフ構造の展開ロジック等）を局所化し、Operation 層を NodeKind の整合性検証に集中させる。

**この設計で表現できる問題の例:** ABC244-E のヘッダ `N M K S T X` + M 行の辺リストは、Builder 層が `build_graph_section(header_vars=6, edge_count=M, weighted=false)` を Action 列に展開し、Operation 層が各 Action を順次適用する。

---

## 第 5 章　StructureAST 設計

### 5.1 NodeKind 一覧（Rev.1）

型情報（`typ` / `element_type`）と区切り文字（`separator`）は ConstraintAST に移動済み。StructureAST は純粋に「何がどこにあるか」のみを表現する。

| # | NodeKind | Slot | 対応する競プロ概念 |
|---|----------|------|--------------------|
| 1 | **Scalar** | `name: Ident` | パラメータ変数 N, M, S |
| 2 | **Array** | `name: Ident`, `length: Reference` | 配列 A_1…A_N |
| 3 | **Matrix** | `name: Ident`, `rows: Reference`, `cols: Reference` | H×W グリッド |
| 4 | **Tuple** | `elements: Vec<NodeId>` | 同一行変数組 (N, M, K) |
| 5 | **Repeat** | `count: Reference`, `body: Vec<NodeId>` | M 行の繰り返し |
| 6 | **Section** | `header: Option<NodeId>`, `body: Vec<NodeId>` | 複合セクション |
| 7 | **Sequence** | `children: Vec<NodeId>` | 入力全体のルート |
| 8 | **Choice** | `tag: Reference`, `variants: Vec<(Literal, Vec<NodeId>)>` | クエリ種別分岐 |
| 9 | **Hole** | `expected_kind: Option<NodeKindHint>` | 未完成位置 |

### 5.2 Rust 型定義

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// Arena ベースの AST（M-1 対応: HashMap → Vec）
pub struct StructureAst {
    pub root: NodeId,
    pub arena: Vec<Option<StructureNode>>,  // NodeId(n) → arena[n]
    pub next_id: u64,
}

pub struct StructureNode {
    pub id: NodeId,
    pub kind: NodeKind,
}

pub enum NodeKind {
    Scalar { name: Ident },
    Array { name: Ident, length: Reference },
    Matrix { name: Ident, rows: Reference, cols: Reference },
    Tuple { elements: Vec<NodeId> },           // Rev.1: Vec<Reference> → Vec<NodeId>
    Repeat { count: Reference, body: Vec<NodeId> },
    Section { header: Option<NodeId>, body: Vec<NodeId> },
    Sequence { children: Vec<NodeId> },
    Choice { tag: Reference, variants: Vec<(Literal, Vec<NodeId>)> },  // Rev.1 追加
    Hole { expected_kind: Option<NodeKindHint> },  // Rev.1: hole_id 除去
}

pub enum Reference {
    VariableRef(NodeId),
    IndexedRef { target: NodeId, indices: Vec<Ident> },
    Unresolved(Ident),
}
```

### 5.3 具体例: ABC284-C（グラフ連結成分数）

入力形式: `N M` / `u_1 v_1` … `u_M v_M`

```
Sequence [id=0]
├── Tuple [id=1, elements=[id=2, id=3]]
│   ├── Scalar [id=2, name="N"]
│   └── Scalar [id=3, name="M"]
└── Repeat [id=4, count=Ref(id=3), body=[id=5]]
    └── Tuple [id=5, elements=[id=6, id=7]]
        ├── Scalar [id=6, name="u"]
        └── Scalar [id=7, name="v"]
```

### 5.4 具体例: AGC062-A（複数テストケース）

入力形式: `T` / (N, S) × T 回

```
Sequence [id=0]
├── Scalar [id=1, name="T"]
└── Repeat [id=2, count=Ref(id=1), body=[id=3]]
    └── Section [id=3, header=None, body=[id=4, id=5]]
        ├── Scalar [id=4, name="N"]
        └── Scalar [id=5, name="S"]
```

### 5.5 具体例: ABC278-D（3種クエリ — Choice の使用）

入力形式: `N` / `A_1…A_N` / `Q` / クエリ行（型コード 1/2/3 で構造が変化）

```
Sequence [id=0]
├── Scalar [id=1, name="N"]
├── Array [id=2, name="A", length=Ref(id=1)]
├── Scalar [id=3, name="Q"]
└── Repeat [id=4, count=Ref(id=3), body=[id=5]]
    └── Choice [id=5, tag=Ref(id=6),
          variants=[
            (1, [Scalar("x")]),              // 1 x
            (2, [Scalar("i"), Scalar("x")]), // 2 i x
            (3, [Scalar("i")])               // 3 i
          ]]
```

---

## 第 6 章　ConstraintAST 設計

### 6.1 Constraint 一覧（Rev.1）

TypeDecl が型情報の Single Source of Truth。CharSet, StringLength, RenderHint を追加し、文字列問題と表示制御に対応。

| # | Constraint | 何を制約するか | 具体例 |
|---|-----------|---------------|--------|
| 1 | **Range** | 変数の値域 | `1 ≤ N ≤ 2×10^5` |
| 2 | **TypeDecl** | 変数の型（唯一の情報源） | `N は整数`, `S は文字列` |
| 3 | **LengthRelation** | 配列長/文字列長と変数の関係 | `len(A) = N` |
| 4 | **Relation** | 変数間の比較・不等式 | `u_i < v_i`, `M ≤ N(N-1)/2` |
| 5 | **Distinct** | 要素の相異条件 | `C_i は相異なる` |
| 6 | **Property** | 構造全体の性質タグ | `グラフは単純かつ連結`, `A は順列` |
| 7 | **SumBound** | テストケース横断の総和 | `N の総和は 3×10^5 以下` |
| 8 | **Sorted** | ソート済み条件 | `p_1 < p_2 < … < p_K` |
| 9 | **Guarantee** | 存在保証・妥当性保証 | `解が存在することは保証される` |
| 10 | **CharSet** | 文字集合制約 | `英小文字のみ`, `{o, x, .}` |
| 11 | **StringLength** | 文字列長制約 | `1 ≤ |S| ≤ 1000` |
| 12 | **RenderHint** | 表示フォーマット | `separator=Space`, `separator=None` |

### 6.2 Rust 型定義

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintId(pub u64);

/// Rev.1: ConstraintId によるアドレッシング。RemoveConstraint が機能する。
pub struct ConstraintSet {
    pub arena: Vec<Option<(ConstraintId, Constraint)>>,
    pub by_node: Vec<(NodeId, Vec<ConstraintId>)>,
    pub global: Vec<ConstraintId>,
    pub next_id: u64,
}

pub enum Constraint {
    Range { target: Reference, lower: Expression, upper: Expression },
    TypeDecl { target: Reference, expected: ExpectedType },
    LengthRelation { target: Reference, length: Expression },
    Relation { lhs: Expression, op: RelationOp, rhs: Expression },
    Distinct { elements: Reference, unit: DistinctUnit },
    Property { target: Reference, tag: PropertyTag },
    SumBound { variable: Reference, upper: Expression },
    Sorted { elements: Reference, order: SortOrder },
    Guarantee { description: String, predicate: Option<Expression> },
    CharSet { target: Reference, charset: CharSetSpec },
    StringLength { target: Reference, min: Expression, max: Expression },
    RenderHint { target: Reference, hint: RenderHintKind },
}

pub enum CharSetSpec {
    LowerAlpha, UpperAlpha, Alpha, Digit, AlphaNumeric,
    Custom(Vec<char>), Range(char, char),
}

pub enum PropertyTag {
    Simple, Connected, Tree, Permutation, Binary, Odd, Even,
    Custom(String),
}

pub enum Expression {
    Lit(i64),
    Var(Reference),
    BinOp { op: ArithOp, lhs: Box<Expression>, rhs: Box<Expression> },
    Pow { base: Box<Expression>, exp: Box<Expression> },
    FnCall { name: Ident, args: Vec<Expression> },
}
```

### 6.3 具体例: ABC284-C の制約表現

```rust
// 1 ≤ N ≤ 100
Range { target: Ref(N), lower: Lit(1), upper: Lit(100) }

// 0 ≤ M ≤ N(N-1)/2
Range { target: Ref(M), lower: Lit(0),
        upper: BinOp(Div, BinOp(Mul, Var(N), BinOp(Sub, Var(N), Lit(1))), Lit(2)) }

// 1 ≤ u_i < v_i ≤ N
Relation { lhs: Var(Ref(u)), op: Lt, rhs: Var(Ref(v)) }
Range { target: Ref(u), lower: Lit(1), upper: Var(Ref(N)) }
Range { target: Ref(v), lower: Lit(1), upper: Var(Ref(N)) }

// N, M は整数
TypeDecl { target: Ref(N), expected: Int }
TypeDecl { target: Ref(M), expected: Int }

// グラフは単純
Property { target: Ref(edge_list), tag: Simple }

// 入力は全て整数
TypeDecl { target: Ref(u), expected: Int }
TypeDecl { target: Ref(v), expected: Int }
```

### 6.4 具体例: AGC062-A の制約表現（複数テストケース + 文字列）

```rust
// T の範囲
Range { target: Ref(T), lower: Lit(1), upper: Pow(Lit(10), Lit(5)) }

// N の範囲
Range { target: Ref(N), lower: Lit(2), upper: BinOp(Mul, Lit(3), Pow(Lit(10), Lit(5))) }

// S の型・長さ・文字集合
TypeDecl { target: Ref(S), expected: Str }
LengthRelation { target: Ref(S), length: Var(Ref(N)) }
CharSet { target: Ref(S), charset: Custom(vec!['A', 'B']) }

// N の総和制約
SumBound { variable: Ref(N), upper: BinOp(Mul, Lit(3), Pow(Lit(10), Lit(5))) }
```

---

## 第 7 章　ProjectionAPI 設計

### 7.1 中核 trait 定義

```rust
/// read-only な像を導出する。GUI/CLI/AI Agent/テストすべてがこの trait を通じて状態を読む。
pub trait ProjectionAPI {
    fn nodes(&self) -> Vec<ProjectedNode>;
    fn children(&self, node: NodeId) -> Vec<SlotEntry>;
    fn inspect(&self, node: NodeId) -> Option<NodeDetail>;
    fn hole_candidates(&self, hole: NodeId) -> Vec<CandidateKind>;
    fn available_actions(&self) -> Vec<AvailableAction>;
    fn why_not_editable(&self, node: NodeId) -> Option<NotEditableReason>;
    fn completeness(&self) -> CompletenessSummary;
}

/// AI Agent 向け補助（全クライアントからアクセス可能、特権なし）
pub trait ProjectionQueryAPI: ProjectionAPI {
    fn ancestors(&self, node: NodeId) -> Vec<NodeId>;
    fn prioritized_holes(&self) -> Vec<PrioritizedHole>;
    fn serialize_as_text(&self) -> String;
    fn render_problem_text(&self) -> String;
}
```

### 7.2 Projection の責務境界

| Projection が持つべきもの | Projection が持つべきでないもの |
|--------------------------|-------------------------------|
| ノードのラベル生成 | AST の変更操作（→ Operation） |
| Hole の候補カテゴリ列挙 | 制約の整合性検証（→ Operation） |
| 実行可能 Action 列挙 | Undo/Redo 管理（→ Operation） |
| 編集不可理由の導出 | UI レイアウト計算（→ View） |
| 完成度サマリ | ファイル I/O（→ 外部） |
| 制約の人間可読表現 | パーサ（→ 将来モジュール） |

**判定基準:**
1. 状態を変えるか？ → Yes なら Operation
2. 現在の状態だけで計算できるか？ → Yes なら Projection 候補
3. 全クライアントで共通か？ → Yes なら Projection、No なら View 側
4. キャッシュ可能か？ → Projection は AST 不変な限り同一結果を返すべき

### 7.3 具体例: ABC284-C の Projection

```
nodes() の返却:
  Root         "Problem: abc284_c"        depth=0
  Section      "Input"                    depth=1
  Header       "Header: N M"             depth=2
  ScalarVar    "N"                        depth=3
  ScalarVar    "M"                        depth=3
  EdgeList     "EdgeList(M edges)"        depth=2
  Section      "Output"                   depth=1
  Hole         "???"                      depth=2  ← is_hole=true

hole_candidates(Hole) → [IntroduceScalar { names: ["ans"] }]

completeness() → { total_holes: 1, filled: 0, is_complete: false }
```

**この設計で表現できる問題の例:** 上記の ABC284-C 構築途中の状態では、Output の Hole が残っており、`completeness().is_complete == false`。FillHole で単一整数出力を指定すると完成状態に遷移する。

---

## 第 8 章　Operation 設計

### 8.1 Action enum

```rust
pub enum Action {
    FillHole { target: NodeId, fill: FillContent },
    ReplaceNode { target: NodeId, replacement: FillContent },
    AddConstraint { target: NodeId, constraint: ConstraintDef },
    RemoveConstraint { constraint_id: ConstraintId },    // Rev.1: ConstraintId
    IntroduceMultiTestCase { count_var_name: String, sum_bound: Option<SumBoundDef> },
    AddSlotElement { parent: NodeId, slot_name: String, element: FillContent },
    RemoveSlotElement { parent: NodeId, slot_name: String, child: NodeId },
}
```

### 8.2 Operation trait

```rust
pub trait Operation {
    /// Action を適用し、新しい AST 状態を返す。
    fn apply(&mut self, action: Action) -> Result<ApplyResult, OperationError>;
    /// dry-run（Rev.1: ProjectionQueryAPI から移動、M-2 対応）
    fn preview(&self, action: &Action) -> Result<PreviewResult, OperationError>;
}

pub struct ApplyResult {
    pub created_nodes: Vec<NodeId>,
    pub removed_nodes: Vec<NodeId>,
    pub created_constraints: Vec<ConstraintId>,   // Rev.1 追加
    pub affected_constraints: Vec<ConstraintId>,
}
```

### 8.3 OperationError

```rust
pub enum OperationError {
    TypeMismatch { expected: ExpectedType, actual: String, context: String },
    NodeNotFound { node: NodeId },
    SlotOccupied { node: NodeId, current_occupant: String },
    ConstraintViolation { violated_constraints: Vec<ViolationDetail> },
    InvalidOperation { action: String, reason: String },
}
```

### 8.4 各 Action の具体例

**FillHole (ABC300-A):**
```rust
Action::FillHole {
    target: NodeId(5),  // ヘッダの3番目の hole
    fill: FillContent::Scalar { name: "B".into(), typ: VarType::Int },
}
// → Ok: NodeId(5) が ScalarVar "B" に変わる
```

**AddConstraint (ABC284-C):**
```rust
Action::AddConstraint {
    target: NodeId(3),  // N
    constraint: ConstraintDef {
        kind: ConstraintDefKind::Range { lower: "1".into(), upper: "100".into() },
    },
}
// → Ok: N に "1 ≤ N ≤ 100" の制約が追加
```

**IntroduceMultiTestCase (AGC062-A):**
```rust
Action::IntroduceMultiTestCase {
    count_var_name: "T".into(),
    sum_bound: Some(SumBoundDef { bound_var: "N".into(), upper: "3e5".into() }),
}
// → Ok: 既存構造が Repeat(T, Section(...)) で包含される
```

**AddConstraint to Hole (Rev.1 L-4 対応):**
```rust
// Hole に先に制約を設定し、後で FillHole する戦略が可能
Action::AddConstraint {
    target: NodeId(7),  // まだ Hole
    constraint: range("1", "100"),
}
// → Ok: Hole に "1 ≤ ? ≤ 100" が紐づく。FillHole 時に新ノードに引き継ぎ。
```

### 8.5 AI Agent の操作フロー（ABC284-C 構築）

```
Step 1:  completeness()            → { total_holes: 5, is_complete: false }
Step 2:  prioritized_holes()       → [Critical: Header を決めないと構造が決まらない]
Step 3:  hole_candidates(header)   → [IntroduceScalar { names: ["N", "M"] }]
Step 4:  preview(FillHole(header)) → Ok(PreviewResult { new_holes: [...] })
Step 5:  apply(FillHole(header, Scalar("N")))
Step 6:  apply(AddSlotElement(header, "M"))
Step 7:  apply(FillHole(data, EdgeList))
Step 8:  apply(AddConstraint(N, Range(1, 100)))
Step 9:  apply(AddConstraint(M, Range(0, N(N-1)/2)))
Step 10: apply(AddConstraint(edges, Property(Simple)))
Step 11: apply(FillHole(output, SingleValue(Int)))
Step 12: completeness() → { total_holes: 0, is_complete: true }
```

---

## 第 9 章　Canonical Rendering 方針

### 9.1 順序決定規則

1. **Sequence.children の順序がマスター**: ルートの children リスト順が入力行の出現順序を決定する。
2. **Tuple.elements の Vec 順序**: 同一行内の変数出現順序。
3. **Repeat.body の順序**: 繰り返し内の各行の構造。
4. **Section の header → body 順**: ヘッダ行が先、データ行が後。

### 9.2 展開規則

| 構造 | 展開パターン | 例 |
|------|-------------|-----|
| Array(長さ N) | `A_1 A_2 … A_N` | `C_1 C_2 … C_N` |
| Repeat(M, Tuple(u,v)) | M 行の `u_i v_i` | 辺リスト |
| Matrix(H×W) + RenderHint(sep=None) | H 行、各行 W 文字連結 | 文字グリッド |
| Matrix(H×W) + RenderHint(sep=Space) | H 行、各行 W 要素空白区切り | 整数行列 |
| Choice(tag, variants) | tag に応じた分岐表示 | `1 x y` / `2 x` |

### 9.3 省略記号の規則

- 水平: `A_1 A_2 … A_N`（要素数が変数依存の場合 `…` で省略）
- 垂直: 繰り返しブロックは `⋮` で省略
- 形式: 常に「最初の 1〜2 要素 + 省略 + 最後の要素」

### 9.4 変数名正規化

| パターン | 正規形 | 例 |
|----------|--------|-----|
| スカラー | 大文字英字 | `N`, `M`, `K` |
| 配列要素 | 名前 + 下付き添字 | `A_i`, `C_i` |
| 行列要素 | 名前 + 二重添字 | `A_{i,j}`, `C[i][j]` |
| 添字変数 | Repeat ネスト深さに応じて `i`, `j`, `k` 自動割り当て |

### 9.5 制約の表示順序

`Range → TypeDecl → LengthRelation → Relation → Distinct → Property → Sorted → SumBound → Guarantee`

同一種別内は対象変数名の辞書順。

### 9.6 具体例: ABC284-C の canonical rendering

**Input:**
```
N M
u_1 v_1
u_2 v_2
⋮
u_M v_M
```

**Constraints:**
```
1 ≤ N ≤ 100
0 ≤ M ≤ N(N-1)/2
1 ≤ u_i < v_i ≤ N
入力は全て整数
与えられるグラフは単純
```

**この設計で表現できる問題の例:** 上記の rendering は StructureAST + ConstraintAST の情報のみから決定論的に生成される。同一 AST からは常に同一テキストが出力される（Arena ベースで順序保証、M-1 対応済み）。

---

## 第 10 章　ランダム Sample Case 生成方針

### 10.1 生成戦略の全体像

```
StructureAST + ConstraintAST
    │
    ▼ 依存グラフ構築
DependencyGraph (DAG)
    │
    ▼ トポロジカルソート
GenerationOrder [N, M, u_1, v_1, ..., u_M, v_M]
    │
    ▼ 順序に従い各変数を生成
GeneratedSample { values: HashMap<NodeId, Value>, warnings: Vec<Warning> }
```

### 10.2 各制約種別の生成戦略

| Constraint | 戦略 | 保証レベル |
|-----------|------|-----------|
| **Range** | `[lower, upper]` の一様ランダム | L1: Guaranteed |
| **TypeDecl(Int)** | Range と組み合わせ | L1 |
| **TypeDecl(Str/Char)** | CharSet 参照で文字列生成 | L2（CharSet 必須） |
| **LengthRelation** | 先に長さ変数を生成、その値で配列長決定 | L1 |
| **Relation（単純）** | 先行変数から後続変数の範囲を絞り込み | L2 |
| **Distinct** | Fisher-Yates / リジェクション | L2 |
| **Property(Permutation)** | Fisher-Yates シャッフル | L2 |
| **Property(Connected)** | ランダム木 + 辺追加 | L2 |
| **Property(Tree)** | Prüfer 列 | L2 |
| **Property(Simple)** | 辺集合管理 | L2 |
| **SumBound** | ケース間で総和を分配 | L2 |
| **Sorted** | 生成後ソート | L2 |
| **Guarantee** | **生成器の責務外**; 事後検証のみ | L3: BestEffort |
| **CharSet** | 指定文字集合からランダムサンプリング | L2 |
| **StringLength** | 先に長さ決定、その長さで文字列生成 | L1 |
| **RenderHint** | 生成には影響しない | — |

### 10.3 3 段階保証レベル

| レベル | 名称 | 意味 | 条件 |
|--------|------|------|------|
| **L1** | `Guaranteed` | 確実に生成可能 | Range + LengthRelation + TypeDecl(Int) のみ |
| **L2** | `HighProbability` | 高確率で生成可能 | L1 + Distinct, Sorted, Relation(単純), Property(定義済み) |
| **L3** | `BestEffort` | 生成試行するが保証なし | Guarantee, Property(Custom), 複合 Relation |

### 10.4 生成可能保証の条件

> StructureAST + ConstraintAST が以下を**すべて**満たすとき、生成器は有限時間内にサンプルを生成できる:
>
> 1. **DAG 条件**: 依存グラフが非巡回
> 2. **Range 非空**: 各 Range の `lower ≤ upper`
> 3. **対応範囲内**: 各制約が生成器の対応集合に含まれる
> 4. **制約相互作用が充足可能**: Distinct + Range → 値域幅 ≥ 配列長、等（M-3 対応）
> 5. **Hole 非存在**: StructureAST に Hole が残っていない

### 10.5 Unsat 判定と報告

```rust
pub enum GenerationResult {
    Ok { sample: GeneratedSample, level: GuaranteeLevel },
    UnsatStatic { reason: UnsatReason, involved_nodes: Vec<NodeId> },
    UnsatDynamic { reason: UnsatReason, partial_env: HashMap<NodeId, Value>, failed_node: NodeId },
}

pub enum UnsatReason {
    CyclicDependency { cycle: Vec<NodeId> },
    EmptyRange { target: NodeId, lower: i64, upper: i64 },
    InsufficientDomain { target: NodeId, required: usize, available: usize },
    RejectionLimitExceeded { target: NodeId, attempts: u32 },
    UnsupportedProperty { tag: PropertyTag },
    HolePresent { hole: NodeId },
}
```

### 10.6 生成が困難な制約への対処

| 困難ケース | 対処方針 |
|-----------|---------|
| 複合 Relation の逆関数 | 単純ケース（線形）は逆関数導出; 一般ケースはリジェクション（上限 1000 回） |
| Distinct + 狭い値域 | 事前検査 `hi - lo + 1 ≥ N`; 集合ベースサンプリング |
| Property の組み合わせ | プリセット（Simple+Connected, Tree）は専用生成器; レイヤー合成 |
| Guarantee の能動的充足 | 生成器の責務外; 事後検証 + 警告。決定不能問題に帰着しうる |

**この設計で表現できる問題の例:** ABC350-C（順列ソート）は Property(Permutation) による Fisher-Yates シャッフルで L2 保証の生成が可能。ABC284-C（グラフ）は Range + Property(Simple) の組み合わせで、N を先に生成し M を N(N-1)/2 以下で生成し、辺を集合管理で重複排除する。

---

## 第 11 章　実サイトサンプルに対するカバレッジ検証結果

> 出典: doc/review/coverage-validation.md — AtCoder 18 問の検証

### 11.1 総合カバー率

| 指標 | 完全対応 | 部分対応以上 | 非対応 |
|------|----------|-------------|--------|
| **StructureAST** | 72.2% (13/18) | **94.4% (17/18)** | 5.6% (1/18) |
| **ConstraintAST** | 72.2% (13/18) | **94.4% (17/18)** | 5.6% (1/18) |
| **Canonical Rendering** | 72.2% (13/18) | **94.4% (17/18)** | 5.6% (1/18) |
| **Sample Generation** | 44.4% (8/18) | **94.4% (17/18)** | 5.6% (1/18) |

### 11.2 カテゴリ別 StructureAST カバー率

| カテゴリ | 問題数 | 対応可能 | 部分対応 | 非対応 |
|----------|--------|----------|----------|--------|
| 整数/配列（基本） | 2 | **2** | 0 | 0 |
| グラフ（木） | 1 | 0 | **1** | 0 |
| グラフ（一般） | 2 | **2** | 0 | 0 |
| 行列/2D配列 | 3 | **3** | 0 | 0 |
| 文字列 | 3 | **3** | 0 | 0 |
| 複数テストケース | 1 | **1** | 0 | 0 |
| クエリ形式 | 2 | 0 | **2** | 0 |
| 可変長入力 | 1 | **1** | 0 | 0 |
| 相互依存制約 | 2 | 1 | **1** | 0 |
| interactive | 1 | 0 | 0 | **1** |

### 11.3 検出されたギャップ一覧

| ID | ギャップ | 影響問題 | 優先度 | Rev.1 対応状況 |
|----|----------|----------|--------|---------------|
| **A** | Repeat.count が Expression を受け取れない | abc270_c (木) | ★★☆ | **延期** (D-8): 暗黙 Scalar 導入で回避 |
| **B** | Variant/Choice NodeKind 欠如 | abc278_d (クエリ) | ★★★ | **対応済み**: Choice NodeKind 追加 |
| **C** | Tuple 内インライン可変長配列 | abc356_c | ★☆☆ | **延期** (Phase 2): Vec\<NodeId\> で部分回避 |
| **D** | 三角行列サポート | abc371_c | ★☆☆ | **延期** (D-1): Repeat + 可変長 Tuple で近似 |
| **E** | 文字列パターン/正規表現制約 | abc350_a | ★☆☆ | **延期** (D-2): Property(Custom) で近似 |
| **F** | 文字列長・文字集合制約 | abc338_b, abc337_d | ★★☆ | **対応済み**: CharSet + StringLength 追加 |
| **G** | BigInt リテラル | abc283_c | ☆☆☆ | **延期** (D-3): i64 で 10^18 まで対応 |

### 11.4 Rev.1 対応後の予測カバー率

| 指標 | Sprint 2 時点 | Rev.1 対応後 |
|------|--------------|-------------|
| StructureAST 完全対応 | 72.2% | **~78%** (Choice 追加による改善) |
| ConstraintAST 完全対応 | 72.2% | **~89%** (CharSet + StringLength による改善) |
| Sample Generation 完全対応 | 44.4% | **~56%** (文字列生成対応による改善) |

### 11.5 判定基準

- **対応可能**: 既存の NodeKind / Constraint で完全に表現可能
- **部分対応**: 近似可能だが一部情報が欠落、または workaround が必要
- **非対応**: 現モデルでは表現不能

---

## 第 12 章　破綻例・未対応例・将来拡張

### 12.1 重大欠陥とその修正（Sprint 3 Critical Review）

| ID | 欠陥 | 深刻度 | Rev.1 修正内容 |
|----|------|--------|---------------|
| **S-1** | 型情報の二重管理（NodeKind と ConstraintAST の両方に型） | 重大 | NodeKind から `typ`/`element_type`/`separator` を除去。TypeDecl を Single Source of Truth に |
| **S-2** | 制約に ID がなく RemoveConstraint が機能しない | 重大 | ConstraintId 導入。ConstraintSet を再設計 |
| **S-3** | FillContent の肥大化で Operation が god object 化 | 重大 | Builder 層分離の方針確定。FillContent は Builder が Action 列に展開 |
| **M-1** | HashMap の順序非保証で canonical rendering が破綻 | 中 | Arena (Vec) ベースに変更 |
| **M-2** | preview_action が Projection の責務境界を侵害 | 中 | Operation trait に移動 |
| **M-3** | 生成可能保証が制約の相互作用を考慮していない | 中 | 制約相互作用条件を追加 |
| **M-4** | Tuple.elements が Vec\<Reference\> で親子関係が曖昧 | 中 | Vec\<NodeId\> に変更 |
| **M-5** | クエリ型コード分岐が表現不能 | 中 | Choice NodeKind 追加 |

### 12.2 非対応の明示（恥ではない）

以下は Phase 1 で意図的に非対応とする。これらは設計の限界ではなく、スコープの選択である。

| # | 非対応項目 | 理由 | 延期先 |
|---|-----------|------|--------|
| D-1 | 三角行列 NodeKind | 出現頻度低; Repeat で近似可能 | Phase 2 |
| D-2 | 文字列パターン/正規表現制約 | Property(Custom) で近似可能 | Phase 2 |
| D-3 | BigInt リテラル | i64 で 10^18 まで対応; 超大整数は稀 | Phase 2 |
| D-4 | Interactive NodeKind | 静的木モデルと根本的に異なる | Phase 2 |
| D-5 | Expression 評価の完全形式化 | 実装フェーズで段階的定義 | Sprint 5 |
| D-6 | 依存グラフ構築ルールの完全形式化 | ヒューリスティックで大半対応 | Sprint 5 |
| D-7 | Repeat スコープ内制約の形式化 | 暗黙規約で実装開始 | Sprint 5 |
| D-8 | Repeat.count の Expression 対応 | 暗黙 Scalar 導入で回避 | Sprint 5 |
| D-9 | ProjectedNodeKind のドメイン固有分類 | annotation tag は Phase 2 | Phase 2 |

### 12.3 将来拡張の方向性

1. **パーサ**: テキスト→AST 変換。AtCoder の問題ページ HTML からの自動構造抽出。
2. **OutputSpec の正式モデル化**: 現在は入力構造に注力。出力仕様の AST 化は Phase 2。
3. **Undo/Redo**: Operation 層に immutable snapshot ベースの履歴管理を追加。
4. **形式的検証**: Hazelnut 流の action semantics の部分的形式化。Sensibility 定理の限定版証明。
5. **Codeforces / AOJ 対応**: サイト間の表記揺れ吸収層の追加。

### 12.4 先行研究との未消化な関係

| 先行研究 | 現状の取り込み度 | 今後の課題 |
|---------|----------------|-----------|
| JetBrains MPS | 概念レベルで参照 | constraints aspect との詳細比較 |
| Attribute Grammars | ConstraintAST が類似の機構 | 制約伝搬の形式化 |
| Liquid Types / Refinement Types | 型と制約の分担に反映 | 理論的位置づけの明確化 |
| Luck (Lampropoulos 2017) | 制約ベース生成の系譜に位置づけ | 生成言語の比較評価 |

---

## 第 13 章　実装着手順（Sprint 5 タスク順序）

### 13.1 今すぐ実装すべき最小コア

以下を Phase 1 の最小実装スコープとする:

1. **StructureAST**: 9 NodeKind + Arena ベース + NodeId 管理
2. **ConstraintAST**: 12 Constraint + ConstraintId + ConstraintSet
3. **Operation**: FillHole, ReplaceNode, AddConstraint, RemoveConstraint, AddSlotElement, RemoveSlotElement
4. **ProjectionAPI**: nodes(), inspect(), hole_candidates(), completeness()
5. **Sample Generator**: Range + LengthRelation + TypeDecl(Int) 限定（L1 保証のみ）
6. **Canonical Renderer**: StructureAST → Input Format テキスト + ConstraintAST → Constraints テキスト

### 13.2 Sprint 5 の具体的タスク順序

```
Week 1: Foundation
  T-01  core 型定義（NodeId, ConstraintId, Ident, Literal, Expression）
  T-02  StructureAst Arena 実装 + StructureNode + NodeKind enum
  T-03  Reference 型 + 解決ユーティリティ
  T-04  ConstraintSet 実装 + Constraint enum
  T-05  Expression 評価器（Lit, Var, BinOp, Pow, FnCall(min/max)）

Week 2: Operation Core
  T-06  OperationError enum 実装
  T-07  FillHole 実装（前提条件チェック + ノード置換 + 子 Hole 生成）
  T-08  AddConstraint / RemoveConstraint 実装
  T-09  ReplaceNode 実装（依存関係チェック含む）
  T-10  AddSlotElement / RemoveSlotElement 実装

Week 3: Projection + Rendering
  T-11  ProjectionAPI trait 実装（nodes, inspect, completeness）
  T-12  hole_candidates 実装（ExpectedType + ConstraintAST からの導出）
  T-13  Canonical Renderer: Input Format テキスト生成
  T-14  Canonical Renderer: Constraints テキスト生成
  T-15  preview (dry-run) 実装

Week 4: Sample Generation + Integration
  T-16  依存グラフ構築 + トポロジカルソート
  T-17  Range + LengthRelation + TypeDecl(Int) の生成器（L1）
  T-18  Distinct + Sorted + Property(Permutation) の生成器（L2 部分）
  T-19  GeneratedSample → テキスト出力
  T-20  統合テスト: ABC284-C, ABC300-A, ABC350-C の end-to-end 検証
```

### 13.3 各タスクの検証基準

| タスク群 | 検証方法 |
|---------|---------|
| T-01〜T-05 | ユニットテスト: 型生成、Arena CRUD、式評価 |
| T-06〜T-10 | ユニットテスト: 各 Action の成功/失敗ケース（§8.4 の具体例） |
| T-11〜T-15 | ユニットテスト: ABC284-C の Projection 出力（§7.3 の具体例） |
| T-16〜T-19 | ユニットテスト + プロパティベーステスト: 生成値が制約充足することを検証 |
| T-20 | 統合テスト: 構築→Rendering→Sample 生成→制約検証のパイプライン |

### 13.4 Phase 2 への引き継ぎ事項

- D-1〜D-9 の延期項目（§12.2）
- Builder 層の実装（EdgeList, QueryList, TriangularBlock の展開ロジック）
- IntroduceMultiTestCase の完全実装（SumBound の分配ロジック含む）
- L2 / L3 生成器の拡充（Property(Connected), Property(Tree), 複合 Relation）
- OutputSpec のモデル化
- Undo/Redo 機構

---

## 付録 A　引用文献

### 必須引用

1. Omar et al., "Hazelnut: A Bidirectionally Typed Structure Editor Calculus," POPL 2017
2. Omar et al., "Live Functional Programming with Typed Holes," ICFP 2019
3. Omar et al., "Filling Typed Holes with Live GUIs," PLDI 2021
4. Voelter et al., "A General Architecture for Heterogeneous Language Engineering," 2015
5. Voelter et al., "Efficient Development of Consistent Projectional Editors using Grammar Cells," SLE 2016

### 強く推奨

6. Berger et al., "Projectional Editors for JSON-Based DSLs," 2023
7. Claessen & Hughes, "QuickCheck: A Lightweight Tool for Random Testing," ICFP 2000
8. Lampropoulos et al., "Luck: A Probabilistic Language for Testing," 2017

### 背景文献

9. Knuth, "Semantics of Context-Free Languages," 1968 (attribute grammars)
10. Rondon et al., "Liquid Types," PLDI 2008
11. Wadler, "A Prettier Printer," 2003
12. Reps & Teitelbaum, "The Synthesizer Generator," 1984

## 付録 B　入力パターンと NodeKind の対応表

| 入力パターン | 対応 NodeKind 構成 |
|-------------|-------------------|
| I-1: ヘッダ行+データ行 | Sequence > Tuple(header) + Repeat(data) |
| I-2: グリッド入力 | Tuple(H,W) + Matrix(H,W) |
| I-3: グラフ入力 | Tuple(N,M) + Repeat(M, Tuple(u,v)) |
| I-4: クエリ形式 | Tuple(L,Q) + Repeat(Q, Choice(tag, variants)) |
| I-5: 複数テストケース | Scalar(T) + Repeat(T, Section(...)) |
| I-6: 複合セクション | Sequence > Section × n |
| I-7: インタラクティブ | **非対応**（Phase 2） |

## 付録 C　制約パターンと Constraint の対応表

| 制約パターン | 対応 Constraint |
|-------------|----------------|
| C-1: 値域 `1 ≤ N ≤ 300` | Range |
| C-2: 型宣言 | TypeDecl |
| C-3: 導出制約 `M ≤ N(N-1)/2` | Range (upper に Expression) |
| C-4: 変数間関係 `u_i < v_i` | Relation |
| C-5: 構造的性質 | Property |
| C-6: 総和制約 | SumBound |
| C-7: 存在保証 | Guarantee |
| 文字列長 `1 ≤ |S| ≤ 1000` | StringLength |
| 文字集合 `英小文字のみ` | CharSet |
| 表示区切り `separator=None` | RenderHint |
