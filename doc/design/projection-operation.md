# ProjectionAPI と Operation の API 面設計

> Sprint 2 — ProjectionOperationAgent による API 面定義
>
> **[Rev.1]** Sprint 3 Synthesis により改訂。S-2 (ConstraintId), S-3 (Builder層), M-1 (Arena化), M-2 (preview移動), L-4 (Hole制約許可) 対応。

---

## 1. ProjectionAPI が返すべき情報

### 1.1 中核 trait 定義

```rust
/// NodeId は AST 内のノードを一意に識別する。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(u64);

/// ProjectionAPI: StructureAST + ConstraintAST から read-only な像を導出する。
/// GUI / CLI / AI Agent / テストコードすべてがこの trait を通じて状態を読む。
pub trait ProjectionAPI {
    /// AST 全体のノードを深さ優先で返す。
    fn nodes(&self) -> Vec<ProjectedNode>;

    /// 指定ノードの子ノード一覧（slot 名付き）。
    fn children(&self, node: NodeId) -> Vec<SlotEntry>;

    /// 指定ノードの詳細情報。
    fn inspect(&self, node: NodeId) -> Option<NodeDetail>;

    /// 指定 hole に対して埋められる候補カテゴリを返す。
    fn hole_candidates(&self, hole: NodeId) -> Vec<CandidateKind>;

    /// 現在の状態で実行可能な Action の一覧を返す。
    fn available_actions(&self) -> Vec<AvailableAction>;

    /// 指定ノードが編集不可の場合、その理由を返す。
    fn why_not_editable(&self, node: NodeId) -> Option<NotEditableReason>;

    /// AST 全体の完成度サマリ（残り hole 数、制約充足状況など）。
    fn completeness(&self) -> CompletenessSummary;
}
```

### 1.2 返却型

```rust
/// ProjectedNode: 外部向けに導出されたノード表現。
pub struct ProjectedNode {
    pub id: NodeId,
    pub label: String,
    pub kind: ProjectedNodeKind,
    pub depth: usize,
    pub is_hole: bool,
}

pub enum ProjectedNodeKind {
    Root,              // Problem 全体
    Section,           // 入力セクション (e.g. グラフ部、条件部)
    Header,            // ヘッダ行
    ScalarVar,         // スカラー変数 (N, M, K ...)
    ArrayVar,          // 配列変数 (A_1 ... A_N)
    GridVar,           // グリッド変数 (C[i][j])
    EdgeList,          // 辺リスト
    QueryList,         // クエリリスト
    MultiTestCase,     // 複数テストケース包含
    TriangularBlock,   // 三角行列
    Constraint,        // 制約ノード
    Hole,              // 未充填の穴
}

/// SlotEntry: あるノードの slot に紐づく子ノード情報。
pub struct SlotEntry {
    pub slot_name: String,
    pub child: Option<NodeId>,
    pub expected_type: Option<ExpectedType>,
    pub is_required: bool,
}

/// ExpectedType: hole / slot に対して期待される型情報。
pub enum ExpectedType {
    IntScalar,
    IntArray { length_depends_on: Option<NodeId> },
    CharGrid { rows: Option<NodeId>, cols: Option<NodeId> },
    StringScalar,
    EdgeListOf { node_count: Option<NodeId>, edge_count: Option<NodeId> },
    QueryListOf { count: Option<NodeId> },
    ConstraintExpr,
    AnyOf(Vec<ExpectedType>),
}

/// NodeDetail: inspect() が返す詳細情報。
pub struct NodeDetail {
    pub id: NodeId,
    pub label: String,
    pub kind: ProjectedNodeKind,
    pub slots: Vec<SlotEntry>,
    pub constraints: Vec<ConstraintSummary>,
    pub editable: bool,
    pub not_editable_reason: Option<NotEditableReason>,
}

/// ConstraintSummary: 制約の人間可読な要約。
pub struct ConstraintSummary {
    pub description: String,     // e.g. "1 ≤ N ≤ 2×10^5"
    pub satisfied: Option<bool>, // None = 判定不能 (hole 残りのため)
}

/// CandidateKind: hole を埋める候補の種別。
pub enum CandidateKind {
    /// スカラー変数を導入する (e.g. N, M)
    IntroduceScalar { suggested_names: Vec<String> },
    /// 配列変数を導入する (e.g. A_1 ... A_N)
    IntroduceArray { suggested_names: Vec<String> },
    /// グリッド変数を導入する
    IntroduceGrid,
    /// 辺リストを導入する
    IntroduceEdgeList,
    /// クエリリストを導入する
    IntroduceQueryList,
    /// セクションを導入する
    IntroduceSection,
    /// 三角行列を導入する
    IntroduceTriangularBlock,
    /// 複数テストケース構造を導入する
    IntroduceMultiTestCase,
    /// 既存変数への参照 (e.g. 配列長の hole に N を入れる)
    ReferenceExisting { candidates: Vec<NodeId> },
}

/// AvailableAction: 現在実行可能な操作。
pub struct AvailableAction {
    pub action: Action,
    pub target: NodeId,
    pub description: String,
}

/// NotEditableReason: 編集不可の理由。
pub enum NotEditableReason {
    /// StructureAST 上の制約で固定されている
    StructurallyFixed { reason: String },
    /// 他ノードが依存しているため変更不可
    HasDependents { dependents: Vec<NodeId> },
    /// 親ノードの hole がまだ埋まっていない
    ParentHoleUnfilled { parent: NodeId },
    /// 制約によりこの位置は確定済み
    ConstraintDetermined { constraint: String },
}

/// CompletenessSummary: AST 全体の完成度。
pub struct CompletenessSummary {
    pub total_holes: usize,
    pub filled_holes: usize,
    pub unsatisfied_constraints: usize,
    pub is_complete: bool,
}
```

### 1.3 具体例: ABC284-C (グラフ連結成分数) の Projection

問題: N 頂点 M 辺の単純無向グラフの連結成分数を出力。

**構築途中の AST に対する `nodes()` の返却値:**

```
ProjectedNode { id: 0, label: "Problem: abc284_c",     kind: Root,       depth: 0, is_hole: false }
ProjectedNode { id: 1, label: "Input",                  kind: Section,    depth: 1, is_hole: false }
ProjectedNode { id: 2, label: "Header: N M",            kind: Header,     depth: 2, is_hole: false }
ProjectedNode { id: 3, label: "N",                      kind: ScalarVar,  depth: 3, is_hole: false }
ProjectedNode { id: 4, label: "M",                      kind: ScalarVar,  depth: 3, is_hole: false }
ProjectedNode { id: 5, label: "EdgeList(M edges)",      kind: EdgeList,   depth: 2, is_hole: false }
ProjectedNode { id: 6, label: "Output",                 kind: Section,    depth: 1, is_hole: false }
ProjectedNode { id: 7, label: "???",                    kind: Hole,       depth: 2, is_hole: true  }
```

**`hole_candidates(NodeId(7))` の返却値:**

```
[
  CandidateKind::IntroduceScalar { suggested_names: ["ans"] },
  CandidateKind::ReferenceExisting { candidates: [] },
]
```

→ Output の hole を `IntroduceScalar` で「単一整数出力」として埋めることができる。

**`available_actions()` の一部:**

```
[
  AvailableAction {
    action: Action::FillHole { target: NodeId(7), fill: ... },
    target: NodeId(7),
    description: "Output の形式を指定する",
  },
  AvailableAction {
    action: Action::AddConstraint { target: NodeId(3), ... },
    target: NodeId(3),
    description: "N に制約を追加する (e.g. 1 ≤ N ≤ 100)",
  },
]
```

---

## 2. Action の分類

### 2.1 Action enum 定義

```rust
/// Action: Operation に渡す編集要求。
/// CLI / AI Agent / テストコード すべてがこの型を通じて状態を変更する。
pub enum Action {
    /// hole を埋める。
    FillHole {
        target: NodeId,
        fill: FillContent,
    },

    /// 既存ノードを差し替える。
    ReplaceNode {
        target: NodeId,
        replacement: FillContent,
    },

    /// 制約を追加する。
    AddConstraint {
        target: NodeId,
        constraint: ConstraintDef,
    },

    /// 制約を削除する。
    /// [Rev.1] S-2 対応: constraint_id を ConstraintId に変更。
    RemoveConstraint {
        constraint_id: ConstraintId,
    },

    /// 複数テストケース構造を導入する。
    IntroduceMultiTestCase {
        count_var_name: String,
        sum_bound: Option<SumBoundDef>,
    },

    /// slot に要素を追加する（Section 追加、ヘッダ変数追加など）。
    AddSlotElement {
        parent: NodeId,
        slot_name: String,
        element: FillContent,
    },

    /// slot から要素を削除する。
    RemoveSlotElement {
        parent: NodeId,
        slot_name: String,
        child: NodeId,
    },
}
```

### 2.2 補助型

```rust
/// FillContent: hole / slot を埋める内容。
/// [Rev.1] S-3 対応: FillContent は Builder 層が Action 列に展開する高レベル意図である。
/// Operation 層は NodeKind レベルの低レベル Action のみを受け付ける。
/// Builder 層は Operation の外に位置し、ドメイン知識（EdgeList, QueryList 等の展開ロジック）を局所化する。
///
/// 展開の流れ:
///   [ユーザー / AI] → 高レベル意図 (FillContent)
///   [Builder 層]    → Action 列に展開 (FillHole, AddSlotElement, AddConstraint の列)
///   [Operation 層]  → 各 Action を検証・適用
pub enum FillContent {
    Scalar { name: String, typ: VarType },
    Array { name: String, element_type: VarType, length: LengthSpec },
    Grid { name: String, rows: LengthSpec, cols: LengthSpec, cell_type: VarType },
    EdgeList { edge_count: LengthSpec, weighted: bool },
    QueryList { query_count: LengthSpec, variants: Vec<QueryVariant> },
    TriangularBlock { size: LengthSpec },
    Section { label: String },
    OutputSingleValue { typ: VarType },
    OutputYesNo,
    OutputModular { modulus: u64 },
    OutputConditional { condition_output: Box<FillContent>, else_output: Box<FillContent> },
    OutputMultiLine { count: LengthSpec },
    Interactive { protocol: InteractiveProtocol },
}

pub enum VarType { Int, Str, Char }

/// LengthSpec: 長さ指定。固定値 or 変数参照。
pub enum LengthSpec {
    Fixed(usize),
    RefVar(NodeId),
    Expr(String),  // e.g. "N-1", "N*(N-1)/2"
}

pub struct QueryVariant {
    pub type_code: String,       // e.g. "1", "2"
    pub param_types: Vec<VarType>,
}

pub struct InteractiveProtocol {
    pub question_prefix: String,   // "?"
    pub answer_prefix: String,     // "!"
    pub max_questions: Option<LengthSpec>,
}

pub struct ConstraintDef {
    pub kind: ConstraintDefKind,
}

pub enum ConstraintDefKind {
    Range { lower: String, upper: String },
    TypeDecl { typ: VarType },
    Relation { op: RelOp, rhs: String },
    Distinct,
    Sorted { order: SortOrder },
    Property { tag: String },
    SumBound { over_var: String, upper: String },
    Derived { expr: String },
    Guarantee { description: String },
}

pub enum RelOp { Lt, Le, Gt, Ge, Ne, Eq }
pub enum SortOrder { Ascending, Descending }

pub struct SumBoundDef {
    pub bound_var: String,  // e.g. "N"
    pub upper: String,      // e.g. "3e5"
}
```

### 2.3 各 Action の仕様

#### FillHole

| 項目 | 内容 |
|------|------|
| **入力パラメータ** | `target: NodeId` (hole のノード), `fill: FillContent` (埋める内容) |
| **前提条件** | ① target が Hole である ② fill の型が slot の ExpectedType と互換 ③ 親ノードが有効な状態にある |
| **成功時の状態変化** | hole が fill の内容で置換され、新ノードが生成される。必要に応じて子 hole が作られる（例: Array を入れると length の hole が生える） |
| **失敗理由** | `NodeNotFound` — target が存在しない / `TypeMismatch` — fill の型と ExpectedType が不一致 / `SlotOccupied` — 既に埋まっている |

**具体例 (ABC300-A):** ヘッダ行の3番目の hole を `Scalar { name: "B", typ: Int }` で埋める。

```rust
Action::FillHole {
    target: NodeId(5),  // ヘッダの3番目の hole
    fill: FillContent::Scalar { name: "B".into(), typ: VarType::Int },
}
// 成功 → NodeId(5) が ScalarVar "B" に変わる
```

#### ReplaceNode

| 項目 | 内容 |
|------|------|
| **入力パラメータ** | `target: NodeId` (差し替え対象), `replacement: FillContent` (新しい内容) |
| **前提条件** | ① target が存在する ② target が Hole でない（Hole なら FillHole を使う） ③ replacement の型が slot の ExpectedType と互換 ④ target に依存するノードが存在しない、または依存が解消可能 |
| **成功時の状態変化** | target のサブツリーが replacement で置換される。旧ノードへの参照（制約含む）は無効化される |
| **失敗理由** | `NodeNotFound` / `TypeMismatch` / `ConstraintViolation` — 置換後に既存の制約が壊れる / `HasDependents` — 他ノードが依存しており安全に差し替え不可 |

**具体例 (ABC300-A → ABC350-C への変更):** ヘッダ行 `N A B` を `N` 単独に差し替え。

```rust
Action::ReplaceNode {
    target: NodeId(2),  // Header "N A B"
    replacement: FillContent::Scalar { name: "N".into(), typ: VarType::Int },
}
// 失敗例 → ConstraintViolation: A, B を参照する制約が存在する
```

#### AddConstraint

| 項目 | 内容 |
|------|------|
| **入力パラメータ** | `target: NodeId` (制約を付与するノード), `constraint: ConstraintDef` |
| **前提条件** | ① target が存在する ② target が制約を受け入れられるノード種別である（**[Rev.1] L-4 対応: Hole を含む**） ③ 制約が既存の制約と矛盾しない |
| **成功時の状態変化** | ConstraintAST に新しい制約が追加される。新しい ConstraintId が割り当てられる |
| **失敗理由** | `NodeNotFound` / `ConstraintViolation` — 既存制約と矛盾（例: `1 ≤ N ≤ 100` が既にあるのに `N ≤ 50` を追加すると上界が競合） |

**具体例 (ABC284-C):**

```rust
Action::AddConstraint {
    target: NodeId(3),  // N
    constraint: ConstraintDef {
        kind: ConstraintDefKind::Range {
            lower: "1".into(),
            upper: "100".into(),
        },
    },
}
// 成功 → N に "1 ≤ N ≤ 100" の制約が追加される
```

```rust
// グラフの構造的制約
Action::AddConstraint {
    target: NodeId(5),  // EdgeList
    constraint: ConstraintDef {
        kind: ConstraintDefKind::Property {
            tag: "simple_graph".into(),
        },
    },
}
```

#### RemoveConstraint

| 項目 | 内容 |
|------|------|
| **入力パラメータ** | `constraint_id: ConstraintId` (制約の ID) |
| **前提条件** | ① constraint_id が ConstraintSet に存在する |
| **成功時の状態変化** | ConstraintAST から該当制約が除去される |
| **失敗理由** | `ConstraintNotFound` — 制約が存在しない / `InvalidOperation` — 削除すると構造的整合性が壊れる制約（例: 配列長制約 `len(A) = N` は配列の構造と不可分） |

**[Rev.1]** S-2 対応: `constraint_id` を `ConstraintId` 型に変更。各制約は一意の ConstraintId で識別されるため、同一ノードに複数制約がある場合でも個別に削除可能。

**具体例:** ABC300-A で「C_i は相異なる」制約を除去する。

```rust
Action::RemoveConstraint {
    constraint_id: ConstraintId(20), // Distinct(C) の制約
}
// 成功 → 相異条件が除去される（問題としては別物になるが、構造的には合法）
```

#### IntroduceMultiTestCase

| 項目 | 内容 |
|------|------|
| **入力パラメータ** | `count_var_name: String` (テストケース数の変数名, 通常 "T"), `sum_bound: Option<SumBoundDef>` (総和制約) |
| **前提条件** | ① 現在の AST が単一テストケース構造である ② MultiTestCase が未導入 |
| **成功時の状態変化** | Root 直下に MultiTestCase ノードが挿入され、既存の入力構造がケース本体として包含される。count_var (T) がヘッダとして追加される |
| **失敗理由** | `InvalidOperation` — 既に MultiTestCase 構造がある / Interactive 問題で MultiTestCase は非対応 |

**具体例 (AGC062-A):** 単一テストケース構造から複数テストケース構造へ変換。

```rust
Action::IntroduceMultiTestCase {
    count_var_name: "T".into(),
    sum_bound: Some(SumBoundDef {
        bound_var: "N".into(),
        upper: "3e5".into(),
    }),
}
// 成功 →
//   Root
//   └─ MultiTestCase(T)
//      ├─ Header: T
//      └─ CaseBody (元の Input 構造)
//         ├─ N
//         └─ S
// 制約に "N の総和 ≤ 3×10^5" が自動追加
```

#### AddSlotElement

| 項目 | 内容 |
|------|------|
| **入力パラメータ** | `parent: NodeId`, `slot_name: String` (追加先スロット名), `element: FillContent` |
| **前提条件** | ① parent が存在する ② slot_name が parent のノード種別で許可されたスロットである ③ スロットが可変長スロットである（固定スロットは FillHole を使う） |
| **成功時の状態変化** | parent の slot に新しい子ノードが末尾追加される |
| **失敗理由** | `NodeNotFound` / `InvalidOperation` — slot_name が存在しない or 固定長スロット / `TypeMismatch` — element の型が slot の期待型と不一致 |

**具体例 (ABC244-E):** ヘッダ行に変数を追加していく。

```rust
// ヘッダに "S" 変数を追加 (既に N, M, K がある状態)
Action::AddSlotElement {
    parent: NodeId(2),  // Header
    slot_name: "variables".into(),
    element: FillContent::Scalar { name: "S".into(), typ: VarType::Int },
}
// 成功 → Header が "N M K S" になる
```

**具体例 (ABC299-E):** 入力に新しいセクションを追加。

```rust
// グラフセクションの後に条件セクションを追加
Action::AddSlotElement {
    parent: NodeId(1),   // Input
    slot_name: "sections".into(),
    element: FillContent::Section { label: "Conditions".into() },
}
// 成功 → Input 配下に新しい空セクション（hole 付き）が追加される
```

#### RemoveSlotElement

| 項目 | 内容 |
|------|------|
| **入力パラメータ** | `parent: NodeId`, `slot_name: String`, `child: NodeId` (削除する子ノード) |
| **前提条件** | ① parent / child が存在する ② child が parent の slot_name に属する ③ child を削除しても構造不整合にならない |
| **成功時の状態変化** | child とそのサブツリーが削除される。child を参照する制約も除去される |
| **失敗理由** | `NodeNotFound` / `InvalidOperation` — 必須スロットの唯一の子を削除しようとした / `ConstraintViolation` — 削除により依存制約が壊れる |

---

## 3. OperationError の設計

### 3.1 Error enum 定義

```rust
/// OperationError: Operation が Action を拒否した場合の失敗理由。
#[derive(Debug)]
pub enum OperationError {
    /// fill/replacement の型が slot の ExpectedType と合わない。
    TypeMismatch {
        expected: ExpectedType,
        actual: String,
        context: String,
    },

    /// 指定された NodeId が AST 内に存在しない。
    NodeNotFound {
        node: NodeId,
    },

    /// FillHole の対象が既に埋まっている。
    SlotOccupied {
        node: NodeId,
        current_occupant: String,
    },

    /// 操作の結果、既存の制約が破れる。
    ConstraintViolation {
        violated_constraints: Vec<ViolationDetail>,
    },

    /// このノード種別・状態ではこの操作が意味をなさない。
    InvalidOperation {
        action: String,
        reason: String,
    },
}

pub struct ViolationDetail {
    /// [Rev.1] S-2 対応: ConstraintId に変更
    pub constraint_id: ConstraintId,
    pub description: String,  // 人間可読な説明
    pub suggestion: Option<String>,  // 修正のヒント
}
```

### 3.2 各エラーの具体例

#### TypeMismatch

**状況:** ABC300-B で、グリッドの hole に EdgeList を入れようとした。

```rust
Action::FillHole {
    target: NodeId(10),  // Grid の cell_type hole
    fill: FillContent::EdgeList { edge_count: LengthSpec::Fixed(5), weighted: false },
}

// → Err(OperationError::TypeMismatch {
//     expected: ExpectedType::CharGrid { rows: Some(NodeId(3)), cols: Some(NodeId(4)) },
//     actual: "EdgeList".into(),
//     context: "Grid slot 'body' expects CharGrid, got EdgeList".into(),
// })
```

#### NodeNotFound

**状況:** 削除済みのノードに制約を追加しようとした。

```rust
Action::AddConstraint {
    target: NodeId(999),  // 存在しない
    constraint: ConstraintDef {
        kind: ConstraintDefKind::Range { lower: "1".into(), upper: "100".into() },
    },
}

// → Err(OperationError::NodeNotFound {
//     node: NodeId(999),
// })
```

#### SlotOccupied

**状況:** ABC300-A で、既に N が入っている hole に再度 FillHole しようとした。

```rust
Action::FillHole {
    target: NodeId(3),  // 既に ScalarVar "N" が入っている
    fill: FillContent::Scalar { name: "X".into(), typ: VarType::Int },
}

// → Err(OperationError::SlotOccupied {
//     node: NodeId(3),
//     current_occupant: "ScalarVar(N)".into(),
// })
// ヒント: ReplaceNode を使うべき
```

#### ConstraintViolation

**状況:** ABC284-C で、辺リストを削除しようとしたが `u_i, v_i` に値域制約が付いている。

```rust
Action::RemoveSlotElement {
    parent: NodeId(1),
    slot_name: "data_blocks".into(),
    child: NodeId(5),  // EdgeList
}

// → Err(OperationError::ConstraintViolation {
//     violated_constraints: vec![
//         ViolationDetail {
//             constraint_id: ConstraintId(15),
//             description: "1 ≤ u_i < v_i ≤ N — u_i, v_i は EdgeList に依存".into(),
//             suggestion: Some("先に関連する制約を RemoveConstraint で削除してください".into()),
//         },
//         ViolationDetail {
//             constraint_id: ConstraintId(16),
//             description: "0 ≤ M ≤ N(N-1)/2 — M は EdgeList の辺数".into(),
//             suggestion: Some("M の制約も合わせて除去してください".into()),
//         },
//     ],
// })
```

#### InvalidOperation

**状況:** 既に MultiTestCase 構造がある問題に再度 IntroduceMultiTestCase を適用しようとした。

```rust
Action::IntroduceMultiTestCase {
    count_var_name: "T".into(),
    sum_bound: None,
}

// → Err(OperationError::InvalidOperation {
//     action: "IntroduceMultiTestCase".into(),
//     reason: "MultiTestCase structure already exists (count_var: T)".into(),
// })
```

**状況:** Hole に対して AddConstraint — **[Rev.1] L-4 対応: Hole への制約追加は許可された。**

```rust
// Hole に対して先に制約を設定し、後で FillHole する戦略が可能。
Action::AddConstraint {
    target: NodeId(7),  // まだ Hole のまま
    constraint: ConstraintDef {
        kind: ConstraintDefKind::Range { lower: "1".into(), upper: "100".into() },
    },
}

// → Ok: Hole に "1 ≤ ? ≤ 100" の制約が紐づけられる。
// FillHole 実行時にこの制約が新ノードに引き継がれる。
```

---

## 4. View を薄く保つ条件

### 4.1 原則

> **Projection は「問い合わせ」のみ。「判断」と「変更」は Operation の責務。**

ProjectionAPI は StructureAST と ConstraintAST を**読むだけ**で、いかなる状態変更もしない。View（GUI/CLI/AI）は Projection から得た情報を表示し、ユーザーの選択を Action に変換して Operation に渡す。

### 4.2 Projection が持つべきもの

| 責務 | 説明 | 例 |
|------|------|-----|
| ノードのラベル生成 | NodeKind + 名前から表示文字列を導出 | `ScalarVar("N")` → `"N"` |
| hole の候補カテゴリ列挙 | ExpectedType と ConstraintAST から候補種別を導出 | `[IntroduceScalar, IntroduceArray, ...]` |
| 実行可能 Action 列挙 | 現在の AST 状態から可能な操作を導出 | `[FillHole(7), AddConstraint(3), ...]` |
| 編集不可理由の導出 | 構造的固定 / 依存関係 を判定し理由を返す | `HasDependents { dependents: [NodeId(15)] }` |
| 完成度サマリ | hole 数・制約充足率の集計 | `{ total_holes: 3, filled: 1, ... }` |
| 制約の人間可読表現 | ConstraintAST の内部表現を文字列化 | `"1 ≤ N ≤ 2×10^5"` |

### 4.3 Projection が持つべきでないもの

| 持つべきでないもの | 理由 | 正しい責務の所在 |
|---|---|---|
| AST の変更操作 | 状態遷移は Operation の専権事項 | **Operation** |
| 制約の整合性検証 | 変更前後の整合性保証は Operation が行う | **Operation** |
| Undo/Redo の管理 | 状態履歴の管理は状態遷移層に属する | **Operation** |
| Action の実行判断 | 「このアクションを実行すべきか」はクライアントの判断 | **View / AI Agent** |
| UI レイアウト計算 | DOM座標、CSS、ウィンドウ幅などは UI 層の責務 | **View (GUI)** |
| ファイル I/O | AST のシリアライズ・デシリアライズ | **外部ユーティリティ** |
| 問題文のパース | テキストから AST への変換は別モジュール | **Parser (将来)** |
| 候補の**優先順位**付け | AI 向けの推奨順は Agent 側が決定する | **AI Agent** |

### 4.4 境界の判定基準

ある機能を Projection / Operation のどちらに置くか迷ったとき:

1. **「状態を変えるか？」** → Yes なら Operation。
2. **「現在の状態だけで計算できるか？」** → Yes なら Projection 候補。
3. **「全クライアント（GUI/CLI/AI/テスト）で共通か？」** → Yes なら Projection。No（特定 View でしか使わない）なら View 側。
4. **「キャッシュ可能か？」** → Projection はキャッシュ可能であるべき。AST が変更されない限り同じ結果を返す。

### 4.5 具体例: ABC300-A の操作フローでの責務分担

```
[状態] Header に N, A, B が入り、配列 C の hole が残っている

1. GUI が ProjectionAPI.nodes() を呼ぶ → ノード一覧を受け取る (Projection)
2. GUI が hole_candidates(NodeId(8)) を呼ぶ → [IntroduceArray, ...] (Projection)
3. ユーザーが IntroduceArray を選択 (View の責務)
4. GUI が Action::FillHole { target: 8, fill: Array { name: "C", ... } } を構築 (View)
5. GUI が Operation.apply(action) を呼ぶ (Operation)
6. Operation が型チェック → 合格 → 新 AST を返す (Operation)
7. GUI が新 AST に対して再度 ProjectionAPI.nodes() を呼ぶ (Projection)
```

この流れにおいて、Projection はステップ 1, 2, 7 でのみ登場し、**変更は一切行わない**。

---

## 5. AI Agent が触れる API 面

### 5.1 結論: 基本的に同一 API で十分

AI Agent は GUI/CLI と**同じ** `ProjectionAPI` + `Operation` API を使う。
追加の特権 API は原則不要である。

理由:

1. **ProjectionAPI** が返す情報（ノード一覧、hole 候補、実行可能 Action）は、AI が次の操作を選択するのに十分な情報を含んでいる。
2. **Operation** の `Action` enum は、AI が生成できる粒度で設計されている（hole ID + FillContent の指定）。
3. AI にだけ「制約を無視する特権」を与えると、一貫性の保証が壊れる。

### 5.2 AI Agent 向けの**補助** API（同じ trait を拡張）

ただし、AI Agent の効率的な意思決定を支援するために、ProjectionAPI に以下の**read-only** メソッドを追加する。これらは GUI/CLI も利用可能であり、特権ではない。

```rust
/// AI Agent 向けの補助 Projection。全クライアントからアクセス可能。
/// [Rev.1] M-2 対応: preview_action は Operation trait に移動。
pub trait ProjectionQueryAPI: ProjectionAPI {
    /// 指定ノードから root までのパスを返す（コンテキスト把握用）。
    fn ancestors(&self, node: NodeId) -> Vec<NodeId>;

    /// 全 hole を優先度付きで返す（未充填で最も影響が大きい hole が先頭）。
    fn prioritized_holes(&self) -> Vec<PrioritizedHole>;

    /// AST を構造化テキストとして返す（LLM のコンテキスト用）。
    fn serialize_as_text(&self) -> String;

    /// 現在の AST から問題文形式のテキストを生成する。
    fn render_problem_text(&self) -> String;
}

pub struct PrioritizedHole {
    pub node: NodeId,
    pub priority: HolePriority,
    pub reason: String,
}

pub enum HolePriority {
    /// これを埋めないと他の hole の候補が決まらない。
    Critical,
    /// 問題として成立するために必要。
    Required,
    /// あると望ましいが、なくても最低限の問題構造は成立する。
    Optional,
}

pub struct PreviewResult {
    pub new_holes_created: Vec<NodeId>,
    pub constraints_affected: Vec<NodeId>,
    pub completeness_after: CompletenessSummary,
}
```

### 5.3 AI Agent の典型的な操作フロー

ABC284-C（グラフ連結成分数）を AI Agent が構築する例:

```
Step 1: agent は ProjectionAPI.completeness() で現状を把握
        → { total_holes: 5, filled: 0, is_complete: false }

Step 2: agent は prioritized_holes() で最優先の hole を取得
        → [PrioritizedHole { node: NodeId(2), priority: Critical,
            reason: "Header を決めないとデータ構造が決まらない" }]

Step 3: agent は hole_candidates(NodeId(2)) で候補を取得
        → [IntroduceScalar { suggested_names: ["N", "M"] }, ...]

Step 4: agent は Operation.preview で dry-run
        action = FillHole { target: NodeId(2),
                 fill: Scalar { name: "N", typ: Int } }
        → Ok(PreviewResult { new_holes_created: [...], ... })

Step 5: agent は Operation.apply(action) で確定
        → Ok(new_ast)

Step 6: agent は AddSlotElement でヘッダに M を追加
Step 7: agent は FillHole で EdgeList を追加
Step 8: agent は AddConstraint で "1 ≤ N ≤ 100" を追加
Step 9: agent は AddConstraint で "0 ≤ M ≤ N(N-1)/2" を追加
Step 10: agent は AddConstraint で Property("simple_graph") を追加
Step 11: agent は FillHole で Output を SingleValue(Int) として埋める
Step 12: completeness() → { total_holes: 0, is_complete: true }
```

### 5.4 CLI / テストコード での同一 API 使用例

```rust
// CLI: 対話的に構築
let proj = ProjectionView::new(&ast, &constraints);
for node in proj.nodes() {
    println!("{}{}", "  ".repeat(node.depth), node.label);
}
// ユーザー入力を Action に変換して apply

// テストコード: ABC300-A の構築を検証
let mut engine = OperationEngine::new();
engine.apply(Action::FillHole { target: header_hole, fill: scalar("N", Int) })?;
engine.apply(Action::AddSlotElement { parent: header, slot_name: "variables".into(),
    element: scalar("A", Int) })?;
engine.apply(Action::AddSlotElement { parent: header, slot_name: "variables".into(),
    element: scalar("B", Int) })?;
engine.apply(Action::FillHole { target: data_hole,
    fill: array("C", Int, ref_var(n_node)) })?;
engine.apply(Action::AddConstraint { target: n_node,
    constraint: range("1", "300") })?;
let proj = engine.projection();
assert!(proj.completeness().is_complete);
```

### 5.5 Agent 向け追加 API が**不要**な領域

以下は明示的に不要と判断した:

| 検討した候補 | 不要な理由 |
|---|---|
| `apply_batch(Vec<Action>)` | 現時点ではアトミック一括適用の需要が薄い。必要になったら Operation 層に追加可能 |
| `suggest_next_action()` | これは AI の推論であり、コア API に含めるべきでない。Agent 側で `prioritized_holes` + `hole_candidates` から自前で推論する |
| `undo()` / `redo()` | Operation 層の将来拡張。ProjectionAPI には不要 |
| `validate_all()` | `completeness()` + `available_actions()` で代替可能 |

---

## 付録: Operation trait 定義

```rust
/// Operation: Action を受け取り、新しい AST 状態を返す。
/// 妥当性検査 → 適用 → 新状態返却 のパイプライン。
/// [Rev.1] M-2 対応: preview メソッドを追加。
pub trait Operation {
    /// Action を適用し、成功すれば新しい AST 状態を返す。
    fn apply(&mut self, action: Action) -> Result<ApplyResult, OperationError>;

    /// [Rev.1] Action を適用した場合の結果をプレビューする（dry-run）。
    /// &self（不変参照）なので状態は変更しない。Operation 内部のロジックを再利用する。
    /// M-2 対応: ProjectionQueryAPI から移動。
    fn preview(&self, action: &Action) -> Result<PreviewResult, OperationError>;
}

/// [Rev.1] S-2 対応: created_constraints を追加、affected_constraints を ConstraintId に変更。
pub struct ApplyResult {
    /// 新しく生成されたノードの ID 一覧。
    pub created_nodes: Vec<NodeId>,
    /// 削除されたノードの ID 一覧。
    pub removed_nodes: Vec<NodeId>,
    /// [Rev.1] 新しく生成された制約の ID 一覧。
    pub created_constraints: Vec<ConstraintId>,
    /// 影響を受けた制約の ID 一覧。
    pub affected_constraints: Vec<ConstraintId>,
}
```

---

## [Rev.1] 改訂履歴

| 版 | 日付 | 変更内容 |
|----|------|----------|
| Sprint 2 (初版) | — | ProjectionOperationAgent による初期設計 |
| Rev.1 (Sprint 3 Synthesis) | — | S-2: RemoveConstraint を ConstraintId 指定に変更。ApplyResult に created_constraints 追加。ViolationDetail.constraint_id を ConstraintId に変更。S-3: FillContent に Builder 層分離の設計方針を文書化。M-2: preview_action を ProjectionQueryAPI から Operation trait に移動。L-4: Hole への AddConstraint を許可（InvalidOperation の例を修正）。 |
