# Competitive Programming AST Editor UI設計書

## 0. この文書の位置づけ

この文書は、プロジェクト全体の設計書を置き換えるものではない。全体方針・ドメインモデル・AST core の責務などはリポジトリの `main.md` に委ねる。

本書の目的は、**今回作る Editor における UI 設計思想を共有すること**である。

特に以下を明確にする。

* ユーザーは何をどういう順番で操作するのか
* UI は AST とどう接続されるべきか
* どこまでを AST 側の責務とし、どこからを UI / EditorState 側の責務とするか
* 「理論的にきれい」よりも「競プロ入力を迷わず作れる」ことをどう優先するか
* 基本ケースを確実に扱い、中程度に複雑な問題まで高い割合でカバーするためのUI方針は何か

---

## 1. Editor の第一目的

Editor の第一目的は、**競技プログラミング問題の入力仕様を、GUI 上で構造的に作成・編集できること**である。

ここで重要なのは、ユーザーが最終的に欲しいのが「美しい AST」そのものではなく、以下の一連の体験だということである。

1. 問題文を読みながら
2. 入力の形を段階的に組み立て
3. 制約を後から足し
4. 見た目としても破綻せず
5. 最終的に AST と sample が得られる

したがって本 Editor は、単なる AST ビューアでも、文字列 DSL の入力フォームでもない。

**構造化 GUI によって AST を直接編集する projectional editor** として設計する。

---

## 2. 根幹となる UI 設計思想

### 2.1 AST を正本とする

UI は正本ではない。正本は AST である。

ただし UI は AST の単なるダンプではなく、ユーザーが編集可能なように整えられた投影である。

原則として次の流れを採る。

```text
AST
  ↓
Projection API
  ↓
UI
  ↓
Operation API
  ↓
AST 更新
```

この方針により、表示と編集の根拠が常に AST に戻る。

---

### 2.2 テキスト編集ではなく構造編集を主とする

本 Editor は、ユーザーに自由入力で DSL を書かせることを主眼にしない。

代わりに、

* クリック
* 選択
* ブロック追加
* 操作テンプレート適用
* 候補選択

を中心とする。

理由は以下。

* 競プロ入力仕様は構造が強い
* 単純な入力ほど GUI で速く作れる
* 未完成テキストより未完成構造のほうが扱いやすい
* 生成器や検証器との接続が簡単になる

---

### 2.3 基本ケースは完璧にする

すべての競プロ問題を最初から一般的に扱うことは目標にしない。

代わりに、以下を迷いなく作れることを最優先にする。

* 単一整数
* 複数整数
* 長さ依存配列
* グリッド
* 辺リスト
* 複数テストケース
* クエリ列
* 基本的な Range / Type / Length / Relation 制約

ここが雑だと、どれだけ高機能でも使われない。

---

### 2.4 複雑さは「AST の一般化」ではなく「操作テンプレート」で吸収する

`A: int[N/2]` のような式付き長さや、`1 <= A[i] <= N` のような制約は必要である。

ただしそれを「欄ごとに専用UIを増やす」ことで対処しない。

代わりに、

* 型付きの式スロット
* 共通の式構築 UI
* 操作テンプレート
* pending action

によって吸収する。

これにより、hole 種類と入力候補の直積爆発を避ける。

---

### 2.5 未完成は AST と UI state に適切に分配する

未完成状態をすべて AST に入れるのでも、すべて UI state に逃がすのでもない。

今回の基本方針は次の通り。

#### Structure の未完成

AST 側に `Hole` を残す。

理由:

* 構造では「まだ何かが入る位置」が本質だから
* 順序・親子関係・位置が意味を持つから
* UI でも空位置が持続的に見えていてほしいから

#### Expression の未完成

AST に partial expr を入れず、EditorState / PendingAction に逃がす。

理由:

* 式編集は局所変形として表現しやすいから
* `/x`, `+x`, `max(x,y)` などは操作テンプレートとして扱えるから
* 完成した瞬間に一発で AST を更新できるから

#### Constraint の未完成

原則として AST に draft constraint を入れず、EditorState に持つ。

理由:

* 制約作成途中はフォーム入力に近いから
* 木の空位置より「いま構築中の1件」という性質が強いから

---

## 3. この Editor が前提とするデータ責務

### 3.1 AST

AST は 1 種類を基本とする。

* Structure は `Hole` を許容する
* Constraint は完成形のみを持つ
* Expression は完成形のみを持つ

この AST が唯一の正本である。

---

### 3.2 EditorState

UI 専用の一時状態は AST に混ぜず、EditorState 側で持つ。

例:

* 選択中 node / expr / slot
* 開いているメニュー
* モーダル状態
* 式構築中の pending action
* 制約作成中の draft
* ドラッグ中状態
* hover / focus

EditorState は意味論の正本ではなく、編集体験のための補助状態である。

---

### 3.3 Projection API

Projection は 3 種類に分ける。

#### Render Projection

描画に必要な最小情報だけ返す。

* node 単位
* expr 単位
* slot 単位

#### Action Projection

クリック時にだけ必要な候補やアクションを返す。

* 置換候補
* wrap 候補
* 入力テンプレート
* 参照候補

#### Global Projection

全体一覧・診断・アウトライン用。

* structure 一覧
* constraint 一覧
* diagnostics
* completeness

この分離により、全体巨大 projection を避ける。

---

## 4. UI 設計の主要な立場

### 4.1 画面は「構造一覧」「詳細」「制約一覧」を基本とする

MVP の基本構成は以下。

#### 左ペイン: Structure

* sequence / section / repeat / tuple / array / scalar を一覧表示
* Hole も可視化
* 順序と親子関係を編集できる

#### 中央 or 右: Detail

* 選択中ノードの詳細編集
* 名前
* 型
* 長さ
* body
* variants
* 関連する制約

#### 右ペイン or 下部: Constraints

* 制約一覧
* 制約追加
* 制約編集
* diagnostics

この構成により、構造と制約を明示的に分離したまま編集できる。

---

### 4.2 ユーザーは「宣言を置いてから制約を足す」

基本的な編集順は以下を想定する。

1. 構造ノードを置く
2. 名前を付ける
3. 型を付ける
4. 長さや body をつなぐ
5. 制約を後から追加する

例:

* `N: int`
* `A: int[N]`
* `1 <= N <= 10^5`
* `1 <= A[i] <= 10^9`

この流れは競プロ入力の理解順に近い。

---

### 4.3 UI は「何を入力するか」より「次に何ができるか」を見せる

本 Editor では、空欄に自由入力させるよりも、

* ここに何が置けるか
* いま選んだものに何を適用できるか
* 次に自然な操作は何か

を見せることを重視する。

例:

* Hole をクリックすると候補カテゴリが出る
* `N` をクリックすると `/x` や `+x` が出る
* `Repeat` を選ぶと body に何を追加できるか出る

これは構造化GUIとしての本質である。

---

## 5. Structure UI 設計思想

### 5.1 Hole は「未完成」ではなく「編集可能な位置」でもある

Structure の Hole は単なる不足表示ではない。

Hole は以下の意味を持つ。

* ここにまだ何かが入る
* この位置は AST 上で意味がある
* ユーザーはここを埋めることで前進する
* GUI 上の挿入位置として可視化される

したがって Hole は、エラー表示ではなく **first-class な編集対象** として扱う。

---

### 5.2 追加は「位置」を意識させる

Structure 編集では、何を置くかだけでなく、どこに置くかが重要である。

したがって UI は以下を明確に見せる。

* 親はどこか
* 子順序はどうか
* ここは single slot か list slot か
* body / header / variant / tuple など、どのスロットか

これが見えないと、木編集はすぐに破綻する。

---

### 5.3 高レベルブロックを直接追加できるようにする

ユーザーに毎回 raw NodeKind を並べさせると重い。

そのため UI では高レベルな追加操作を用意する。

例:

* 変数を追加
* 配列を追加
* グリッドを追加
* 辺リストを追加
* テストケースを導入
* クエリ列を追加

内部ではこれらが AST / Operation API に展開される。

つまり UI では Builder 的な操作を前面に出す。

---

## 6. Expression UI 設計思想

### 6.1 式は partial AST を持たず、操作テンプレートで構築する

式はテキスト入力中心ではなく、操作テンプレートで構築する。

例:

* 参照を選ぶ
* 定数を入れる
* `/x` を適用
* `+x` を適用
* `max(x,y)` を適用

重要なのは、`N / ?` を AST に入れないこと。

代わりに UI は、

* `N` に対して `/x` を選ぶ
* 右辺候補を選ぶ
* 確定時に `N / 2` に置換する

という遷移で扱う。

---

### 6.2 「式エディタ」ではなく「式遷移UI」と考える

一般的な数式エディタのように、完全な自由構文編集を最初から目指さない。

今回の UI はむしろ、

* 現在の式ノードを選ぶ
* そのノードに対する遷移候補を出す
* 必要なら追加引数を選ばせる
* 最後に一括で AST へ反映する

という遷移型 UI として考える。

---

### 6.3 式スロットは共通基盤で扱う

長さ、range upper、repeat count などを別々の UI にしない。

それぞれを **型付き式スロット** として扱う。

例:

* 配列長 slot
* Range の lower slot
* Range の upper slot
* Relation の lhs / rhs slot

違うのはラベルではなく、

* expected sort
* 参照可能 scope
* 許される操作
* slot rule

である。

---

## 7. Constraint UI 設計思想

### 7.1 制約は「フォームとして組み立てる」

Constraint は構造ノードほど位置依存ではない。

したがって制約作成 UI は、Hole を埋めるよりもフォーム組み立てに近くてよい。

たとえば Range なら:

* target を選ぶ
* lower を設定
* upper を設定
* 完成したら AST に追加

という流れにする。

---

### 7.2 制約は node に従属しつつ、一覧としても見えるべき

Constraint は target を持つが、ユーザー視点では一覧でも見たい。

したがって UI では両方必要。

* 選択中 node に関連する constraint を Detail に出す
* 全 constraint を一覧でも見せる

これにより、局所編集と全体把握を両立する。

---

### 7.3 基本制約にはショートカットを用意する

よくある制約はワンクリックで足せるようにする。

例:

* Range を追加
* Type を追加
* 長さ = 変数
* Distinct を追加
* Sorted を追加
* Tree 性質を追加

ここで重要なのは、複雑な一般制約のために基本制約の操作を重くしないこと。

---

## 8. ユーザー操作順序の原則

### 8.1 問題読解順に近い操作列を採る

ユーザーが競プロ問題を読むとき、多くは次の順で理解する。

1. 入力の骨格
2. 各変数や配列
3. 長さ依存関係
4. 制約
5. 特殊性質

Editor もこの順を自然に踏めるべきである。

---

### 8.2 先に完全性を要求しない

途中状態で以下があってよい。

* 名前未設定
* 型未設定
* 構造 Hole 未充足
* 制約がまだ足りない

ただしそれは silent failure ではなく、diagnostics として可視化する。

---

### 8.3 「まだ決めなくてよい」を尊重する

Editor はユーザーに過剰な即時決定を強制しない。

例:

* 配列は先に置ける
* 長さは後で決められる
* 制約は後から足せる
* section の body は空でもよい

これは、問題文理解と構築が必ずしも一直線ではないからである。

---

## 9. Preact と wasm(AST core) の UI 境界思想

### 9.1 wasm は意味論、Preact は体験

Rust/wasm 側は以下に集中する。

* AST
* Operation
* Projection
* 検証
* sample generation への橋渡し

Preact 側は以下に集中する。

* コンポーネント表示
* 選択状態
* メニュー
* pending action
* draft form
* drag & drop
* focus / hover

つまり wasm は意味論の中核、Preact は編集体験の中核である。

---

### 9.2 クリック可能性と候補列挙を分ける

描画時には、

* どこがクリックできるか
* 何を指しているか

だけ分かればよい。

何を置換できるか、何を適用できるかは、クリック時に遅延取得する。

これにより projection を過剰に肥大化させない。

---

## 10. この UI 設計が目指す体験

ユーザーは以下のように感じるべきである。

* とりあえず骨格を置ける
* どこを埋めればよいか分かる
* よくある入力はすぐ作れる
* 複雑な式も手順で組み立てられる
* 制約は後から整理して追加できる
* 途中状態でも壊れて見えない
* 最終的に sample までつながる

この体験を支えるために、UI は「理論を見せる」のではなく、**AST の意味論を壊さずに自然な編集動線を与える**ことを目標とする。

---

## 11. MVP で守るべき UI 要件

MVP では最低限以下を満たす。

### 必須

* Structure 一覧
* Hole 可視化
* Scalar / Array / Tuple / Repeat / Section / Sequence / Choice の基本編集
* Constraint 一覧
* Range / TypeDecl / LengthRelation / Relation の追加
* 式スロットに対する基本的な構築 UI
* diagnostics 表示
* AST から sample を出力できる

### できれば早めに欲しい

* 高レベルブロック追加
* 辺リストテンプレート
* グリッドテンプレート
* 複数テストケース導入テンプレート
* Tree / Distinct / Sorted などの簡易ショートカット

### 後回しにできる

* 自由入力式エディタ
* 高度な drag & drop
* collaborative editing
* 高機能 undo/redo
* interactive 問題専用 UI

---

## 12. 最終的な推奨方針

今回の Editor UI は、以下の立場を採る。

1. AST を正本とする projectional editor にする
2. Structure の未完成は AST の Hole で扱う
3. Expression の未完成は pending action に逃がす
4. Constraint の未完成は draft form として扱う
5. Projection は render / action / global に分ける
6. UI は block / template / candidate driven にする
7. 基本ケースを確実にし、中程度の複雑さまで高い割合でカバーする

この方針により、

* AST core の意味論
* Preact UI の自然さ
* wasm 境界の明快さ
* sample generation との接続

を同時に満たすことを目指す。

---

## 付録: この文書の使い方

この文書は、実装担当 AI Agent に対して以下の使い方を想定する。

* `main.md` を全体設計の根幹として読む
* 本書を UI 設計思想の基盤として共有する
* 実際のタスクでは、本書の方針に沿って画面設計・API設計・状態分離を具体化する
* 実問題カバレッジ検証では、本書の思想がユーザー操作として自然かを検証する

この文書は「完成した詳細仕様」ではなく、**Editor を設計・実装する際の共通思想を固定するための土台**である。

---

## 13. 具体設計の叩き台

この章では、思想レベルから一歩進めて、実装直前に近い粒度で Editor の具体像を示す。

ここでの目的は、クラス名や関数名を最終確定することではなく、

* どの画面があるか
* どの状態があるか
* どの API が必要か
* どの操作がどう流れるか

を明確にすることである。

---

## 14. 画面構成案

### 14.1 全体レイアウト

MVP の基本レイアウトは 3 カラム + 下部パネルとする。

```text
┌──────────────────────────────────────────────────────────────┐
│ Header: project / save / sample / diagnostics summary        │
├───────────────┬───────────────────────────┬──────────────────┤
│ Structure     │ Main Editor / Detail      │ Constraints      │
│ Tree / List   │ Selected node inspector    │ List / Inspector │
│ + add buttons │ inline expression slots    │ + add constraint │
├───────────────┴───────────────────────────┴──────────────────┤
│ Bottom Panel: sample output / logs / validation / preview    │
└──────────────────────────────────────────────────────────────┘
```

### 14.2 各領域の責務

#### Header

* プロジェクト名
* 保存
* Sample 生成
* diagnostics 件数
* 完成度サマリ

#### Structure ペイン

* Sequence / Section / Repeat / Tuple / Scalar / Array / Choice のツリー表示
* Hole の可視化
* 追加ボタン
* 移動 / 並び替え

#### Main Editor / Detail

* 選択中ノードの詳細編集
* 名前・型・body・length・variant などの編集
* 式スロットの編集入口

#### Constraints ペイン

* 全制約の一覧
* 対象ノードに紐づく制約の絞り込み
* 制約追加
* 制約編集

#### Bottom Panel

* generated sample
* canonical preview
* validation / warnings
* operation log

---

## 15. 主要 UI コンポーネント

### 15.1 Preact コンポーネント境界

以下の単位で切る。

```ts
<AppShell />
  <HeaderBar />
  <StructurePane />
    <StructureTree />
      <StructureNodeView nodeId=... />
  <MainPane />
    <NodeInspector nodeId=... />
      <SlotField slotId=... />
      <ExprSlotField slotId=... />
  <ConstraintPane />
    <ConstraintList />
      <ConstraintCard constraintId=... />
    <ConstraintDraftPanel />
  <BottomPanel />
    <SamplePreview />
    <DiagnosticsPanel />
```

### 15.2 コンポーネントの原則

* `StructureNodeView` は node render projection だけを見る
* `ConstraintCard` は constraint render projection だけを見る
* 候補一覧や遷移可能操作は click 時に取得する
* UI 状態は local に持ちすぎず、editor store に集約する

---

## 16. EditorState の具体像

### 16.1 中央集約 state の例

```ts
interface EditorState {
  selectedNodeId: NodeId | null;
  selectedConstraintId: ConstraintId | null;
  selectedExprSource: SourceRef | null;

  openPopover: PopoverState | null;
  openModal: ModalState | null;

  pendingExprAction: PendingExprAction | null;
  draftConstraint: DraftConstraint | null;

  dragState: DragState | null;
  diagnosticsFilter: DiagnosticsFilter;

  samplePreview: SamplePreviewState;
}
```

### 16.2 PendingExprAction の例

```ts
type PendingExprAction =
  | {
      kind: "wrapBinary";
      targetExpr: ExprId;
      op: "+" | "-" | "*" | "/";
      phase: "select-lhs-or-rhs" | "select-other-side";
      fixedSide: "lhs" | "rhs";
    }
  | {
      kind: "wrapCall";
      targetExpr: ExprId;
      func: "min" | "max" | "abs";
      nextArgIndex: number;
      args: (ResolvedExpr | null)[];
    }
  | {
      kind: "replaceExpr";
      targetExpr: ExprId;
    };
```

### 16.3 DraftConstraint の例

```ts
type DraftConstraint =
  | {
      kind: "Range";
      target: PlaceRef | null;
      lower: ResolvedExpr | null;
      upper: ResolvedExpr | null;
    }
  | {
      kind: "TypeDecl";
      target: PlaceRef | null;
      expectedType: "Int" | "Str" | "Char" | null;
    }
  | {
      kind: "LengthRelation";
      target: PlaceRef | null;
      length: ResolvedExpr | null;
    }
  | {
      kind: "Relation";
      lhs: ResolvedExpr | null;
      op: "<" | "<=" | "==" | "!=" | ">=" | ">" | null;
      rhs: ResolvedExpr | null;
    };
```

ここで重要なのは、DraftConstraint は AST に入れず、EditorState にのみ存在することである。

---

## 17. wasm 側 API の具体像

### 17.1 公開したい API の分類

#### A. Render projection API

```rust
project_structure_outline() -> StructureOutlineView
project_node_render(node_id: NodeId) -> NodeRenderView
project_constraint_render(constraint_id: ConstraintId) -> ConstraintRenderView
project_expr_render(expr_id: ExprId) -> ExprRenderView
project_slot_render(slot_id: SlotId) -> SlotRenderView
project_diagnostics() -> DiagnosticsView
project_completeness() -> CompletenessView
```

#### B. Action projection API

```rust
project_node_actions(node_id: NodeId) -> NodeActionMenu
project_hole_candidates(node_id: NodeId) -> HoleCandidateMenu
project_expr_actions(source: SourceRef) -> ExprActionMenu
project_expr_slot_candidates(slot_id: SlotId) -> ExprCandidateMenu
project_constraint_target_candidates(kind: ConstraintKind) -> TargetCandidateMenu
```

#### C. Operation API

```rust
apply_action(action: EditorAction) -> ApplyResult
preview_action(action: EditorAction) -> PreviewResult
undo() -> UndoResult
redo() -> RedoResult
```

#### D. Sample API

```rust
generate_sample() -> SampleGenerationResult
render_canonical_spec() -> String
```

---

### 17.2 EditorAction の具体像

```rust
enum EditorAction {
    FillHole { target: NodeId, fill: FillContent },
    ReplaceNode { target: NodeId, replacement: FillContent },
    MoveNode { node: NodeId, new_parent: NodeId, index: usize },
    RemoveNode { node: NodeId },

    AddConstraint { constraint: ConstraintDef },
    UpdateConstraint { id: ConstraintId, patch: ConstraintPatch },
    RemoveConstraint { id: ConstraintId },

    SetNodeName { node: NodeId, name: String },
    SetTypeDecl { target: PlaceRef, expected: ExpectedType },
    SetArrayLengthExpr { node: NodeId, expr: Expression },

    ReplaceExpr { target: ExprId, expr: Expression },
    WrapExprBinary { target: ExprId, op: ArithOp, other: Expression, side: Side },
    WrapExprCall { target: ExprId, func: FuncName, extra_args: Vec<Expression> },
}
```

---

## 18. Projection の返り値イメージ

### 18.1 NodeRenderView

```ts
interface NodeRenderView {
  nodeId: NodeId;
  label: string;
  kindLabel: string;
  isHole: boolean;
  expectedKindHint: string | null;
  badges: string[];
  children: ChildEntryView[];
  diagnostics: DiagnosticSummary[];
  clickableRegions: ClickableRegion[];
}
```

### 18.2 ExprRenderView

```ts
interface ExprRenderView {
  exprId: ExprId;
  root: InlineNode;
}

type InlineNode =
  | {
      kind: "token";
      text: string;
      source: SourceRef;
      clickable: boolean;
    }
  | {
      kind: "group";
      source: SourceRef;
      children: InlineNode[];
    };
```

### 18.3 ExprActionMenu

```ts
interface ExprActionMenu {
  source: SourceRef;
  actions: {
    id: string;
    label: string;
    kind: "replace" | "wrap-binary" | "wrap-call" | "delete";
    metadata?: unknown;
  }[];
}
```

ここでは候補の完全列挙を render 時に埋め込まず、click 時取得に徹する。

---

## 19. 具体的な操作フロー

### 19.1 `N: int` を作る

1. Structure ペインで `+ 変数を追加`
2. wasm に `FillHole` ないし `AddScalar` 相当を送る
3. `NodeInspector` で名前 `N`
4. `TypeDecl(Int)` を追加

UI 的には:

* structure に `N`
* constraints に `N is int`

が見える。

---

### 19.2 `A: int[N]` を作る

1. Structure ペインで `+ 配列を追加`
2. 名前 `A`
3. 要素型 `int`
4. 長さ slot をクリック
5. 候補として `N`, 定数, 式テンプレートを表示
6. `N` を選択
7. `SetArrayLengthExpr(A, Ref(N))`

---

### 19.3 `A: int[N/2]` を作る

1. `A` の長さ slot をクリック
2. `N` を選択
3. 式表示上の `N` をクリック
4. action menu から `/x`
5. RHS 候補から `2`
6. `WrapExprBinary(target=N, op=/, other=2, side=rhs)` を適用

この間、AST には `N/?` は存在しない。
PendingExprAction が UI state にのみ存在し、確定時に一発更新する。

---

### 19.4 `1 <= A[i] <= 10^9` を追加する

1. Constraints ペインで `+ Range`
2. target として `A[i]` を選択
3. lower に `1`
4. upper に `10^9`
5. 確定して `AddConstraint`

`A[i]` は PlaceRef として扱う。

---

### 19.5 Tree 入力を追加する

MVP では raw NodeKind を直接組み立てさせず、Builder 的ショートカットを使う。

1. `+ 辺リスト`
2. `N`, `N-1`, endpoints を使うテンプレート選択
3. 必要なら `Tree` property を追加

内部では

* `Tuple(N)`
* `Repeat(N-1, Tuple(u,v))`
* `Property(Tree)`

に展開される。

---

## 20. UI で優先的に用意すべきテンプレート

### 20.1 Structure テンプレート

* 単一変数
* 配列
* グリッド
* tuple 行
* repeat ブロック
* section
* 複数テストケース
* 辺リスト
* クエリ列

### 20.2 Expression テンプレート

* 定数
* 参照
* `x + y`
* `x - y`
* `x * y`
* `x / y`
* `max(x, y)`
* `min(x, y)`
* `len(x)`

### 20.3 Constraint テンプレート

* Range
* TypeDecl
* LengthRelation
* Relation
* Distinct
* Sorted
* Property(Tree)
* Property(Simple)
* SumBound

---

## 21. Diagnostics の設計

### 21.1 diagnostics のレベル

* Error: sample 生成や canonical rendering ができない
* Warning: 意味は通るが不自然 / 不完全
* Info: 補助的提案

### 21.2 代表例

#### Error

* root に Hole が残っている
* 必須の length が未設定
* 制約 target が不正

#### Warning

* 型宣言がない
* 配列長に Range がなく sample 生成が不安定
* Tree property はあるが辺数 relation が未設定

#### Info

* このパターンは辺リストテンプレートでも作れる
* この配列に Distinct を足す候補がある

---

## 22. Sample 生成との接続

### 22.1 Bottom Panel に出すもの

* canonical input format preview
* generated sample
* generation guarantee level
* warnings

### 22.2 ボタンの意味

#### Validate

AST と constraints の整合性検査のみ

#### Generate Sample

現在の AST から sample を 1 件生成

#### Regenerate

同じ仕様から別 sample を再生成

これにより、Editor は「仕様を作って終わり」ではなく、すぐ結果確認できるようにする。

---

## 23. MVP 実装順の具体案

### Phase 1

* Structure ペイン
* Scalar / Array / Tuple / Sequence / Repeat / Hole
* Constraint: TypeDecl / Range / LengthRelation
* expression slot: 定数・参照・`/x` まで
* sample preview

### Phase 2

* Section / Choice
* Relation / Distinct / Sorted / Property
* 辺リスト / グリッド / 複数テストケーステンプレート
* 中程度の複雑さの式

### Phase 3

* クエリ列テンプレート強化
* 高度な制約テンプレート
* 複数 sample 比較
* 保存/読み込み/共有

---

## 24. 最終的な実装方針の要約

今回の Editor は、次の設計を採る。

* AST は 1 種類
* Structure の未完成は Hole で表す
* Expression の未完成は pending action で表す
* Constraint の未完成は draft form で表す
* Preact は体験、wasm は意味論を担う
* render 時は局所 projection、候補列挙は click 時 projection
* ユーザーには raw AST より template-driven な編集を見せる

この方針により、理論と操作性の両立を図る。

---

## 25. AI Agent 向け実装契約

この章は、AI Agent が実装時に迷わないようにするための**契約仕様**である。

ここでいう契約とは、

* どの状態をどこに置くか
* 何を許し何を許さないか
* 各 API が何を受け取り何を返すか
* どの層が何を知らなくてよいか

を固定するためのものである。

本章に反する実装を行う場合は、必ず理由と代替案を設計書に明記すること。

---

## 26. 未完成状態の配置契約

### 26.1 基本原則

未完成状態は、意味論上必要なものだけを AST に置く。

それ以外の未完成は EditorState に置く。

この原則により、AST を過剰に editor 専用の一時状態で汚染しない。

---

### 26.2 Structure 未完成の契約

#### 採用

Structure の未完成は AST に置く。

具体的には `NodeKind::Hole` を用いる。

#### 理由

Structure の未完成は、

* 位置
* 順序
* 親子関係
* スロット種別

に意味があるため、UI state だけで持つと構造そのものを UI が肩代わりしてしまうから。

#### 許される状態

* `Sequence` の child に Hole が存在してよい
* `Section.body` に Hole が存在してよい
* `Repeat.body` に Hole が存在してよい
* `Choice.variant` 内に Hole が存在してよい

#### 禁止

* Hole を UI の見た目だけで表現し、AST から消すこと
* 構造上意味のある空位置を EditorState にしか持たないこと

---

### 26.3 Expression 未完成の契約

#### 採用

Expression の未完成は AST に置かない。

具体的には `N / ?` や `max(?, 1)` のような partial expr を AST に入れない。

未完成状態は `PendingExprAction` として EditorState に置く。

#### 理由

式編集は局所的な変形として表現できるため、

* 対象式
* 適用したい操作
* 未確定引数

を UI state で持てば十分だから。

#### 許される状態

EditorState には以下が存在してよい。

* `wrapBinary(target=N, op=/, rhs未選択)`
* `wrapCall(target=N, func=max, arg2未選択)`
* `replaceExpr(target=exprId, candidate未確定)`

#### 禁止

* AST に expression hole variant を追加すること
* AST に editor 専用の未完成式を保存すること
* `Expression` を UI 入力途中の一時木として使うこと

#### 例

`A: int[N/2]` を作るとき:

1. AST 上の長さ式は最初 `N`
2. UI で `N` を選択
3. PendingExprAction = `/x`
4. RHS に `2` を選択
5. その瞬間に `N/2` の完成式を生成し AST を置換

途中で AST に `N/?` は現れない。

---

### 26.4 Constraint 未完成の契約

#### 採用

Constraint の未完成は AST に置かず、`DraftConstraint` として EditorState に置く。

#### 理由

Constraint 作成途中は構造的空位置というより、1件のフォーム入力途中だから。

#### 許される状態

* target 未設定の Range draft
* lower だけ設定済みの Range draft
* TypeDecl の expectedType 未設定 draft

#### 禁止

* ConstraintAST に DraftRange や ConstraintHole を入れること
* 未完成 constraint を AST 一覧に混在させること

---

## 27. AST 契約

### 27.1 AST の種類

AST は 1 種類である。

#### 含むもの

* StructureAST（Hole 許容）
* ConstraintAST（完成形のみ）
* Expression（完成形のみ）

#### 含まないもの

* UI 選択状態
* hover/focus
* メニュー開閉状態
* draft constraint
* pending expression action
* drag state

---

### 27.2 AST の役割

AST は以下の正本である。

* canonical rendering
* sample generation 入力
* validation の意味論対象
* projection の入力

AST は Editor 用の ephemeral state を持ってはならない。

---

### 27.3 AST 不変条件

以下を満たすこと。

#### Structure

* root は常に存在する
* NodeId は stable である
* NodeKind は明示的である
* Structure Hole は正規のノードとして扱う

#### Expression

* 常に完成式である
* partial expr を含まない
* slot 内に入る式は expected sort を満たす

#### Constraint

* 常に完成済みである
* target / lhs / rhs / lower / upper 等の必須フィールドが埋まっている

---

## 28. EditorState 契約

### 28.1 EditorState の責務

EditorState は次を持つ。

* selection
* pending action
* draft form
* popover/modal 開閉
* drag state
* 一時 preview

### 28.2 EditorState の非責務

EditorState は次を持たない。

* canonical meaning
* sample generation の入力仕様本体
* 完成済み structure / constraint / expression

### 28.3 EditorState の寿命

* 画面遷移やモーダル終了で消えてよい
* 保存対象ではない
* undo/redo の正本ではない

必要なら UI セッション復元用に別永続化を考えてよいが、AST と同一視してはならない。

---

## 29. Projection API 契約

### 29.1 原則

Projection API は read-only である。

* 状態変更しない
* 副作用を起こさない
* 同じ AST / 同じ EditorState から同じ結果を返す

---

### 29.2 Render Projection 契約

#### 役割

描画に必要な最小情報だけ返す。

#### 入力

* AST
* 必要なら EditorState の一部（selection など）

#### 出力

* label
* kind
* children
* diagnostics summary
* clickable region
* source ref

#### 禁止

* 候補一覧の全列挙
* 重い推論結果の過剰埋め込み
* UI レイアウトそのものの決定

---

### 29.3 Action Projection 契約

#### 役割

クリック時に必要な候補や遷移可能操作を返す。

#### 入力

* node_id / expr_id / slot_id / source_ref
* 必要な文脈

#### 出力

* 遷移可能操作
* 候補式
* 候補 target
* 操作テンプレート

#### 禁止

* AST 更新
* draft の暗黙確定

---

### 29.4 Global Projection 契約

#### 役割

全体一覧と状態サマリ。

#### 出力例

* structure outline
* constraint list summary
* diagnostics list
* completeness summary

#### 禁止

* 各ノードの重い候補列挙まで抱え込むこと

---

## 30. Operation API 契約

### 30.1 Operation の役割

Operation API は唯一 AST を更新してよい層である。

### 30.2 Operation の入力

* 明示的 action
* 必要なら action に付随する payload

### 30.3 Operation の出力

* 成功/失敗
* 作成/更新/削除された ID
* diagnostics 再計算のヒント

### 30.4 禁止

* UI に依存する判断を持つこと
* モーダル遷移や focus 遷移を持つこと
* 「たぶんこれだろう」で draft を補完すること

---

## 31. Preact / wasm 境界契約

### 31.1 wasm 側が担うもの

* AST
* validation
* projection
* operation
* sample generation
* stable id を持つ意味論的オブジェクト

### 31.2 Preact 側が担うもの

* コンポーネント構成
* EditorState
* pending action
* draft constraint
* 表示遷移
* menu / modal / popover
* drag and drop

### 31.3 禁止

#### wasm 側でやってはいけないこと

* hover 管理
* popover 開閉管理
* pending action の UI 遷移管理

#### Preact 側でやってはいけないこと

* AST の意味論的整合性判定を独自に持つこと
* sample generation を独自に再実装すること
* projection を自前で組み立てること

---

## 32. 式スロット契約

### 32.1 スロットの定義

式スロットとは、完成した expression を受け取る入力位置である。

例:

* array length
* range lower
* range upper
* relation lhs / rhs
* repeat count

### 32.2 スロットが持つべき情報

各スロットは最低限次を持つ。

* slot_id
* label
* expected_sort
* scope
* slot_rule
* current expr (存在するなら)

### 32.3 expected_sort

MVP では以下を採用する。

* IntExpr
* BoolExpr
* PlaceExpr

### 32.4 slot_rule

例:

* NonNegative
* NoSelfReference
* ComparableWithTarget
* AllowIndexRef
* IntegerOnly

### 32.5 契約

* スロットに入るのは常に完成式
* 未完成式は slot 内に入れてはならない
* スロットの候補列挙は click 時 projection で取得する

---

## 33. PendingExprAction 契約

### 33.1 役割

式編集の途中状態を保持する。

### 33.2 含むべき情報

* target expr id
* 操作種別
* 現在フェーズ
* 未充足引数
* 一時的な候補選択結果

### 33.3 含んではならないもの

* AST の正本コピー
* 完成した expression の別管理
* projection 結果そのもの

### 33.4 フェーズ遷移

例: `/x`

1. target expr 選択
2. op=`/` を選択
3. rhs 候補選択待ち
4. rhs 確定
5. completed expression を生成
6. Operation API に渡して AST 更新
7. PendingExprAction 破棄

---

## 34. DraftConstraint 契約

### 34.1 役割

制約 1 件の構築途中状態を保持する。

### 34.2 必須フィールド

* kind
* 各フィールドの current value
* completion 状態
* validation message

### 34.3 完了条件

DraftConstraint は kind ごとの必須項目が埋まったときのみ `AddConstraint` に変換してよい。

例:

#### Range

* target != null
* lower != null
* upper != null

#### TypeDecl

* target != null
* expectedType != null

---

## 35. SourceRef 契約

### 35.1 役割

UI のクリック可能領域と AST 上の意味論的対象を結ぶ。

### 35.2 最低限必要な粒度

MVP では以下を持てばよい。

* NodeRef(node_id)
* ExprRef(expr_id)
* SlotRef(slot_id)
* ConstraintRef(constraint_id)

### 35.3 禁止

* DOM 位置だけで対象を特定すること
* text offset ベースに依存すること

---

## 36. 具体的な契約付き操作例

### 36.1 `N: int`

#### UI

* `+ 変数`
* 名前 `N`
* TypeDecl `int`

#### AST

* Structure に Scalar `N`
* Constraint に TypeDecl(N, Int)

#### EditorState

* 一時的 selection はあってよい
* draft constraint は確定後に破棄

---

### 36.2 `A: int[N]`

#### UI

* `+ 配列`
* 名前 `A`
* 要素型 `int`
* 長さ slot で `N`

#### 契約

* 長さ slot は IntExpr slot
* `N` は completed expression
* AST には `Expression::Var(N)` が入る
* pending expr action は確定時点で消える

---

### 36.3 `A: int[N/2]`

#### 契約

* AST に `N/?` は出ない
* `/x` 選択中は EditorState にのみ未完成状態がある
* 右辺 `2` が確定した瞬間に `WrapExprBinary` を実行

---

### 36.4 `1 <= A[i] <= 10^9`

#### 契約

* 制約追加途中は DraftConstraint(Range)
* target=`A[i]`, lower=`1`, upper=`10^9` がそろうまで AST 追加しない
* そろった瞬間に AddConstraint

---

## 37. AI Agent 実装ルール

AI Agent は以下を守ること。

### 必須

* AST を唯一の正本として扱う
* 未完成 expression を AST に導入しない
* draft constraint を AST に混ぜない
* Structure Hole は正規ノードとして扱う
* projection と operation を混ぜない
* wasm に UI state を押し込まない

### 望ましい

* まず基本ケースを通す
* 高レベルテンプレートを先に作る
* click 時候補列挙を基本にする
* diagnostics を常に見えるようにする

### 禁止

* convenient だからという理由で AST に UI state を混ぜること
* Preact 側で意味論的制約を再実装すること
* render projection に全候補を埋め込むこと
* partial expr を安易に追加すること

---

## 38. 実装前に AI Agent が確認すべきチェックリスト

1. この未完成状態は本当に AST に必要か
2. それは Structure の空位置か、ただの UI 入力途中か
3. その候補列挙は render 時に必要か、click 時でよいか
4. この state は保存対象か、一時状態か
5. この責務は wasm に置くべきか、Preact に置くべきか
6. この UI は基本ケースを速くしているか、それとも重くしているか
7. この追加で AST core の意味論が汚れていないか

---

## 39. この章の使い方

この章は、AI Agent が

* 画面実装
* API 実装
* wasm 境界設計
* state 分離
* 式編集フロー
* 制約追加フロー

を迷わず進めるための契約である。

実装タスクを切るときは、必ず

* AST に何が入るか
* EditorState に何が入るか
* Projection は何を返すか
* Operation は何を更新するか

をこの契約に照らして確認すること。
