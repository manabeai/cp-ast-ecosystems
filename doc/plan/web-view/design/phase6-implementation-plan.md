# CP AST Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an MVP AST Editor UI that enables constructing competitive programming input specifications via a 3-column projectional editor, backed by wasm-compiled Rust AST engine.

**Architecture:** Preact + TypeScript frontend communicates with Rust cp-ast-core via 8 stateless wasm functions (JSON string boundary). EditorState lives in Preact signals; AST is the single source of truth rebuilt from JSON on each wasm call.

**Tech Stack:** Rust 2021 (cp-ast-core, cp-ast-wasm), wasm-pack, Preact + Vite + TypeScript, @preact/signals

---

## 依存関係グラフ

```
Task 1 (SlotKind/SetExpr) ──► Task 3 (Projection API)
                              Task 3 ──► Task 5 (wasm 8関数)
Task 2 (OperationError)   ──► Task 5
                              Task 5 ──► Task 6 (EditorState)
                              Task 6 ──► Task 7 (StructurePane)
                              Task 6 ──► Task 8 (DetailPane)
                              Task 6 ──► Task 9 (ConstraintPane)
                              Task 6 ──► Task 10 (BottomPanel)
Task 4 (JSON DTO拡張)     ──► Task 5
                              Task 7-10 ──► Task 11 (統合テスト)
```

---

### Task 1: SlotKind enum と SetExpr Action 追加

**Dependencies:** なし
**Files:**
- Create: `crates/cp-ast-core/src/operation/slot_kind.rs`
- Modify: `crates/cp-ast-core/src/operation/mod.rs`
- Modify: `crates/cp-ast-core/src/operation/action.rs`
- Test: `crates/cp-ast-core/tests/slot_kind_tests.rs`

**Success Criteria:** `SlotKind` enumが定義され、`Action::SetExpr`が追加され、`cargo test --all-targets`が通る。

- [ ] **Step 1: テスト作成**

```rust
// crates/cp-ast-core/tests/slot_kind_tests.rs
use cp_ast_core::operation::{SlotKind, SlotId};

#[test]
fn slot_kind_display() {
    assert_eq!(SlotKind::ArrayLength.as_str(), "ArrayLength");
    assert_eq!(SlotKind::RepeatCount.as_str(), "RepeatCount");
}

#[test]
fn slot_id_construction() {
    let slot = SlotId {
        owner: NodeId::new(),
        kind: SlotKind::ArrayLength,
    };
    assert_eq!(slot.kind, SlotKind::ArrayLength);
}
```

- [ ] **Step 2: テストが失敗することを確認**

Run: `cargo test --all-targets 2>&1 | tail -5`
Expected: コンパイルエラー（SlotKind未定義）

- [ ] **Step 3: SlotKind enum と SlotId struct を実装**

```rust
// crates/cp-ast-core/src/operation/slot_kind.rs
use crate::structure::NodeId;

/// 式スロットの種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotKind {
    /// Array.length
    ArrayLength,
    /// Repeat.count
    RepeatCount,
    /// Range.lower
    RangeLower,
    /// Range.upper
    RangeUpper,
    /// Relation.lhs
    RelationLhs,
    /// Relation.rhs
    RelationRhs,
    /// LengthRelation.length
    LengthLength,
}

impl SlotKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArrayLength => "ArrayLength",
            Self::RepeatCount => "RepeatCount",
            Self::RangeLower => "RangeLower",
            Self::RangeUpper => "RangeUpper",
            Self::RelationLhs => "RelationLhs",
            Self::RelationRhs => "RelationRhs",
            Self::LengthLength => "LengthLength",
        }
    }
}

/// 式スロットの識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlotId {
    pub owner: NodeId,
    pub kind: SlotKind,
}
```

- [ ] **Step 4: Action に SetExpr を追加**

`crates/cp-ast-core/src/operation/action.rs` に `SetExpr` variant を追加:

```rust
/// 式スロットに式を設定
SetExpr {
    slot: SlotId,
    expr: Expression,
},
```

- [ ] **Step 5: mod.rs の pub export を更新**

`crates/cp-ast-core/src/operation/mod.rs` に `slot_kind` モジュールを追加し、`SlotKind`, `SlotId` を re-export。

- [ ] **Step 6: テスト実行**

Run: `cargo test --all-targets`
Expected: PASS

- [ ] **Step 7: lint & format**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings`
Expected: 警告なし

- [ ] **Step 8: コミット**

```bash
git add crates/cp-ast-core/src/operation/slot_kind.rs \
        crates/cp-ast-core/src/operation/mod.rs \
        crates/cp-ast-core/src/operation/action.rs \
        crates/cp-ast-core/tests/slot_kind_tests.rs
git commit -m "feat(core): add SlotKind enum, SlotId struct, and SetExpr Action

Add SlotKind to identify expression slots (ArrayLength, RepeatCount, etc.)
and SetExpr Action variant for slot-level expression updates.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 2: OperationError 拡充

**Dependencies:** なし
**Files:**
- Create: `crates/cp-ast-core/src/operation/error.rs`
- Modify: `crates/cp-ast-core/src/operation/mod.rs`
- Test: `crates/cp-ast-core/tests/operation_error_tests.rs`

**Success Criteria:** 構造化エラー型 `OperationError` が定義され、`ApplyResult` が `Result<ApplySuccess, OperationError>` 型として使用可能。

- [ ] **Step 1: テスト作成**

```rust
// crates/cp-ast-core/tests/operation_error_tests.rs
use cp_ast_core::operation::{OperationError, ApplySuccess};

#[test]
fn error_display_node_not_found() {
    let err = OperationError::NodeNotFound { node_id: "42".into() };
    assert!(format!("{err}").contains("42"));
}

#[test]
fn apply_success_has_created_nodes() {
    let success = ApplySuccess {
        document_json: String::new(),
        created_nodes: vec!["1".into()],
        created_constraints: vec![],
        affected_constraints: vec![],
    };
    assert_eq!(success.created_nodes.len(), 1);
}
```

- [ ] **Step 2: テスト失敗を確認**

Run: `cargo test --all-targets 2>&1 | tail -5`

- [ ] **Step 3: OperationError と ApplySuccess を実装**

```rust
// crates/cp-ast-core/src/operation/error.rs
use std::fmt;

/// Action適用の成功結果
#[derive(Debug, Clone)]
pub struct ApplySuccess {
    pub document_json: String,
    pub created_nodes: Vec<String>,
    pub created_constraints: Vec<String>,
    pub affected_constraints: Vec<String>,
}

/// Action適用のエラー
#[derive(Debug, Clone)]
pub enum OperationError {
    NodeNotFound { node_id: String },
    SlotAlreadyOccupied { node_id: String, slot: String },
    TypeMismatch { expected: String, actual: String },
    InvalidFill { reason: String },
    ConstraintViolation { violations: Vec<String> },
    DeserializationError { message: String },
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound { node_id } => write!(f, "Node not found: {node_id}"),
            Self::SlotAlreadyOccupied { node_id, slot } =>
                write!(f, "Slot {slot} on node {node_id} is already occupied"),
            Self::TypeMismatch { expected, actual } =>
                write!(f, "Type mismatch: expected {expected}, got {actual}"),
            Self::InvalidFill { reason } => write!(f, "Invalid fill: {reason}"),
            Self::ConstraintViolation { violations } =>
                write!(f, "Constraint violations: {}", violations.join(", ")),
            Self::DeserializationError { message } =>
                write!(f, "Deserialization error: {message}"),
        }
    }
}

impl std::error::Error for OperationError {}
```

- [ ] **Step 4: テスト実行・lint・コミット**

Run: `cargo test --all-targets && cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings`

```bash
git add crates/cp-ast-core/src/operation/error.rs \
        crates/cp-ast-core/src/operation/mod.rs \
        crates/cp-ast-core/tests/operation_error_tests.rs
git commit -m "feat(core): add OperationError and ApplySuccess types

Structured error types for operation results, enabling the wasm boundary
to return actionable error information to the frontend.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 3: Projection API 統合版実装

**Dependencies:** Task 1
**Files:**
- Create: `crates/cp-ast-core/src/projection/full.rs`
- Create: `crates/cp-ast-core/src/projection/node_detail.rs`
- Create: `crates/cp-ast-core/src/projection/candidates.rs`
- Create: `crates/cp-ast-core/src/projection/types.rs`
- Modify: `crates/cp-ast-core/src/projection/mod.rs`
- Test: `crates/cp-ast-core/tests/projection_tests.rs`

**Success Criteria:** `project_full()`, `project_node_detail()`, `get_hole_candidates()`, `get_expr_candidates()`, `get_constraint_targets()` が実装され、テストが通る。

- [ ] **Step 1: Projection返却型をtypes.rsに定義**

```rust
// crates/cp-ast-core/src/projection/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullProjection {
    pub outline: Vec<OutlineNode>,
    pub diagnostics: Vec<Diagnostic>,
    pub completeness: CompletenessInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineNode {
    pub id: String,
    pub label: String,
    pub kind_label: String,
    pub depth: usize,
    pub is_hole: bool,
    pub child_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub node_id: Option<String>,
    pub constraint_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessInfo {
    pub total_holes: usize,
    pub is_complete: bool,
    pub missing_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetail {
    pub slots: Vec<SlotInfo>,
    pub related_constraints: Vec<ConstraintSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub kind: String,
    pub current_expr: Option<String>,
    pub is_editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSummary {
    pub id: String,
    pub label: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoleCandidate {
    pub kind: String,
    pub suggested_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprCandidateMenu {
    pub references: Vec<ReferenceCandidate>,
    pub literals: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCandidate {
    pub node_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintTargetMenu {
    pub targets: Vec<ReferenceCandidate>,
}
```

- [ ] **Step 2: project_full実装**

`crates/cp-ast-core/src/projection/full.rs`: AstEngineからOutline, Diagnostics, Completenessを生成。ノードを深さ優先で走査し、OutlineNode列に変換。Holeカウント、未定義制約を検出。

- [ ] **Step 3: project_node_detail実装**

`crates/cp-ast-core/src/projection/node_detail.rs`: 指定NodeIdのスロット情報と関連制約を返す。

- [ ] **Step 4: candidates実装**

`crates/cp-ast-core/src/projection/candidates.rs`: `get_hole_candidates`, `get_expr_candidates`, `get_constraint_targets` の3関数。ノード種別に基づく候補生成。

- [ ] **Step 5: テスト作成・実行**

テストは基本的なAST（Scalar + Array）に対してprojectionが正しい構造を返すことを検証。

- [ ] **Step 6: lint・コミット**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

```bash
git add crates/cp-ast-core/src/projection/
git commit -m "feat(core): implement integrated Projection API (8 functions)

Add project_full, project_node_detail, get_hole_candidates,
get_expr_candidates, get_constraint_targets with structured return types.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 4: cp-ast-json DTO 拡張

**Dependencies:** Task 1, Task 2, Task 3
**Files:**
- Modify: `crates/cp-ast-json/src/lib.rs` (or relevant DTO files)
- Test: `crates/cp-ast-json/tests/`

**Success Criteria:** Projection型、OperationError、SlotKind、SetExpr の JSON シリアライズ/デシリアライズが動作する。

- [ ] **Step 1: SlotKind, SlotId の JSON DTO追加**

SlotKind は `"ArrayLength"` などの文字列としてシリアライズ。SlotId は `{ "owner": "123", "kind": "ArrayLength" }` 形式。

- [ ] **Step 2: SetExpr Action の DTO追加**

既存の Action DTO に SetExpr variant を追加。

- [ ] **Step 3: OperationError, ApplySuccess の DTO追加**

ApplyResult を `{ "success": {...} }` or `{ "error": {...} }` 形式でシリアライズ。

- [ ] **Step 4: Projection型の DTO追加**

FullProjection, NodeDetail, HoleCandidate, ExprCandidateMenu, ConstraintTargetMenu の JSON roundtrip テスト。

- [ ] **Step 5: テスト・lint・コミット**

Run: `cargo test --all-targets && cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings`

```bash
git add crates/cp-ast-json/
git commit -m "feat(json): add DTOs for Projection, OperationError, SlotKind

Extend cp-ast-json with serialization support for new editor types.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 5: cp-ast-wasm 8関数実装

**Dependencies:** Task 3, Task 4
**Files:**
- Modify: `crates/cp-ast-wasm/src/lib.rs`
- Test: wasm-pack test (or manual browser test)

**Success Criteria:** 8つのwasm関数がエクスポートされ、`wasm-pack build`が成功する。

- [ ] **Step 1: project_full wasm関数**

```rust
#[wasm_bindgen]
pub fn project_full(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let projection = cp_ast_core::projection::project_full(&engine);
    serde_json::to_string(&projection).map_err(|e| JsError::new(&e.to_string()))
}
```

- [ ] **Step 2: project_node_detail wasm関数**

```rust
#[wasm_bindgen]
pub fn project_node_detail(document_json: &str, node_id: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let id = parse_node_id(node_id)?;
    let detail = cp_ast_core::projection::project_node_detail(&engine, id);
    serde_json::to_string(&detail).map_err(|e| JsError::new(&e.to_string()))
}
```

- [ ] **Step 3: get_hole_candidates, get_expr_candidates, get_constraint_targets**

3つの候補取得関数を追加。

- [ ] **Step 4: apply_action wasm関数**

```rust
#[wasm_bindgen]
pub fn apply_action(document_json: &str, action_json: &str) -> Result<String, JsError> {
    let mut engine = deserialize(document_json)?;
    let action: ActionDto = serde_json::from_str(action_json)
        .map_err(|e| JsError::new(&e.to_string()))?;
    let result = engine.apply(action.into());
    serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
}
```

- [ ] **Step 5: check_generatability wasm関数**

生成可能性を判定し、`{ "generatable": true/false, "blockers": [...] }` を返す。

- [ ] **Step 6: wasm-pack build**

Run: `wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm`
Expected: ビルド成功

- [ ] **Step 7: web/src/wasm.ts に新関数の import追加**

```typescript
export {
  // 既存
  render_input_format, render_structure_tree, ...
  // 新規
  project_full, project_node_detail,
  get_hole_candidates, get_expr_candidates, get_constraint_targets,
  apply_action, generate_sample, check_generatability,
} from '../wasm/cp_ast_wasm';
```

- [ ] **Step 8: コミット**

```bash
git add crates/cp-ast-wasm/src/lib.rs web/src/wasm.ts
git commit -m "feat(wasm): implement 8-function editor API

Add project_full, project_node_detail, get_hole/expr/constraint_candidates,
apply_action, generate_sample, check_generatability wasm exports.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 6: EditorState signal 設計

**Dependencies:** Task 5
**Files:**
- Create: `web/src/editor/state.ts`
- Create: `web/src/editor/types.ts`
- Create: `web/src/editor/actions.ts`
- Create: `web/src/editor/index.ts`

**Success Criteria:** EditorState がPreact signalとして管理され、document更新時にprojectionが自動計算される。

- [ ] **Step 1: TypeScript型定義**

```typescript
// web/src/editor/types.ts
export interface FullProjection {
  outline: OutlineNode[];
  diagnostics: Diagnostic[];
  completeness: CompletenessInfo;
}

export interface OutlineNode {
  id: string;
  label: string;
  kindLabel: string;
  depth: number;
  isHole: boolean;
  childIds: string[];
}

export interface Diagnostic {
  level: 'error' | 'warning' | 'info';
  message: string;
  nodeId?: string;
  constraintId?: string;
}

export interface CompletenessInfo {
  totalHoles: number;
  isComplete: boolean;
  missingConstraints: string[];
}

export type SlotKind =
  | 'ArrayLength' | 'RepeatCount'
  | 'RangeLower' | 'RangeUpper'
  | 'RelationLhs' | 'RelationRhs'
  | 'LengthLength';

export interface SlotId {
  owner: string;
  kind: SlotKind;
}

export type DraftConstraint =
  | { kind: 'Range'; target: string | null; lower: string | null; upper: string | null }
  | { kind: 'TypeDecl'; target: string | null; expectedType: string | null }
  | { kind: 'LengthRelation'; target: string | null; length: string | null }
  | { kind: 'Relation'; lhs: string | null; op: string | null; rhs: string | null };
```

- [ ] **Step 2: EditorState signals**

```typescript
// web/src/editor/state.ts
import { signal, computed } from '@preact/signals';
import { project_full } from '../wasm';
import type { FullProjection } from './types';

export const editorDocumentJson = signal<string>('');
export const selectedNodeId = signal<string | null>(null);
export const selectedConstraintId = signal<string | null>(null);
export const draftConstraint = signal<DraftConstraint | null>(null);

export const projection = computed<FullProjection | null>(() => {
  const doc = editorDocumentJson.value;
  if (!doc) return null;
  try {
    return JSON.parse(project_full(doc));
  } catch {
    return null;
  }
});
```

- [ ] **Step 3: Action dispatch関数**

```typescript
// web/src/editor/actions.ts
import { editorDocumentJson, selectedNodeId } from './state';
import { apply_action } from '../wasm';

export function dispatchAction(action: unknown): boolean {
  const result = JSON.parse(apply_action(
    editorDocumentJson.value,
    JSON.stringify(action)
  ));
  if (result.document) {
    editorDocumentJson.value = result.document;
    return true;
  }
  console.error('Action failed:', result.error);
  return false;
}
```

- [ ] **Step 4: index.ts でre-export**

- [ ] **Step 5: コミット**

```bash
git add web/src/editor/
git commit -m "feat(web): add EditorState signal management

Preact signals for document JSON, selection, draft constraint.
Computed projection auto-updates on document change.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 7: StructurePane コンポーネント

**Dependencies:** Task 6
**Files:**
- Create: `web/src/components/editor/StructurePane.tsx`
- Create: `web/src/components/editor/StructureNodeView.tsx`
- Create: `web/src/components/editor/AddNodeMenu.tsx`
- Create: `web/src/components/editor/HoleIndicator.tsx`

**Success Criteria:** 左カラムにAST構造がツリー表示され、ノード選択・Hole候補選択が動作する。

- [ ] **Step 1: StructureNodeView — 単一ノード表示**

各OutlineNodeを再帰的にレンダリング。is_holeならHoleIndicatorを表示。クリックでselectedNodeId更新。

- [ ] **Step 2: HoleIndicator — Hole表示と候補選択**

`[?]` アイコン表示。クリックで`get_hole_candidates`呼び出し、ポップオーバーで候補表示。候補選択で`FillHole` Action dispatch。

- [ ] **Step 3: AddNodeMenu — ノード追加ボタン**

`[+ 変数]`, `[+ 配列]`, `[+ タプル]`, `[+ 繰り返し]` のドロップダウン。選択するとルートSequenceにHole追加→即FillHole。

- [ ] **Step 4: StructurePane — ツリー全体**

projection.outlineからツリーを構築し、StructureNodeViewを再帰表示。AddNodeMenuをツリー下部に配置。

- [ ] **Step 5: スタイリング**

ツリーインデント、選択状態のハイライト、Hole視覚表示（点線境界、灰色背景）。

- [ ] **Step 6: コミット**

```bash
git add web/src/components/editor/
git commit -m "feat(web): implement StructurePane with tree view

Tree visualization of AST outline, hole indicators with candidate
selection, and add-node menu for creating new structure elements.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 8: DetailPane コンポーネント

**Dependencies:** Task 6
**Files:**
- Create: `web/src/components/editor/DetailPane.tsx`
- Create: `web/src/components/editor/NodeInspector.tsx`
- Create: `web/src/components/editor/ExprSlotField.tsx`
- Create: `web/src/components/editor/NameField.tsx`

**Success Criteria:** 選択ノードの詳細（名前、型、式スロット、関連制約）が中央カラムに表示され、式スロット編集が候補選択で動作する。

- [ ] **Step 1: NodeInspector — ノード詳細表示**

selectedNodeIdが変わるたびに`project_node_detail`を呼び出し、スロット一覧と関連制約を表示。

- [ ] **Step 2: NameField — ノード名編集**

テキスト入力。変更時にReplaceNode Actionをdispatch。

- [ ] **Step 3: ExprSlotField — 式スロット編集**

現在の式を表示。クリックで`get_expr_candidates`呼び出し、候補をドロップダウンで表示。選択でSetExpr Actionをdispatch。

- [ ] **Step 4: DetailPane — 統合レイアウト**

ノード選択時: NodeInspector表示。未選択時: 空状態メッセージ。

- [ ] **Step 5: コミット**

```bash
git add web/src/components/editor/DetailPane.tsx \
        web/src/components/editor/NodeInspector.tsx \
        web/src/components/editor/ExprSlotField.tsx \
        web/src/components/editor/NameField.tsx
git commit -m "feat(web): implement DetailPane with node inspector

Node detail view showing slots, expression editors with candidate
selection, and related constraints for the selected node.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 9: ConstraintPane コンポーネント

**Dependencies:** Task 6
**Files:**
- Create: `web/src/components/editor/ConstraintPane.tsx`
- Create: `web/src/components/editor/ConstraintCard.tsx`
- Create: `web/src/components/editor/DraftConstraintPanel.tsx`
- Create: `web/src/components/editor/AddConstraintButton.tsx`

**Success Criteria:** 右カラムに制約一覧が表示され、4種のDraftConstraintフォームから制約追加が動作する。

- [ ] **Step 1: ConstraintCard — 制約表示**

各制約のラベル（`1 ≤ N ≤ 10⁵` 形式）、削除ボタン。削除でRemoveConstraint Actionをdispatch。

- [ ] **Step 2: AddConstraintButton — 制約追加開始**

`[+ 制約]` ボタン。クリックで4種（Range, TypeDecl, LengthRelation, Relation）を選択。選択でdraftConstraint signalを初期化。

- [ ] **Step 3: DraftConstraintPanel — 4種フォーム**

draftConstraintが非nullの時に表示。各フィールドの入力UI:
- target: ドロップダウン（`get_constraint_targets`から候補取得）
- lower/upper/length: 式スロット（ExprSlotField再利用）
- op: 演算子選択
- 完了ボタン: canFinalize判定後にAddConstraint dispatch

- [ ] **Step 4: ConstraintPane — 統合レイアウト**

制約リスト + AddConstraintButton + DraftConstraintPanel。

- [ ] **Step 5: コミット**

```bash
git add web/src/components/editor/ConstraintPane.tsx \
        web/src/components/editor/ConstraintCard.tsx \
        web/src/components/editor/DraftConstraintPanel.tsx \
        web/src/components/editor/AddConstraintButton.tsx
git commit -m "feat(web): implement ConstraintPane with draft forms

Constraint list display, 4-type draft constraint forms (Range,
TypeDecl, LengthRelation, Relation), and constraint management.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 10: BottomPanel (Sample/Preview/Diagnostics)

**Dependencies:** Task 6
**Files:**
- Create: `web/src/components/editor/BottomPanel.tsx`
- Create: `web/src/components/editor/SamplePreview.tsx`
- Create: `web/src/components/editor/DiagnosticsPanel.tsx`
- Create: `web/src/components/editor/CanonicalPreview.tsx`

**Success Criteria:** 下部パネルにタブ切替でSample出力、入力形式プレビュー、Diagnostics一覧が表示される。

- [ ] **Step 1: SamplePreview — サンプル生成表示**

`check_generatability` → 生成可能なら `generate_sample` 呼び出し。結果をmonospace表示。Seed変更ボタン。

- [ ] **Step 2: CanonicalPreview — 入力形式テキスト**

既存の `render_input_format` を活用。TeX表示も `render_input_tex` で表示。

- [ ] **Step 3: DiagnosticsPanel — 診断一覧**

projection.diagnosticsをリスト表示。level別アイコン（🔴 error, ⚠️ warning, ℹ️ info）。クリックで対象ノードを選択。

- [ ] **Step 4: BottomPanel — タブ切替**

`[Sample] [Preview] [Diagnostics]` タブ。アクティブタブの内容を表示。

- [ ] **Step 5: コミット**

```bash
git add web/src/components/editor/BottomPanel.tsx \
        web/src/components/editor/SamplePreview.tsx \
        web/src/components/editor/DiagnosticsPanel.tsx \
        web/src/components/editor/CanonicalPreview.tsx
git commit -m "feat(web): implement BottomPanel with sample/preview/diagnostics

Tabbed bottom panel showing generated samples, input format preview,
and diagnostic messages with click-to-select-node navigation.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 11: EditorPage 統合と 3カラムレイアウト

**Dependencies:** Task 7, Task 8, Task 9, Task 10
**Files:**
- Create: `web/src/components/editor/EditorPage.tsx`
- Create: `web/src/components/editor/HeaderBar.tsx`
- Modify: `web/src/app.tsx` — ルーティング追加
- Modify: `web/src/state.ts` — editor ページ追加

**Success Criteria:** `/editor` パスで3カラムEditorページが表示され、全コンポーネントが連動する。

- [ ] **Step 1: HeaderBar — ヘッダーバー**

プロジェクト名、完成度表示（completeness.isComplete）、エラー数表示、`[Sample生成]` ボタン。

- [ ] **Step 2: EditorPage — 3カラムレイアウト**

```tsx
export function EditorPage() {
  return (
    <div class="editor-page">
      <HeaderBar />
      <div class="editor-main">
        <StructurePane />
        <DetailPane />
        <ConstraintPane />
      </div>
      <BottomPanel />
    </div>
  );
}
```

CSS: `editor-main` は `display: grid; grid-template-columns: 250px 1fr 300px;`

- [ ] **Step 3: app.tsx にルーティング追加**

`#/editor` で EditorPage を表示。既存の viewer/preview と共存。

- [ ] **Step 4: 初期document生成**

EditorPage マウント時に空のAstEngine documentを生成（wasm `create_empty_document()` 関数を追加するか、空JSON定数を使用）。

- [ ] **Step 5: E2E動作確認**

ブラウザで `#/editor` にアクセス。3カラム表示、Hole追加、ノード選択、制約追加、Sample生成の一連フローを手動確認。

- [ ] **Step 6: コミット**

```bash
git add web/src/components/editor/EditorPage.tsx \
        web/src/components/editor/HeaderBar.tsx \
        web/src/app.tsx web/src/state.ts
git commit -m "feat(web): integrate EditorPage with 3-column layout

Full editor page with Structure, Detail, Constraint panes and
bottom panel. Routing via #/editor hash.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 12: 統合テスト — 基本ケース操作フロー

**Dependencies:** Task 11
**Files:**
- Create: `web/tests/editor-flows.test.ts` (or manual test protocol)
- Create: `crates/cp-ast-core/tests/editor_integration.rs`

**Success Criteria:** 以下の7基本ケースがそれぞれ操作フローとして構築可能。

- [ ] **Step 1: Rust統合テスト — Scalar構築**

```rust
// crates/cp-ast-core/tests/editor_integration.rs
#[test]
fn build_scalar_n() {
    let mut engine = AstEngine::new();
    // FillHole → Scalar(N)
    // AddConstraint → TypeDecl(N, Int)
    // AddConstraint → Range(N, 1, 100000)
    // project_full → outlineにN表示
    // generate_sample → 有効な値
}
```

- [ ] **Step 2: Rust統合テスト — Array構築**

N + A[N] を構築し、project_full, generate_sampleが正しく動作することを検証。

- [ ] **Step 3: Rust統合テスト — Tuple構築**

(N, M) タプルを構築。

- [ ] **Step 4: Rust統合テスト — Edge List構築**

N + Repeat(N-1, Tuple(u, v)) + 制約。

- [ ] **Step 5: フロントエンドビルド確認**

Run: `cd web && npx tsc --noEmit && npx vite build`
Expected: ビルド成功

- [ ] **Step 6: MVP成功基準テスト**

手動テスト: 空ASTから「N: int (1≤N≤10^5), A: int[N] (1≤A_i≤10^9)」を構築し、Sample生成まで完了することを確認。目標: 3分以内。

- [ ] **Step 7: コミット**

```bash
git add crates/cp-ast-core/tests/editor_integration.rs web/tests/
git commit -m "test: add editor integration tests for basic cases

Verify Scalar, Array, Tuple, EdgeList construction flows work
end-to-end through Projection and Operation APIs.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## 実装順序サマリ

| 順序 | Task | 領域 | 依存 |
|------|------|------|------|
| 1 | Task 1: SlotKind/SetExpr | cp-ast-core | なし |
| 2 | Task 2: OperationError | cp-ast-core | なし |
| 3 | Task 3: Projection API | cp-ast-core | Task 1 |
| 4 | Task 4: JSON DTO拡張 | cp-ast-json | Task 1,2,3 |
| 5 | Task 5: wasm 8関数 | cp-ast-wasm | Task 3,4 |
| 6 | Task 6: EditorState | web | Task 5 |
| 7 | Task 7: StructurePane | web | Task 6 |
| 8 | Task 8: DetailPane | web | Task 6 |
| 9 | Task 9: ConstraintPane | web | Task 6 |
| 10 | Task 10: BottomPanel | web | Task 6 |
| 11 | Task 11: EditorPage統合 | web | Task 7-10 |
| 12 | Task 12: 統合テスト | all | Task 11 |

Task 1 と Task 2 は並行実行可能。Task 7, 8, 9, 10 は並行実行可能。
