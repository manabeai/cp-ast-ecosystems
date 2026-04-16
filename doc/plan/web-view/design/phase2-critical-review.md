# Phase 2: Critical Reviewer Agent Report

本文書は、Editor UI設計に対する批判的レビューを記録する。

---

## 1. 前提

### 1.1 レビュー対象

| 文書 | 行数 | 評価焦点 |
|------|------|---------|
| plan.md | 1793行 | 設計思想・契約仕様の妥当性 |
| phase2-domain-model.md | ~600行 | 型設計の過不足 |
| phase2-gui-interaction.md | ~800行 | 操作フローの実現可能性 |
| phase2-real-problem-coverage.md | ~700行 | カバレッジの偏り・漏れ |
| phase2-wasm-boundary.md | ~950行 | API粒度・性能リスク |

### 1.2 レビュー基準

本レビューは以下の観点で批評を行う：

1. **過剰設計**: 複雑性に見合う価値がない設計
2. **過少設計**: 後で必ず問題になる手抜き
3. **リスク**: 実装時に破綻する可能性が高い箇所
4. **MVPスコープ**: 初回リリースで本当に必要か

---

## 2. リスク一覧

| ID | Risk | Severity | Likelihood | Mitigation |
|-----|------|----------|------------|------------|
| R-1 | ExprId導入の実装コストがplan.mdで過小評価されている。現行Expressionはvalue型でありArenaで管理されていないため、部分式選択のためにExpression全体のデータ構造変更が必要 | High | High | MVP Phase 1ではExprId未導入で開始し、式スロットは「全置換」のみに限定する |
| R-2 | PendingExprAction の複雑なフェーズ遷移（wrapBinary, wrapCall等）がUIバグの温床になる | Medium | High | フェーズ数を最小化（2フェーズ以内）。wrapCallは MVP後回し |
| R-3 | JSON roundtrip による状態管理がundo/redoと相性が悪い。plan.md §28.3で「undo/redoの正本ではない」と書いているが、結局どう実装するか未定義 | High | High | MVP ではundo/redo非対応を明示。後続で Command pattern 検討 |
| R-4 | 3カラムレイアウトがモバイル/狭画面で破綻する | Low | Medium | MVP はデスクトップのみ。狭画面対応は MVP後 |
| R-5 | project_structure_outline の計算量が O(n) でノード数増加時にUI遅延 | Medium | Low | 現実の競プロ入力で n>100 は稀。問題発生時にpagination検討 |
| R-6 | DraftConstraint の種別ごとのフォーム実装が12種類×各フィールドで膨れ上がる | Medium | High | MVP は Range, TypeDecl, LengthRelation, Relation の4種に限定 |
| R-7 | テンプレート展開（辺リスト、グリッド等）の複数Action atomicity が未検討。途中失敗時のロールバックがない | High | Medium | テンプレートは「単一 Action で全展開」として実装。部分適用を許さない |
| R-8 | Coverage Agent の36問が ABC/ARC に偏り、Codeforces や特殊形式（interactive）を含まない | Medium | Medium | MVP は AtCoder バッチ問題のみ対象と明示。偏りは許容 |
| R-9 | 下三角行列（Problem 11）の「非対応」が放置されている。頻出ではないが、対応できない理由がRepeat設計の根本問題を示唆 | Medium | Low | MVP後回し。将来的にはRepeat.index_varを式から参照可能にする拡張が必要 |
| R-10 | project_expr_actions の「クリック時遅延取得」がユーザー体感遅延につながる可能性 | Low | Low | 実測で10ms以下であれば許容。超過時はプリフェッチ検討 |

---

## 3. 過剰設計の指摘

### 3.1 Projection 3分割（render/action/global）

**問題**: plan.md §29 で render/action/global の3分割を定義しているが、この分離が本当に必要か疑問。

**根拠**:
- 現行 cp-ast-core の ProjectionAPI は7メソッドで十分機能している
- Domain Model Agent は約20個の新Projection関数を提案しているが、これは過剰
- 特に `project_node_render` と `project_structure_outline` の責務分離は曖昧

**具体例**:
```rust
// Domain Model提案: 4つの関数
project_structure_outline() -> StructureOutlineView
project_node_render(node_id) -> NodeRenderView
project_slot_render(slot_id) -> SlotRenderView
project_diagnostics() -> DiagnosticsView

// 実際に必要なもの: 2つで十分
project_outline_with_diagnostics() -> FullOutlineView  // 一括取得
project_node_detail(node_id) -> NodeDetailView         // 選択時のみ
```

**推奨**: MVP では render/action/global の分離を緩め、5-7個の関数に絞る。

### 3.2 EditorState の型定義が細かすぎる

**問題**: GUI Interaction Agent の EditorState 定義が過度に詳細。

```typescript
// 提案されている型
interface EditorState {
  selectedNodeId: NodeId | null;
  selectedConstraintId: ConstraintId | null;
  focusedSlotId: SlotId | null;  // ← 本当に必要？
  openPopover: PopoverState | null;
  openModal: ModalState | null;
  pendingExprAction: PendingExprAction | null;
  draftConstraint: DraftConstraint | null;
  bottomPanelTab: 'sample' | 'preview' | 'diagnostics' | 'log';
  structureTreeExpanded: Set<NodeId>;  // ← MVP で必要？
}
```

**根拠**:
- `focusedSlotId` は `selectedNodeId` + ローカル状態で十分
- `structureTreeExpanded` はツリー展開状態だが、初期は全展開で十分
- `bottomPanelTab` は React state で十分

**推奨**: MVP では EditorState を以下に絞る：
```typescript
interface EditorState {
  documentJson: string;           // 正本
  selectedNodeId: NodeId | null;
  pendingExprAction: PendingExprAction | null;
  draftConstraint: DraftConstraint | null;
}
```

### 3.3 SlotId の過剰設計

**問題**: Domain Model Agent の `SlotId` 定義が複雑。

```rust
pub struct SlotId {
    pub owner: NodeId,
    pub slot_name: String,  // ← String は過剰
    pub index: Option<usize>,
}
```

**根拠**:
- slot_name は実質 `"length"`, `"lower"`, `"upper"`, `"count"` 等の固定セット
- String ではなく enum にすべき
- `index` が必要になるケースは MVP では発生しない

**推奨**:
```rust
pub enum SlotKind {
    ArrayLength,
    RepeatCount,
    RangeLower,
    RangeUpper,
    RelationLhs,
    RelationRhs,
    // ...
}

pub struct SlotId {
    pub owner: NodeId,
    pub kind: SlotKind,
}
```

### 3.4 ExprSort/SlotRule の早期設計

**問題**: plan.md §32.3-32.4 で `ExprSort`（IntExpr, BoolExpr, PlaceExpr）と `SlotRule`（NonNegative, NoSelfReference 等）を定義しているが、MVP で使わない。

**根拠**:
- MVP の式スロットは全て IntExpr 想定
- BoolExpr は Relation 制約の左右辺のみで、これは MVP では式スロット扱いしない
- SlotRule の検証は sample generation 側で行えばよい

**推奨**: MVP では ExprSort = IntExpr 固定、SlotRule 未実装。

---

## 4. 過少設計の指摘

### 4.1 エラーハンドリングの具体設計がない

**問題**: plan.md/wasm Boundary Agent ともに「エラーは Result で返す」と書いているが、具体的なエラーケースの列挙がない。

**欠落しているエラーケース**:
1. 参照先ノードが削除された時の LengthRelation の挙動
2. 循環参照（A.length = Ref(B), B.length = Ref(A)）の検出
3. 制約追加時の型不整合（TypeDecl(Int) だが CharSet を追加）
4. sample generation 失敗時の診断情報

**推奨**: OperationError の完全列挙を設計に追加：
```rust
pub enum OperationError {
    NodeNotFound(NodeId),
    SlotAlreadyOccupied { node: NodeId, slot: SlotKind },
    CircularReference { path: Vec<NodeId> },
    TypeConflict { existing: TypeDecl, conflicting: Constraint },
    InvalidFill { target: NodeId, reason: String },
    // ...
}
```

### 4.2 キーボードナビゲーションの未設計

**問題**: GUI Interaction Agent はクリックベースの操作フローのみ記述。キーボード操作の設計がない。

**欠落しているケース**:
- Tab でフォーカス移動
- Enter で確定、Escape でキャンセル
- 矢印キーでツリーナビゲーション
- Ctrl+Z/Ctrl+Y (undo/redo は MVP 外だが、キーバインドだけでも)

**推奨**: MVP でもフォーカス管理と Enter/Escape は実装必須。Tab order を設計に追加。

### 4.3 名前衝突の検出がない

**問題**: 同名の変数が作成された時の挙動が未定義。

**具体例**:
1. Scalar(N) を追加後、別の Scalar(N) を追加 → どうなる？
2. Repeat 内で index_var "i" を使用中に、外部で Scalar "i" を追加

**推奨**: 
- 名前衝突時は警告（diagnostics）を出すが、許容する
- Reference 解決は「最も近いスコープ」を優先
- plan.md §21 の diagnostics にこれを追加

### 4.4 コピー&ペースト

**問題**: 構造のコピー&ペーストの設計がない。

**根拠**: 類似構造の繰り返し入力（例: 2つの配列 A, B に同じ制約）でユーザーはコピペを期待する。

**推奨**: MVP 後回しを明示。ただし「制約の複製」は MVP でも欲しい機能。

### 4.5 空の Sequence/Tuple の扱い

**問題**: Sequence や Tuple の子が0個の場合の挙動が未定義。

**疑問**:
- 空の Sequence は許容するか？
- Tuple(1要素) と Scalar は同一視するか？
- Repeat(0回) は許容するか？

**推奨**: 以下を不変条件に追加：
- Sequence は 0 要素を許容（root が空の初期状態）
- Tuple は 1 要素以上必須
- Repeat は count >= 0 を許容（生成時は0なら出力なし）

---

## 5. MVPで切るべきもの

| 機能 | 切る理由 | 残すリスク |
|------|---------|-----------|
| ExprId（部分式選択） | Expression のデータ構造変更が大規模。式スロットは「全置換」で十分 | 「N を N/2 に変更」の操作が「スロット全体を再入力」になるが許容可能 |
| wrapCall（max, min 等） | フェーズ遷移が複雑。/x, +x で十分カバーできる | max(N, M) は「(N > M ? N : M)」相当を直接入力で代替 |
| undo/redo | 状態管理の複雑化。JSON roundtrip との相性が悪い | ブラウザの「戻る」で致命的になる可能性あり。説明文で警告 |
| drag & drop（構造並べ替え） | 複雑かつバグりやすい。手動削除→再追加で代替可能 | 操作ステップ増加だが MVP では許容 |
| Section, Choice | 頻度が低い。Sequence + Repeat で大半カバー | クエリ列問題が作りにくくなる → MVP Phase 2 で対応 |
| Distinct, Sorted, Property, SumBound, Guarantee, CharSet, StringLength 制約 | 12種のうち4種（Range, TypeDecl, LengthRelation, Relation）で基本ケースをカバー | 木入力で Property(Tree) がないと不完全だが、Guarantee で代用可能 |
| 高レベルテンプレート（辺リスト、グリッド、クエリ列） | 実装コストが高い。手動構築でも作成可能 | UX が大幅に劣化するが、MVP では許容。Phase 2 で追加 |
| モバイル対応 | 3カラムレイアウトがそもそも合わない | デスクトップのみと明示 |
| sample 複数生成・比較 | 単一生成で十分 | 1クリック1生成で MVP 十分 |
| Bottom Panel の Log タブ | デバッグ用であり必須ではない | 開発時は console.log で代替 |

### MVP Phase 1 スコープ（最小）

**Structure**: Scalar, Array, Tuple, Sequence, Repeat, Hole
**Constraint**: Range, TypeDecl, LengthRelation, Relation
**Expression**: Lit, Var, BinOp（+, -, *, /）
**Operation**: FillHole, ReplaceNode, AddConstraint, RemoveConstraint, AddSlotElement, RemoveSlotElement
**UI**: 3カラム基本レイアウト、式スロット全置換、DraftConstraint 4種

**切る**: ExprId, wrapCall, undo/redo, drag&drop, Section, Choice, 8種のConstraint, 高レベルテンプレート

---

## 6. 代替案

### 6.1 Projection分離の代替

**現行設計**: render/action/global の3分割、約20関数

**代替案**: 2分割、5-7関数

```rust
// === 描画 + 診断 (document 更新時に呼ぶ) ===
fn project_full(document: &str) -> FullProjection;
// 返り値: { outline, diagnostics, completeness }

fn project_node_detail(document: &str, node_id: &str) -> NodeDetail;
// 返り値: { slots, relatedConstraints, actions }

// === 候補取得 (click 時に呼ぶ) ===
fn get_hole_candidates(document: &str, hole_id: &str) -> Vec<Candidate>;
fn get_expr_candidates(document: &str, parent_id: &str, slot_kind: &str) -> Vec<ExprCandidate>;
fn get_constraint_targets(document: &str, constraint_kind: &str) -> Vec<PlaceRef>;

// === 操作 ===
fn apply_action(document: &str, action: &str) -> ApplyResult;

// === サンプル ===
fn generate_sample(document: &str, seed: u32) -> String;
```

**利点**: 
- 関数数が約1/3に削減
- 呼び出しパターンがシンプル
- render/action の境界があいまいにならない

### 6.2 EditorState の代替

**現行設計**: 10+ フィールドの複雑な EditorState

**代替案**: 最小 EditorState + ローカル状態

```typescript
// グローバル状態（Preact signal）
const documentJson = signal<string | null>(null);
const selectedNodeId = signal<NodeId | null>(null);

// ローカル状態（コンポーネント内 useState）
// - openPopover: Popover コンポーネント内
// - draftConstraint: DraftConstraintPanel コンポーネント内
// - pendingExprAction: ExprSlotField コンポーネント内

// computed projection
const fullProjection = computed(() => {
  if (!documentJson.value) return null;
  return JSON.parse(project_full(documentJson.value));
});
```

**利点**:
- グローバル状態が2つだけで済む
- 各コンポーネントが自分の状態を管理
- テストしやすい

### 6.3 式スロット編集の代替

**現行設計**: ExprId + PendingExprAction + wrapBinary/wrapCall

**代替案**: テキスト入力 + パース + バリデーション

```typescript
// 式スロットに「N/2」と直接入力
<ExprSlotField 
  value={currentExpr}  // "N/2"
  onSubmit={(text) => {
    const parseResult = parseExpr(text, availableRefs);
    if (parseResult.ok) {
      applyAction({ kind: "SetExpr", slot, expr: parseResult.expr });
    } else {
      showError(parseResult.error);
    }
  }}
/>
```

**利点**:
- PendingExprAction の複雑なフェーズ遷移が不要
- ExprId 導入が不要
- ユーザーは馴染みのある「テキスト入力」で式を書ける

**欠点**:
- パース実装が必要（ただし単純な算術式なら容易）
- 参照候補の補完 UI が別途必要

**推奨**: MVP ではテキスト入力を主、候補選択を補助とする。

### 6.4 DraftConstraint の代替

**現行設計**: kind ごとに異なるフォーム（12種 × 各フィールド）

**代替案**: 共通フォーム + 動的フィールド

```typescript
interface GenericDraftConstraint {
  kind: ConstraintKind;
  fields: Map<string, FieldValue | null>;
}

// フォームは kind に応じて必要フィールドを表示
function ConstraintForm({ draft }: { draft: GenericDraftConstraint }) {
  const schema = CONSTRAINT_SCHEMAS[draft.kind];
  return (
    <form>
      {schema.fields.map(field => (
        <FieldEditor key={field.name} field={field} value={draft.fields.get(field.name)} />
      ))}
    </form>
  );
}
```

**利点**:
- 新しい Constraint 種別追加時にフォームコードを変更不要
- コード量削減

**欠点**:
- 型安全性が下がる
- 特殊なフィールド（PlaceRef 選択等）の扱いが複雑

---

## 7. 他Agent報告への批評

### 7.1 Domain Model Agent 批評

**型が多すぎないか**: **多すぎる**

| 提案された型 | 必要性 |
|-------------|-------|
| ExprId | MVP不要。式スロット全置換で十分 |
| SlotId | 必要だが String → enum に簡略化すべき |
| SourceRef | 必要 |
| ExprSort | MVP不要。IntExpr 固定 |
| SlotRule | MVP不要 |
| InlineNode | 過剰。単純な String でよい |

**不足はないか**: **ある**

- `ReferenceScope`: 参照可能な変数の範囲を定義する型がない
- `ValidationContext`: 検証コンテキストの型がない
- `OperationBatch`: 複数 Action の原子性保証に必要

**総評**: 方向性は正しいが、MVP には過剰。半分に削るべき。

### 7.2 GUI Interaction Agent 批評

**操作フローに無理はないか**: **一部あり**

| フロー | 問題点 |
|-------|--------|
| Scalar 追加 | 自然。問題なし |
| Array 追加 | 長さスロット設定が2ステップ（クリック→選択）で冗長。1ステップ（ドロップダウン選択）で十分 |
| 式編集 (N → N/2) | PendingExprAction のフェーズ遷移が複雑。テキスト入力の方がシンプル |
| 制約追加 | DraftConstraint の「追加ボタンが全必須項目埋まるまで無効」はUXが悪い。部分入力で追加→後から編集のほうが自然 |
| テンプレート適用 | モーダルでのフィールド入力が多すぎ。プリセット選択→ワンクリック適用のほうがよい |

**総評**: クリックベースの設計は妥当だが、ステップ数削減の余地あり。

### 7.3 Coverage Agent 批評

**36問で十分か**: **不十分だがMVPとしては許容**

| カテゴリ | 問題数 | 評価 |
|---------|--------|-----|
| 単一値 | 2 | 十分 |
| 複数値 | 2 | 十分 |
| 配列 | 3 | 十分 |
| グリッド | 4 | 十分 |
| 木 | 2 | やや少ない |
| 一般グラフ | 2 | 十分 |
| 辺リスト | 3 | 十分 |
| クエリ列 | 3 | 十分 |
| 複数テストケース | 3 | 十分 |
| 総和制約 | 2 | 十分 |
| sorted/distinct | 2 | 十分 |
| 式付き境界 | 3 | 十分 |
| choice/variant | 3 | 十分 |

**バイアスはないか**: **ある**

1. **AtCoder 偏り**: 全問が AtCoder。Codeforces, USACO, AOJ なし
2. **難易度偏り**: ABC 中心。ARC/AGC の複雑な入力形式が少ない
3. **interactive 問題なし**: MVP 対象外とはいえ、完全に欠落
4. **文字列問題が少ない**: CharSet, StringLength の検証が不十分

**非対応問題**:
- Problem 11（下三角行列）: **重大な設計欠陥を示唆**
- Problem 13（各行可変長）: **部分対応だが workaround が複雑**

**総評**: MVP としては十分だが、「非対応」の根本原因（Repeat.index_var の参照不可）は将来対応が必要。

### 7.4 wasm Boundary Agent 批評

**API粒度は適切か**: **やや細かすぎる**

| 提案 | 評価 |
|------|------|
| Render Projection 6関数 | 多い。3関数に統合可能 |
| Action Projection 5関数 | 妥当 |
| Operation 2関数 | 妥当 |
| Global Projection 2関数 | Render に統合可能 |
| Sample 2関数 | 妥当 |

**パフォーマンスリスク評価**: **概ね妥当だが楽観的**

- JSON roundtrip の「100KB で 2-5ms」は環境依存。低スペック端末では10ms+
- computed signal の「document変更時のみ再計算」は、頻繁な変更時に cascade する

**Error handling**: **不十分**

- `WasmOperationError` の列挙は良いが、**回復可能/不可能の分類がない**
- ユーザーに見せるメッセージと内部エラーの分離がない

**総評**: 方向性は正しいが、API 統合と Error 分類の追加が必要。

---

## 8. 具体例: 設計が破綻するシナリオ

### 8.1 Scenario 1: 下三角行列（Problem 11）

**入力形式**:
```
N
A_{1,1}
A_{2,1} A_{2,2}
...
A_{N,1} ... A_{N,N}
```

**問題点**: 行 i に i 個の要素がある構造を現行 AST で表現できない。

**現行設計での試み**:
```
Sequence(
  Scalar(N),
  Repeat(count=N, index_var=i, body=???)
)
```

- `body` が `Array(length=i)` となるべきだが、`i` は Expression で参照できない
- 現行 `Reference::VariableRef(NodeId)` は「NodeId で特定されたノードの値」しか参照できない
- `index_var` は String であり、NodeId を持たない

**根本原因**: Repeat.index_var がスコープ変数として Expression から参照可能でない設計。

**影響範囲**: 下三角行列だけでなく、「行 i に 2i+1 個」のような可変長行全般が非対応。

**緩和策**: 
- MVP では「非対応」と明示
- 将来的には `index_var` に NodeId を割り当てるか、`Expression::IndexVar(String)` を追加

### 8.2 Scenario 2: 式編集のキャンセルが期待と異なる

**シナリオ**:
1. 配列 A の長さを N/2 に設定したい
2. 長さスロットをクリック → N を選択 → 「/x」を選択
3. この時点で「やっぱりやめた」と思いキャンセル

**現行設計での動作**:
- PendingExprAction がクリアされる
- AST の長さスロットは N のまま（最初に選択した値）

**ユーザーの期待**:
- 「長さスロットをクリック」する前の状態に戻る
- しかし、その前の状態は Hole だったかもしれないし、別の式だったかもしれない

**問題点**: 
- 「キャンセル」の意味が曖昧
- PendingExprAction 開始時に「元の状態」を保存していない

**緩和策**:
- PendingExprAction に `originalExpr: Expression | null` を追加
- キャンセル時に originalExpr を復元
- または、式スロットの「最初の選択」を Operation とせず、PendingExprAction の開始トリガーとする

### 8.3 Scenario 3: テンプレート適用の途中失敗

**シナリオ**:
1. 「辺リストテンプレート」を選択
2. 頂点数 N、辺数 M、Tree 性質を設定
3. 「適用」をクリック

**テンプレート展開が行う操作**:
1. Tuple(N, M) を追加 ← 成功
2. Repeat(M, Tuple(u, v)) を追加 ← 成功
3. Range(N, 1, 10^5) を追加 ← 成功
4. Range(M, 1, N-1) を追加 ← **失敗（N-1 の式構築に失敗）**
5. Property(Tree) を追加 ← 実行されない

**現行設計での動作**:
- 1-3 は AST に反映済み
- 4 で失敗
- 中途半端な状態が残る

**問題点**: 
- テンプレート適用の atomicity がない
- 部分適用状態のロールバックがない

**緩和策**:
- テンプレートを「単一 Action」として実装
- `Action::ApplyTemplate { kind, config }` を追加
- wasm 側で展開→検証→全適用 or 全却下
- **これは plan.md に記載がなく、設計の漏れ**

---

## 9. 総合評価

### 9.1 現行設計の成熟度: **3/5**

**良い点**:
- AST を正本とする思想は一貫
- Hole ファーストの設計は Hazelnut の知見を活かしている
- 未完成状態の配置（Structure→AST, Expr/Constraint→EditorState）は妥当

**悪い点**:
- ExprId/SlotId 等の詳細設計が実装コストを過小評価
- エラーハンドリングの具体設計が欠落
- テンプレート適用の atomicity 未検討
- Repeat.index_var の参照問題が放置

### 9.2 MVP実現可能性: **4/5**

**実現可能な理由**:
- 基本ケース（Scalar, Array, Tuple, Repeat + 基本制約）の設計は堅実
- wasm 境界の JSON roundtrip は既存実装で実績あり
- Coverage の 36 問中 31 問は完全対応

**リスク要因**:
- ExprId を MVP に入れると工数爆発
- PendingExprAction の複雑さがバグを招く可能性
- undo/redo 不在がユーザー体験を大きく損なう

### 9.3 最大のリスク1つ

**ExprId の実装コスト**

plan.md と Domain Model Agent が提案する「ExprId による部分式選択」は、現行 Expression のデータ構造を根本から変更する必要がある。

現行:
```rust
pub enum Expression {
    Lit(Literal),
    Var(Reference),
    BinOp { op: ArithOp, lhs: Box<Expression>, rhs: Box<Expression> },
    Pow { base: Box<Expression>, exp: Box<Expression> },
    FnCall { func: FuncName, args: Vec<Expression> },
}
```

ExprId 導入後（最小変更案）:
```rust
pub struct ExprNode {
    pub id: ExprId,
    pub kind: ExprKind,
}

pub enum ExprKind {
    Lit(Literal),
    Var(Reference),
    BinOp { op: ArithOp, lhs: Box<ExprNode>, rhs: Box<ExprNode> },
    // ...
}
```

**影響範囲**:
- cp-ast-core の Expression 関連コード全般
- cp-ast-json のシリアライゼーション
- cp-ast-tree の式表示
- sample generation の式評価
- render / render_tex の式出力

**推奨**: MVP Phase 1 では ExprId を導入せず、式スロットは「全置換」で対応。
