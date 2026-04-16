# Phase 4: 相互批判レポート

> 作成日: 2025-07-18
> プロセス: process.md §Phase 4 準拠
> 依存: phase2-domain-model.md, phase2-gui-interaction.md, phase2-real-problem-coverage.md, phase2-wasm-boundary.md, phase2-sample-generation.md, phase2-critical-review.md

---

## 1. GUI → Domain Model 批判

GUI Interaction Agentの観点から、Domain Model Agentの提案に対する批判を行う。

### 1.1 ExprId導入の過剰さ

**問題**: Domain Model Agent は ExprId(u64) の導入を提案している（phase2-domain-model.md §2.1.1）が、これは実装コストに見合わない。

**批判根拠**:
- GUI Interaction Agent の操作フロー（§2.6 PendingExprAction）では、式スロットの「全置換」で十分機能する
- 「N → N/2」の変換は `ReplaceExpr { target: length_slot, expr: BinOp(...) }` で実現可能
- 部分式選択（例: `N/2` の `N` だけを選択）は、UIとしてクリック位置で特定でき、AST側にExprIdは不要
- 現行 Expression は value型であり、Arena化は大規模変更を伴う

**影響度**: 高（実装コスト増大、MVPスコープ肥大化）

**推奨解決策**: 
- MVP では ExprId を導入しない
- 式スロットは「全置換」のみを提供
- 式内部の操作が必要な場合は、PendingExprAction で候補を絞り込む（GUI側で対応）

### 1.2 SlotId の String 型設計

**問題**: Domain Model Agent の SlotId 定義で `slot_name: String` としている（phase2-domain-model.md §2.1.2）が、これは型安全性を損なう。

**批判根拠**:
- GUI Interaction Agent は SlotId を `"42:length"` のような文字列で表現（phase2-gui-interaction.md §7.1）
- スロット名は固定セット（"length", "lower", "upper", "count", "lhs", "rhs" 等）
- String にすると typo によるランタイムエラーが発生しうる

**推奨解決策**:
```rust
pub enum SlotKind {
    ArrayLength,
    RepeatCount,
    RangeLower,
    RangeUpper,
    RelationLhs,
    RelationRhs,
}
```

### 1.3 Projection API の過剰分割

**問題**: Domain Model Agent は約20個の Projection 関数を提案している（phase2-domain-model.md §2.4）が、UI 側の呼び出しパターンと合致しない。

**批判根拠**:
- GUI Interaction Agent の「Render時取得情報」（§2.5）では、4-5種の情報を**同時に**必要とする
- 個別に `project_structure_outline()`, `project_diagnostics()`, `project_completeness()` を呼ぶより、`project_full()` で一括取得のほうがwasm呼び出し回数を削減できる
- Action Projection の5関数は妥当だが、Render Projection は統合すべき

**推奨解決策**:
```rust
// 統合版
fn project_full(document: &str) -> FullProjection;  // outline + diagnostics + completeness
fn project_node_detail(document: &str, node_id: &str) -> NodeDetail;  // 選択時のみ
```

### 1.4 EditorState の型定義と実際の運用の乖離

**問題**: Domain Model Agent が提案する EditorState 型（phase2-domain-model.md §2.3）と、GUI Interaction Agent が必要とする状態の粒度が異なる。

**批判根拠**:
- Domain Model は `samplePreview: SamplePreviewState` を EditorState に含めているが、これは Bottom Panel のローカル状態で十分
- `diagnosticsFilter` は UI コンポーネントのローカル状態で管理すべき
- GUI Agent の EditorState（phase2-gui-interaction.md §2.4）はより実用的な分割になっている

**推奨解決策**: EditorState は以下に限定
- `documentJson`, `selectedNodeId`, `pendingExprAction`, `draftConstraint`
- それ以外は各コンポーネントのローカル状態として管理

---

## 2. Domain Model → GUI 批判

Domain Model Agentの観点から、GUI Interaction Agentの提案に対する批判を行う。

### 2.1 PendingExprAction のフェーズ遷移の意味論的危険

**問題**: GUI Interaction Agent の PendingExprAction 設計（phase2-gui-interaction.md §2.6, §3.2）は、意味論的整合性の検証が不十分。

**批判根拠**:
- `wrapBinary { op: "/", phase: "select-other-side", fixedSide: "lhs" }` というフェーズ遷移は GUI 都合であり、AST の不変条件との関係が曖昧
- 例: `N/2` を構築中に `N` が削除された場合の挙動が未定義
- `targetExpr: ExprId` が必要と書いているが、ExprId を導入しないなら `targetSlot: SlotId` で代替するはずだが、その場合の式の同一性判定が困難

**意味論的リスク**:
1. 構築中の式が参照するノードが削除された場合、orphan reference が発生
2. PendingExprAction 中に他の操作が行われた場合の整合性

**推奨解決策**:
- PendingExprAction 開始時に、対象スロットと参照可能な変数のスナップショットを取得
- AST 変更時は PendingExprAction を自動キャンセル（または警告）
- AST 不変条件 E-INV-3「Reference.Unresolved は一時状態のみ」（phase2-domain-model.md §2.2）との整合性を保証

### 2.2 DraftConstraint の target 選択における PlaceRef の曖昧さ

**問題**: GUI Interaction Agent は `PlaceRef` を `{ nodeId, indexed, indexVar }` で表現（phase2-gui-interaction.md §3.3）しているが、これは AST 層の Reference 型と一致しない。

**批判根拠**:
- AST 層の Reference は `VariableRef(NodeId) | IndexedRef { target, indices } | Unresolved(Ident)`
- `PlaceRef` が `indexed: true, indexVar: "i"` の場合、内部表現は `IndexedRef { target, indices: [Ident("i")] }` となるべき
- 多次元（`A[i][j]`）への拡張性が PlaceRef 設計で考慮されていない

**推奨解決策**:
```typescript
// GUI側
interface PlaceRef {
  nodeId: NodeId;
  indices: string[];  // ["i"] or ["i", "j"] for multi-dimensional
}

// これは AST 側の IndexedRef と1:1対応
```

### 2.3 テンプレート展開の原子性未保証

**問題**: GUI Interaction Agent は `ApplyTemplate` Action を単一呼び出しとしている（phase2-gui-interaction.md §3.4）が、内部で複数の Operation が発生する場合の原子性が未定義。

**批判根拠**:
- 「辺リスト（木）」テンプレートは以下を一括実行:
  1. FillHole(Sequence)
  2. 複数の AddConstraint
  3. Property(Tree) 追加
- 途中で失敗した場合（例: 名前衝突）、部分適用状態になりうる
- undo が1ステップか複数ステップか未定義

**推奨解決策**:
- ApplyTemplate は AstEngine 内部で transaction として実装
- 部分適用は許可しない（all or nothing）
- 返却値に `reverted: bool` を含め、失敗時は元の状態を返す

### 2.4 キーボードナビゲーションとフォーカス管理の欠落

**問題**: GUI Interaction Agent の操作フローはすべてクリックベースであり、キーボード操作の設計がない。

**批判根拠**:
- フォーム入力（名前、式スロット）では Tab/Enter/Escape が期待される
- ツリーナビゲーションでは矢印キーが期待される
- これは「体験を担う」UI 側の責務だが、AST 側の「フォーカス対象」の概念と接続が必要

**推奨解決策**:
- EditorState に `focusedSourceRef: SourceRef | null` を追加
- SourceRef は Domain Model Agent 提案の `Node | Expr | Slot | Constraint` enum
- Tab 順序は `project_structure_outline` の depth-first order に従う

---

## 3. Coverage → Domain Model + GUI 批判

Real Problem Coverage Agentの観点から、Domain Model Agent と GUI Interaction Agent の提案を実問題ベースで批判する。

### 3.1 下三角行列（Problem 11）で設計が破綻

**問題**: ABC370-B（下三角行列）が「非対応」となる根本原因は、Repeat.index_var を Array.length で参照できない設計にある。

**批判対象**: Domain Model Agent の Repeat 設計、GUI Agent の操作フロー

**批判根拠**:
```
入力形式:
N
A_{1,1}
A_{2,1} A_{2,2}
...
A_{N,1} ... A_{N,N}
```

- 現行設計では `Repeat(count=N, index_var=i, body=Array(length=???))` となるが、Array.length に `i`（ループ変数）を参照する手段がない
- Domain Model Agent は Repeat.index_var を `Option<Ident>` としているが、これは「存在する」だけで「参照可能」ではない
- GUI Agent は「???（現行では表現不能）」と記載しているだけで、workaround も示していない

**影響度**: 中（出現頻度は低いが、設計の根本的な制限を示す）

**推奨解決策**:
```rust
// 案1: Expression.Var に index_var 参照を追加
pub enum Reference {
    VariableRef(NodeId),
    IndexedRef { target: NodeId, indices: Vec<Ident> },
    IndexVarRef(Ident),  // ← 追加: Repeat.index_var を参照
    Unresolved(Ident),
}
```

**MVP判断**: 非対応を許容（年数回の出現頻度）。ただし将来対応のための設計余地を残す。

### 3.2 Tuple内インライン配列（Problem 13, 32）で workaround が不自然

**問題**: ABC259-E, ABC356-C の「行内可変長配列」が部分対応となる workaround は、canonical rendering が破綻する。

**批判対象**: Domain Model Agent の Tuple 設計、GUI Agent の操作フロー

**批判根拠**:
```
入力形式 (ABC356-C):
C_1 A_{1,1} ... A_{1,C_1} R_1
...
```

- 現行 Tuple.elements は `Vec<NodeId>` であり、Array を含めることは可能
- しかし `Sequence[Scalar(C_i), Array(A_i, C_i), Scalar(R_i)]` では改行が入る
- Coverage Agent は「RenderHint.Separator(None) で同一行出力を強制」と提案しているが、Domain Model Agent の設計にこれがない

**影響度**: 中（月1-2問程度の出現頻度）

**推奨解決策**:
```rust
pub enum RenderHintKind {
    Separator(Separator),
    InlineLayout,  // ← 追加: 子要素を同一行に配置
}
```

**MVP判断**: 早期対応推奨。RenderHint 拡張で対応可能。

### 3.3 クエリ列テンプレートの UI 設計が未検証

**問題**: GUI Interaction Agent のクエリ列操作フロー（§2.3.7, §3.5）は、Coverage Agent の Problem 19-21 を検証していない。

**批判根拠**:
- ABC278-D（3種クエリ）では `1 x`, `2 i x`, `3 i` という異なる引数数のバリアント
- GUI Agent のモーダル設計（§3.5 T3-T4）では `body: ['l', 'r']` のようにフィールド名のリストを入力するが、これは不自然
- 実際のユーザーは「クエリ1は x のみ、クエリ2は i と x」のように理解するため、構造的入力より自然言語的入力が望ましい

**推奨解決策**:
- テンプレートモーダルを2段階に分割:
  1. クエリ種別数と各タグ値の入力
  2. 各バリアントの構造をインライン編集（Detail Pane 上で直接）

### 3.4 SumBound 対応の欠落

**問題**: Coverage Agent の Problem 25（複数テストケース + 総和制約）は「完全対応」となっているが、Sample Generation Agent は SumBound が未実装と報告している（phase2-sample-generation.md §2.3）。

**批判対象**: Coverage Agent の判定、Domain Model/GUI の SumBound 扱い

**批判根拠**:
- Coverage は「AST/Constraint としては表現可能」の意味で完全対応としている
- しかし Sample Generation ができないなら、Editor としての価値が半減
- AtCoder マルチテストケース問題の80%以上が ΣN 制約を持つ

**影響度**: 高（マルチテストケース問題の主要カバレッジに影響）

**推奨解決策**:
- SumBound を MVP 必須に格上げ
- Sample Generation Agent と協調して Generator 拡張を優先

---

## 4. wasm Boundary → All 批判

wasm Boundary Agentの観点から、全Agentの提案に対する境界責務・パフォーマンス観点の批判を行う。

### 4.1 Domain Model Agent の Projection API 粒度が境界コストを増大

**問題**: Domain Model Agent の約20個の Projection 関数（phase2-domain-model.md §2.4）は、wasm 呼び出し回数を不必要に増やす。

**批判根拠**:
- wasm ↔ JS 境界の各呼び出しには JSON serialize/deserialize のオーバーヘッドがある
- wasm Boundary Agent の計測基準（phase2-wasm-boundary.md §4.3）では、100KB で 2-5ms
- 初回描画で outline, diagnostics, completeness を個別に呼ぶと、document JSON が3回 parse される

**推奨解決策**:
- Render Projection を統合: `project_full()` で一括返却
- 返却 JSON は nested object で、必要部分のみ使用

### 4.2 GUI Interaction Agent の PendingExprAction が境界越えを複雑化

**問題**: GUI Agent の PendingExprAction 設計では、式構築中に複数回の wasm 呼び出しが発生する。

**批判根拠**:
- `N → N/2` の構築フロー（phase2-gui-interaction.md §2.6）:
  1. `project_expr_actions(expr_N)` で候補取得
  2. `/x` 選択後、`project_expr_slot_candidates()` で rhs 候補取得
  3. `apply_action(ReplaceExpr)` で確定
- 3回の wasm 呼び出しが発生
- ステップ2と3の間で AST が変更されていない保証がない

**推奨解決策**:
- 式構築を**テキスト入力 + パース**に変更（Critical Reviewer 提案と一致）
- `parseExpr(text, availableRefs)` を wasm 側に追加し、1回の呼び出しで完結

### 4.3 Sample Generation Agent の check_generatability API が未実装

**問題**: Sample Generation Agent は `check_generatability()` API の追加を提案（phase2-sample-generation.md §6.1）しているが、これが wasm 境界設計に反映されていない。

**批判根拠**:
- wasm Boundary Agent の API 提案（phase2-wasm-boundary.md §2.1）には `check_generatability` がない
- 「Generate ボタン有効化条件」（phase2-sample-generation.md §2.5）には事前検査が必要
- 現状では Generate ボタン押下 → エラー、というUXになる

**推奨解決策**:
```rust
#[wasm_bindgen]
pub fn check_generatability(document_json: &str) -> Result<String, JsError>;
// 返却: { generatable: bool, level: "L1"|"L2"|"L3"|"E0", warnings: [...], errors: [...] }
```

### 4.4 undo/redo の境界責務が未定義

**問題**: 全 Agent が undo/redo を「MVP 後回し」としているが、境界設計への影響が未検討。

**批判根拠**:
- wasm Boundary Agent は「wasm stateless 設計」を採用（phase2-wasm-boundary.md §2.2）
- これは「各呼び出しに document_json を渡す」パターン
- undo 実装時に選択肢が2つ:
  1. Preact 側で document history を保持 → 境界変更なし
  2. wasm 側で history stack を保持 → stateless 破棄、境界変更大
- 選択肢1の場合、巨大 document のメモリ消費が問題
- 選択肢2の場合、wasm 関数が state-aware に変更

**推奨解決策**:
- MVP は選択肢1（Preact 側 history）で実装
- document 圧縮（差分保持）を検討
- MVP 後に wasm 側 history を検討する場合は境界 API を破壊的変更

---

## 5. Sample Generation → All 批判

Sample Generation Agentの観点から、全Agentの提案に対する生成器接続観点の批判を行う。

### 5.1 Domain Model Agent の Expression 型が FnCall を十分に表現していない

**問題**: Domain Model Agent は Expression.FnCall を維持しているが、対応関数リストが未定義。

**批判根拠**:
- Sample Generation Agent は `evaluate(Expression)` で min, max, abs を使用（phase2-sample-generation.md §2.2）
- Domain Model Agent の棚卸し（phase2-domain-model.md §1.2）には `FnCall` があるが、対応関数の列挙がない
- GUI Interaction Agent の式構築フロー（§2.6）で `wrapCall { func: "max" }` を示しているが、Generator 側で max が評価可能か未検証

**推奨解決策**:
- Generator 対応関数リストを明示: `min, max, abs, len`
- これ以外の FnCall は生成不可（L3 降格）とする
- GUI 側は対応関数のみを候補として表示

### 5.2 GUI Interaction Agent のテンプレート設計が Generator の要件を無視

**問題**: GUI Agent のテンプレート（辺リスト、グリッド等）が、Generator が要求する Constraint を自動追加しない可能性。

**批判根拠**:
- Sample Generation は DependencyGraph.build() で Structure 参照を解決（phase2-sample-generation.md §2.2）
- 「辺リスト」テンプレートで `Property(Tree)` を追加しないと、Generator はランダム辺リストを生成
- GUI Agent のテンプレート展開（§2.3.5）では Property 追加が「任意」となっている

**推奨解決策**:
- テンプレートには「最小必須 Constraint セット」を定義
- 「辺リスト（木）」は Property(Tree) を必須とする
- GUI のモーダルで「Property を外す」オプションを提供

### 5.3 Coverage Agent の「完全対応」判定が Generator 可能性を考慮していない

**問題**: Coverage Agent の36問中、Sample Generation 観点では L2/L3 にとどまるものがある。

**批判根拠**:
- Problem 5（順列配列）: Property(Permutation) は Generator 対応済み → L2
- Problem 12（木）: Property(Tree) は Generator 対応済み → L2
- Problem 11（下三角行列）: 構造自体が非対応 → E0
- Problem 13（行内可変長）: Generator は Repeat を処理するが、canonical output が保証されない → L3

**推奨解決策**:
- Coverage 判定に「Generator 対応レベル」を追加
- 「完全対応」は AST 表現 + Generator L1/L2 を要件とする

### 5.4 wasm Boundary Agent の generate_sample エラーが診断に不十分

**問題**: wasm Boundary Agent の `generate_sample` 返却は文字列エラーのみ。

**批判根拠**:
- Sample Generation Agent は `GenerationError` の6種を定義（phase2-sample-generation.md §2.4）
- wasm Boundary Agent の返却例（phase2-wasm-boundary.md §3.3）は `JsError("Cannot generate: 2 holes remain")` という文字列
- GUI が「Hole を埋めてください」「制約を修正してください」の切り分けができない

**推奨解決策**:
```rust
pub struct GenerationResult {
    pub success: bool,
    pub text: Option<String>,
    pub error: Option<GenerationErrorDto>,
    pub level: GenerationLevel,  // L1, L2, L3
    pub warnings: Vec<String>,
}
```

---

## 6. Critical Reviewer → All 批判

Critical Reviewer Agentの観点から、全Agentの提案に対する総合的批判を行う。

### 6.1 MVP スコープの膨張リスク

**問題**: 各 Agent が「MVP 対応」としているものを合計すると、MVP が巨大化する。

**Critical 分析**:
- Domain Model: ExprId, SlotId, SourceRef, 20+ Projection API → **過剰**
- GUI: 7種基本ケースフロー、DraftConstraint 12種 → **妥当だが DraftConstraint は4種に絞るべき**
- Coverage: テンプレート4種（辺リスト、グリッド、クエリ列、複数テストケース）→ **MVP は手動構築で代替可能**
- wasm: 17関数 → **10関数に絞るべき**
- Sample: check_generatability, generate_sample_with_level → **generate_sample のみで十分**

**推奨 MVP Phase 1 スコープ**:
| 領域 | 含む | 含まない |
|------|------|---------|
| Structure | Scalar, Array, Tuple, Sequence, Repeat, Hole | Matrix, Section, Choice |
| Constraint | Range, TypeDecl, LengthRelation, Relation | Distinct, Sorted, Property, SumBound, Guarantee, CharSet, StringLength |
| Expression | Lit, Var, BinOp(+,-,*,/) | Pow, FnCall |
| wasm API | project_full, project_node_detail, get_hole_candidates, get_expr_candidates, apply_action, generate_sample | 20+ 個別 API |
| テンプレート | なし（手動構築） | 辺リスト、グリッド等 |
| UI | 3カラム基本レイアウト、式スロット全置換 | drag&drop, undo/redo |

### 6.2 設計間の不整合

**問題**: 各 Agent の設計間で整合性が取れていない箇所がある。

| 不整合 | Agent A | Agent B | 解決方針 |
|--------|---------|---------|---------|
| ExprId 必要性 | Domain: 必要 | Critical: 不要 | MVP では不要、式は全置換 |
| SlotId 型 | Domain: String | Critical: enum | enum に統一 |
| Projection 粒度 | Domain: 20関数 | wasm/Critical: 5-7関数 | 統合版で実装 |
| SumBound | Coverage: 完全対応 | Sample: 未実装 | Generator 拡張が必要 |
| undo/redo | 全員: MVP後回し | - | 明示的に「非対応」と文書化 |

### 6.3 代替案の未検討

**問題**: 各 Agent は自分の領域で最適な設計を追求しているが、領域横断の代替案が検討されていない。

**例: 式スロット編集の代替**

| 方式 | 提案元 | 長所 | 短所 |
|------|--------|------|------|
| ExprId + PendingExprAction | Domain/GUI | 構造的操作 | 実装複雑 |
| テキスト入力 + パース | Critical | シンプル | パーサ実装が必要 |
| 候補選択のみ | - | 最小実装 | 複雑な式が作れない |

**推奨**: MVP は「候補選択のみ」、Phase 2 で「テキスト入力 + パース」を追加

### 6.4 実装コストの見積もり欠如

**問題**: 全 Agent が「実装コスト」の見積もりを行っていない。

**Critical による粗見積もり**:
| 機能 | 見積もり (人日) | リスク |
|------|-----------------|--------|
| ExprId 導入 | 5-10 | 高（Expression 全面改修） |
| 20 Projection API | 10-15 | 中 |
| 7 基本ケースフロー | 15-20 | 低 |
| DraftConstraint 12種 | 10-15 | 中 |
| テンプレート4種 | 8-12 | 中 |
| Sample Generation 拡張 | 5-8 | 中 |

**MVP Phase 1 目標**: 30-40人日で動作する最小プロトタイプ

---

## 7. 合意事項

全 Agent が支持する設計方針:

| # | 合意事項 | 支持 Agent | 根拠 |
|---|----------|-----------|------|
| C-1 | AST は1種類（StructureAST + ConstraintAST + Expression） | 全員 | 状態同期問題の回避 |
| C-2 | Structure Hole は AST 内に配置 | 全員 | 位置・順序が意味論的 |
| C-3 | Expression 未完成は EditorState に配置（AST に入れない） | 全員 | AST 汚染回避 |
| C-4 | Constraint 未完成は EditorState に配置 | 全員 | フォーム入力モデルに自然 |
| C-5 | wasm ↔ JS 間は JSON string | wasm/Domain/GUI | 可視化・デバッグ容易 |
| C-6 | ID は decimal string（53bit 問題回避） | wasm/Domain | JS 互換性 |
| C-7 | click 時遅延取得（render 時に候補全列挙しない） | GUI/wasm | パフォーマンス |
| C-8 | undo/redo は MVP 後回し | 全員 | 実装複雑性 |
| C-9 | ABC A〜D 問題で 90% 以上カバレッジが目標 | Coverage/Critical | 実用性基準 |
| C-10 | 下三角行列は MVP 非対応 | Coverage/Critical | 出現頻度が低い |

---

## 8. 未解決争点

| ID | Topic | Position A | Position B | Recommended Resolution |
|----|-------|------------|------------|------------------------|
| D-1 | ExprId の必要性 | Domain: 部分式選択に必須 | Critical/GUI: 式全置換で十分 | **MVP では ExprId 不要**。式スロットは「全置換」のみ。部分式操作は Phase 2 で検討 |
| D-2 | SlotId の slot_name 型 | Domain: String | Critical: enum (SlotKind) | **enum に統一**。typo 防止、型安全性向上 |
| D-3 | Projection API 粒度 | Domain: 20関数 | Critical/wasm: 5-7関数 | **統合版採用**。`project_full()` + `project_node_detail()` + 候補取得3関数 + `apply_action()` + `generate_sample()` |
| D-4 | SumBound の MVP 優先度 | Sample: MVP 必須 | Critical: Phase 2 | **Phase 2**。Guarantee で代替可能。Generator 拡張コストが高い |
| D-5 | テンプレートの MVP 包含 | GUI/Coverage: MVP 必須 | Critical: 手動構築で代替 | **Phase 2**。MVP は手動構築のみ。テンプレートは UX 改善として Phase 2 |
| D-6 | 式スロット編集方式 | GUI: 候補選択 + PendingExprAction | Critical: テキスト入力 + パース | **MVP は候補選択のみ**。複雑な式は Phase 2 で対応 |
| D-7 | Tuple内インライン配列 | Coverage: 早期対応推奨 | - | **Phase 2**。RenderHint.InlineLayout で対応 |
| D-8 | check_generatability API | Sample: 必要 | wasm: 未提案 | **MVP 必要**。Generate ボタン有効化に必須。簡易版で実装 |

---

## 9. 設計修正提案

### 優先度: High（MVP Phase 1 必須）

#### P-H1: Projection API 統合

**現状**: Domain Model が20+関数を提案
**修正**: 以下の7関数に統合

```rust
// Render (document 更新時)
fn project_full(document: &str) -> String;
// 返却: { outline, diagnostics, completeness }

fn project_node_detail(document: &str, node_id: &str) -> String;
// 返却: { slots, relatedConstraints }

// Action (click 時)
fn get_hole_candidates(document: &str, hole_id: &str) -> String;
fn get_expr_candidates(document: &str, parent_id: &str, slot_kind: &str) -> String;
fn get_constraint_targets(document: &str, constraint_kind: &str) -> String;

// Operation
fn apply_action(document: &str, action: &str) -> String;

// Sample
fn generate_sample(document: &str, seed: u32) -> String;
fn check_generatability(document: &str) -> String;  // 追加
```

#### P-H2: ExprId 導入の延期

**現状**: Domain Model が ExprId 導入を提案
**修正**: MVP では導入しない

- 式スロットは「全置換」のみ
- `SetExpr { slot: SlotId, expr: ExpressionDto }` Action で対応
- 部分式操作は Phase 2 で ExprId 導入時に検討

#### P-H3: SlotId の型安全化

**現状**: `slot_name: String`
**修正**: `slot_kind: SlotKind` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotKind {
    ArrayLength,
    MatrixRows,
    MatrixCols,
    RepeatCount,
    RangeLower,
    RangeUpper,
    RelationLhs,
    RelationRhs,
}

pub struct SlotId {
    pub owner: NodeId,
    pub kind: SlotKind,
}
```

#### P-H4: DraftConstraint 4種に限定

**現状**: 12種の Constraint すべてに DraftConstraint
**修正**: MVP は4種のみ

```typescript
type DraftConstraint =
  | { kind: 'Range'; target: PlaceRef | null; lower: Expression | null; upper: Expression | null }
  | { kind: 'TypeDecl'; target: PlaceRef | null; expectedType: 'Int' | 'Str' | 'Char' | null }
  | { kind: 'LengthRelation'; target: PlaceRef | null; length: Expression | null }
  | { kind: 'Relation'; lhs: Expression | null; op: RelationOp | null; rhs: Expression | null };
```

### 優先度: Medium（Phase 2）

#### P-M1: テンプレート実装

- 辺リスト、グリッド、クエリ列、複数テストケースの4種
- ApplyTemplate Action を AstEngine に追加
- GUI にテンプレートモーダルを追加

#### P-M2: RenderHint.InlineLayout 追加

```rust
pub enum RenderHintKind {
    Separator(Separator),
    InlineLayout,  // Tuple 内の配列を同一行に
}
```

#### P-M3: SumBound Generator 対応

- Generator に SumBound 考慮の分配ロジックを追加
- `remaining` トラッキングで各イテレーションの範囲を動的調整

#### P-M4: テキスト入力式構築

- `parseExpr(text, scope)` wasm 関数を追加
- GUI に式スロットテキスト入力を追加
- PendingExprAction は不要に

### 優先度: Low（Phase 3 以降）

#### P-L1: undo/redo 実装

- Preact 側で document history を保持（差分圧縮）
- または wasm 側で Command pattern を実装

#### P-L2: 下三角行列対応

- `Reference::IndexVarRef(Ident)` を追加
- Repeat.index_var を Array.length から参照可能に

#### P-L3: Matrix, Section, Choice の GUI 対応

- 現行は Sequence + Repeat で代替
- 専用 UI を追加

---

## 10. 結論

### Phase 4 完了条件の確認

> 各主要設計判断について、最低 1 回は批判的検証を受けていること（process.md §Phase 4）

| 設計判断 | 批判元 Agent | 検証結果 |
|----------|-------------|---------|
| AST 1種類 | Critical | 支持 |
| Hole 配置 | Domain/GUI | 支持 |
| Expression partial 不配置 | GUI/Domain | 支持 |
| ExprId 導入 | Critical/GUI | **争点** → MVP 延期 |
| SlotId 設計 | Critical | **修正** → enum 化 |
| Projection 分割 | Critical/wasm | **修正** → 統合 |
| PendingExprAction | Domain | **リスク指摘** → 簡略化 |
| DraftConstraint | Critical | **修正** → 4種に限定 |
| テンプレート | Critical | **延期** → Phase 2 |
| SumBound | Sample/Coverage | **延期** → Phase 2 |
| undo/redo | 全員 | **延期** → Phase 3 |

### 次ステップ

Phase 5（統合設計）では、本文書の以下を入力として最終設計を収束させる：

1. **合意事項 C-1〜C-10** を設計の基盤とする
2. **未解決争点 D-1〜D-8** の Recommended Resolution を採用
3. **設計修正提案 P-H1〜P-H4** を MVP Phase 1 に含める
4. **P-M1〜P-M4** は Phase 2 ロードマップに記載
