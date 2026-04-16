# Phase 2: Domain Model Agent Report

本文書は、Editor UI設計における**ドメインモデル設計**の判断と提案を記録する。

**担当領域**: AST / Structure / Constraint / Expression の責務分離、Hole意味論、EditorState境界、Projection / Operation API骨格、Stable ID / SourceRef設計、型設計提案

---

## 1. 前提

### 1.1 固定前提の確認

以下のphase1-premises.md前提を継承・確認した:

| # | 前提 | 本Agentの解釈 |
|---|------|---------------|
| A-1 | AST は1種類（StructureAST + ConstraintAST + Expression） | **完全支持**。AST分離は複雑性を招く |
| A-2 | AST が唯一の正本 | **完全支持**。EditorStateは補助にすぎない |
| A-5 | Operation API経由でAST更新、Projection APIはread-only | **完全支持** |
| A-6 | Arena-based ID管理（NodeId, ConstraintId） | **完全支持**。stable identity必須 |
| A-7 | Holeは第一級市民 | **完全支持**。Hazelnut方式の継承 |
| B-2 | Expression未完成 → EditorState/PendingExprAction | **完全支持**。AST汚染回避 |
| B-3 | Constraint未完成 → EditorState/DraftConstraint | **完全支持** |

### 1.2 現行cp-ast-coreの型棚卸し（実装確認済み）

**Structure層**:
- `NodeId(u64)`: グローバル一意ID（AtomicU64で生成）
- `NodeKind`: Scalar, Array, Matrix, Tuple, Repeat, Section, Sequence, Choice, Hole の9 variants
- `Reference`: VariableRef(NodeId), IndexedRef{target, indices}, Unresolved(Ident)
- `Ident`: 名前wrapper
- `Literal`: IntLit(i64), StrLit(String)
- `NodeKindHint`: Hole期待型ヒント

**Constraint層**:
- `ConstraintId(u64)`: 制約一意ID
- `Constraint`: 12 variants（Range, TypeDecl, LengthRelation, Relation, Distinct, Property, SumBound, Sorted, Guarantee, CharSet, StringLength, RenderHint）
- `Expression`: Lit, Var, BinOp, Pow, FnCall

**Operation層**:
- `AstEngine`: structure + constraints を管理
- `Action`: FillHole, ReplaceNode, AddConstraint, RemoveConstraint, IntroduceMultiTestCase, AddSlotElement, RemoveSlotElement

**Projection層（現行）**:
- `ProjectionAPI` trait: nodes(), children(), inspect(), hole_candidates(), available_actions(), why_not_editable(), completeness()
- 返却型は基本的なもののみ実装済み

---

## 2. 主要判断

### 2.1 ExprId / SlotId / SourceRef の型設計（U-1, U-2）

#### 2.1.1 ExprId

**判断: 新規ID型 `ExprId(u64)` を導入する**

```rust
/// 式ノードの一意識別子
/// Expression内の部分式を個別に選択・操作するために必要
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(u64);

impl ExprId {
    #[must_use]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
    
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}
```

**必要性の根拠**:
- plan.md §18.2 `ExprRenderView` が `exprId: ExprId` を要求
- 式の部分選択（`N` in `N/2`）にはサブ式IDが必須
- PendingExprActionの `targetExpr` に使用

**実装方針**:
- Expression構築時にIDを割り当てる
- 現行Expressionはvalue型なので、`AnnotatedExpression { id: ExprId, expr: Expression }` または arena-based 管理を検討

**MVP判断**: **MVP対応**（式スロット編集に必須）

#### 2.1.2 SlotId

**判断: 新規ID型 `SlotId` を導入する**

```rust
/// 式スロットの識別子
/// (所属ノード, スロット名, 配列インデックス) の組み合わせで一意に特定
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlotId {
    /// スロットを持つノード
    pub owner: NodeId,
    /// スロット名（"length", "lower", "upper", "count" など）
    pub slot_name: String,
    /// 配列スロットの場合のインデックス（通常はNone）
    pub index: Option<usize>,
}
```

**必要性の根拠**:
- plan.md §32 式スロット契約で slot_id が必要
- `project_slot_render(slot_id)` API に必須
- 長さスロット、Range上下界、Relation左右辺を統一的に扱う

**MVP判断**: **MVP対応**

#### 2.1.3 SourceRef

**判断: 4種類のSourceRefをenumで統一**

```rust
/// UI のクリック可能領域と AST 上の対象を結ぶ参照
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceRef {
    /// ノード全体
    Node(NodeId),
    /// 式（部分式含む）
    Expr(ExprId),
    /// 式スロット
    Slot(SlotId),
    /// 制約
    Constraint(ConstraintId),
}
```

**設計根拠**:
- plan.md §35 SourceRef契約に完全準拠
- DOM位置依存を回避し、意味論的対象を直接指す
- Preact側は `data-source-ref` 属性でこれを保持

**MVP判断**: **MVP対応**

---

### 2.2 AST不変条件リスト

plan.md §27.3 に基づき、以下の不変条件を維持する:

#### Structure不変条件

| # | 不変条件 | 検証タイミング |
|---|----------|---------------|
| S-INV-1 | rootは常に存在する | AstEngine::new()後、全Operation後 |
| S-INV-2 | NodeIdはstableである（変更されない） | 全Operation |
| S-INV-3 | 親子参照の整合性（childのparentが正しい） | AddSlotElement, RemoveSlotElement後 |
| S-INV-4 | Holeは正規ノード（null/undefined禁止） | 全Operation |
| S-INV-5 | Reference.VariableRef(id) のidは存在するノード | FillHole, ReplaceNode後 |

#### Expression不変条件

| # | 不変条件 | 検証タイミング |
|---|----------|---------------|
| E-INV-1 | 常に完成式（Holeを含まない） | スロット代入時 |
| E-INV-2 | 部分式を含まない（`?` 的な穴なし） | スロット代入時 |
| E-INV-3 | Reference.Unresolved は一時状態のみ | AST正規化後 |

#### Constraint不変条件

| # | 不変条件 | 検証タイミング |
|---|----------|---------------|
| C-INV-1 | 常に完成済み（全必須フィールド充足） | AddConstraint時 |
| C-INV-2 | target/lhs/rhs等が有効なReferenceを持つ | AddConstraint時 |
| C-INV-3 | ConstraintIdは一意 | ConstraintSet::add()時 |

---

### 2.3 EditorState vs AST の境界

| 状態 | 配置先 | 根拠 |
|------|--------|------|
| Structure Hole | **AST** | 位置・順序・親子関係に意味論的価値（plan.md §26.2） |
| 完成済みExpression | **AST** | 正本として保存・生成に使用 |
| 未完成Expression（`N/?`） | **EditorState** | PendingExprActionとして管理（plan.md §26.3） |
| 完成済みConstraint | **AST** | 正本 |
| 未完成Constraint | **EditorState** | DraftConstraintとして管理（plan.md §26.4） |
| 選択状態 | **EditorState** | UI一時状態 |
| ドラッグ状態 | **EditorState** | UI一時状態 |
| メニュー開閉 | **EditorState** | UI一時状態 |
| undo/redo履歴 | **AST層管理だがASTに混在させない** | 別データ構造 |

**EditorState型定義提案**:

```typescript
// TypeScript側（Preact）
interface EditorState {
    // Selection
    selectedNodeId: NodeId | null;
    selectedConstraintId: ConstraintId | null;
    selectedExprSource: SourceRef | null;
    
    // Pending actions
    pendingExprAction: PendingExprAction | null;
    draftConstraint: DraftConstraint | null;
    
    // UI state
    openPopover: PopoverState | null;
    openModal: ModalState | null;
    dragState: DragState | null;
    
    // Preview
    samplePreview: SamplePreviewState;
    diagnosticsFilter: DiagnosticsFilter;
}
```

---

### 2.4 Projection API 拡張骨格（U-4）

現行 `ProjectionAPI` trait を拡張し、plan.md §17.1 の分類に対応する:

```rust
// ===== Render Projection（描画最小情報） =====

/// 構造全体のアウトラインを返す
fn project_structure_outline(&self) -> StructureOutlineView;

/// 単一ノードの描画情報
fn project_node_render(&self, node_id: NodeId) -> Option<NodeRenderView>;

/// 制約の描画情報
fn project_constraint_render(&self, constraint_id: ConstraintId) -> Option<ConstraintRenderView>;

/// 式の描画情報（部分式にSourceRef付与）
fn project_expr_render(&self, source: &SourceRef) -> Option<ExprRenderView>;

/// スロットの描画情報
fn project_slot_render(&self, slot_id: &SlotId) -> Option<SlotRenderView>;

/// 診断情報一覧
fn project_diagnostics(&self) -> DiagnosticsView;

/// 完成度サマリ
fn project_completeness(&self) -> CompletenessView;

// ===== Action Projection（click時候補） =====

/// ノードに対するアクション一覧
fn project_node_actions(&self, node_id: NodeId) -> NodeActionMenu;

/// Holeの候補一覧
fn project_hole_candidates(&self, hole_id: NodeId) -> HoleCandidateMenu;

/// 式に対するアクション一覧
fn project_expr_actions(&self, source: &SourceRef) -> ExprActionMenu;

/// 式スロットの候補一覧
fn project_expr_slot_candidates(&self, slot_id: &SlotId) -> ExprCandidateMenu;

/// 制約targetの候補一覧
fn project_constraint_target_candidates(&self, kind: ConstraintKind) -> TargetCandidateMenu;

// ===== Global Projection（全体一覧） =====

/// 全ノード一覧（現行nodes()と互換）
fn nodes(&self) -> Vec<ProjectedNode>;

/// 全制約一覧
fn constraints(&self) -> Vec<ProjectedConstraint>;

/// 参照可能な変数一覧（scope考慮）
fn available_references(&self, context: &SlotId) -> Vec<ReferenceCandidate>;
```

**返却型の拡張**:

```rust
/// ノードの描画情報（plan.md §18.1準拠）
pub struct NodeRenderView {
    pub node_id: NodeId,
    pub label: String,
    pub kind_label: String,
    pub is_hole: bool,
    pub expected_kind_hint: Option<NodeKindHint>,
    pub badges: Vec<String>,
    pub children: Vec<ChildEntryView>,
    pub diagnostics: Vec<DiagnosticSummary>,
    pub clickable_regions: Vec<ClickableRegion>,
}

/// 式の描画情報（plan.md §18.2準拠）
pub struct ExprRenderView {
    pub expr_id: ExprId,
    pub root: InlineNode,
}

pub enum InlineNode {
    Token {
        text: String,
        source: SourceRef,
        clickable: bool,
    },
    Group {
        source: SourceRef,
        children: Vec<InlineNode>,
    },
}

/// スロットの描画情報
pub struct SlotRenderView {
    pub slot_id: SlotId,
    pub label: String,
    pub current_expr: Option<ExprRenderView>,
    pub expected_sort: ExprSort,
    pub slot_rule: SlotRule,
    pub is_empty: bool,
}

/// 式スロットの期待される式の種別（plan.md §32.3）
pub enum ExprSort {
    IntExpr,
    BoolExpr,
    PlaceExpr,
}

/// スロット固有のルール（plan.md §32.4）
pub enum SlotRule {
    NonNegative,
    NoSelfReference,
    ComparableWithTarget,
    AllowIndexRef,
    IntegerOnly,
}

/// クリック可能領域
pub struct ClickableRegion {
    pub source: SourceRef,
    pub region_kind: RegionKind,
}

pub enum RegionKind {
    NodeHeader,
    ExprToken,
    SlotEmpty,
    ConstraintTarget,
}
```

---

### 2.5 wasm-exposed core API 提案

```rust
// ===== wasm-bindgen公開関数（cp-ast-wasm crate） =====

// --- Render Projection ---
#[wasm_bindgen]
pub fn project_structure_outline(engine_json: &str) -> String; // JSON

#[wasm_bindgen]
pub fn project_node_render(engine_json: &str, node_id: &str) -> String;

#[wasm_bindgen]
pub fn project_constraint_render(engine_json: &str, constraint_id: &str) -> String;

#[wasm_bindgen]
pub fn project_expr_render(engine_json: &str, source_ref_json: &str) -> String;

#[wasm_bindgen]
pub fn project_slot_render(engine_json: &str, slot_id_json: &str) -> String;

#[wasm_bindgen]
pub fn project_diagnostics(engine_json: &str) -> String;

#[wasm_bindgen]
pub fn project_completeness(engine_json: &str) -> String;

// --- Action Projection ---
#[wasm_bindgen]
pub fn project_node_actions(engine_json: &str, node_id: &str) -> String;

#[wasm_bindgen]
pub fn project_hole_candidates(engine_json: &str, hole_id: &str) -> String;

#[wasm_bindgen]
pub fn project_expr_actions(engine_json: &str, source_ref_json: &str) -> String;

#[wasm_bindgen]
pub fn project_expr_slot_candidates(engine_json: &str, slot_id_json: &str) -> String;

// --- Operation ---
#[wasm_bindgen]
pub fn apply_action(engine_json: &str, action_json: &str) -> String; // Result<ApplyResult>

#[wasm_bindgen]
pub fn preview_action(engine_json: &str, action_json: &str) -> String; // Result<PreviewResult>

// --- Sample ---
#[wasm_bindgen]
pub fn generate_sample(engine_json: &str) -> String;

#[wasm_bindgen]
pub fn render_canonical_spec(engine_json: &str) -> String;
```

**API境界の原則**:
- wasm ↔ JS間はすべてJSON string（E-4前提）
- IDはdecimal string（E-5前提、JS 53bit精度問題回避）
- 各関数は状態を持たず、engine_jsonを受け取り結果を返す

---

## 3. 具体例

### 3.1 `N: int` の型フロー

**ユーザー操作**: Structure ペインで「+変数」→ 名前「N」→ 型「int」

**関与する型・API**:

```
[UI] "+ 変数" クリック
  ↓
[Preact] project_hole_candidates(root_hole_id) 呼び出し
  ↓
[wasm] HoleCandidateMenu { candidates: [IntroduceScalar { suggested_names: ["N", "M"] }] }
  ↓
[Preact] Scalar選択 → 名前入力 → DraftConstraint(TypeDecl) 作成
  ↓
[Preact] 確定時: apply_action(FillHole { target: hole_id, fill: Scalar { name: "N" } })
         + apply_action(AddConstraint { target: N, constraint: TypeDecl { expected: Int } })
  ↓
[wasm] ApplyResult { created_nodes: [NodeId(1)], created_constraints: [ConstraintId(1)] }
```

**AST状態**:
```
StructureAst:
  root: NodeId(0) = Sequence { children: [NodeId(1)] }
  NodeId(1) = Scalar { name: Ident("N") }

ConstraintSet:
  ConstraintId(1) = TypeDecl { target: VariableRef(NodeId(1)), expected: Int }
```

**EditorState変化**:
- `selectedNodeId: NodeId(1)`
- `draftConstraint: null` （確定後破棄）

---

### 3.2 `A: int[N]` の型フロー

**ユーザー操作**: 「+配列」→ 名前「A」→ 型「int」→ 長さスロットで「N」選択

**関与する型・API**:

```
[UI] "+ 配列" クリック
  ↓
[Preact] FillHole { fill: Array { name: "A", length: Hole } } として仮挿入
  ↓
[wasm] NodeId(2) = Array { name: "A", length: Unresolved("?") } が作成される
       ※ 長さスロットは未設定状態
  ↓
[UI] 長さスロットをクリック
  ↓
[Preact] project_expr_slot_candidates(SlotId { owner: NodeId(2), slot_name: "length", index: None })
  ↓
[wasm] ExprCandidateMenu {
         references: [ReferenceCandidate { target: NodeId(1), label: "N", ref_type: IntExpr }],
         constants: [1, 10, 100],
         templates: ["+x", "-x", "/x", "*x"]
       }
  ↓
[Preact] "N" 選択
  ↓
[Preact] apply_action(SetArrayLengthExpr { node: NodeId(2), expr: Var(VariableRef(NodeId(1))) })
  ↓
[wasm] ApplyResult { affected_nodes: [NodeId(2)] }
```

**AST状態**:
```
StructureAst:
  NodeId(2) = Array { name: Ident("A"), length: Var(VariableRef(NodeId(1))) }

ConstraintSet:
  ConstraintId(2) = TypeDecl { target: IndexedRef { target: NodeId(2), indices: [Ident("i")] }, expected: Int }
```

**関与する型**:
- `SlotId { owner: NodeId(2), slot_name: "length" }`
- `Expression::Var(Reference::VariableRef(NodeId(1)))`
- `ExprCandidateMenu`
- `ReferenceCandidate`

---

### 3.3 `1 <= A[i] <= 10^9` の型フロー

**ユーザー操作**: Constraintsペインで「+Range」→ target「A[i]」→ lower「1」→ upper「10^9」

**関与する型・API**:

```
[UI] "+ Range" クリック
  ↓
[Preact] DraftConstraint::Range { target: None, lower: None, upper: None } を EditorStateに作成
  ↓
[UI] target フィールドクリック
  ↓
[Preact] project_constraint_target_candidates(ConstraintKind::Range)
  ↓
[wasm] TargetCandidateMenu {
         candidates: [
           TargetCandidate { ref: IndexedRef { target: NodeId(2), indices: ["i"] }, label: "A[i]" },
           TargetCandidate { ref: VariableRef(NodeId(1)), label: "N" }
         ]
       }
  ↓
[Preact] "A[i]" 選択 → DraftConstraint更新
  ↓
[UI] lower フィールドに "1" 入力 → DraftConstraint更新
[UI] upper フィールドで式構築
  ↓
[Preact] PendingExprAction 不要（直接入力）
         または 式構築UI: "10" → "^9" 適用 → Pow { base: Lit(10), exp: Lit(9) }
  ↓
[Preact] DraftConstraint完成: { target: A[i], lower: Lit(1), upper: Pow(10,9) }
  ↓
[Preact] apply_action(AddConstraint {
           target: NodeId(2),  // A
           constraint: ConstraintDef::Range {
             target: IndexedRef { target: NodeId(2), indices: ["i"] },
             lower: Lit(1),
             upper: Pow { base: Lit(10), exp: Lit(9) }
           }
         })
  ↓
[wasm] ApplyResult { created_constraints: [ConstraintId(3)] }
```

**AST状態**:
```
ConstraintSet:
  ConstraintId(3) = Range {
    target: IndexedRef { target: NodeId(2), indices: [Ident("i")] },
    lower: Lit(1),
    upper: Pow { base: Box::new(Lit(10)), exp: Box::new(Lit(9)) }
  }
```

**EditorState変化**:
- 構築中: `draftConstraint: Some(DraftConstraint::Range { ... })`
- 確定後: `draftConstraint: None`

---

### 3.4 Tree 辺リストの型フロー

**ユーザー操作**: 「+辺リスト」テンプレート選択 → 辺数「N-1」→ Property「Tree」追加

**関与する型・API**:

```
[UI] "+ 辺リスト" テンプレート選択
  ↓
[Preact] 高レベルテンプレート展開:
         Builder層が以下のAction列を生成:
         1. AddSlotElement { parent: root, slot_name: "children", element: Repeat { count: ?, body: Tuple(u,v) } }
         2. FillHole (count slot)
         3. AddConstraint (u: 1 <= u[i] <= N)
         4. AddConstraint (v: 1 <= v[i] <= N)
         5. AddConstraint (Relation: u[i] < v[i])  // optional
  ↓
[Preact] 辺数スロットの式構築UI表示
  ↓
[UI] "N" 選択 → "-1" 適用
  ↓
[Preact] PendingExprAction = { kind: "wrapBinary", targetExpr: ExprId(N), op: Sub, phase: "select-rhs" }
  ↓
[UI] "1" 選択
  ↓
[Preact] PendingExprAction完成 → Expression::BinOp { op: Sub, lhs: Var(N), rhs: Lit(1) }
  ↓
[Preact] apply_action(SetRepeatCountExpr { node: repeat_id, expr: N-1 })
  ↓
[UI] Property追加
  ↓
[Preact] apply_action(AddConstraint { target: repeat_id, constraint: Property { tag: Tree } })
```

**AST状態**:
```
StructureAst:
  NodeId(10) = Repeat {
    count: BinOp { op: Sub, lhs: Var(VariableRef(NodeId(1))), rhs: Lit(1) },
    index_var: Some(Ident("i")),
    body: [NodeId(11)]
  }
  NodeId(11) = Tuple { elements: [NodeId(12), NodeId(13)] }
  NodeId(12) = Scalar { name: Ident("u") }
  NodeId(13) = Scalar { name: Ident("v") }

ConstraintSet:
  ConstraintId(10) = Property { target: VariableRef(NodeId(10)), tag: Tree }
  ConstraintId(11) = Range { target: IndexedRef(NodeId(12), ["i"]), lower: Lit(1), upper: Var(N) }
  ConstraintId(12) = Range { target: IndexedRef(NodeId(13), ["i"]), lower: Lit(1), upper: Var(N) }
```

---

## 4. 現行方針に対する支持/反対/留保

### 4.1 完全支持

| # | 方針 | 理由 |
|---|------|------|
| plan.md §27.1 | AST 1種類 | 分離ASTは状態同期問題を招く。1種類で十分 |
| plan.md §26.2 | Structure HoleをAST内に配置 | 位置・順序が意味論的（Q-2回答: **十分**） |
| plan.md §26.3 | Expression未完成をEditorStateに | AST汚染回避、操作テンプレート方式と整合 |
| plan.md §26.4 | Constraint未完成をEditorStateに | フォーム的入力と相性良好 |
| plan.md §29 | Projection 3分類 | render/action/global分離で過剰projection回避 |
| plan.md §35 | SourceRefで意味論的対象を指す | DOM依存排除、stable identity確保 |

### 4.2 部分支持（条件付き）

| # | 方針 | 条件 |
|---|------|------|
| plan.md §32 式スロット統一設計 | **完全対応するが段階的実装を推奨** — MVP Phase 1では IntExpr + 参照のみ、Phase 2で BoolExpr, PlaceExpr追加 |
| plan.md §17.2 EditorAction列挙 | **完全対応するが拡張可能設計が必要** — MoveNode, UpdateConstraint は追加候補 |

### 4.3 留保

| # | 方針 | 留保理由 |
|---|------|----------|
| plan.md §33 PendingExprAction | 詳細フェーズ遷移はGUI Interaction Agentに委ねる（U-5） |
| plan.md §34 DraftConstraint完了条件 | 詳細ロジックはGUI Interaction Agentに委ねる（U-6） |

---

## 5. 不足点

### 5.1 ExprIdの具体的管理方式

**現状**: Expression は value型で定義されており、ExprId を持たない

**提案**: 2つの選択肢

1. **Annotated Expression 方式**:
   ```rust
   pub struct AnnotatedExpr {
       pub id: ExprId,
       pub expr: Expression,
   }
   ```
   - Pro: 既存Expressionと互換
   - Con: ネストした式でIDを全部持つと冗長

2. **Arena-based Expression 方式**:
   ```rust
   pub struct ExprArena {
       exprs: Vec<Option<Expression>>,
       next_id: u64,
   }
   ```
   - Pro: NodeId/ConstraintIdと同じ管理方式
   - Con: 現行Expression APIの変更が大きい

**推奨**: MVP では **Option 1（Annotated方式）** を採用し、wasm公開時のみID付与。内部Expressionはvalue型のまま。

### 5.2 SlotIdの親ノード決定問題

現行の Array.length は `Expression` 型だが、slot として管理するには owner NodeId が必要。

**提案**: `SlotId` は Projection 層で動的に生成する（ASTに保存しない）

```rust
fn project_slot_render(&self, slot_id: &SlotId) -> Option<SlotRenderView> {
    let node = self.structure.get(slot_id.owner)?;
    match &node.kind() {
        NodeKind::Array { length, .. } if slot_id.slot_name == "length" => {
            Some(SlotRenderView {
                slot_id: slot_id.clone(),
                current_expr: Some(self.render_expr(length)),
                expected_sort: ExprSort::IntExpr,
                // ...
            })
        }
        // ...
    }
}
```

### 5.3 undefinedフィールドの表現

現行 `Array.length` は `Expression` 型で「未設定」を表現できない。

**提案**: `Option<Expression>` に変更、または `Expression::Placeholder` を導入

```rust
// 案1: Option化（推奨）
Array { name: Ident, length: Option<Expression> }

// 案2: Placeholder variant
pub enum Expression {
    // 既存...
    Placeholder { slot_id: SlotId }, // 未設定を表現
}
```

**推奨**: **案1** — Optionの方がRustイディオムに沿う

### 5.4 undo/redo戦略（U-10）

plan.md §11で後回し候補だが、AstEngine レベルでのスナップショット or command履歴は考慮が必要。

**MVP判断**: **MVP後回し** — ただしAstEngine::apply()の返り値に逆操作情報を含めておくと後から対応しやすい

---

## 6. 他Agentに渡すべき論点

### 6.1 GUI Interaction Agent へ

| 論点 | 詳細 |
|------|------|
| PendingExprAction フェーズ遷移詳細 | `/x` 適用時の候補絞り込み、キャンセル処理、部分確定など |
| DraftConstraint 完了条件ロジック | 各ConstraintKindごとの必須フィールド検証 |
| 式構築UIの操作フロー | Token選択 → Action選択 → 引数選択 の具体的遷移 |
| Q-3: Expr partialを持たない設計で十分か | 複雑な式（三項演算等）でPendingだけで足りるか |
| Q-7: click時候補列挙で十分か | パフォーマンス含めた検証依頼 |

### 6.2 wasm Boundary Agent へ

| 論点 | 詳細 |
|------|------|
| ExprIdのJSON表現 | string vs number、ネスト式のID割り当てタイミング |
| Projection API分割粒度 | 1関数 vs 複数関数、バッチAPIの必要性 |
| U-8: diagnostics計算タイミング | 毎回全計算 vs 差分計算 vs 遅延計算 |
| U-11: diff-based更新 vs full再描画 | projection結果の差分検出方式 |

### 6.3 Sample Generation Agent へ

| 論点 | 詳細 |
|------|------|
| Q-9: sample generationへの接続 | **完全対応** — AST不変条件（Hole非存在）を満たせば生成可能 |
| Expression評価 | Var参照解決、BinOp/Pow/FnCall評価の優先度依存 |
| Constraint相互作用検出 | Distinct + Range の充足可能性判定 |

### 6.4 Critical Reviewer へ

| 論点 | 詳細 |
|------|------|
| Q-1: ASTは1種類でよいか | **回答: 十分** — 設計検証依頼 |
| Q-10: MVPで何を切るか | ExprId完全実装、undo/redo、複雑な式テンプレート |

---

## 7. Q&A 回答

### Q-1: ASTは1種類で十分か

**回答: 完全対応**

理由:
1. Structure Hole が未完成状態を AST 内で表現
2. Expression/Constraint の未完成は EditorState で管理
3. 別 AST（DraftAST, EditAST等）を導入すると同期問題が発生
4. 1 種類で canonical rendering, sample generation, validation すべてに対応可能

### Q-2: Structure Hole の役割は十分か

**回答: 完全対応**

理由:
1. Hole は位置・順序・親子関係を保持（意味論的価値）
2. NodeKindHint で期待される種別を表現
3. Hole への制約付与が可能（L-4対応済み）
4. FillHole 時に制約継承

### Q-5: Expression slot 設計の具体化

**回答: 部分対応**（本文書で SlotId, ExprSort, SlotRule を定義）

追加検討が必要な点:
- BoolExpr の具体的ケース（MVP後回し候補）
- PlaceExpr と Reference の関係整理
- 複合スロット（複数式を受け取る）の扱い

### Q-9: sample generation につながるか

**回答: 完全対応**

接続条件:
1. AST 不変条件を満たす（Hole 非存在、Expression 完成、Constraint 完成）
2. 依存グラフが非巡回
3. 各 Constraint が生成器対応範囲内

現行 sample モジュールとの接続:
- `AstEngine.structure` + `AstEngine.constraints` をそのまま入力
- `project_completeness().is_complete == true` で生成可能判定

---

## 8. 結論

本 Domain Model Agent Report により、以下を確定した:

1. **ExprId, SlotId, SourceRef** の型設計提案（MVP対応）
2. **AST不変条件** 15項目のリスト化
3. **EditorState vs AST** の境界明確化
4. **Projection API 拡張骨格** 17メソッド提案
5. **wasm-exposed API** 15関数提案
6. **具体例** 4件による型フロー検証
7. **Q-1, Q-2, Q-5, Q-9** への回答

次ステップとして、GUI Interaction Agent / wasm Boundary Agent への論点引き継ぎを推奨する。
