# Phase 2: GUI Interaction Agent レポート

本文書は、Editor UI設計におけるGUI Interaction Agentの検討結果である。

---

## 1. 前提

### 1.1 採用する固定前提

phase1-premises.md および plan.md から以下を前提とする。

| ID | 前提 | 本Agentへの影響 |
|---|------|---------------|
| A-1 | AST は1種類（StructureAST + ConstraintAST + Expression） | UIは単一ASTを投影する |
| A-3 | UIはASTの投影（projectional editor） | 自由テキスト入力ではなく構造編集 |
| B-1 | Structure未完成 → AST の NodeKind::Hole | Holeは第一級市民として可視化 |
| B-2 | Expression未完成 → EditorState/PendingExprAction | 部分式は AST に入らない |
| B-3 | Constraint未完成 → EditorState/DraftConstraint | 未完成制約は AST に入らない |
| C-1 | Render Projection: 描画最小情報のみ | render時に候補全列挙しない |
| C-2 | Action Projection: click時に候補・操作を遅延取得 | click時にwasmを呼ぶ |
| D-2 | template-driven編集 | 高レベルブロック操作を前面に出す |
| D-3 | 3カラム+下部パネルレイアウト | plan.md §14.1準拠 |
| D-4 | 式スロットは統一設計 | expected_sort, scope, slot_rule で制御 |

### 1.2 本Agent の担当範囲

1. 画面レイアウト設計
2. Preactコンポーネント階層と責務
3. 基本ケース全ての操作フロー
4. UI状態遷移
5. Render時 vs Click時のデータ取得タイミング
6. PendingExprAction の具体的UI体験
7. DraftConstraint のフォーム設計

---

## 2. 主要判断

### 2.1 画面レイアウト設計

plan.md §14.1 の3カラム+下部パネルを具体化する。

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
│ [+ テンプレート]│ │ 関連制約:            │  │ ─────────────────────  │
│                │ │   • 1 ≤ A_i ≤ 10⁹   │  │ DraftConstraint:       │
│                │ └─────────────────────┘  │ [Range を作成中...]     │
├────────────────┴───────────────────────────┴─────────────────────────┤
│ Bottom Panel                                                         │
│ [Sample] [Preview] [Diagnostics] [Log]                               │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ 3                                                                │ │
│ │ 1 5 2                                                            │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

#### 各領域の固定責務

| 領域 | 責務 | render時取得情報 | click時取得情報 |
|-----|------|----------------|----------------|
| Header | プロジェクト操作・サマリ表示 | completeness, diagnostics count | なし |
| Structure Pane | 構造ツリー表示・追加UI | nodes(), is_hole, label, depth | hole_candidates() |
| Detail Pane | 選択ノードの詳細編集 | node_render(), slot_render() | expr_actions(), slot_candidates() |
| Constraint Pane | 制約一覧・追加UI | constraint_list() | constraint_target_candidates() |
| Bottom Panel | Sample・Preview・診断 | diagnostics(), sample_preview() | なし |

### 2.2 Preactコンポーネント階層

```typescript
<App>
  <EditorProvider>                    // EditorState を提供
    <HeaderBar />
    <MainLayout>
      <StructurePane>
        <StructureTree>
          <StructureNodeView nodeId />  // 各ノードの1行表示
            <HoleIndicator />          // Hole の場合の可視化
        </StructureTree>
        <AddMenu />                    // + ボタンからのドロップダウン
      </StructurePane>

      <DetailPane>
        <NodeInspector nodeId>
          <NameField />
          <TypeField />
          <SlotFieldList>
            <ExprSlotField slotId>    // 長さ、Range上下限など
              <InlineExprView />       // 現在の式表示
              <ExprPopover />          // click時の候補・操作
            </ExprSlotField>
          </SlotFieldList>
          <RelatedConstraintList />
        </NodeInspector>
      </DetailPane>

      <ConstraintPane>
        <ConstraintList>
          <ConstraintCard constraintId />
        </ConstraintList>
        <AddConstraintButton />
        <DraftConstraintPanel />       // 作成中制約のフォーム
      </ConstraintPane>
    </MainLayout>

    <BottomPanel>
      <TabBar />
      <SamplePreview />
      <CanonicalPreview />
      <DiagnosticsPanel />
      <OperationLog />
    </BottomPanel>

    <Modal />                          // PendingExprAction の選択UI
    <Popover />                        // click時候補表示
  </EditorProvider>
</App>
```

#### コンポーネント責務の明確化

| コンポーネント | 責務 | 状態保持 | wasm呼び出しタイミング |
|--------------|------|---------|---------------------|
| EditorProvider | EditorState管理 | selectedNodeId, pendingExprAction, draftConstraint | なし |
| StructureNodeView | ノード1行の表示 | なし（propsのみ） | render時: なし |
| HoleIndicator | Holeの視覚表現 | なし | click時: hole_candidates() |
| ExprSlotField | 式スロットの表示・編集入口 | なし | click時: expr_slot_candidates(), expr_actions() |
| DraftConstraintPanel | 制約フォーム | ローカルフォーム状態 | 確定時: apply_action(AddConstraint) |
| Modal | PendingExprAction の進行UI | なし（EditorStateから読む） | 選択確定時: apply_action(WrapExpr等) |

### 2.3 基本ケースの操作フロー

#### 2.3.1 Single Scalar (`N: int`)

**想定操作列:**
1. Structure Pane の `[+ 変数]` をクリック
2. ポップオーバーで「整数スカラー」を選択
3. Detail Pane で名前を `N` に設定
4. TypeDecl `int` が自動追加される（または明示選択）

**EditorState遷移:**
```
S0: { selectedNodeId: null, pendingExprAction: null, draftConstraint: null }
    ↓ click [+ 変数]
S1: { openPopover: { kind: "add-node", candidates: [...] }, ... }
    ↓ select "整数スカラー"
    ↓ wasm: apply_action(FillHole { target: root_hole, fill: Scalar { name: "" } })
S2: { selectedNodeId: new_scalar_id, openPopover: null, ... }
    ↓ input name "N"
    ↓ wasm: apply_action(SetNodeName { node: id, name: "N" })
S3: { selectedNodeId: id, ... }  // AST には Scalar(N) が存在
    ↓ TypeDecl追加（自動 or 明示）
    ↓ wasm: apply_action(AddConstraint { target: N, constraint: TypeDecl(Int) })
S4: 完了状態
```

#### 2.3.2 Array (`A: int[N]`)

**想定操作列:**
1. Structure Pane の `[+ 配列]` をクリック
2. Detail Pane で名前を `A` に設定
3. 要素型を `int` に設定
4. 長さスロットをクリック
5. 候補から `N` を選択

**EditorState遷移:**
```
S0: { selectedNodeId: scalar_N_id, ... }
    ↓ click [+ 配列]
    ↓ wasm: apply_action(AddSlotElement { parent: root, slot: "children", element: Array { name: "", length: Hole } })
S1: { selectedNodeId: new_array_id, ... }
    ↓ input name "A", select type "int"
S2: { selectedNodeId: array_id, ... }
    ↓ click 長さスロット
    ↓ wasm: project_expr_slot_candidates(length_slot_id)
S3: { openPopover: { kind: "expr-candidates", candidates: [{ kind: "ref", target: N }, { kind: "literal" }, ...] }, ... }
    ↓ select N
    ↓ wasm: apply_action(SetArrayLengthExpr { node: array_id, expr: Var(N) })
S4: 完了状態  // AST に Array(A, length=Ref(N)) が存在
```

#### 2.3.3 Tuple (`N M K` on same line)

**想定操作列:**
1. Structure Pane の `[+ タプル]` をクリック
2. タプルの子として3つのスカラーを追加
3. 各スカラーに名前 N, M, K を設定

**EditorState遷移:**
```
S0: { ... }
    ↓ click [+ タプル]
    ↓ wasm: apply_action(AddSlotElement { parent: root, slot: "children", element: Tuple { children: [Hole, Hole] } })
S1: { selectedNodeId: tuple_id, ... }
    // Tuple内にHole 2つ（デフォルト）
    ↓ click Hole[0], select "整数スカラー", name "N"
S2: { selectedNodeId: scalar_N_id, ... }
    ↓ click Hole[1], select "整数スカラー", name "M"
S3: { selectedNodeId: scalar_M_id, ... }
    ↓ click [+] on Tuple（子追加ボタン）
    ↓ wasm: apply_action(AddSlotElement { parent: tuple_id, slot: "children", element: Scalar { name: "K" } })
S4: 完了状態
```

#### 2.3.4 Grid (`A[H][W]`)

**想定操作列:**
1. `[+ テンプレート]` → 「グリッド」を選択
2. テンプレートが H, W, A[H][W] を一括生成
3. Detail Pane で名前を確認・調整

**EditorState遷移:**
```
S0: { ... }
    ↓ click [+ テンプレート] → "グリッド"
    ↓ wasm: apply_action(ApplyTemplate { kind: "grid", config: { rows: "H", cols: "W", name: "A" } })
    // 内部で以下を一括実行:
    //   FillHole(Tuple(H, W))
    //   AddSlotElement(Matrix { name: "A", rows: Ref(H), cols: Ref(W) })
    //   AddConstraint(TypeDecl(A, Char))  // デフォルト
S1: { selectedNodeId: matrix_A_id, ... }
    // Detail Pane で rows/cols のスロット確認・要素型変更
```

#### 2.3.5 Edge List (`N vertices, M edges as u_i v_i`)

**想定操作列:**
1. `[+ テンプレート]` → 「辺リスト」を選択
2. 頂点数 N、辺数 M を設定
3. Tree property を追加（任意）

**EditorState遷移:**
```
S0: { ... }
    ↓ click [+ テンプレート] → "辺リスト"
S1: { openModal: { kind: "template-config", template: "edge-list", fields: { vertex_count: "", edge_count: "", is_tree: false } }, ... }
    ↓ input vertex_count: "N", edge_count: "M"
    ↓ toggle is_tree: true
    ↓ click [適用]
    ↓ wasm: apply_action(ApplyTemplate { kind: "edge-list", config: { ... } })
    // 内部で以下を一括実行:
    //   AddSlotElement(Tuple(Scalar(N), Scalar(M)))
    //   AddSlotElement(Repeat { count: Ref(M), index_var: "i", body: Tuple(Scalar(u), Scalar(v)) })
    //   AddConstraint(Range(N, 1, 10^5))
    //   AddConstraint(Property(Tree))  // is_tree の場合
S2: { selectedNodeId: repeat_id, openModal: null, ... }
```

#### 2.3.6 Multi-testcase Wrapper

**想定操作列:**
1. 既存構造がある状態で `[+ テンプレート]` → 「複数テストケース化」
2. テストケース数の変数名を設定
3. 総和制約の有無を選択

**EditorState遷移:**
```
S0: { ... }  // 既存AST: Sequence(Scalar(N), Array(A))
    ↓ click [+ テンプレート] → "複数テストケース化"
S1: { openModal: { kind: "multi-testcase-config", fields: { count_var: "T", sum_bound: null } }, ... }
    ↓ input count_var: "T"
    ↓ set sum_bound: "N" (ΣN ≤ 2×10⁵)
    ↓ click [適用]
    ↓ wasm: apply_action(IntroduceMultiTestCase { count_var_name: "T", sum_bound: Some({ var: "N", upper: 2*10^5 }) })
S2: { ... }  // AST: Sequence(Scalar(T), Repeat(count=T, body=Sequence(Scalar(N), Array(A))))
```

#### 2.3.7 Query with Choice

**想定操作列:**
1. `[+ テンプレート]` → 「クエリ列」を選択
2. クエリ数の変数名を設定
3. クエリ種別（Choice）を追加
4. 各バリアントの構造を定義

**EditorState遷移:**
```
S0: { ... }
    ↓ click [+ テンプレート] → "クエリ列"
S1: { openModal: { kind: "query-list-config", fields: { count_var: "Q", variants: [] } }, ... }
    ↓ input count_var: "Q"
    ↓ click [バリアント追加]
    ↓ input variant[0]: { tag: "1", body: "l r" }  // l, r の意味
    ↓ click [バリアント追加]
    ↓ input variant[1]: { tag: "2", body: "x" }
    ↓ click [適用]
    ↓ wasm: apply_action(ApplyTemplate { kind: "query-list", config: { ... } })
    // 内部で以下を一括実行:
    //   AddSlotElement(Scalar(Q))
    //   AddSlotElement(Repeat { count: Ref(Q), body: Choice { tag: "type", variants: [
    //     Variant { tag: "1", body: Tuple(Scalar(l), Scalar(r)) },
    //     Variant { tag: "2", body: Scalar(x) }
    //   ] } })
S2: { selectedNodeId: choice_id, openModal: null, ... }
```

### 2.4 UI状態遷移マトリクス

#### EditorState 型定義（TypeScript）

```typescript
interface EditorState {
  // Selection
  selectedNodeId: NodeId | null;
  selectedConstraintId: ConstraintId | null;
  focusedSlotId: SlotId | null;

  // Popover/Modal
  openPopover: PopoverState | null;
  openModal: ModalState | null;

  // Pending Actions
  pendingExprAction: PendingExprAction | null;
  draftConstraint: DraftConstraint | null;

  // View State
  bottomPanelTab: 'sample' | 'preview' | 'diagnostics' | 'log';
  structureTreeExpanded: Set<NodeId>;
}

type PopoverState =
  | { kind: 'add-node'; anchorRef: RefObject; candidates: CandidateKind[] }
  | { kind: 'expr-candidates'; slotId: SlotId; candidates: ExprCandidate[] }
  | { kind: 'expr-actions'; exprId: ExprId; actions: ExprAction[] }
  | { kind: 'constraint-targets'; constraintKind: string; targets: PlaceRef[] };

type ModalState =
  | { kind: 'template-config'; template: string; fields: Record<string, unknown> }
  | { kind: 'pending-expr-select'; phase: PendingExprPhase };

type PendingExprAction =
  | { kind: 'wrapBinary'; targetExpr: ExprId; op: ArithOp; phase: 'select-other-side'; fixedSide: 'lhs' | 'rhs' }
  | { kind: 'wrapCall'; targetExpr: ExprId; func: FuncName; argsNeeded: number; collectedArgs: Expression[] }
  | { kind: 'replaceExpr'; targetSlotId: SlotId };

type DraftConstraint =
  | { kind: 'Range'; target: PlaceRef | null; lower: Expression | null; upper: Expression | null }
  | { kind: 'TypeDecl'; target: PlaceRef | null; expectedType: ExpectedType | null }
  | { kind: 'LengthRelation'; target: PlaceRef | null; length: Expression | null }
  | { kind: 'Relation'; lhs: Expression | null; op: RelationOp | null; rhs: Expression | null }
  | { kind: 'Distinct'; target: PlaceRef | null }
  | { kind: 'Sorted'; target: PlaceRef | null; order: SortOrder | null }
  | { kind: 'Property'; target: PlaceRef | null; tag: PropertyTag | null };
```

### 2.5 Render時 vs Click時 データ取得の分離

#### Render時に取得する情報（頻繁に更新）

| API | 返却情報 | 用途 |
|-----|---------|------|
| `project_structure_outline()` | NodeId, label, depth, is_hole の配列 | Structure Pane のツリー表示 |
| `project_node_render(nodeId)` | kindLabel, badges, children, clickableRegions | Detail Pane の選択ノード表示 |
| `project_slot_render(slotId)` | label, expected_sort, current_expr | 式スロットの現在値表示 |
| `project_diagnostics()` | errors, warnings の配列 | Header のカウント、Bottom Panel |
| `project_completeness()` | total_holes, filled_holes, is_complete | Header の完成度表示 |

#### Click時に取得する情報（遅延評価）

| API | トリガー | 返却情報 | 用途 |
|-----|---------|---------|------|
| `project_hole_candidates(nodeId)` | Hole ノードクリック | CandidateKind[] | 追加候補ポップオーバー |
| `project_expr_slot_candidates(slotId)` | 式スロットクリック | ExprCandidate[] | 参照・定数候補 |
| `project_expr_actions(exprId)` | 式内部クリック | ExprAction[] | wrap操作候補 |
| `project_constraint_target_candidates(kind)` | 制約追加時 | PlaceRef[] | target選択候補 |

#### 理由

- **Render時**: ASTの構造変更で再描画が必要。最小情報で効率化。
- **Click時**: ユーザー操作に応じて詳細取得。候補数が多い場合の事前計算を避ける。

**Q-7への回答**: click時候補列挙で**十分**。理由:
1. 候補数は通常10-50程度で、click時のwasm呼び出しは10ms以下
2. render時に全候補を埋め込むと、ノード数×候補数の計算が毎描画で発生
3. 候補はscopeに依存し、AST変更で無効化されるため事前計算のキャッシュが困難

### 2.6 PendingExprAction 具体的UIウォークスルー

#### シナリオ: `A: int[N/2]` の作成

**初期状態**: Array `A` の長さスロットに `N` が入っている

```
Detail Pane:
┌─────────────────────────────────────┐
│ Array: A                            │
├─────────────────────────────────────┤
│ 名前: [A          ]                 │
│ 要素型: int                          │
│ 長さ: [ N ]  ← クリック対象           │
│            ↑ この "N" をクリック      │
└─────────────────────────────────────┘
```

**ステップ1: 式内部をクリック**

```typescript
// EditorState変化
{ selectedNodeId: array_A_id, pendingExprAction: null, openPopover: null }
  ↓ click on "N" (ExprId: expr_N)
  ↓ wasm: project_expr_actions(expr_N)
  ↓ returns: [
      { id: "wrap-div", label: "/x (N を割る)", kind: "wrap-binary", op: "/" },
      { id: "wrap-add", label: "+x", kind: "wrap-binary", op: "+" },
      { id: "wrap-max", label: "max(N, x)", kind: "wrap-call", func: "max" },
      { id: "replace", label: "置換", kind: "replace" }
    ]

{ openPopover: { kind: "expr-actions", exprId: expr_N, actions: [...] }, ... }
```

**UI表示**:
```
長さ: [ N ]
         ↓ Popover
       ┌─────────────────┐
       │ ÷  /x (N を割る) │  ← 選択
       │ +  +x           │
       │ ↑  max(N, x)    │
       │ ⟳  置換         │
       └─────────────────┘
```

**ステップ2: `/x` を選択**

```typescript
// EditorState変化
{ openPopover: { ... }, pendingExprAction: null, ... }
  ↓ select "/x"

{
  openPopover: null,
  pendingExprAction: {
    kind: "wrapBinary",
    targetExpr: expr_N,
    op: "/",
    phase: "select-other-side",
    fixedSide: "lhs"  // N が左辺に固定
  },
  openModal: { kind: "pending-expr-select", phase: { ... } }
}
```

**UI表示**: モーダルが開く
```
┌──────────────────────────────────────┐
│ N / ?                                │
├──────────────────────────────────────┤
│ 右辺を選択:                          │
│                                      │
│ [2]  [10]  [100]  [他の値...]        │ ← よく使う定数
│                                      │
│ 参照:                                │
│ [ N ]  [ M ]  [ Q ]                  │ ← scope内の変数
│                                      │
│ [キャンセル]               [直接入力] │
└──────────────────────────────────────┘
```

**ステップ3: `2` を選択**

```typescript
// EditorState変化
{ pendingExprAction: { ... }, ... }
  ↓ select "2"
  ↓ 完成式を構築: BinOp(Var(N), Div, Lit(2))
  ↓ wasm: apply_action(ReplaceExpr { target: expr_N, expr: BinOp(Var(N), Div, Lit(2)) })
  ↓ 成功

{ pendingExprAction: null, openModal: null, ... }
```

**最終UI**:
```
長さ: [ N/2 ]  ← 完成した式が表示される
```

#### 契約確認

- **AST に `N/?` は存在しない**: PendingExprAction が EditorState にのみ存在
- **確定時に一発で更新**: `ReplaceExpr` または `WrapExprBinary` で AST を更新
- **キャンセル可能**: PendingExprAction をクリアするだけで AST に影響なし

**Q-3への回答**: PendingExprAction（partial ASTなし）で**十分**。理由:
1. 式操作は2-3ステップで完結し、複雑な分岐がない
2. 中間状態はUI表現だけで十分（"N / ?" のテキスト表示）
3. AST を汚さないことで、キャンセル・undo が自然に実装できる

### 2.7 DraftConstraint フォーム設計

#### 基本構造

```
Constraint Pane:
┌─────────────────────────────────┐
│ 📋 制約                          │
├─────────────────────────────────┤
│ ● N: int                        │
│ ● 1 ≤ N ≤ 10⁵                   │
│ ● A_i: int                      │
│ ● 1 ≤ A_i ≤ 10⁹                 │
├─────────────────────────────────┤
│ [+ Range] [+ Type] [+ その他 ▼] │
├─────────────────────────────────┤
│ ◐ Range を作成中                │  ← DraftConstraint
│   target: [    ▼]               │
│   lower:  [1     ]              │
│   upper:  [      ]              │
│   [キャンセル]         [追加]   │
└─────────────────────────────────┘
```

#### DraftConstraint種別ごとのフォーム

##### Range

```typescript
interface DraftRange {
  kind: 'Range';
  target: PlaceRef | null;    // 必須
  lower: Expression | null;   // 必須
  upper: Expression | null;   // 必須
}
```

```
┌─────────────────────────────────┐
│ Range 制約                      │
├─────────────────────────────────┤
│ 対象: [A[i] ▼]                  │  ← PlaceRef選択
│                                 │
│ 下限: [1     ] [式編集...]      │  ← ExprSlotField
│ 上限: [10⁹   ] [式編集...]      │
│                                 │
│ ⚠️ 上限が未設定です              │  ← バリデーション
│                                 │
│ [キャンセル]         [追加]     │  ← 全必須項目が埋まるまで無効
└─────────────────────────────────┘
```

##### TypeDecl

```typescript
interface DraftTypeDecl {
  kind: 'TypeDecl';
  target: PlaceRef | null;        // 必須
  expectedType: ExpectedType | null;  // 必須: Int, Str, Char
}
```

```
┌─────────────────────────────────┐
│ 型宣言                          │
├─────────────────────────────────┤
│ 対象: [N ▼]                     │
│ 型: (●) int  ( ) str  ( ) char  │
│                                 │
│ [キャンセル]         [追加]     │
└─────────────────────────────────┘
```

##### Property（Tree, Distinct, etc.）

```typescript
interface DraftProperty {
  kind: 'Property';
  target: PlaceRef | null;
  tag: PropertyTag | null;
}
```

```
┌─────────────────────────────────┐
│ 性質制約                        │
├─────────────────────────────────┤
│ 対象: [グラフ ▼]                │
│ 性質: [Tree ▼]                  │
│   ・Tree                        │
│   ・Simple                      │
│   ・Connected                   │
│   ・Permutation                 │
│                                 │
│ [キャンセル]         [追加]     │
└─────────────────────────────────┘
```

#### 完了条件ロジック

```typescript
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
    case 'Distinct':
    case 'Sorted':
    case 'Property':
      return draft.target !== null && draft.tag !== null;
  }
}

function finalizeDraft(draft: DraftConstraint): EditorAction {
  // DraftConstraint → AddConstraint Action への変換
  return { kind: 'AddConstraint', constraint: toConstraintDef(draft) };
}
```

**Q-4への回答**: DraftConstraint を EditorState に逃がして**OK**。理由:
1. 制約作成はフォーム入力であり、構造的な位置を持たない
2. 必須フィールドが埋まるまで AST に追加しない方が明快
3. キャンセル時に AST から削除する処理が不要

---

## 3. 具体例

### 3.1 `N: int` を空ASTから作成する

#### 操作列

1. エディタ起動（空AST = root にHole 1つ）
2. Structure Pane で Hole をクリック
3. ポップオーバーから「整数スカラー」を選択
4. Detail Pane で名前を `N` に入力
5. （任意）制約ペインで Range を追加

#### EditorState 変化

```
T0: 初期状態
  AST: { root: Hole }
  EditorState: {
    selectedNodeId: null,
    pendingExprAction: null,
    draftConstraint: null,
    openPopover: null
  }

T1: Hole をクリック
  → wasm: project_hole_candidates(hole_id)
  → returns: [IntroduceScalar, IntroduceArray, IntroduceTuple, ...]

  EditorState: {
    selectedNodeId: hole_id,
    openPopover: { kind: 'add-node', candidates: [...] }
  }

T2: 「整数スカラー」を選択
  → wasm: apply_action(FillHole { target: hole_id, fill: Scalar { name: "" } })
  → returns: { created_id: scalar_id, ... }

  AST: { root: Scalar { id: scalar_id, name: "" } }
  EditorState: {
    selectedNodeId: scalar_id,
    openPopover: null
  }

T3: 名前に "N" を入力（debounce後）
  → wasm: apply_action(SetNodeName { node: scalar_id, name: "N" })

  AST: { root: Scalar { id: scalar_id, name: "N" } }

T4: TypeDecl 自動追加（設定による）
  → wasm: apply_action(AddConstraint { target: scalar_id, constraint: TypeDecl(Int) })

  Constraints: [TypeDecl { target: N, expected: Int }]

T5: Range 追加（任意）
  → [+ Range] をクリック
  → DraftConstraint が開始

  EditorState: {
    draftConstraint: { kind: 'Range', target: null, lower: null, upper: null }
  }

T6: target を N に、lower を 1、upper を 10^5 に設定
  EditorState: {
    draftConstraint: { kind: 'Range', target: Ref(N), lower: Lit(1), upper: Pow(10, 5) }
  }

T7: [追加] をクリック
  → wasm: apply_action(AddConstraint { target: N, constraint: Range(1, 10^5) })

  Constraints: [TypeDecl(N, Int), Range(N, 1, 10^5)]
  EditorState: { draftConstraint: null }
```

### 3.2 `A: int[N/2]`（式編集とPendingAction）

#### 操作列

1. 既存の `N` がある状態で `[+ 配列]`
2. 名前を `A` に設定
3. 長さスロットをクリック → 候補から `N` を選択
4. 長さスロット内の `N` をクリック
5. `/x` を選択
6. 右辺に `2` を選択

#### EditorState 変化

```
T0: 初期状態
  AST: { root: Sequence([Scalar(N)]) }
  EditorState: { selectedNodeId: N_id, ... }

T1: [+ 配列] をクリック
  → wasm: apply_action(AddSlotElement { parent: seq_id, slot: "children", element: Array { name: "", length: Hole } })

  AST: { root: Sequence([Scalar(N), Array { name: "", length: Hole }]) }
  EditorState: { selectedNodeId: array_id }

T2: 名前を "A" に、長さスロットをクリック
  → wasm: project_expr_slot_candidates(length_slot_id)
  → returns: [Ref(N), Lit(任意), ...]

  EditorState: {
    selectedNodeId: array_id,
    openPopover: { kind: 'expr-candidates', slotId: length_slot_id, candidates: [...] }
  }

T3: N を選択
  → wasm: apply_action(SetArrayLengthExpr { node: array_id, expr: Var(N) })

  AST: { root: Sequence([Scalar(N), Array { name: "A", length: Var(N) }]) }
  EditorState: { openPopover: null }

T4: 長さスロット内の "N" をクリック
  → wasm: project_expr_actions(expr_N_in_length)
  → returns: ["/x", "+x", "max(N,x)", "replace"]

  EditorState: {
    openPopover: { kind: 'expr-actions', exprId: expr_N_id, actions: [...] }
  }

T5: "/x" を選択
  EditorState: {
    openPopover: null,
    pendingExprAction: {
      kind: 'wrapBinary',
      targetExpr: expr_N_id,
      op: '/',
      phase: 'select-other-side',
      fixedSide: 'lhs'
    },
    openModal: { kind: 'pending-expr-select', ... }
  }

  // この時点で AST には N/? は存在しない

T6: 右辺に 2 を選択
  → 完成式を構築: BinOp(Var(N), Div, Lit(2))
  → wasm: apply_action(ReplaceExpr { target: expr_N_id, expr: BinOp(...) })
  // 注: ReplaceExprがない場合は、SetArrayLengthExprで全体置換

  AST: { root: Sequence([Scalar(N), Array { name: "A", length: BinOp(Var(N), Div, Lit(2)) }]) }
  EditorState: {
    pendingExprAction: null,
    openModal: null
  }
```

### 3.3 `1 <= A[i] <= 10^9`（制約ドラフトフロー）

#### 操作列

1. 既存の配列 `A` がある状態
2. Constraint Pane で `[+ Range]` をクリック
3. target として `A[i]` を選択
4. lower に `1` を入力
5. upper に `10^9` を入力（式編集UI使用）
6. `[追加]` をクリック

#### EditorState 変化

```
T0: 初期状態
  Constraints: [TypeDecl(A, Int)]
  EditorState: { draftConstraint: null }

T1: [+ Range] をクリック
  EditorState: {
    draftConstraint: {
      kind: 'Range',
      target: null,
      lower: null,
      upper: null
    }
  }

  // Constraint Pane に DraftConstraintPanel が表示される

T2: target ドロップダウンをクリック
  → wasm: project_constraint_target_candidates('Range')
  → returns: [PlaceRef(N), PlaceRef(A, indexed=true), ...]

  // PlaceRef(A, indexed=true) は "A[i]" として表示

T3: A[i] を選択
  EditorState: {
    draftConstraint: {
      kind: 'Range',
      target: { nodeId: A_id, indexed: true, indexVar: 'i' },
      lower: null,
      upper: null
    }
  }

T4: lower に 1 を入力
  EditorState: {
    draftConstraint: {
      ...
      lower: Lit(1),
      upper: null
    }
  }

T5: upper スロットで式編集
  5a: スロットをクリック
      EditorState: { openPopover: { kind: 'expr-candidates', ... } }

  5b: "10^x" テンプレートを選択
      EditorState: {
        pendingExprAction: {
          kind: 'literalPow',
          base: 10,
          phase: 'select-exponent'
        }
      }

  5c: 指数に 9 を入力
      → 完成式: Pow(Lit(10), Lit(9))

  EditorState: {
    draftConstraint: {
      ...
      lower: Lit(1),
      upper: Pow(Lit(10), Lit(9))
    },
    pendingExprAction: null
  }

T6: [追加] をクリック（全必須項目が埋まっている）
  → wasm: apply_action(AddConstraint {
      target: A[i],
      constraint: Range { lower: 1, upper: 10^9 }
    })

  Constraints: [TypeDecl(A, Int), Range(A[i], 1, 10^9)]
  EditorState: { draftConstraint: null }
```

### 3.4 Tree入力をテンプレートで構築

#### 操作列

1. 空のエディタで `[+ テンプレート]` → 「辺リスト（木）」
2. 頂点数変数名を `N` に設定
3. `[適用]` をクリック
4. （自動生成された制約を確認）

#### EditorState 変化

```
T0: 初期状態
  AST: { root: Hole }
  EditorState: { ... }

T1: [+ テンプレート] をクリック
  EditorState: {
    openPopover: {
      kind: 'template-menu',
      items: ['単一変数', '配列', 'グリッド', '辺リスト', '辺リスト（木）', 'クエリ列', '複数テストケース']
    }
  }

T2: 「辺リスト（木）」を選択
  EditorState: {
    openPopover: null,
    openModal: {
      kind: 'template-config',
      template: 'tree-edge-list',
      fields: {
        vertex_count_var: 'N',    // デフォルト
        edge_count_expr: 'N-1',   // 木なので N-1 固定
        endpoint_vars: ['u', 'v'] // デフォルト
      }
    }
  }

T3: 設定を確認して [適用] をクリック
  → wasm: apply_action(ApplyTemplate {
      kind: 'tree-edge-list',
      config: { vertex_count_var: 'N', edge_count_expr: 'N-1', endpoint_vars: ['u', 'v'] }
    })

  // 内部で以下を一括実行:
  // 1. FillHole(Sequence([
  //      Scalar(N),
  //      Repeat { count: BinOp(Var(N), Sub, Lit(1)), index_var: 'i', body:
  //        Tuple([Scalar(u), Scalar(v)])
  //      }
  //    ]))
  // 2. AddConstraint(TypeDecl(N, Int))
  // 3. AddConstraint(Range(N, 2, 10^5))  // デフォルト
  // 4. AddConstraint(TypeDecl(u, Int))
  // 5. AddConstraint(TypeDecl(v, Int))
  // 6. AddConstraint(Range(u, 1, N))
  // 7. AddConstraint(Range(v, 1, N))
  // 8. AddConstraint(Property(Tree))

  AST: 上記の構造
  Constraints: [TypeDecl(N, Int), Range(N, ...), ..., Property(Tree)]
  EditorState: {
    openModal: null,
    selectedNodeId: root_sequence_id  // 作成された構造を選択
  }

T4: Detail Pane で確認・調整
  // N の Range を変更したい場合は制約を編集
```

### 3.5 Query問題のChoice追加

#### 操作列

1. 既存構造に `[+ テンプレート]` → 「クエリ列」
2. クエリ数変数を `Q` に設定
3. バリアント1: tag=1, body=「l r」（範囲クエリ）
4. バリアント2: tag=2, body=「x」（点クエリ）
5. `[適用]` をクリック

#### EditorState 変化

```
T0: 初期状態（N, Aがある）
  AST: { root: Sequence([Scalar(N), Array(A, length=N)]) }

T1: [+ テンプレート] → 「クエリ列」
  EditorState: {
    openModal: {
      kind: 'template-config',
      template: 'query-list',
      fields: {
        count_var: 'Q',
        variants: []  // 初期は空
      }
    }
  }

T2: [バリアント追加] をクリック
  EditorState: {
    openModal: {
      ...
      fields: {
        count_var: 'Q',
        variants: [{ tag: '', body: [] }]
      }
    }
  }

T3: バリアント1を設定: tag=1, body=[l, r]
  // モーダル内のフォームで入力
  EditorState: {
    openModal: {
      ...
      fields: {
        count_var: 'Q',
        variants: [{ tag: '1', body: ['l', 'r'] }]
      }
    }
  }

T4: [バリアント追加] → バリアント2を設定: tag=2, body=[x]
  EditorState: {
    openModal: {
      ...
      fields: {
        count_var: 'Q',
        variants: [
          { tag: '1', body: ['l', 'r'] },
          { tag: '2', body: ['x'] }
        ]
      }
    }
  }

T5: [適用] をクリック
  → wasm: apply_action(ApplyTemplate { kind: 'query-list', config: { ... } })

  // 内部で以下を生成:
  // AddSlotElement(Scalar(Q))
  // AddSlotElement(Repeat {
  //   count: Var(Q),
  //   index_var: 'i',
  //   body: Choice {
  //     tag_var: 'type',
  //     variants: [
  //       Variant { tag: '1', body: Tuple([Scalar(l), Scalar(r)]) },
  //       Variant { tag: '2', body: Scalar(x) }
  //     ]
  //   }
  // })

  AST: Sequence([Scalar(N), Array(A), Scalar(Q), Repeat(Q, Choice(...))])
  EditorState: {
    openModal: null,
    selectedNodeId: choice_id
  }
```

---

## 4. 現行方針に対する支持/反対/留保

### 4.1 支持

| 方針 | 支持理由 |
|-----|---------|
| AST は1種類 | UIの複雑さを抑え、単一の正本で全派生物を生成できる |
| Structure Hole は AST に置く | 構造的位置が意味を持ち、UI で持続的に可視化したい |
| Expression partial を AST に入れない | 式操作は2-3ステップで完結し、PendingExprAction で十分管理可能 |
| DraftConstraint を EditorState に逃がす | 制約作成はフォーム入力であり、位置を持たない |
| Render/Action/Global Projection の分離 | 描画効率と遅延取得の明確な分離 |
| click時候補列挙 | パフォーマンス上適切、候補の無効化管理が簡潔 |
| template-driven 編集 | 基本ケースの操作効率を大幅に向上 |

### 4.2 反対

なし。現行方針は GUI 操作体験の観点から適切。

### 4.3 留保

| 方針 | 留保理由 | 必要な追加検討 |
|-----|---------|--------------|
| ExprSlotField の統一設計 | expected_sort, slot_rule の具体値が未定義 | Domain Model Agent と協議して enum を確定 |
| SourceRef の粒度 | NodeRef, ExprRef, SlotRef, ConstraintRef の4種で十分か | wasm Boundary Agent と協議 |
| テンプレート展開の内部表現 | ApplyTemplate という単一 Action か、複数 Action のバッチか | Operation API 設計で確定 |

---

## 5. 不足点

### 5.1 型定義の不足

| 不足項目 | 現状 | 必要な追加 |
|---------|------|----------|
| ExprId | plan.md で言及あるが未実装 | `crates/cp-ast-core/src/expression/mod.rs` に追加 |
| SlotId | 概念のみ | 式スロットを識別するための型を追加 |
| PlaceRef | DraftConstraint で必要 | NodeId + indexed + indexVar の構造体 |
| SourceRef | clickable region の特定に必要 | enum { NodeRef, ExprRef, SlotRef, ConstraintRef } |

### 5.2 API の不足

| 不足API | 用途 | 現行で代替可能か |
|--------|------|----------------|
| `project_expr_slot_candidates(SlotId)` | 式スロットの候補取得 | `hole_candidates` で部分代替 |
| `project_expr_actions(ExprId)` | 式ノードのwrap/replace候補 | 未実装 |
| `project_constraint_target_candidates(kind)` | 制約 target の候補 | 未実装 |
| `apply_action(ApplyTemplate)` | テンプレート一括適用 | 複数 Action を順次呼び出しで代替可能 |
| `project_slot_render(SlotId)` | スロットの現在値表示 | `inspect` で部分取得可能 |

### 5.3 UI実装上の不足

| 不足項目 | 説明 |
|---------|------|
| drag & drop | ノード並び替えの UI が未設計 |
| undo/redo UI | Header に配置予定だが詳細未定 |
| キーボードショートカット | Tab でスロット移動、Enter で確定、Esc でキャンセル |
| エラー表示 | 式スロットに不正値が入った場合のインライン表示 |

---

## 6. 他Agentに渡すべき論点

### 6.1 → Domain Model Agent

1. **ExprId / SlotId の型設計を確定してほしい**
   - Expression は arena-based か、インライン埋め込みか
   - SlotId は NodeId + slot_name の複合か、独立IDか

2. **PlaceRef の構造を確定してほしい**
   - `A[i]` を表す PlaceRef: `{ nodeId: NodeId, indexed: bool, indexVar: Ident }`
   - 多次元 `A[i][j]` への拡張性

3. **ApplyTemplate Action は単一か複数バッチか**
   - 単一: undo が1ステップ、内部で複数操作を実行
   - バッチ: 呼び出し側で複数 Action を送る、undo は複数ステップ

### 6.2 → wasm Boundary Agent

1. **Projection API の拡張を実装してほしい**
   - `project_expr_slot_candidates`
   - `project_expr_actions`
   - `project_constraint_target_candidates`

2. **click時API呼び出しのレイテンシ目標**
   - 10ms以下を想定しているが、候補数が100を超える場合の対策

3. **差分更新の必要性**
   - 現状は全再描画を想定
   - ノード数1000を超える場合の最適化検討

### 6.3 → Critical Reviewer Agent

1. **PendingExprAction が複雑な式で破綻しないか検証してほしい**
   - 例: `max(N-1, M/2)` のような入れ子式の構築フロー
   - 深さ3以上の式編集は現実的に発生するか

2. **DraftConstraint の制約種別が足りているか**
   - MVP では Range, TypeDecl, LengthRelation, Relation, Distinct, Sorted, Property
   - SumBound, CharSet, StringLength は後回しで問題ないか

3. **テンプレートの粒度は適切か**
   - 「辺リスト」と「辺リスト（木）」を分けるべきか
   - 「グリッド（文字）」と「グリッド（数値）」を分けるべきか

### 6.4 → Real Problem Coverage Agent

1. **操作フローで想定外のパターンがないか**
   - 本レポートの基本ケース7種で頻出問題の90%をカバーできるか
   - 特殊な入力構造（三角行列、対称行列など）の頻度

2. **テンプレートの必要十分性**
   - 「辺リスト」「グリッド」「複数テストケース」「クエリ列」で足りるか
   - 「対称グラフ入力」「重み付き辺」などの追加が必要か

---

## 7. Q-5: 式スロット設計の具体化

plan.md §32 の式スロット契約を具体化する。

### 7.1 SlotId の設計

```typescript
type SlotId = `${NodeId}:${SlotName}`;
// 例: "42:length", "73:lower", "73:upper"

interface SlotInfo {
  slotId: SlotId;
  nodeId: NodeId;
  slotName: string;       // "length", "lower", "upper", "count", "lhs", "rhs"
  label: string;          // 表示用: "長さ", "下限", "上限"
  expectedSort: ExpectedSort;
  slotRule: SlotRule[];
  currentExpr: Expression | null;
}
```

### 7.2 ExpectedSort

```typescript
type ExpectedSort =
  | 'IntExpr'     // 整数値を返す式
  | 'BoolExpr'    // 真偽値を返す式（Relation用）
  | 'PlaceExpr';  // 場所参照（制約target用）
```

### 7.3 SlotRule

```typescript
type SlotRule =
  | 'NonNegative'           // 0以上の値
  | 'Positive'              // 1以上の値
  | 'NoSelfReference'       // 自己参照禁止（配列長に自身を使えない）
  | 'IntegerOnly'           // 整数定数のみ（Relation等では変数OK）
  | 'AllowIndexRef'         // A[i] のようなインデックス参照を許可
  | 'ComparableWithTarget'; // target と比較可能な型
```

### 7.4 スロット種別ごとの設定

| スロット | ExpectedSort | SlotRule |
|---------|-------------|----------|
| Array.length | IntExpr | NonNegative, NoSelfReference |
| Matrix.rows | IntExpr | Positive, NoSelfReference |
| Matrix.cols | IntExpr | Positive, NoSelfReference |
| Repeat.count | IntExpr | NonNegative |
| Range.lower | IntExpr | なし |
| Range.upper | IntExpr | ComparableWithTarget |
| Relation.lhs | IntExpr | AllowIndexRef |
| Relation.rhs | IntExpr | AllowIndexRef |
| LengthRelation.length | IntExpr | NonNegative |

---

## 8. まとめ

本レポートでは、GUI Interaction Agent として以下を明確化した。

1. **画面レイアウト**: 3カラム+下部パネルの具体的な領域責務
2. **コンポーネント階層**: Preact コンポーネントの構造と責務分離
3. **基本ケース7種の操作フロー**: Single Scalar, Array, Tuple, Grid, Edge List, Multi-testcase, Query with Choice
4. **EditorState 型定義**: 選択状態、PendingExprAction、DraftConstraint の具体構造
5. **Render時/Click時データ分離**: 効率的なProjection利用
6. **PendingExprAction UI**: 式編集の3ステップフロー
7. **DraftConstraint フォーム**: 制約種別ごとの必須項目と完了条件
8. **式スロット設計**: SlotId, ExpectedSort, SlotRule の具体化

現行方針（plan.md の契約）は GUI 操作体験の観点から**支持**する。
Expression partial を AST に入れず PendingExprAction で管理する設計は**十分**である。
DraftConstraint を EditorState に逃がす設計は**適切**である。
Click時候補列挙は**十分**であり、パフォーマンス問題は発生しない。

不足点として、ExprId/SlotId/PlaceRef の型定義、Projection API の拡張、テンプレート展開の Action 設計が必要である。これらは Domain Model Agent および wasm Boundary Agent と協議して確定する。
