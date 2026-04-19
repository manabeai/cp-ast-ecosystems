# Sub-project B+C 設計書: Projectional Editor 実装

## 概要

競技プログラミング AST Editor の編集機能を実装する。
Rust WASM バックエンド（Projection + Actions API）と Preact フロントエンドを構築し、
26個の E2E テストを全て通すことをゴールとする。

**スコープ**: E2Eテストが通る最小限の実装。ノード削除・Undo等の汎用機能は対象外。

---

## 1. アーキテクチャ: TEA (The Elm Architecture)

### 1.1 データフロー

```
┌──────────────────────────────────────────────────────┐
│  Model: Preact Signal (documentJson)                  │
│  - documentJson: string          … AST 全体の状態     │
│  - projection: FullProjection    … computed signal     │
│  - popupState: PopupState        … ポップアップ局所状態│
└───────────┬────────────────────────────┬──────────────┘
            │ View                       │ Update
            ▼                            ▼
  project_full(docJson)         apply_action(docJson, actionJson)
  render_*_tex(docJson)         → 新しい docJson が返る
  generate_sample(docJson)      → Signal 更新 → 再描画
```

### 1.2 状態管理方針

| 状態 | 保持場所 | 理由 |
|------|---------|------|
| documentJson (AST全体) | Preact Signal | WASMはステートレス。Signal変更→computed再計算が自然 |
| projection (UI構造) | computed Signal | documentJson変更時に自動再計算 |
| popupState (中間操作) | Preact Signal (局所) | ポップアップ確定までWASMに送らない |
| tex/sample (プレビュー) | computed Signal | documentJson依存で自動更新 |

### 1.3 ポップアップ分離原則

ポップアップ内の中間状態はWASMに送らない。

- 例: NをN-1にする場合、N→N-→N-1の過程でN-の状態ではActionを投げない
- 確定ボタン押下時にのみAction JSONを構築してWASMに送信
- フロントが持つのは「確定に必要な最小限のバリデーション」のみ（正整数チェック等）

### 1.4 制約bound入力のポップアップ

draft制約の `?` をクリックするとポップアップが表示される:
- 参照可能な変数リスト（N, M, ...）をクリックで選択
- 自由入力欄（正整数のみ）
- 変数選択後、その変数をクリックすると関数適用ポップアップ（+,-,*,/,min,max）が表示
- 関数のオペランドも自由入力（正整数）
- 確定時にExpression全体をAction化

---

## 2. WASM API 設計 (Sub-project B)

### 2.1 DTO変換境界の設計原則

**DTO変換は WASM エントリーポイント（`#[wasm_bindgen]` 関数）だけが担う。**

```rust
// WASM境界層: DTO ↔ AstEngine 変換はここだけ
#[wasm_bindgen]
pub fn project_full(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;      // DTO → AstEngine
    let projection = projection::project(&engine); // AstEngine で操作
    serde_json::to_string(&projection)             // 結果 → JSON
        .map_err(|e| JsError::new(&e.to_string()))
}
```

内部のprojection関数やaction適用関数は `AstEngine` を直接受け取り、
DTO/JSONを一切意識しない:

```rust
// 内部API: AstEngine だけを扱う（DTOフリー）
pub fn project(engine: &AstEngine) -> FullProjection { ... }
pub fn apply(engine: &mut AstEngine, action: &Action) -> ApplyResult { ... }
```

### 2.2 新規追加関数 (6個)

#### 2.2.1 `new_document() → String`

空のAST（Sequenceルート + 1つのHole）をJSON文字列で返す。

```rust
#[wasm_bindgen]
pub fn new_document() -> Result<String, JsError> {
    let engine = AstEngine::new(); // 空のエンジン生成
    serialize(&engine)             // JSON化
}
```

#### 2.2.2 `project_full(document_json) → String (JSON)`

Rich Projection。フロントエンドはこの出力をそのまま描画する。

**出力スキーマ:**

```typescript
interface FullProjection {
  nodes: ProjectedNode[];        // 表示順のノード一覧
  hotspots: Hotspot[];           // 挿入ポイント一覧
  constraints: {
    drafts: DraftConstraint[];   // 未完了制約
    completed: CompletedConstraint[]; // 完了制約
  };
  available_vars: string[];      // 式入力で使える変数名
  completeness: CompletenessSummary;
}

interface ProjectedNode {
  id: string;
  label: string;       // "N", "A₁ A₂ … Aₙ", etc.
  kind: string;        // "scalar", "array", "repeat", etc.
  depth: number;       // ネスト深さ
  is_hole: boolean;
}

interface Hotspot {
  parent_id: string;
  direction: "below" | "right" | "inside" | "variant";
  candidates: string[];  // "scalar", "array", "grid-template", etc.
}

interface DraftConstraint {
  index: number;         // data-testid="draft-constraint-{index}"
  constraint_id: string;
  target_id: string;
  target_name: string;
  display: string;       // "? ≤ N ≤ ?"
  template: string;      // "range", "type_decl", etc.
}

interface CompletedConstraint {
  index: number;         // data-testid="completed-constraint-{index}"
  constraint_id: string;
  display: string;       // "1 ≤ Aᵢ ≤ 10⁹"
}
```

#### 2.2.3 `apply_action(document_json, action_json) → String`

Action適用後の新しい document JSON を返す。

**境界層での処理フロー:**
1. `document_json` → `AstEngine` にデシリアライズ
2. `action_json` → `Action` にデシリアライズ
3. `engine.apply(&action)` で適用（AstEngine型で操作）
4. 結果の `AstEngine` → JSON にシリアライズして返す

#### 2.2.4 `get_hole_candidates(document_json, hole_id) → String (JSON)`

特定Holeに対する候補一覧と、各候補に必要な入力フィールド情報を返す。

```typescript
interface HoleCandidateDetail {
  kind: string;          // "scalar", "array", etc.
  label: string;         // 表示名
  fields: CandidateField[];
}

interface CandidateField {
  name: string;          // "name", "type", "length", "count", "weight_name"
  field_type: string;    // "text", "select", "var_select"
  options?: string[];    // select時の選択肢
  default_value?: string;
}
```

#### 2.2.5 `get_expr_candidates(document_json) → String (JSON)`

式入力で参照可能な変数一覧を返す。
制約 bound 入力ポップアップで `?` クリック時に表示される変数リスト。

```typescript
interface ExprCandidate {
  name: string;     // "N", "M", etc.
  node_id: string;  // 対応するノードID
}
```

#### 2.2.6 `get_constraint_targets(document_json) → String (JSON)`

Property / SumBound ショートカットの対象ノード一覧。

```typescript
interface ConstraintTarget {
  node_id: string;
  name: string;
  applicable_properties: string[];  // "Tree", "Connected", etc.
  can_sumbound: boolean;
}
```

---

## 3. Rust 内部拡張 (Sub-project B)

### 3.1 FullProjection 型の追加

`crates/cp-ast-core/src/projection/` に以下を追加:

- `FullProjection` 構造体（Serialize derive付き）
- `project(engine: &AstEngine) -> FullProjection` 関数
- 既存の `ProjectionAPI` trait実装を利用して構築

### 3.2 Action JSON デシリアライズ

`crates/cp-ast-json/` に Action ↔ JSON 変換を追加:

- `ActionDto` — Action の JSON 表現
- `FillContentDto` — FillContent の JSON 表現
- `ConstraintDefDto` — ConstraintDef の JSON 表現
- `serialize_action(action) -> String`
- `deserialize_action(json) -> Result<Action, ConversionError>`

### 3.3 Draft 制約自動生成

**Draft制約はprojection層がオンザフライで生成する（ASTには保存しない）。**

`project_full()` 内で、各ノードに対し「必要だが未定義の制約」を検出してdraftとして返す:

- Scalar (Int型) が Range 制約を持たない → Range draft (`? ≤ N ≤ ?`)
- Array 要素が Range 制約を持たない → Range draft (`? ≤ A_i ≤ ?`)
- Grid (Str型) が CharSet 制約を持たない → CharSet draft
- 既に対応する制約が存在するノードにはdraftを生成しない

Draft はUI概念であり、ユーザーが制約を入力すると AddConstraint Action で実体制約がASTに追加される。
この時点でdraftは自然に消滅する（projection再計算時に「制約あり」と判定されるため）。

### 3.4 Hotspot 生成ロジック

`project_full()` 内で:

- Sequence直下 → `below` hotspot (全候補)
- Scalar/Array の右隣 → `right` hotspot (scalar のみ)
- Repeat の body → `inside` hotspot
- Choice → `variant` hotspot

### 3.5 FillContent 拡張

E2Eテストで必要な以下のFillContent variantが未実装の場合は追加:

- `EdgeList { edge_count: LengthSpec, weighted: bool, weight_name: Option<String> }`
- `QueryList { query_count: LengthSpec }`
- `MultiTestCase { count_var_name: String }`
- `GridTemplate { name: String, row_var: LengthSpec, col_var: LengthSpec, cell_type: VarType }`

---

## 4. フロントエンド設計 (Sub-project C)

### 4.1 ファイル構成

```
web/src/
├── main.tsx                   (既存: エントリーポイント、WASM初期化)
├── app.tsx                    (改修: EditorPage をデフォルトに)
├── wasm.ts                    (改修: 新規WASM関数をexport追加)
├── state.ts                   (既存: viewer用シグナル、改修不要)
├── editor/
│   ├── editor-state.ts        (新規: エディタ用シグナル)
│   ├── popup-state.ts         (新規: ポップアップ局所状態)
│   ├── action-builder.ts      (新規: UIイベント → Action JSON変換)
│   ├── EditorPage.tsx         (新規: 3ペインレイアウト)
│   ├── StructurePane.tsx      (新規: 構造ペイン)
│   ├── NodePopup.tsx          (新規: ノード作成ウィザード)
│   ├── ExpressionBuilder.tsx  (新規: 式構築 N-1等)
│   ├── ConstraintPane.tsx     (新規: 制約ペイン)
│   ├── ConstraintEditor.tsx   (新規: 制約編集)
│   ├── ValueInput.tsx         (新規: 値入力ポップアップ)
│   └── PreviewPane.tsx        (新規: TeX+サンプル表示)
├── index.css                  (既存: スタイル拡張)
```

### 4.2 状態管理 (`editor-state.ts`)

```typescript
import { signal, computed } from '@preact/signals';

// Model
export const documentJson = signal<string>('');
export const sampleSeed = signal<number>(42);

// Computed (WASM呼び出し)
export const projection = computed(() => {
  if (!documentJson.value) return null;
  return JSON.parse(project_full(documentJson.value));
});

export const texInput = computed(() => {
  if (!documentJson.value) return '';
  return render_input_tex(documentJson.value);
});

export const texConstraints = computed(() => {
  if (!documentJson.value) return '';
  return render_constraints_tex(documentJson.value);
});

export const sampleOutput = computed(() => {
  if (!documentJson.value) return '';
  return generate_sample(documentJson.value, sampleSeed.value);
});

// Update
export function dispatchAction(actionJson: string): void {
  const newDoc = apply_action(documentJson.value, actionJson);
  documentJson.value = newDoc;
}

// Initialize
export function initEditor(): void {
  documentJson.value = new_document();
}
```

### 4.3 ポップアップ状態 (`popup-state.ts`)

```typescript
import { signal } from '@preact/signals';

export type PopupMode =
  | { type: 'closed' }
  | { type: 'node'; holeId: string; direction: string }
  | { type: 'constraint'; constraintId: string; bound: 'lower' | 'upper' }
  | { type: 'property' }
  | { type: 'sumbound' };

export const popupMode = signal<PopupMode>({ type: 'closed' });

// ノード作成ウィザードの中間状態
export const selectedCandidate = signal<string | null>(null);
export const nameValue = signal<string>('');
export const typeValue = signal<string>('number');
export const lengthValue = signal<string>('');

// 式ビルダーの中間状態
export const expressionBase = signal<string | null>(null);
export const expressionOp = signal<string | null>(null);
export const expressionOperand = signal<string>('');
```

### 4.4 コンポーネント概要

#### EditorPage.tsx
- 3ペインのgridレイアウト
- `initEditor()` を `useEffect` で呼ぶ
- `projection` signal を各ペインに渡す

#### StructurePane.tsx
- `projection.nodes` をリスト描画（depth に応じてインデント）
- Hole ノードの位置に InsertionHotspot を表示
- Hotspot クリック → `popupMode` を `node` に変更

#### NodePopup.tsx
- `popupMode.type === 'node'` 時に表示
- 候補一覧 → 選択 → フィールド入力 → 確定
- 確定時: `action-builder` でAction JSON構築 → `dispatchAction()`
- data-testid: `node-popup`, `popup-option-{x}`, `name-input`, `type-select`, `length-select`, `confirm-button`

#### ExpressionBuilder.tsx
- `count-field` クリック → 変数一覧表示
- 変数クリック → `expression-element-{var}` → 関数ポップアップ
- 関数選択 → オペランド入力 → Enter で確定
- data-testid: `count-field`, `count-var-option-{x}`, `expression-element-{x}`, `function-op-{x}`, `function-operand-input`

#### ConstraintPane.tsx
- draft一覧: `draft-constraint-{index}` クリックで ConstraintEditor 表示
- completed一覧: `completed-constraint-{index}`
- Property/SumBound ショートカットボタン

#### ConstraintEditor.tsx
- `constraint-lower-input`, `constraint-upper-input` クリック → ValueInput ポップアップ表示
- 確定時: AddConstraint Action を dispatch

#### ValueInput.tsx (ポップアップ)
- `?` クリック時にポップアップとして表示
- 変数リスト（`constraint-var-option-{x}`）をクリックで選択
- 自由入力欄（`constraint-value-literal`）正整数のみ
- 変数選択後、変数クリック → 関数適用（ExpressionBuilder 共用）
- data-testid: `constraint-value-literal`, `constraint-var-option-{x}`, `constraint-{bound}-expression`

#### PreviewPane.tsx
- `tex-input-format`: KaTeX で TeX レンダリング
- `tex-constraints`: KaTeX で制約レンダリング
- `sample-output`: サンプルテキスト表示

### 4.5 単体テスト

各コンポーネントに Preact Testing Library によるユニットテスト:

```
web/tests/unit/
├── editor-state.test.ts       (状態管理ロジック)
├── action-builder.test.ts     (Action JSON構築)
├── NodePopup.test.tsx         (ポップアップ操作)
├── ConstraintEditor.test.tsx  (制約編集)
├── ValueInput.test.tsx        (値入力ポップアップ)
└── ExpressionBuilder.test.tsx (式構築)
```

テストフレームワーク: Vitest + @testing-library/preact

---

## 5. testid 契約一覧

E2E テストの POM (editor-page.ts) で使用される全 testid:

### Structure Pane

| testid | 要素 | コンポーネント |
|--------|------|---------------|
| `structure-pane` | ペイン全体 | StructurePane |
| `insertion-hotspot-below` | 下方向挿入ボタン | StructurePane |
| `insertion-hotspot-right` | 右方向挿入ボタン | StructurePane |
| `insertion-hotspot-inside` | 内部挿入ボタン | StructurePane |
| `insertion-hotspot-variant` | バリアント追加ボタン | StructurePane |
| `node-popup` | ノード作成ポップアップ | NodePopup |
| `popup-option-scalar` | scalar選択 | NodePopup |
| `popup-option-array` | array選択 | NodePopup |
| `popup-option-grid-template` | gridテンプレート | NodePopup |
| `popup-option-edge-list-template` | 辺リストテンプレート | NodePopup |
| `popup-option-query-template` | クエリテンプレート | NodePopup |
| `popup-option-multi-testcase-template` | 複数テストケース | NodePopup |
| `popup-option-weighted-edge-list` | 重み付き辺リスト | NodePopup |
| `popup-option-tuple` | タプル | NodePopup |
| `name-input` | 変数名入力 | NodePopup |
| `type-select` | 型選択 | NodePopup |
| `length-select` | 長さ変数選択 | NodePopup |
| `weight-name-input` | 重み変数名入力 | NodePopup |
| `confirm-button` | 確定ボタン | NodePopup |
| `count-field` | カウント入力フィールド | ExpressionBuilder |
| `count-var-option-{var}` | カウント変数選択肢 | ExpressionBuilder |
| `expression-element-{var}` | 式の変数要素 | ExpressionBuilder |
| `function-op-{op}` | 関数操作ボタン | ExpressionBuilder |
| `function-operand-input` | 関数オペランド入力 | ExpressionBuilder |

### Constraint Pane

| testid | 要素 | コンポーネント |
|--------|------|---------------|
| `constraint-pane` | ペイン全体 | ConstraintPane |
| `draft-constraint-{index}` | draft制約アイテム | ConstraintPane |
| `completed-constraint-{index}` | 完了制約アイテム | ConstraintPane |
| `constraint-lower-input` | 下限入力エリア | ConstraintEditor |
| `constraint-upper-input` | 上限入力エリア | ConstraintEditor |
| `constraint-value-literal` | 自由入力（正整数） | ValueInput |
| `constraint-var-option-{var}` | 変数選択肢 | ValueInput |
| `constraint-lower-expression` | 下限の式要素 | ConstraintEditor |
| `constraint-upper-expression` | 上限の式要素 | ConstraintEditor |
| `constraint-confirm` | 制約確定ボタン | ConstraintEditor |
| `property-shortcut` | Property追加ボタン | ConstraintPane |
| `property-option-{name}` | Property選択肢 | ConstraintPane |
| `sumbound-shortcut` | SumBound追加ボタン | ConstraintPane |
| `sumbound-var-select` | SumBound変数選択 | ConstraintPane |
| `sumbound-upper-input` | SumBound上限入力 | ConstraintPane |
| `sumbound-upper-expression` | SumBound上限式要素 | ConstraintPane |

### Preview Pane

| testid | 要素 | コンポーネント |
|--------|------|---------------|
| `preview-pane` | ペイン全体 | PreviewPane |
| `tex-input-format` | TeX入力形式 | PreviewPane |
| `tex-constraints` | TeX制約 | PreviewPane |
| `sample-output` | サンプル出力 | PreviewPane |

### Math Editing

| testid | 要素 | コンポーネント |
|--------|------|---------------|
| `math-editable-{id}` | 編集可能な数式要素 | ConstraintEditor |
| `math-editor-input` | 数式エディタ入力 | ConstraintEditor |
| `math-editor-confirm` | 数式エディタ確定 | ConstraintEditor |

---

## 6. 制約事項

- 既存の22個のE2Eテスト（5ファイル）は一切変更しない
- 新規追加の4個のE2Eテスト（graph.spec.ts）も変更しない
- helpers.ts は変更しない
- editor-page.ts の既存メソッドは変更しない（追加のみ可）
- スコープ: 26個のE2Eテストが通る最小限の実装
