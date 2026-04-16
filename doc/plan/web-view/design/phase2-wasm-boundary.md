# Phase 2: wasm/Frontend境界設計

本文書は、Editor UI設計におけるRust/wasm層とPreact/TypeScript層の境界設計を定めるものである。

---

## 1. 前提

本設計は以下の固定前提に基づく（phase1-premises.md準拠）：

| 前提 | 内容 |
|------|------|
| A-4 | wasm(Rust)は意味論、Preact(TS)は体験を担う |
| C-1〜C-4 | Projection分離（render/action/global）、click時遅延取得 |
| E-4 | wasm ↔ JS間はJSON string（cp-ast-json経由） |
| E-5 | IDはdecimal string（JS 53bit精度問題回避） |

現行cp-ast-wasm crateは10個のwasm-bindgen関数（viewer用）を持ち、本設計はこれをeditor用に拡張する。

---

## 2. 主要判断

### 2.1 wasm exported API proposal

#### A. Render Projection Functions

```rust
/// Structure全体のoutline（ツリー構築用）
#[wasm_bindgen]
pub fn project_structure_outline(document_json: &str) -> Result<String, JsError>;

/// 単一ノードの描画情報
#[wasm_bindgen]
pub fn project_node_render(
    document_json: &str,
    node_id: &str,
) -> Result<String, JsError>;

/// 単一制約の描画情報
#[wasm_bindgen]
pub fn project_constraint_render(
    document_json: &str,
    constraint_id: &str,
) -> Result<String, JsError>;

/// Expression inline表示用（token tree）
#[wasm_bindgen]
pub fn project_expr_render(
    document_json: &str,
    slot_id: &str,
) -> Result<String, JsError>;

/// Diagnostics一覧（Error/Warning/Info）
#[wasm_bindgen]
pub fn project_diagnostics(document_json: &str) -> Result<String, JsError>;

/// 完成度サマリ（hole数、未充足制約数）
#[wasm_bindgen]
pub fn project_completeness(document_json: &str) -> Result<String, JsError>;
```

返却JSON例（`project_structure_outline`）:
```json
{
  "nodes": [
    {
      "id": "1",
      "label": "N",
      "kindLabel": "Scalar",
      "depth": 1,
      "isHole": false,
      "childIds": []
    },
    {
      "id": "2",
      "label": "A",
      "kindLabel": "Array",
      "depth": 1,
      "isHole": false,
      "childIds": []
    }
  ],
  "rootId": "0"
}
```

#### B. Action Projection Functions

```rust
/// Holeに入れられる候補一覧
#[wasm_bindgen]
pub fn project_hole_candidates(
    document_json: &str,
    hole_id: &str,
) -> Result<String, JsError>;

/// ノードに対して実行可能なアクション一覧
#[wasm_bindgen]
pub fn project_node_actions(
    document_json: &str,
    node_id: &str,
) -> Result<String, JsError>;

/// 式スロットに入れられる候補一覧（参照・定数・式テンプレート）
#[wasm_bindgen]
pub fn project_expr_slot_candidates(
    document_json: &str,
    slot_json: &str, // { "parentId": "...", "slotName": "..." }
) -> Result<String, JsError>;

/// 既存式に対する変形操作一覧（wrap binary、wrap call等）
#[wasm_bindgen]
pub fn project_expr_actions(
    document_json: &str,
    slot_json: &str,
) -> Result<String, JsError>;

/// 制約targetの候補一覧
#[wasm_bindgen]
pub fn project_constraint_target_candidates(
    document_json: &str,
    constraint_kind: &str,
) -> Result<String, JsError>;
```

返却JSON例（`project_hole_candidates`）:
```json
{
  "candidates": [
    { "kind": "IntroduceScalar", "suggestedNames": ["N", "M", "K"] },
    { "kind": "IntroduceArray", "suggestedNames": ["A", "B", "C"] },
    { "kind": "IntroduceMatrix" },
    { "kind": "IntroduceTuple" },
    { "kind": "IntroduceRepeat" },
    { "kind": "IntroduceSection" }
  ]
}
```

#### C. Operation Functions

```rust
/// Actionを適用してASTを更新（成功時は新document JSON）
#[wasm_bindgen]
pub fn apply_action(
    document_json: &str,
    action_json: &str,
) -> Result<String, JsError>;

/// Actionのdry-run（新Hole数・影響制約を返す）
#[wasm_bindgen]
pub fn preview_action(
    document_json: &str,
    action_json: &str,
) -> Result<String, JsError>;
```

Action JSON例:
```json
{
  "kind": "FillHole",
  "target": "3",
  "fill": {
    "kind": "Scalar",
    "name": "N"
  }
}
```

ApplyResult JSON例:
```json
{
  "success": true,
  "document": "...(updated JSON)...",
  "createdNodes": ["4"],
  "createdConstraints": [],
  "affectedConstraints": []
}
```

Error JSON例:
```json
{
  "success": false,
  "error": {
    "kind": "NodeNotFound",
    "nodeId": "99"
  }
}
```

#### D. Global Projection Functions

```rust
/// 全制約の一覧（軽量サマリ）
#[wasm_bindgen]
pub fn project_constraint_list(document_json: &str) -> Result<String, JsError>;

/// 指定ノードに紐づく制約一覧
#[wasm_bindgen]
pub fn project_node_constraints(
    document_json: &str,
    node_id: &str,
) -> Result<String, JsError>;
```

#### E. Sample Generation Functions

```rust
/// サンプル生成（既存関数を継承）
#[wasm_bindgen]
pub fn generate_sample(document_json: &str, seed: u32) -> Result<String, JsError>;

/// Canonical入力形式preview
#[wasm_bindgen]
pub fn render_canonical_preview(document_json: &str) -> Result<String, JsError>;
```

---

### 2.2 State Ownership Model

| 状態 | 所有者 | 理由 |
|------|--------|------|
| AstEngine (Structure + Constraints) | wasm memory (immutable snapshot per call) | 正本はwasm側だが、各呼び出しはstateless |
| document JSON | Preact signal | フロントが最新JSONを保持し、各wasm呼び出しに渡す |
| selectedNodeId | Preact signal | UI選択状態 |
| selectedConstraintId | Preact signal | UI選択状態 |
| pendingExprAction | Preact signal | 式編集中の一時状態（ASTに入れない） |
| draftConstraint | Preact signal | 制約作成中の一時状態（ASTに入れない） |
| openPopover, openModal | Preact signal | UI開閉状態 |
| dragState | Preact signal | D&D状態 |
| diagnosticsCache | Preact signal | project_diagnostics結果のキャッシュ |

**重要な設計判断**: wasm側はstatelessとする。

- 各wasm関数呼び出しにdocument_jsonを渡す
- Operation適用後は新しいdocument_jsonが返る
- Preact側がdocument signal を更新することで状態が進む

理由:
- wasm memory内にmutableな状態を持つと、JS↔wasm間の同期が複雑化する
- JSON roundtripによりatomicな状態更新が保証される
- Preactのsignal/computed systemと自然に統合できる

---

### 2.3 Invocation Pattern

#### 同期呼び出し（sync）

全wasm関数は同期呼び出しとする。

```typescript
// Preact側の呼び出しパターン
function onClickHole(holeId: string) {
  const doc = documentJson.value;
  const candidatesJson = project_hole_candidates(doc, holeId);
  const candidates: HoleCandidateMenu = JSON.parse(candidatesJson);
  openCandidatePopover(holeId, candidates);
}
```

理由:
- 現行cp-ast-wasm関数も全て同期
- AST操作は十分高速（<10msを想定）
- 非同期化の複雑さを避ける

#### batch呼び出し vs 個別呼び出し

| 方式 | 採用 | 理由 |
|------|------|------|
| 初回描画時 | outline + diagnostics + completenessをbatch | 画面構築に必要な最小セット |
| click時 | 個別にaction projectionを呼ぶ | 遅延取得の原則 |
| operation後 | apply_action → outline再取得 | 楽観的更新は行わない（正本はwasm側） |

```typescript
// 初回/document更新後
async function refreshUI() {
  const doc = documentJson.value;
  const [outline, diag, comp] = await Promise.all([
    JSON.parse(project_structure_outline(doc)),
    JSON.parse(project_diagnostics(doc)),
    JSON.parse(project_completeness(doc)),
  ]);
  structureOutline.value = outline;
  diagnostics.value = diag;
  completeness.value = comp;
}
```

---

### 2.4 Serialization Strategy

**採用方針**: JSON string（現行維持）+ 型付きDTO

| 層 | 形式 |
|----|------|
| wasm boundary | JSON string |
| Rust内部 | cp-ast-json DTO経由で変換 |
| TypeScript内部 | `JSON.parse` + TypeScript interface |

理由:
- wasm-bindgen複合型（serde-wasm-bindgen）は依存追加・複雑化を招く
- JSON stringは可視化・デバッグが容易
- 現行cp-ast-json crateとの整合性維持

**パフォーマンス考慮**:
- 典型的AST（ノード数<100, 制約数<50）で<5ms想定
- 大規模AST（ノード数>500）が頻発する場合のみ、差分プロトコル検討

TypeScript側の型定義例:
```typescript
interface NodeRenderView {
  id: string;        // decimal string
  label: string;
  kindLabel: string;
  isHole: boolean;
  expectedKindHint: string | null;
  childIds: string[];
  diagnosticSummary: DiagnosticSummary[];
}

interface HoleCandidateMenu {
  candidates: HoleCandidate[];
}

type HoleCandidate =
  | { kind: "IntroduceScalar"; suggestedNames: string[] }
  | { kind: "IntroduceArray"; suggestedNames: string[] }
  | { kind: "IntroduceMatrix" }
  | { kind: "IntroduceTuple" }
  | { kind: "IntroduceRepeat" }
  | { kind: "IntroduceSection" };
```

---

### 2.5 Update Notification Pattern

**採用方針**: Pull-based（明示的再取得）

wasm側はpush通知を行わない。Preact側が以下のタイミングでprojectionを再取得する：

| トリガー | 再取得対象 |
|----------|-----------|
| documentJson.value変更 | outline, diagnostics, completeness |
| node選択変更 | project_node_render (optional) |
| operation成功 | documentJson自体を更新 → 自動で上記が走る |

```typescript
// Preact signals computed pattern
const structureOutline = computed(() => {
  if (!documentJson.value) return null;
  return JSON.parse(project_structure_outline(documentJson.value));
});

const diagnostics = computed(() => {
  if (!documentJson.value) return [];
  return JSON.parse(project_diagnostics(documentJson.value));
});
```

理由:
- wasmからJS callbackを呼ぶパターンは複雑
- Preact signalsのreactive systemで十分
- pull-basedの方がデバッグ・テストが容易

---

### 2.6 Error Handling across Boundary

```rust
// Rust側
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum WasmOperationResult {
    Success {
        document: String,
        created_nodes: Vec<String>,
        created_constraints: Vec<String>,
        affected_constraints: Vec<String>,
    },
    Error {
        error: WasmOperationError,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum WasmOperationError {
    NodeNotFound { node_id: String },
    SlotOccupied { node_id: String, current_occupant: String },
    TypeMismatch { expected: String, actual: String, context: String },
    ConstraintViolation { violations: Vec<ViolationDto> },
    InvalidOperation { action: String, reason: String },
    DeserializationError { message: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct ViolationDto {
    pub constraint_id: String,
    pub description: String,
    pub suggestion: Option<String>,
}
```

TypeScript側:
```typescript
interface ApplyResult {
  success: true;
  document: string;
  createdNodes: string[];
  createdConstraints: string[];
  affectedConstraints: string[];
}

interface ApplyError {
  success: false;
  error: OperationError;
}

type ApplyResponse = ApplyResult | ApplyError;

function applyActionSafe(doc: string, action: ActionDto): ApplyResponse {
  try {
    const result = JSON.parse(apply_action(doc, JSON.stringify(action)));
    return result;
  } catch (e) {
    // JsError from wasm (deserialization failure等)
    return {
      success: false,
      error: { kind: "DeserializationError", message: String(e) }
    };
  }
}
```

---

## 3. 具体例

### 3.1 User clicks "add scalar N" → wasm call → UI update

```
┌──────────────────────────────────────────────────────────────────────┐
│ 1. User clicks "+ 変数を追加" button in StructurePane               │
├──────────────────────────────────────────────────────────────────────┤
│ 2. Preact: openHoleCandidatePopover()                               │
│    - 現在のfocus位置（root sequence末尾）を特定                        │
│    - project_hole_candidates(doc, holeId) 呼び出し                   │
├──────────────────────────────────────────────────────────────────────┤
│ 3. wasm: project_hole_candidates()                                  │
│    - JSON deserialize → AstEngine                                   │
│    - hole_candidates(holeId) 計算                                   │
│    - JSON serialize → 返却                                          │
│    - 戻り値: { candidates: [{ kind: "IntroduceScalar", ... }, ...]}│
├──────────────────────────────────────────────────────────────────────┤
│ 4. Preact: showCandidatePopover(candidates)                         │
│    - User selects "IntroduceScalar"                                 │
│    - 名前入力フォーム表示 → "N" 入力                                  │
├──────────────────────────────────────────────────────────────────────┤
│ 5. Preact: buildAction & call wasm                                  │
│    const action = {                                                  │
│      kind: "FillHole",                                               │
│      target: holeId,                                                 │
│      fill: { kind: "Scalar", name: "N" }                             │
│    };                                                                │
│    const result = applyActionSafe(doc, action);                      │
├──────────────────────────────────────────────────────────────────────┤
│ 6. wasm: apply_action()                                             │
│    - deserialize document + action                                   │
│    - engine.apply(FillHole { target, fill })                        │
│    - 成功: 新document JSON生成                                       │
│    - serialize ApplyResult                                           │
├──────────────────────────────────────────────────────────────────────┤
│ 7. Preact: update state                                             │
│    if (result.success) {                                             │
│      documentJson.value = result.document;  // signal update        │
│      closePopover();                                                 │
│    } else {                                                          │
│      showError(result.error);                                        │
│    }                                                                 │
├──────────────────────────────────────────────────────────────────────┤
│ 8. Preact: computed signals auto-update                             │
│    - structureOutline.value 再計算（project_structure_outline）      │
│    - diagnostics.value 再計算（project_diagnostics）                 │
│    - UI自動再描画                                                    │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 3.2 User edits expression slot N → N/2

```
┌──────────────────────────────────────────────────────────────────────┐
│ 1. 前提: Array A の length slot に Expression::Var(N) が入っている   │
├──────────────────────────────────────────────────────────────────────┤
│ 2. User clicks on "N" token in the length slot UI                    │
├──────────────────────────────────────────────────────────────────────┤
│ 3. Preact: project_expr_actions(doc, slotJson) 呼び出し              │
│    - slotJson = { parentId: "A's id", slotName: "length" }          │
├──────────────────────────────────────────────────────────────────────┤
│ 4. wasm: 返却 ExprActionMenu                                         │
│    {                                                                 │
│      actions: [                                                      │
│        { id: "wrap-div", label: "/ x", kind: "wrap-binary" },       │
│        { id: "wrap-mul", label: "* x", kind: "wrap-binary" },       │
│        { id: "wrap-add", label: "+ x", kind: "wrap-binary" },       │
│        { id: "wrap-sub", label: "- x", kind: "wrap-binary" },       │
│        { id: "replace", label: "置換", kind: "replace" }            │
│      ]                                                               │
│    }                                                                 │
├──────────────────────────────────────────────────────────────────────┤
│ 5. Preact: showExprActionPopover(actions)                           │
│    - User selects "/ x"                                              │
├──────────────────────────────────────────────────────────────────────┤
│ 6. Preact: setPendingExprAction()                                   │
│    pendingExprAction.value = {                                       │
│      kind: "wrapBinary",                                             │
│      slotJson: { parentId: "...", slotName: "length" },             │
│      op: "Div",                                                      │
│      phase: "select-rhs",                                            │
│      lhs: { kind: "Var", reference: { kind: "VariableRef", ... } }, │
│      rhs: null  // 未確定                                            │
│    };                                                                │
│    ※ ASTは変更しない                                                 │
├──────────────────────────────────────────────────────────────────────┤
│ 7. Preact: project_expr_slot_candidates(doc, slotJson) 呼び出し     │
│    - rhs候補を取得                                                   │
├──────────────────────────────────────────────────────────────────────┤
│ 8. wasm: 返却 ExprCandidateMenu                                      │
│    {                                                                 │
│      candidates: [                                                   │
│        { kind: "Literal", value: 2 },                                │
│        { kind: "Literal", value: 3 },                                │
│        { kind: "Reference", nodeId: "1", label: "N" },              │
│        { kind: "Reference", nodeId: "...", label: "M" }             │
│      ]                                                               │
│    }                                                                 │
├──────────────────────────────────────────────────────────────────────┤
│ 9. User selects "2" from candidate list                              │
├──────────────────────────────────────────────────────────────────────┤
│ 10. Preact: completePendingAction()                                  │
│     - pendingからcompleted expressionを構築:                          │
│       { kind: "BinOp", op: "Div",                                    │
│         lhs: { kind: "Var", ... },                                   │
│         rhs: { kind: "Lit", value: 2 } }                             │
│     - action = { kind: "SetExpr",                                    │
│         slot: slotJson,                                              │
│         expr: completedExpr }                                        │
│     - apply_action(doc, action) 呼び出し                             │
├──────────────────────────────────────────────────────────────────────┤
│ 11. wasm: apply_action()                                             │
│     - ASTのArray.lengthを N/2 に更新                                 │
│     - 新document JSON返却                                            │
├──────────────────────────────────────────────────────────────────────┤
│ 12. Preact: finalize                                                 │
│     documentJson.value = result.document;                            │
│     pendingExprAction.value = null;  // クリア                       │
│     closePopover();                                                  │
└──────────────────────────────────────────────────────────────────────┘
```

**重要**: ステップ6〜9の間、ASTには「N/?」のような不完全式は存在しない。未完成状態は`pendingExprAction` signal内にのみ存在する。

---

### 3.3 User clicks "generate sample" → wasm call → result display

```
┌──────────────────────────────────────────────────────────────────────┐
│ 1. User clicks "Generate Sample" button in BottomPanel              │
├──────────────────────────────────────────────────────────────────────┤
│ 2. Preact: check completeness first                                 │
│    const comp = JSON.parse(project_completeness(doc));               │
│    if (!comp.isComplete) {                                           │
│      showWarning("Holes remain: " + comp.totalHoles);               │
│      // 続行可能だが警告表示                                          │
│    }                                                                 │
├──────────────────────────────────────────────────────────────────────┤
│ 3. Preact: generate_sample(doc, seed) 呼び出し                       │
│    - seed = sampleSeed.value（ランダムまたはユーザー指定）            │
├──────────────────────────────────────────────────────────────────────┤
│ 4. wasm: generate_sample()                                          │
│    - deserialize document                                            │
│    - 依存グラフ解析                                                  │
│    - 制約に基づく値生成                                              │
│    - sample_to_text() でテキスト化                                   │
│    - 返却: "3\n1 2 3\n"                                              │
│                                                                      │
│    エラーケース:                                                     │
│    - Hole残存で生成不能 → JsError("Cannot generate: 2 holes remain")│
│    - 制約矛盾 → JsError("Constraint conflict: N > M but M > N")      │
├──────────────────────────────────────────────────────────────────────┤
│ 5. Preact: display result                                           │
│    try {                                                             │
│      const sample = generate_sample(doc, seed);                      │
│      samplePreview.value = { success: true, text: sample };         │
│    } catch (e) {                                                     │
│      samplePreview.value = { success: false, error: String(e) };    │
│    }                                                                 │
├──────────────────────────────────────────────────────────────────────┤
│ 6. BottomPanel renders:                                              │
│    ┌─────────────────────────────────────┐                           │
│    │ Sample Output                       │                           │
│    │ ─────────────────────────────────── │                           │
│    │ 3                                   │                           │
│    │ 1 2 3                               │                           │
│    │                                     │                           │
│    │ [Regenerate] [Copy]                │                           │
│    └─────────────────────────────────────┘                           │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 4. パフォーマンスリスク一覧

### 4.1 Arena Size Scaling

| リスク | 影響度 | 対策 |
|--------|--------|------|
| ノード数 > 500 | 中 | project_structure_outlineの返却サイズ増大。pagination/virtualizationを検討 |
| 制約数 > 200 | 低 | project_constraint_listで軽量サマリを返す。詳細は個別取得 |
| 深いネスト（depth > 20） | 低 | 描画自体がボトルネック。wasmは問題なし |

**実測基準**（要検証）:
- project_structure_outline: <5ms (ノード100個)
- apply_action: <10ms (通常操作)

### 4.2 Projection Computation Cost

| 操作 | 想定コスト | 備考 |
|------|-----------|------|
| project_structure_outline | O(n) | 全ノード走査、毎document更新時 |
| project_node_render | O(1) | 単一ノード |
| project_hole_candidates | O(n) | スコープ内変数列挙が必要な場合 |
| project_expr_slot_candidates | O(n) | 参照可能変数の列挙 |
| project_diagnostics | O(n + c) | n=ノード数、c=制約数 |

**最適化余地**:
- diagnosticsはdocument変更箇所に限定した差分計算が可能（MVP後）
- 大規模ASTではoutlineをlazy loadingに変更可能

### 4.3 JSON Serialization Overhead

| サイズ | 想定時間 | 備考 |
|--------|---------|------|
| 10KB (小規模AST) | <1ms | 問題なし |
| 100KB (中規模AST) | 2-5ms | 許容範囲 |
| 1MB (大規模AST) | 20-50ms | UIジャンクの可能性。差分プロトコル要検討 |

**緩和策**:
- 大規模ASTは通常の競プロ入力では発生しない
- 発生した場合はWeb Worker移行を検討（MVP外）

### 4.4 Candidate Enumeration Cost

| 操作 | 最悪ケース | 対策 |
|------|-----------|------|
| hole_candidates | 固定カテゴリ列挙のみ | O(1)、問題なし |
| expr_slot_candidates | スコープ内全変数 | n < 100なら問題なし。上限設定可能 |
| constraint_target_candidates | 全ノード列挙 | フィルタリングUIで絞り込み |

**click時遅延取得の効果**:
- 描画時に候補を埋め込まないため、初回描画は高速
- クリック時のみ計算するため、使用されない候補のコストがゼロ

### 4.5 Re-render Frequency

| トリガー | 再描画範囲 | 最適化 |
|----------|-----------|-------|
| document更新 | 全体outline再取得 | Preact diffing最適化 |
| selection変更 | 局所（選択ノードのみ） | project_node_renderは個別呼び出し可能 |
| pending state変更 | Preact内のみ | wasmは呼ばない |

**Preact signals活用**:
- `computed`によりdocument変更時のみ再計算
- 細粒度signalで不要な再描画を抑制

---

## 5. 現行方針に対する支持/反対/留保

### 5.1 支持

| 方針 | 評価 | 理由 |
|------|------|------|
| render/action/global projection分離 (C-1〜C-4) | **完全支持** | 描画時の候補列挙回避により、初回描画が高速化。click時遅延取得は実用的 |
| JSON string境界 (E-4) | **完全支持** | 可視化・デバッグ容易。serde-wasm-bindgen導入の複雑さを回避 |
| Expression未完成をAST外に置く (B-2) | **完全支持** | pendingExprActionパターンにより、ASTの整合性が常に保証される |
| Constraint未完成をAST外に置く (B-4) | **完全支持** | DraftConstraintパターンは自然。フォーム入力の一般的なUXに一致 |
| Structure HoleをAST内に置く (B-1) | **完全支持** | 位置・順序の意味論があり、AST外では管理困難 |

### 5.2 反対

該当なし。設計方針は境界設計の観点から妥当。

### 5.3 留保

| 方針 | 評価 | 理由 |
|------|------|------|
| wasm stateless設計 | **条件付き支持** | 現時点では最もシンプル。undo/redo実装時に再検討が必要。wasm内にhistory stackを持つ方が効率的な可能性あり |
| 全document JSON受け渡し | **条件付き支持** | MVP規模では問題なし。大規模AST対応時に差分プロトコル（JSON Patch等）を検討 |
| 同期呼び出しのみ | **条件付き支持** | 現行のviewer同様に十分高速。sample生成が重くなった場合にasync化を検討 |

---

## 6. 不足点

### 6.1 未定義事項

| 項目 | 状態 | 必要なアクション |
|------|------|-----------------|
| ExprIdの具体的な型 | 未定義 | Domain Model Agentが定義。SlotRef = (NodeId, SlotName) で代替可能か検討 |
| SlotIdの具体的な型 | 未定義 | { parentId: NodeId, slotName: String } でJSON表現可能だが、型定義が必要 |
| undo/redo across boundary | 未定義 | wasm側でhistory stack管理 vs Preact側でdocument history管理。どちらを採用するか未決定 |
| 高レベルテンプレートのAction定義 | 部分定義 | "辺リスト追加"などはFillHole + 複数Operationの組み合わせ。Builder層の詳細設計が必要 |
| preview_actionの活用方法 | 概念のみ | UIでどうプレビューを見せるか。ツールチップ？インライン表示？ |

### 6.2 検証が必要な仮定

| 仮定 | 検証方法 |
|------|---------|
| apply_action < 10ms | 実装後のベンチマーク |
| JSON parse/stringify < 5ms (100KB) | プロトタイプでの計測 |
| click時候補列挙がUXを損なわない | ユーザーテスト（200ms以内ならOK） |

---

## 7. 他Agentに渡すべき論点

### 7.1 → Domain Model Agent

- **ExprId / SlotId型の確定**: wasm境界で使用するため、JSON serializable な形式が必要。`{ nodeId: string, slotName: string }` で十分か、専用の `SlotId` stable identifier が必要か？
- **SourceRef型の詳細設計**: NodeRef / ExprRef / SlotRef / ConstraintRef の統一的な表現。click targetの識別に使用。

### 7.2 → GUI Interaction Agent

- **pendingExprAction state machine**: phase遷移の詳細（select-lhs-or-rhs → select-other-side → confirm）。キャンセル時のリセット処理。
- **draftConstraint completion logic**: 各Constraint kindごとの完了条件。バリデーションメッセージの表示タイミング。
- **popover/modal state管理**: Preact signalでの状態管理パターン。複数popoverの排他制御。

### 7.3 → Critical Reviewer

- **Q-11 diff-based更新の必要性**: 現時点ではfull re-render（project_structure_outline再取得）で十分か？大規模ASTでの劣化を許容するか？
- **undo/redo責務分担**: MVP後回しだが、wasm側でhistory管理する場合、境界設計に影響。早期決定が望ましい。

### 7.4 → Sample Generation Agent

- **generate_sample errorの詳細化**: 現行は文字列エラーのみ。構造化エラー（Hole残存、制約矛盾、生成timeout等）の区別が必要か？
- **incremental sample preview**: document編集中にlive previewを出す場合、頻繁なgenerate_sample呼び出しが発生。throttling戦略が必要か？

---

## 8. 主要論点への回答

### Q-6: Is render/action/global projection split workable at the wasm boundary?

**回答: Yes（完全に実現可能）**

- Render projection: `project_structure_outline`, `project_node_render`, `project_expr_render` で描画情報のみ返却
- Action projection: `project_hole_candidates`, `project_node_actions`, `project_expr_slot_candidates` でclick時のみ呼び出し
- Global projection: `project_diagnostics`, `project_completeness`, `project_constraint_list`

分離により:
- 初回描画: outline + diagnostics + completeness のみ（高速）
- click時: 対象に応じたaction projection（遅延取得）
- operation後: document更新 → outline再取得

### Q-7: Is click-time candidate enumeration performant enough?

**回答: Yes（ほぼ確実に十分）**

- `project_hole_candidates`: 固定カテゴリ列挙（O(1)）
- `project_expr_slot_candidates`: スコープ内変数列挙（O(n), n < 100が典型）
- `project_node_actions`: 固定操作列挙（O(1)）

想定レイテンシ: <50ms（click後のpopover表示まで）

200ms以内であればユーザーは遅延を感じない。wasm呼び出し + JSON parse + popover renderを含めても十分余裕がある。

### Q-11: Diff-based update vs full re-render?

**回答: MVP では full re-render で十分。条件付きで diff 検討。**

理由:
1. 典型的AST規模（ノード<100）では `project_structure_outline` は<5ms
2. Preact の virtual DOM diffing が効率的
3. diff プロトコル（JSON Patch等）の実装コストが高い

条件付き検討:
- ノード数 > 500 が頻発する場合
- re-render が 100ms を超える場合

その場合の対策:
- `project_structure_diff(oldDoc, newDoc)` 関数の追加
- 返却: 追加/削除/更新されたノードIDのみ
- Preact側で部分更新

---

## 9. Appendix: TypeScript型定義ドラフト

```typescript
// ── IDs ──
type NodeId = string;    // decimal string
type ConstraintId = string;
type SlotRef = { parentId: NodeId; slotName: string };

// ── Render Projection ──
interface StructureOutlineView {
  rootId: NodeId;
  nodes: OutlineNode[];
}

interface OutlineNode {
  id: NodeId;
  label: string;
  kindLabel: string;
  depth: number;
  isHole: boolean;
  expectedKindHint: string | null;
  childIds: NodeId[];
}

interface DiagnosticsView {
  items: DiagnosticItem[];
  errorCount: number;
  warningCount: number;
  infoCount: number;
}

interface DiagnosticItem {
  level: "error" | "warning" | "info";
  nodeId: NodeId | null;
  constraintId: ConstraintId | null;
  message: string;
  suggestion: string | null;
}

interface CompletenessView {
  totalHoles: number;
  filledSlots: number;
  unsatisfiedConstraints: number;
  isComplete: boolean;
}

// ── Action Projection ──
interface HoleCandidateMenu {
  candidates: HoleCandidate[];
}

type HoleCandidate =
  | { kind: "IntroduceScalar"; suggestedNames: string[] }
  | { kind: "IntroduceArray"; suggestedNames: string[] }
  | { kind: "IntroduceMatrix" }
  | { kind: "IntroduceTuple" }
  | { kind: "IntroduceRepeat" }
  | { kind: "IntroduceSection" }
  | { kind: "IntroduceChoice" };

interface ExprActionMenu {
  actions: ExprAction[];
}

interface ExprAction {
  id: string;
  label: string;
  kind: "replace" | "wrap-binary" | "wrap-call" | "delete";
}

interface ExprCandidateMenu {
  candidates: ExprCandidate[];
}

type ExprCandidate =
  | { kind: "Literal"; value: number }
  | { kind: "Reference"; nodeId: NodeId; label: string }
  | { kind: "IndexedReference"; nodeId: NodeId; label: string; indexVar: string };

// ── Operations ──
type ActionDto =
  | { kind: "FillHole"; target: NodeId; fill: FillContentDto }
  | { kind: "ReplaceNode"; target: NodeId; replacement: FillContentDto }
  | { kind: "AddConstraint"; target: NodeId; constraint: ConstraintDefDto }
  | { kind: "RemoveConstraint"; constraintId: ConstraintId }
  | { kind: "SetExpr"; slot: SlotRef; expr: ExpressionDto }
  // ... other actions

type FillContentDto =
  | { kind: "Scalar"; name: string }
  | { kind: "Array"; name: string; length: ExpressionDto }
  | { kind: "Matrix"; name: string; rows: ReferenceDto; cols: ReferenceDto }
  | { kind: "Tuple"; elements: NodeId[] }
  | { kind: "Repeat"; count: ExpressionDto; indexVar: string | null; body: NodeId[] }
  | { kind: "Section"; header: NodeId | null; body: NodeId[] }
  | { kind: "Sequence"; children: NodeId[] }
  | { kind: "Choice"; tag: ReferenceDto; variants: ChoiceVariantDto[] }
  | { kind: "Hole"; expectedKind: string | null };

// ── EditorState (Preact側) ──
interface EditorState {
  documentJson: string;
  selectedNodeId: NodeId | null;
  selectedConstraintId: ConstraintId | null;
  pendingExprAction: PendingExprAction | null;
  draftConstraint: DraftConstraint | null;
  openPopover: PopoverState | null;
}

type PendingExprAction =
  | {
      kind: "wrapBinary";
      slot: SlotRef;
      op: "Add" | "Sub" | "Mul" | "Div";
      phase: "select-lhs-or-rhs" | "select-other-side";
      lhs: ExpressionDto | null;
      rhs: ExpressionDto | null;
    }
  | {
      kind: "wrapCall";
      slot: SlotRef;
      func: "min" | "max" | "abs" | "len";
      phase: "select-args";
      args: (ExpressionDto | null)[];
    }
  | {
      kind: "replace";
      slot: SlotRef;
      phase: "select-replacement";
    };

type DraftConstraint =
  | { kind: "Range"; target: ReferenceDto | null; lower: ExpressionDto | null; upper: ExpressionDto | null }
  | { kind: "TypeDecl"; target: ReferenceDto | null; expectedType: "Int" | "Str" | "Char" | null }
  | { kind: "LengthRelation"; target: ReferenceDto | null; length: ExpressionDto | null }
  | { kind: "Relation"; lhs: ExpressionDto | null; op: string | null; rhs: ExpressionDto | null };
```

---

## 10. 結論

本設計により、以下を実現する:

1. **明確な責務分離**: wasm（意味論）/ Preact（体験）の境界が明確
2. **stateless wasm**: 各呼び出しにdocument JSONを渡し、結果を受け取る単純なパターン
3. **遅延候補列挙**: render時は軽量projection、click時に詳細取得
4. **型安全な境界**: JSON DTOによる明示的な契約
5. **エラーハンドリング**: 構造化エラーによりUI側で適切なフィードバック可能

MVP実装において、この境界設計は十分に実用的であり、将来の拡張（undo/redo、大規模AST対応）にも対応可能な余地を残している。
