# Phase 1: 前提固定

本文書は、Editor UI設計の全Subagentが共有する前提を固定するものである。

---

## 1. 固定前提一覧

### 1.1 アーキテクチャ前提

| # | 前提 | 根拠 |
|---|------|------|
| A-1 | ASTは1種類。StructureAST + ConstraintAST + Expressionで構成 | plan.md §27.1 |
| A-2 | ASTが唯一の正本（canonical rendering, sample generation, validation, projectionの入力） | plan.md §27.2 |
| A-3 | UIはASTの投影（projectional editor） | main.md 設計原則1, plan.md §1 |
| A-4 | wasm(Rust)は意味論、Preact(TS)は体験を担う | plan.md §9.1 |
| A-5 | 操作はOperation APIを通じてASTを更新（Projection APIはread-only） | plan.md §29.1, §30.1 |
| A-6 | Arena-basedのID管理（NodeId, ConstraintId）でstable identity | main.md, cp-ast-core実装 |
| A-7 | Holeは第一級市民（未完成でもシステムは壊れない） | main.md 設計原則4 |

### 1.2 未完成状態の配置契約

| # | 前提 | 根拠 |
|---|------|------|
| B-1 | Structure未完成 → ASTのNodeKind::Hole | plan.md §26.2 |
| B-2 | Expression未完成 → EditorState/PendingExprAction（ASTに部分式を入れない） | plan.md §26.3 |
| B-3 | Constraint未完成 → EditorState/DraftConstraint（ASTに未完成制約を入れない） | plan.md §26.4 |

### 1.3 Projection分離

| # | 前提 | 根拠 |
|---|------|------|
| C-1 | Render Projection: 描画最小情報のみ返す | plan.md §29.2 |
| C-2 | Action Projection: click時に候補・操作を遅延取得 | plan.md §29.3 |
| C-3 | Global Projection: 全体一覧・サマリ | plan.md §29.4 |
| C-4 | render時に候補全列挙しない（click時遅延取得が基本） | plan.md §9.2 |

### 1.4 UI方針

| # | 前提 | 根拠 |
|---|------|------|
| D-1 | 基本ケース優先（scalar, array, grid, edge list, multi-testcase, query） | process.md §2.3 |
| D-2 | template-driven編集（raw ASTより高レベルブロック操作を見せる） | plan.md §8, §20 |
| D-3 | 3カラム+下部パネルレイアウト（Structure / Detail / Constraints + Bottom） | plan.md §14.1 |
| D-4 | 式スロットは統一設計（expected_sort, scope, slot_rule） | plan.md §32 |

### 1.5 技術制約

| # | 前提 | 根拠 |
|---|------|------|
| E-1 | Rust 2021 edition, 1.75+ | AGENTS.md |
| E-2 | cp-ast-core → wasm-packでwasm32ビルド | 既存cp-ast-wasm crate |
| E-3 | Preact + Vite + TypeScript | 既存web/ディレクトリ |
| E-4 | wasm ↔ JS間はJSON string（cp-ast-json経由） | 既存cp-ast-wasm/src/lib.rs |
| E-5 | IDはdecimal string（JS 53bit精度問題回避） | cp-ast-json DTO設計 |

---

## 2. 既存cp-ast-core型・API棚卸し

### 2.1 Structure types

```
NodeKind: Scalar, Array, Matrix, Tuple, Repeat, Section, Sequence, Choice, Hole
NodeId: u64ベースstable ID
Reference: VariableRef(NodeId), IndexedRef{target, indices}, Unresolved(Ident)
Ident: 名前（String wrapper）
Literal: IntLit(i64), StrLit(String)
NodeKindHint: AnyScalar, AnyArray, AnyMatrix, AnyTuple, AnyRepeat, AnySection, AnyChoice, Any
StructureAst: arena-based, add_node/get/get_mut/remove/root/set_root/contains/len/iter
```

### 2.2 Constraint types

```
Constraint: Range, TypeDecl, LengthRelation, Relation, Distinct, Property, SumBound, Sorted, Guarantee, CharSet, StringLength, RenderHint (12 variants)
ConstraintId: u64ベースstable ID
ConstraintSet: arena-based, add/get/remove/for_node/global/len/iter
ExpectedType: Int, Str, Char
Expression: Lit, Var, BinOp, Pow, FnCall
ArithOp: Add, Sub, Mul, Div
RelationOp: Lt, Le, Gt, Ge, Eq, Ne
SortOrder: Ascending, NonDecreasing, Descending, NonIncreasing
DistinctUnit: Element, Tuple
PropertyTag: Simple, Connected, Tree, Permutation, Binary, Odd, Even, Custom(String)
CharSetSpec: LowerAlpha, UpperAlpha, Alpha, Digit, AlphaNumeric, Custom(Vec<char>), Range(char, char)
RenderHintKind: Separator(Separator)
Separator: Space, None
```

### 2.3 Operation types

```
AstEngine: { structure: StructureAst, constraints: ConstraintSet }
  - apply(action) -> Result<ApplyResult, OperationError>
  - preview(action) -> Result<PreviewResult, OperationError>

Action: FillHole, ReplaceNode, AddConstraint, RemoveConstraint, IntroduceMultiTestCase, AddSlotElement, RemoveSlotElement
FillContent: Scalar{name}, Array{name,length}, Matrix{name,rows,cols}, Tuple, Repeat{count,index_var}, Section, Sequence, Choice{tag}
```

### 2.4 Projection types (現行)

```
ProjectionAPI trait:
  - nodes() -> Vec<ProjectedNode>
  - children(node) -> Vec<SlotEntry>
  - inspect(node) -> Option<NodeDetail>
  - hole_candidates(hole) -> Vec<CandidateKind>
  - available_actions() -> Vec<AvailableAction>
  - why_not_editable(node) -> Option<NotEditableReason>
  - completeness() -> CompletenessSummary

ProjectedNode: { id, label, depth, is_hole }
CandidateKind: IntroduceScalar, IntroduceArray, IntroduceMatrix, IntroduceSection
```

### 2.5 既存派生物

```
Render: render_input_format, render_constraints_text
Render TeX: render_input_tex, render_constraints_tex, render_full_tex
Tree Viewer: cp-ast-tree crate（ASCII tree renderer）
JSON: cp-ast-json crate（lossless roundtrip）
Sample: dependency graph analysis + constraint-based random generation
wasm: cp-ast-wasm crate（10 wasm-bindgen functions）
```

---

## 3. 未確定事項一覧

| # | 事項 | 現状 | 決定すべきAgent |
|---|------|------|----------------|
| U-1 | ExprId / SlotIdの具体的な型設計 | plan.mdで言及あるが未実装 | Domain Model |
| U-2 | SourceRefの具体実装 | plan.md §35で概念定義のみ | Domain Model + GUI |
| U-3 | EditorActionの完全列挙 | plan.md §17.2で素案 | Domain Model + GUI |
| U-4 | Projection API拡張（現行は基本のみ） | 7メソッドのみ | Domain Model + wasm |
| U-5 | PendingExprActionのフェーズ管理詳細 | plan.md §33で概念のみ | GUI Interaction |
| U-6 | DraftConstraintの完了条件ロジック | plan.md §34で概念のみ | GUI Interaction |
| U-7 | 高レベルテンプレートの具体実装方式 | plan.md §20でリストのみ | GUI + Domain Model |
| U-8 | diagnostics計算のタイミングとコスト | plan.md §21で概念のみ | wasm Boundary |
| U-9 | sample generationのeditor連携詳細 | plan.md §22で概念のみ | Sample Generation |
| U-10 | undo/redo戦略（MVPスコープ） | plan.md §11後回し候補 | Critical Reviewer |
| U-11 | diff-based更新 vs full再描画 | 未検討 | wasm Boundary |

---

## 4. 主要論点一覧（process.md §8ベース）

| # | 論点 | 主担当 | 必ず確認するAgent |
|---|------|--------|------------------|
| Q-1 | ASTは1種類でよいか | Domain Model | Critical Reviewer |
| Q-2 | Structure Holeの役割（十分か？不足か？） | Domain Model | GUI Interaction |
| Q-3 | Expr partialを持たない設計で十分か | GUI Interaction | Critical Reviewer |
| Q-4 | Constraint draftをEditorStateに逃がしてよいか | GUI Interaction | Domain Model |
| Q-5 | 式スロット設計の具体化 | GUI Interaction | Domain Model |
| Q-6 | Projection分離方針（render/action/global） | wasm Boundary | GUI Interaction |
| Q-7 | click時候補列挙で十分か（パフォーマンス含む） | GUI Interaction | wasm Boundary |
| Q-8 | 実問題30件でどこまでカバーできるか | Coverage | 全員 |
| Q-9 | sample generationにつながるか | Sample Generation | Domain Model |
| Q-10 | MVPで何を切るか | Critical Reviewer | 全員 |

---

## 5. Subagentへの共有指示

### 5.1 出力形式（process.md §5.1準拠）

各Subagentは以下のセクションを含むこと:
1. 前提
2. 自分の担当範囲における主要判断
3. 具体例（最低3個）
4. 現行方針に対する支持/反対/留保
5. 不足点
6. 他Agentに渡すべき論点

### 5.2 曖昧語禁止

「だいたい対応できる」「工夫すればいける」は禁止。
「完全対応」「部分対応」「非対応」「MVP後回し」を明示。

### 5.3 参照ドキュメント

全Agent共通:
- 本文書（phase1-premises.md）
- `doc/plan/web-view/plan.md`（設計思想・契約）
- `doc/plan/main.md`（プロジェクト全体像）
- `doc/plan/web-view/process.md`（プロセス定義）

追加参照（必要に応じて）:
- `doc/design/domain-model.md`
- `doc/design/projection-operation.md`
- `doc/review/coverage-validation.md`
- `doc/review/critical-review.md`
- `doc/survey/atcoder-coverage-extended.md`
