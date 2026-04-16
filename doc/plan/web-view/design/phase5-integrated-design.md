# Phase 5: 統合設計書

> 作成日: 2025-07-18
> プロセス: process.md §Phase 5 準拠
> 依存: phase1-premises.md, phase2-*-report.md (6件), phase4-cross-critique.md

---

## 1. Executive Summary

本文書は、CP AST Editor UIの最終アーキテクチャを定める統合設計書である。

### 設計の核心

1. **AST単一正本主義**: StructureAST + ConstraintAST + Expressionの3層で入力仕様を表現。UIはASTの投影であり、正本ではない。

2. **未完成状態の明確な配置**:
   - Structure未完成 → AST内のHoleノード
   - Expression未完成 → EditorState（PendingExprAction）
   - Constraint未完成 → EditorState（DraftConstraint）

3. **wasm stateless設計**: 各wasm呼び出しにdocument JSONを渡し、状態を持たない。Preact側がdocumentを保持し、signalで管理。

4. **MVP Phase 1スコープ**:
   - Structure: Scalar, Array, Tuple, Sequence, Repeat, Hole
   - Constraint: Range, TypeDecl, LengthRelation, Relation
   - Expression: Lit, Var, BinOp (+, -, *, /)
   - wasm API: 8関数に統合
   - UI: 3カラムレイアウト、式スロット全置換、DraftConstraint 4種

### 主要決定事項

| 決定 | 採用方針 | 根拠 |
|------|----------|------|
| ExprId | **MVP延期** | 実装コスト高、式スロット全置換で十分 |
| Projection API | **7-8関数に統合** | 境界呼び出し回数削減 |
| SlotId.slot_name | **enum (SlotKind)** | 型安全性向上 |
| SumBound | **Phase 2** | Generator拡張コスト高 |
| テンプレート | **Phase 2** | 手動構築で代替可能 |
| undo/redo | **Phase 3** | 実装複雑性が高い |

---

## 2. 前提

### 2.1 固定前提（Phase 1より）

| ID | 前提 | 根拠 |
|----|------|------|
| A-1 | ASTは1種類（StructureAST + ConstraintAST + Expression） | plan.md §27.1 |
| A-2 | ASTが唯一の正本 | plan.md §27.2 |
| A-3 | UIはASTの投影（projectional editor） | main.md 設計原則1 |
| A-4 | wasm(Rust)は意味論、Preact(TS)は体験を担う | plan.md §9.1 |
| A-5 | 操作はOperation API経由でAST更新、Projection APIはread-only | plan.md §29.1, §30.1 |
| A-6 | Arena-basedのID管理（NodeId, ConstraintId） | main.md, cp-ast-core実装 |
| A-7 | Holeは第一級市民 | main.md 設計原則4 |
| B-1 | Structure未完成 → ASTのNodeKind::Hole | plan.md §26.2 |
| B-2 | Expression未完成 → EditorState/PendingExprAction | plan.md §26.3 |
| B-3 | Constraint未完成 → EditorState/DraftConstraint | plan.md §26.4 |
| C-1〜C-4 | Projection分離（render/action/global）、click時遅延取得 | plan.md §29 |
| D-1〜D-4 | 基本ケース優先、template-driven、3カラムレイアウト、式スロット統一 | process.md §2.3 |
| E-1〜E-5 | Rust 2021, wasm-pack, Preact+Vite, JSON string境界, decimal string ID | 技術制約 |

### 2.2 解決済み未確定事項

Cross-critique (Phase 4) で解決された事項:

| ID | 事項 | 解決 |
|----|------|------|
| U-1 | ExprId/SlotIdの型設計 | ExprIdはMVP延期、SlotIdはenum化 |
| U-2 | SourceRefの具体実装 | 4種enum採用（Node/Slot/Constraint/Expr） |
| U-3 | EditorActionの完全列挙 | 7種Action + SetExpr追加 |
| U-4 | Projection API拡張 | 統合版8関数採用 |
| U-5 | PendingExprActionのフェーズ管理 | MVP簡略化（候補選択のみ） |
| U-6 | DraftConstraintの完了条件 | 4種に限定、必須フィールド埋め確認 |
| U-7 | 高レベルテンプレート実装方式 | Phase 2延期 |
| U-8 | diagnostics計算タイミング | project_full()で一括取得 |
| U-9 | sample generation連携 | generate_sample() + check_generatability() |
| U-10 | undo/redo戦略 | Phase 3延期（Preact側history予定） |
| U-11 | diff-based更新 vs full再描画 | full再描画（MVP）、差分はPhase 2以降 |

---

## 3. 各Subagentの結論サマリ

### 3.1 Domain Model Agent

**主要提案**:
- ExprId, SlotId, SourceRefの新規ID型導入
- Projection API 20関数拡張
- AST不変条件の明文化

**統合への採用**:
- ✅ SourceRef (4種enum)
- ✅ SlotId (ただしslot_nameはenum化)
- ✅ AST不変条件
- ❌ ExprId (MVP延期)
- ❌ Projection API 20関数 (8関数に統合)

### 3.2 GUI Interaction Agent

**主要提案**:
- 3カラム+下部パネルレイアウト
- 7種基本ケースの操作フロー
- PendingExprAction/DraftConstraintの詳細設計

**統合への採用**:
- ✅ 3カラムレイアウト
- ✅ 操作フロー（基本ケース）
- ✅ DraftConstraint（4種に限定）
- △ PendingExprAction（簡略化: 候補選択のみ）

### 3.3 Real Problem Coverage Agent

**主要提案**:
- 36問の検証（31完全対応、3部分対応、2非対応）
- 下三角行列・行内可変長の設計限界指摘
- SumBound/Propertyの重要性

**統合への採用**:
- ✅ 基本ケースカバレッジ目標（ABC A-D 90%以上）
- ✅ 下三角行列はMVP非対応と明示
- △ SumBound (Phase 2)
- △ Property (MVPはGuaranteeで代替)

### 3.4 wasm Boundary Agent

**主要提案**:
- wasm stateless設計
- JSON string境界維持
- 17関数のAPI提案

**統合への採用**:
- ✅ stateless設計
- ✅ JSON string境界
- △ API数 (8関数に統合)

### 3.5 Sample Generation Agent

**主要提案**:
- check_generatability API追加
- 保証レベル(L1/L2/L3)の明示
- SumBound Generator拡張の必要性

**統合への採用**:
- ✅ check_generatability（簡易版）
- ✅ 保証レベル概念
- △ SumBound (Phase 2)

### 3.6 Critical Reviewer Agent

**主要提案**:
- ExprId実装コストの過小評価指摘
- MVPスコープの絞り込み
- Projection API統合案

**統合への採用**:
- ✅ ExprId延期
- ✅ MVPスコープ絞り込み
- ✅ Projection API統合

---

## 4. 統合アーキテクチャ

### 4.1 AST契約

#### 4.1.1 StructureAST型定義

```rust
/// ノードの一意識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

/// ノード種別
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// 単一変数: N, M, S
    Scalar { name: Ident },
    /// 一次元配列: A_1 ... A_N
    Array { name: Ident, length: Expression },
    /// 二次元配列: A[i][j]（MVP Phase 2）
    Matrix { name: Ident, rows: Reference, cols: Reference },
    /// 同一行変数群: (N, M, K)
    Tuple { elements: Vec<NodeId> },
    /// 繰り返し構造: M行、Tテストケース
    Repeat {
        count: Expression,
        index_var: Option<Ident>,
        body: Vec<NodeId>,
    },
    /// セマンティックブロック（MVP Phase 2）
    Section { header: Option<NodeId>, body: Vec<NodeId> },
    /// 入力全体の順序付きルート
    Sequence { children: Vec<NodeId> },
    /// 分岐構造（MVP Phase 2）
    Choice { tag: Reference, variants: Vec<(Literal, Vec<NodeId>)> },
    /// 未定義位置
    Hole { expected_kind: Option<NodeKindHint> },
}

/// 参照
#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
    VariableRef(NodeId),
    IndexedRef { target: NodeId, indices: Vec<Ident> },
    Unresolved(Ident),
}
```

#### 4.1.2 ConstraintAST型定義

```rust
/// 制約の一意識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintId(u64);

/// 制約種別（MVP: Range, TypeDecl, LengthRelation, Relation）
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// 値範囲: lower ≤ target ≤ upper
    Range { target: Reference, lower: Expression, upper: Expression },
    /// 型宣言
    TypeDecl { target: Reference, expected: ExpectedType },
    /// 長さ関係
    LengthRelation { target: Reference, length: Expression },
    /// 変数関係: lhs op rhs
    Relation { lhs: Expression, op: RelationOp, rhs: Expression },
    // --- 以下MVP後回し ---
    Distinct { elements: Reference, unit: DistinctUnit },
    Property { target: Reference, tag: PropertyTag },
    SumBound { variable: Reference, upper: Expression },
    Sorted { elements: Reference, order: SortOrder },
    Guarantee { description: String, predicate: Option<Expression> },
    CharSet { target: Reference, charset: CharSetSpec },
    StringLength { target: Reference, min: Expression, max: Expression },
    RenderHint { target: Reference, hint: RenderHintKind },
}

/// 型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedType { Int, Str, Char }

/// 比較演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationOp { Lt, Le, Gt, Ge, Eq, Ne }
```

#### 4.1.3 Expression型定義

```rust
/// 式（MVP: Lit, Var, BinOp）
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// 整数リテラル
    Lit(i64),
    /// 変数参照
    Var(Reference),
    /// 二項演算: lhs op rhs
    BinOp { op: ArithOp, lhs: Box<Expression>, rhs: Box<Expression> },
    // --- 以下MVP後回し ---
    Pow { base: Box<Expression>, exp: Box<Expression> },
    FnCall { name: Ident, args: Vec<Expression> },
}

/// 算術演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp { Add, Sub, Mul, Div }
```

#### 4.1.4 AST不変条件

| ID | 不変条件 | 検証タイミング |
|----|----------|---------------|
| S-INV-1 | rootは常に存在する | AstEngine構築後、全Operation後 |
| S-INV-2 | NodeIdはstable（変更されない） | 全Operation |
| S-INV-3 | 親子参照の整合性 | AddSlotElement, RemoveSlotElement後 |
| S-INV-4 | Holeは正規ノード（null禁止） | 全Operation |
| S-INV-5 | VariableRef(id)のidは存在するノード | FillHole, ReplaceNode後 |
| E-INV-1 | Expressionは常に完成式（Holeなし） | スロット代入時 |
| C-INV-1 | Constraintは常に完成済み | AddConstraint時 |
| C-INV-2 | target/lhs/rhsは有効なReference | AddConstraint時 |

**提案元**: Domain Model Agent §2.2
**採用理由**: AST整合性の保証がEditor安定性の基盤

---

### 4.2 EditorState契約

#### 4.2.1 EditorState型定義

```typescript
// TypeScript側（Preact）
interface EditorState {
  // 正本
  documentJson: string;

  // 選択状態
  selectedNodeId: NodeId | null;
  selectedConstraintId: ConstraintId | null;

  // 未完成状態（ASTに入れない）
  pendingExprAction: PendingExprAction | null;
  draftConstraint: DraftConstraint | null;
}

// SlotId: スロット識別子
interface SlotId {
  owner: NodeId;      // decimal string
  kind: SlotKind;
}

// SlotKind: スロット種別（enum化）
type SlotKind =
  | 'ArrayLength'
  | 'RepeatCount'
  | 'RangeLower'
  | 'RangeUpper'
  | 'RelationLhs'
  | 'RelationRhs';

// SourceRef: クリック可能領域の参照
type SourceRef =
  | { kind: 'Node'; nodeId: NodeId }
  | { kind: 'Slot'; slotId: SlotId }
  | { kind: 'Constraint'; constraintId: ConstraintId };
```

#### 4.2.2 PendingExprAction（簡略化版）

```typescript
// MVP: 候補選択のみ（wrapBinary/wrapCallはPhase 2）
type PendingExprAction =
  | { kind: 'selectExpr'; targetSlot: SlotId; candidates: ExprCandidate[] };

// 候補
type ExprCandidate =
  | { kind: 'Reference'; nodeId: NodeId; label: string }
  | { kind: 'Literal'; value: number }
  | { kind: 'BinOp'; op: ArithOp; placeholder: string };
```

**MVP判断**: Critical Reviewer提案により、wrapBinary等の複雑なフェーズ遷移はPhase 2に延期。式スロットは「全置換」のみ。

#### 4.2.3 DraftConstraint（4種限定）

```typescript
// MVP: 4種のみ
type DraftConstraint =
  | { kind: 'Range'; target: PlaceRef | null; lower: Expression | null; upper: Expression | null }
  | { kind: 'TypeDecl'; target: PlaceRef | null; expectedType: ExpectedType | null }
  | { kind: 'LengthRelation'; target: PlaceRef | null; length: Expression | null }
  | { kind: 'Relation'; lhs: Expression | null; op: RelationOp | null; rhs: Expression | null };

// PlaceRef: 制約対象参照
interface PlaceRef {
  nodeId: NodeId;
  indices: string[];  // ["i"] or ["i", "j"]
}

// 完了条件
function canFinalize(draft: DraftConstraint): boolean {
  switch (draft.kind) {
    case 'Range':
      return draft.target !== null && draft.lower !== null && draft.upper !== null;
    case 'TypeDecl':
      return draft.target !== null && draft.expectedType !== null;
    case 'LengthRelation':
      return draft.target !== null && draft.length !== null;
    case 'Relation':
      return draft.lhs !== null && draft.op !== null && draft.rhs !== null;
  }
}
```

**提案元**: GUI Interaction Agent §2.7、Critical Reviewer §5による絞り込み
**採用理由**: 12種全対応は実装コストが高い。4種で基本ケースをカバー。

#### 4.2.4 EditorState不変条件

| ID | 不変条件 |
|----|----------|
| ES-INV-1 | documentJsonは常に有効なAstEngine JSON |
| ES-INV-2 | pendingExprActionとdraftConstraintは同時に非nullにならない |
| ES-INV-3 | selectedNodeIdが非nullならdocumentJson内に存在するNodeId |

---

### 4.3 Projection契約

#### 4.3.1 最終決定: 統合版API（8関数）

**争点**: Domain Model Agentは20関数を提案、Critical Reviewer/wasm Boundaryは5-7関数を主張
**解決**: **統合版8関数**を採用（process.md §11基準: 実装容易性 > 意味論の明快さ）

```rust
// ===== Render Projection (document更新時) =====

/// Structure + diagnostics + completenessを一括取得
#[wasm_bindgen]
pub fn project_full(document_json: &str) -> Result<String, JsError>;
// 返却: { outline: [...], diagnostics: [...], completeness: {...} }

/// 選択ノードの詳細（スロット、関連制約）
#[wasm_bindgen]
pub fn project_node_detail(document_json: &str, node_id: &str) -> Result<String, JsError>;
// 返却: { slots: [...], relatedConstraints: [...] }

// ===== Action Projection (click時) =====

/// Holeの候補一覧
#[wasm_bindgen]
pub fn get_hole_candidates(document_json: &str, hole_id: &str) -> Result<String, JsError>;

/// 式スロットの候補一覧
#[wasm_bindgen]
pub fn get_expr_candidates(
    document_json: &str,
    parent_id: &str,
    slot_kind: &str
) -> Result<String, JsError>;

/// 制約targetの候補一覧
#[wasm_bindgen]
pub fn get_constraint_targets(
    document_json: &str,
    constraint_kind: &str
) -> Result<String, JsError>;

// ===== Operation =====

/// Action適用
#[wasm_bindgen]
pub fn apply_action(document_json: &str, action_json: &str) -> Result<String, JsError>;

// ===== Sample =====

/// サンプル生成
#[wasm_bindgen]
pub fn generate_sample(document_json: &str, seed: u32) -> Result<String, JsError>;

/// 生成可能性事前検査
#[wasm_bindgen]
pub fn check_generatability(document_json: &str) -> Result<String, JsError>;
```

#### 4.3.2 返却型定義

```typescript
// project_full返却型
interface FullProjection {
  outline: OutlineNode[];
  diagnostics: Diagnostic[];
  completeness: CompletenessInfo;
}

interface OutlineNode {
  id: string;           // decimal string
  label: string;
  kindLabel: string;
  depth: number;
  isHole: boolean;
  childIds: string[];
}

interface Diagnostic {
  level: 'error' | 'warning' | 'info';
  message: string;
  nodeId?: string;
  constraintId?: string;
}

interface CompletenessInfo {
  totalHoles: number;
  isComplete: boolean;
  missingConstraints: string[];
}

// get_hole_candidates返却型
interface HoleCandidateMenu {
  candidates: HoleCandidate[];
}

type HoleCandidate =
  | { kind: 'IntroduceScalar'; suggestedNames: string[] }
  | { kind: 'IntroduceArray'; suggestedNames: string[] }
  | { kind: 'IntroduceTuple' }
  | { kind: 'IntroduceRepeat' };

// get_expr_candidates返却型
interface ExprCandidateMenu {
  references: ReferenceCandidate[];
  literals: number[];
}

interface ReferenceCandidate {
  nodeId: string;
  label: string;
}
```

**提案元**: Critical Reviewer §6.1、wasm Boundary Agent §2との協調
**採用理由**: wasm呼び出し回数削減、境界コスト低減

---

### 4.4 Operation契約

#### 4.4.1 Action型定義

```rust
/// ASTを変更するAction
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Holeを具体ノードで埋める
    FillHole { target: NodeId, fill: FillContent },
    /// 既存ノードを置換
    ReplaceNode { target: NodeId, replacement: FillContent },
    /// 制約追加
    AddConstraint { target: NodeId, constraint: ConstraintDef },
    /// 制約削除
    RemoveConstraint { constraint_id: ConstraintId },
    /// マルチテストケース化
    IntroduceMultiTestCase { count_var_name: String, sum_bound: Option<SumBoundDef> },
    /// スロットに要素追加
    AddSlotElement { parent: NodeId, slot_name: String, element: FillContent },
    /// スロットから要素削除
    RemoveSlotElement { parent: NodeId, slot_name: String, child: NodeId },
    /// 式スロット設定（追加）
    SetExpr { slot: SlotId, expr: ExpressionDto },
}

/// 埋め込みコンテンツ
#[derive(Debug, Clone, PartialEq)]
pub enum FillContent {
    Scalar { name: String },
    Array { name: String, length: ExpressionDto },
    Tuple { element_count: usize },
    Repeat { count: ExpressionDto, index_var: Option<String> },
    Sequence,
    Hole { expected_kind: Option<String> },
}
```

#### 4.4.2 Operation結果

```rust
/// Action適用結果
#[derive(Debug, Clone)]
pub enum ApplyResult {
    Success {
        document: String,             // 更新後JSON
        created_nodes: Vec<String>,
        created_constraints: Vec<String>,
        affected_constraints: Vec<String>,
    },
    Error {
        error: OperationError,
    },
}

/// 操作エラー
#[derive(Debug, Clone)]
pub enum OperationError {
    NodeNotFound { node_id: String },
    SlotAlreadyOccupied { node_id: String, slot: String },
    TypeMismatch { expected: String, actual: String },
    InvalidFill { reason: String },
    ConstraintViolation { violations: Vec<ViolationDto> },
}
```

**提案元**: Domain Model Agent §2.5、Critical Reviewer §4.1による補完
**採用理由**: エラーケースの明確化でUI側のハンドリングが容易に

---

### 4.5 Preact/wasm境界契約

#### 4.5.1 State Ownership

| 状態 | 所有者 | 理由 |
|------|--------|------|
| AstEngine | wasm (stateless per call) | 各呼び出しでJSON受け渡し |
| documentJson | Preact signal | フロントが最新JSONを保持 |
| selectedNodeId | Preact signal | UI選択状態 |
| pendingExprAction | Preact signal | 式編集一時状態 |
| draftConstraint | Preact signal | 制約作成一時状態 |
| openPopover/Modal | Preact local state | コンポーネント内部状態 |

#### 4.5.2 シリアライゼーション形式

```
wasm boundary: JSON string
Rust内部: cp-ast-json DTO変換
TypeScript: JSON.parse + interface
```

- IDはdecimal string (JS 53bit精度問題回避)
- 100KB JSONで2-5ms想定（Critical Reviewer §7.4より）

#### 4.5.3 呼び出しパターン

```typescript
// 初回 / document更新時
async function refreshUI() {
  const doc = documentJson.value;
  const projection = JSON.parse(project_full(doc));
  structureOutline.value = projection.outline;
  diagnostics.value = projection.diagnostics;
  completeness.value = projection.completeness;
}

// click時（遅延取得）
function onClickHole(holeId: string) {
  const doc = documentJson.value;
  const candidates = JSON.parse(get_hole_candidates(doc, holeId));
  openCandidatePopover(holeId, candidates);
}

// operation適用後
function applyAndUpdate(action: ActionDto) {
  const result = JSON.parse(apply_action(documentJson.value, JSON.stringify(action)));
  if (result.success) {
    documentJson.value = result.document;  // trigger re-render
  } else {
    showError(result.error);
  }
}
```

**提案元**: wasm Boundary Agent §2.2, §2.3
**採用理由**: stateless設計でatomicな状態更新保証

---

## 5. UI / Interaction 設計

### 5.1 画面レイアウト

```
┌──────────────────────────────────────────────────────────────────────┐
│ Header                                                               │
│ [プロジェクト名] [保存] [Sample生成] │ ⚠️ 2 errors  ✓ 完成度 80%      │
├────────────────┬───────────────────────────┬─────────────────────────┤
│ Structure Pane │ Detail Pane               │ Constraint Pane         │
│ (幅: 250px)    │ (flex: 1)                 │ (幅: 300px)             │
│                │                           │                         │
│ ▼ root         │ ┌─────────────────────┐  │ ● N: int                │
│   ├─ N: int    │ │ Array: A            │  │ ● 1 ≤ N ≤ 10⁵          │
│   ├─ A: int[N] │ ├─────────────────────┤  │ ● A_i: int              │
│   └─ [Hole]    │ │ 名前: [A        ]   │  │ ● 1 ≤ A_i ≤ 10⁹        │
│                │ │ 要素型: int          │  │                         │
│ [+ 変数]       │ │ 長さ: [N ▼]         │  │ [+ 制約]                │
│ [+ 配列]       │ │                     │  │                         │
│ [+ タプル]     │ │ 関連制約:            │  │ ─────────────────────  │
│                │ │   • 1 ≤ A_i ≤ 10⁹   │  │ DraftConstraint:       │
│                │ └─────────────────────┘  │ [Range を作成中...]     │
├────────────────┴───────────────────────────┴─────────────────────────┤
│ Bottom Panel                                                         │
│ [Sample] [Preview] [Diagnostics]                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ 3                                                                │ │
│ │ 1 5 2                                                            │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

### 5.2 Preactコンポーネント階層

```typescript
<App>
  <EditorProvider>                    // EditorState signal提供
    <HeaderBar />
    <MainLayout>
      <StructurePane>
        <StructureTree>
          <StructureNodeView nodeId />
            <HoleIndicator />
        </StructureTree>
        <AddNodeMenu />               // + ボタンからのドロップダウン
      </StructurePane>

      <DetailPane>
        <NodeInspector nodeId>
          <NameField />
          <TypeField />
          <SlotFieldList>
            <ExprSlotField slotId />  // 式スロット（候補選択UI）
          </SlotFieldList>
          <RelatedConstraintList />
        </NodeInspector>
      </DetailPane>

      <ConstraintPane>
        <ConstraintList>
          <ConstraintCard constraintId />
        </ConstraintList>
        <AddConstraintButton />
        <DraftConstraintPanel />      // 4種フォーム
      </ConstraintPane>
    </MainLayout>

    <BottomPanel>
      <TabBar />
      <SamplePreview />
      <CanonicalPreview />
      <DiagnosticsPanel />
    </BottomPanel>

    <Popover />                       // click時候補表示
  </EditorProvider>
</App>
```

### 5.3 基本ケース操作フロー

#### 5.3.1 Single Scalar (`N: int`)

```
1. Structure Pane の [+ 変数] をクリック
2. ポップオーバーで「整数スカラー」を選択
3. Detail Pane で名前を "N" に入力
4. TypeDecl(Int) が自動追加される
5. Constraint Pane で Range(1, 10^5) を追加（任意）
```

**EditorState遷移**:
```
S0: { selectedNodeId: null }
  ↓ [+ 変数] クリック
S1: { openPopover: { kind: 'add-node' } }
  ↓ "整数スカラー" 選択
  ↓ wasm: apply_action(FillHole { fill: Scalar { name: "" } })
S2: { selectedNodeId: new_id, documentJson: updated }
  ↓ 名前入力 "N"
  ↓ wasm: apply_action(ReplaceNode { ... name: "N" })
S3: 完了
```

#### 5.3.2 Array (`A: int[N]`)

```
1. N が存在する状態で [+ 配列] をクリック
2. Detail Pane で名前を "A" に入力
3. 長さスロットをクリック → 候補から N を選択
4. TypeDecl(A, Int) が自動追加
```

**EditorState遷移**:
```
S0: { selectedNodeId: N_id }
  ↓ [+ 配列] クリック
  ↓ wasm: apply_action(AddSlotElement { element: Array { length: Hole } })
S1: { selectedNodeId: array_id }
  ↓ 長さスロットクリック
  ↓ wasm: get_expr_candidates(array_id, "ArrayLength")
S2: { openPopover: { kind: 'expr-candidates', candidates: [N, ...] } }
  ↓ N 選択
  ↓ wasm: apply_action(SetExpr { slot: { owner: array_id, kind: "ArrayLength" }, expr: Var(N) })
S3: 完了
```

#### 5.3.3 Grid/Matrix (`A[H][W]`)

```
1. H, W スカラーを作成
2. [+ 配列] → Matrix を選択（MVP Phase 2）
3. または: Array of Array として手動構築
   - Repeat(count=H, body=Array(W))
```

**MVP対応**: Phase 1 は Array of Array で代替。Matrixノードは Phase 2。

#### 5.3.4 Edge List (Tree)

```
1. N スカラーを作成
2. [+ 繰り返し] → count に N-1 を設定
3. body に Tuple(u, v) を追加
4. Range(u, 1, N), Range(v, 1, N) を追加
5. Guarantee("木である") を追加（MVP では Property(Tree) の代替）
```

**EditorState遷移**:
```
S0: { selectedNodeId: N_id }
  ↓ [+ 繰り返し] クリック
  ↓ wasm: apply_action(AddSlotElement { element: Repeat { count: Hole } })
S1: { selectedNodeId: repeat_id }
  ↓ count スロットで N を選択
  ↓ (N-1 は Phase 2 の式構築 or テキスト入力で対応)
S2: repeat body で [+ タプル]
  ↓ 2要素追加 → u, v
S3: 制約追加
```

#### 5.3.5 Multi-testcase Wrapper

```
1. 既存構造がある状態
2. [Multi-testcase化] ボタン（MVP Phase 2）
3. または手動:
   - T スカラーを追加
   - 既存構造を Repeat(count=T) で包む
```

**MVP対応**: Phase 1 は手動構築。IntroduceMultiTestCase Action は Phase 2。

#### 5.3.6 Query Sequence

```
1. Q スカラーを作成
2. [+ 繰り返し] → count に Q を設定
3. body に Choice を追加（MVP Phase 2）
4. または: 各クエリ種別を個別に表現
```

**MVP対応**: Phase 1 は Repeat + Tuple で代替。Choice ノードは Phase 2。

### 5.4 テンプレートシステム設計

**MVP Phase 1**: テンプレートなし（手動構築）

**Phase 2 計画**:
```rust
enum TemplateKind {
    EdgeList { vertex_var: String, edge_var: String, is_tree: bool },
    Grid { rows_var: String, cols_var: String, element_type: ExpectedType },
    QueryList { count_var: String, variants: Vec<QueryVariant> },
    MultiTestCase { count_var: String, sum_bound: Option<SumBoundDef> },
}
```

テンプレートは `Action::ApplyTemplate` として単一 Action で原子的に適用。

---

## 6. 実問題カバレッジ評価

### 6.1 検証サマリ（36問）

| カテゴリ | 問題数 | 完全対応 | 部分対応 | 非対応 |
|---------|--------|----------|----------|--------|
| 単一値 | 2 | 2 | 0 | 0 |
| 複数値 | 2 | 2 | 0 | 0 |
| 配列 | 3 | 3 | 0 | 0 |
| グリッド | 4 | 3 | 1 | 0 |
| 木 | 2 | 2 | 0 | 0 |
| 一般グラフ | 2 | 2 | 0 | 0 |
| 辺リスト | 3 | 3 | 0 | 0 |
| クエリ列 | 3 | 2 | 1 | 0 |
| 複数テストケース | 3 | 3 | 0 | 0 |
| 総和制約 | 2 | 1 | 1 | 0 |
| sorted/distinct | 2 | 2 | 0 | 0 |
| 式付き境界 | 3 | 3 | 0 | 0 |
| choice/variant | 3 | 2 | 1 | 0 |
| 下三角/可変長 | 2 | 0 | 0 | 2 |
| **合計** | **36** | **30 (83%)** | **4 (11%)** | **2 (6%)** |

### 6.2 非対応問題の根本原因

| 問題 | 根本原因 | 対応方針 |
|------|----------|---------|
| ABC370-B 下三角行列 | Repeat.index_varがArray.lengthから参照不可 | Phase 3: Reference::IndexVarRef追加 |
| ABC259-E 行内可変長 | Tuple内のインライン配列で改行が入る | Phase 2: RenderHint.InlineLayout追加 |

### 6.3 MVP目標

**ABC A-D問題で90%以上のカバレッジ**

- 基本ケース（Scalar, Array, Tuple, Repeat）で構築可能
- Property/SumBoundはGuaranteeで代替

---

## 7. リスクと代替案

### 7.1 リスクテーブル

| ID | リスク | 重大度 | 発生可能性 | 緩和策 |
|----|--------|--------|-----------|--------|
| R-1 | ExprId導入コスト過小評価 | High | High | MVP延期、式全置換で対応 |
| R-2 | PendingExprAction複雑化 | Medium | High | 簡略化（候補選択のみ） |
| R-3 | JSON roundtrip + undo相性 | High | High | MVP undo非対応、Phase 3検討 |
| R-4 | 3カラムレイアウト狭画面破綻 | Low | Medium | デスクトップのみ |
| R-5 | project_full計算量 | Medium | Low | n>100は稀、必要時pagination |
| R-6 | DraftConstraint12種実装爆発 | Medium | High | 4種限定 |
| R-7 | テンプレート原子性未保証 | High | Medium | Action::ApplyTemplateで単一実行 |
| R-8 | カバレッジAtCoder偏り | Medium | Medium | MVP対象明示 |

### 7.2 主要決定の代替案

#### ExprId

| 方式 | 採用 | 長所 | 短所 |
|------|------|------|------|
| ExprId導入（Domain提案） | ❌ Phase 2 | 部分式選択可能 | Expression全面改修 |
| 式全置換（採用） | ✅ MVP | シンプル | N→N/2が再入力 |
| テキスト入力+パース | Phase 2 | 自由度高 | パーサ実装必要 |

#### Projection API粒度

| 方式 | 採用 | 長所 | 短所 |
|------|------|------|------|
| 20関数（Domain提案） | ❌ | 責務明確 | 境界呼び出し多 |
| 8関数統合（採用） | ✅ MVP | 呼び出し削減 | 粒度粗い |
| 3関数超統合 | ❌ | 最小呼び出し | 柔軟性低 |

#### 式スロット編集

| 方式 | 採用 | 長所 | 短所 |
|------|------|------|------|
| PendingExprAction（GUI提案） | Phase 2 | 構造的操作 | フェーズ遷移複雑 |
| 候補選択のみ（採用） | ✅ MVP | 最小実装 | 複雑式不可 |
| テキスト入力+パース | Phase 2 | 自由度 | パーサ必要 |

---

## 8. MVPと後回し項目

### 8.1 MVP Phase 1 スコープ

#### 含む

| 領域 | 内容 |
|------|------|
| Structure | Scalar, Array, Tuple, Sequence, Repeat, Hole |
| Constraint | Range, TypeDecl, LengthRelation, Relation |
| Expression | Lit, Var, BinOp (+, -, *, /) |
| wasm API | 8関数（project_full, project_node_detail, get_hole/expr/constraint_candidates, apply_action, generate_sample, check_generatability） |
| UI | 3カラムレイアウト、式スロット候補選択、DraftConstraint 4種 |
| テンプレート | なし（手動構築） |
| undo/redo | なし |

#### MVP成功基準

1. 空ASTから「N: int (1≤N≤10^5), A: int[N] (1≤A_i≤10^9)」を3分以内に構築可能
2. Generateボタンで有効なサンプルが生成される
3. ABC A-D問題の90%を表現可能
4. ブラウザで60fpsで動作

### 8.2 後回し項目

#### Phase 2 (次リリース)

| 項目 | 理由 | 代替手段 |
|------|------|---------|
| Matrix, Section, Choice | 頻度低、Repeat代替可 | Repeat + Tuple |
| Distinct, Sorted, Property等8種 | 4種で基本カバー | Guarantee |
| Pow, FnCall | BinOpで十分 | 手動計算 |
| テンプレート4種 | UX改善だが必須でない | 手動構築 |
| テキスト式入力 | 候補選択で代替可 | 候補選択 |
| SumBound Generator | Generator拡張コスト高 | Guarantee |
| RenderHint.InlineLayout | 出現頻度低 | 改行許容 |

#### Phase 3 (将来)

| 項目 | 理由 |
|------|------|
| undo/redo | 実装複雑性 |
| Reference::IndexVarRef | 下三角対応、出現頻度極低 |
| drag & drop | バグりやすい |
| モバイル対応 | レイアウト再設計必要 |

---

## 9. 実装計画概要

### 9.1 高レベルタスク分解

```
Phase 1 MVP (目標: 30-40人日)
├── 1. cp-ast-core拡張 (8人日)
│   ├── 1.1 SlotId/SlotKind enum追加
│   ├── 1.2 SetExpr Action追加
│   ├── 1.3 Projection API統合版実装
│   └── 1.4 OperationError拡充
├── 2. cp-ast-wasm拡張 (5人日)
│   ├── 2.1 8関数実装
│   ├── 2.2 JSON DTO定義
│   └── 2.3 エラーハンドリング
├── 3. web/src UI (15人日)
│   ├── 3.1 EditorState signal設計
│   ├── 3.2 3カラムレイアウト
│   ├── 3.3 StructurePane
│   ├── 3.4 DetailPane + ExprSlotField
│   ├── 3.5 ConstraintPane + DraftConstraint 4種
│   ├── 3.6 BottomPanel (Sample/Preview/Diagnostics)
│   └── 3.7 Popover/候補選択UI
├── 4. 統合テスト (5人日)
│   ├── 4.1 基本ケース7種の操作フローテスト
│   ├── 4.2 wasm境界テスト
│   └── 4.3 E2Eテスト
└── 5. ドキュメント (2人日)
    ├── 5.1 ユーザーガイド
    └── 5.2 API仕様書
```

### 9.2 依存関係

```
[1.1 SlotId] ──► [1.2 SetExpr] ──► [2.1 wasm API]
                                         │
[1.3 Projection] ──► [2.1 wasm API] ──► [3.1 EditorState]
                                         │
                                         ▼
                                   [3.2-3.7 UI]
                                         │
                                         ▼
                                   [4.1-4.3 テスト]
```

### 9.3 実装順序

1. **cp-ast-core拡張** → wasm依存のため最初
2. **cp-ast-wasm拡張** → UI依存のため次
3. **EditorState設計** → UI全体の基盤
4. **StructurePane** → 最も使用頻度高い
5. **DetailPane** → ノード編集
6. **ConstraintPane** → 制約追加
7. **BottomPanel** → サンプル確認
8. **統合テスト** → 全体検証

---

## 10. 次に着手すべきタスク

### 実装タスク（優先順）

1. **SlotId/SlotKind enum追加** (cp-ast-core)
   - `crates/cp-ast-core/src/operation/types.rs` に SlotKind enum 定義
   - SlotId struct 追加
   - 1人日

2. **project_full API実装** (cp-ast-core/projection)
   - `crates/cp-ast-core/src/projection/` に統合版実装
   - FullProjection, OutlineNode, Diagnostic, CompletenessInfo 型定義
   - 2人日

3. **wasm 8関数実装** (cp-ast-wasm)
   - `crates/cp-ast-wasm/src/lib.rs` に新API追加
   - JSON DTO (cp-ast-json) 拡張
   - 3人日

4. **EditorState Preact signal設計** (web/src)
   - `web/src/state.ts` 改修
   - documentJson, selectedNodeId signal
   - computed: structureOutline, diagnostics
   - 2人日

5. **StructurePane実装** (web/src/components)
   - ツリー表示
   - ノード選択
   - [+ 変数/配列/タプル] ボタン
   - 3人日

---

## 付録: 用語集

| 用語 | 定義 |
|------|------|
| AST | Abstract Syntax Tree. StructureAST + ConstraintAST + Expression |
| EditorState | UI一時状態。ASTに入らない未完成情報を保持 |
| Projection | ASTからUIへの読み取り専用投影 |
| Operation | ASTを変更するAction |
| Hole | 未定義位置を表す第一級ノード |
| DraftConstraint | 作成中の制約（EditorStateに配置） |
| PendingExprAction | 式編集中の一時状態（EditorStateに配置） |
| SlotId | 式スロットの識別子（owner NodeId + SlotKind） |
| SourceRef | クリック可能領域とAST対象を結ぶ参照 |
| PlaceRef | 制約targetの参照（NodeId + indices） |

---

## 変更履歴

| 日付 | 変更内容 |
|------|----------|
| 2025-07-18 | 初版作成（Phase 5統合設計完了） |
