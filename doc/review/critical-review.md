# 破壊的設計レビュー — StructureAST / ConstraintAST / ProjectionAPI / Operation

> Sprint 3 — CriticalReviewAgent による設計評価
> 対象: domain-model.md, projection-operation.md（Sprint 2 成果物）
> 比較参照: ast-draft.md（Sprint 1 設計ドラフト）

---

## 重大欠陥（設計の根幹に関わるもの）

### S-1. 構造と制約の分離原則が StructureAST 自身で破られている

**問題の説明:**

設計文書は「StructureAST = 何がどこにあるか」「ConstraintAST = その構造にどのような制約が課されているか」と明確に分離を宣言している。しかし NodeKind の定義を見ると、以下のフィールドが StructureAST 側に存在する:

- `Scalar.typ: ExpectedType`（型は制約情報）
- `Array.element_type: ExpectedType`（同上）
- `Matrix.element_type: ExpectedType`（同上）
- `Matrix.separator: Separator`（区切り文字はレンダリング/制約に属する情報）

一方で ConstraintAST 側にも `TypeDecl { target, expected: ExpectedType }` が存在する。つまり**型情報が二重に管理される**。

**破綻する具体例:**

1. `Scalar { name: "N", typ: ExpectedType::Int }` として StructureAST に格納された後、ConstraintAST で `TypeDecl { target: Ref(N), expected: ExpectedType::Str }` が追加された場合、どちらが正か？
2. `Matrix { separator: Separator::Space, ... }` としてスペース区切りで定義した後、実はこの行列は文字グリッドで separator=None が正しいと判明した場合、separator の変更は StructureAST の編集なのか、制約の修正なのか？ Operation は Action のどれでこれを変更すべきか？ `ReplaceNode` でマトリクス全体を差し替えるしかないが、それは名前・参照すべてを破壊する。

**推奨される修正方向:**

- `ExpectedType` を NodeKind から除去し、すべての型情報を ConstraintAST の `TypeDecl` に一元化する。
- `Separator` を ConstraintAST 側に `RenderHint { target, hint: RenderHintKind }` のような制約として移動するか、Canonical Rendering 層の責務にする。
- NodeKind は純粋に構造（名前・子ノードの接続）のみを持つべき:
  ```rust
  Scalar { name: Ident },
  Array { name: Ident, length: Reference },
  Matrix { name: Ident, rows: Reference, cols: Reference },
  ```

---

### S-2. 制約の NodeId 管理が未定義 — RemoveConstraint が機能しない

**問題の説明:**

`RemoveConstraint { constraint_id: NodeId }` は制約を NodeId で指定して削除する。しかし `ConstraintSet` の定義は:

```rust
pub struct ConstraintSet {
    pub constraints: HashMap<NodeId, Vec<Constraint>>,
    pub global_constraints: Vec<Constraint>,
}
```

これは「構造ノードの NodeId → その構造ノードに紐づく制約のリスト」というマッピングであり、**制約自身には NodeId が付与されていない**。`Vec<Constraint>` の中の個別の Constraint に ID がなければ、`RemoveConstraint` はどの制約を指しているのか特定できない。

**破綻する具体例:**

ABC284-C で N に以下の2つの制約がある場合:
- `Range { target: Ref(N), lower: Lit(1), upper: Lit(100) }`
- `Relation { lhs: Var(Ref(M)), op: Le, rhs: BinOp(Mul, Var(Ref(N)), ...) }`

`RemoveConstraint { constraint_id: NodeId(3) }` が渡されたとき、NodeId(3) は N のノード ID であって制約の ID ではない。制約リスト `[Range, Relation]` のどちらを削除すべきか Operation は判断できない。

また `global_constraints: Vec<Constraint>` に格納された制約（Guarantee 等）は構造ノードに紐づかないため、NodeId による指定自体が不可能。

**推奨される修正方向:**

- 制約に一意の `ConstraintId` を付与する:
  ```rust
  pub struct ConstraintId(pub u64);

  pub struct ConstraintSet {
      pub by_node: HashMap<NodeId, Vec<ConstraintId>>,
      pub constraints: HashMap<ConstraintId, Constraint>,
      pub global: Vec<ConstraintId>,
  }
  ```
- `RemoveConstraint` を `RemoveConstraint { constraint_id: ConstraintId }` に変更する。
- 制約追加時に `ApplyResult` が `created_constraint_ids` を返すようにする。

---

### S-3. FillContent → NodeKind の変換にドメイン知識が暗黙に必要で、Operation の責務が爆発する

**問題の説明:**

`FillContent` は13以上のバリアントを持ち（Scalar, Array, Grid, EdgeList, QueryList, TriangularBlock, Section, OutputSingleValue, OutputYesNo, OutputModular, OutputConditional, OutputMultiLine, Interactive）、一方 NodeKind はわずか8バリアントしかない。

つまり Operation は `FillContent::EdgeList` を受け取ったとき、以下の内部変換を暗黙に行う必要がある:
- `Repeat { count: M, body: [Tuple { elements: [Ref(u), Ref(v)] }] }` + 個別の Scalar ノード u, v の生成 + LengthRelation + Range 制約の自動追加

この「FillContent → NodeKind の複数ノード + 制約への展開」はドメイン固有のマクロ展開であり、Operation 層の中にグラフ理論の知識や三角行列の構造知識が埋め込まれることを意味する。

**破綻する具体例:**

1. `FillContent::QueryList { query_count: LengthSpec::RefVar(Q), variants: [QueryVariant { type_code: "1", params: [Int, Int] }, QueryVariant { type_code: "2", params: [Int] }] }` — この変換には「type_code に応じて body の Tuple 構造が変わる繰り返し」が必要だが、NodeKind に条件分岐付き Repeat は存在しない。Repeat は固定の body を繰り返すだけ。**可変構造のクエリは現 NodeKind で表現不能**。
2. `FillContent::TriangularBlock { size: N }` — 「行ごとに列数が減少する」構造は `Matrix` でも `Repeat + Tuple` でも正確に表現できないと domain-model.md §4.1 自身が認めている。FillContent にバリアントがあるが、展開先の NodeKind が存在しない。

**推奨される修正方向:**

- **選択肢A（推奨）**: FillContent を廃止し、Action は NodeKind レベルの操作のみを受け付ける。高レベルの構築パターン（EdgeList, QueryList 等）は Operation の外に「Builder / Template」層として分離する:
  ```rust
  // Builder 層（Operation の外）
  fn build_edge_list(count: Reference, weighted: bool) -> Vec<Action>;
  ```
  これにより Operation は NodeKind と Constraint の整合性のみに責任を持つ。

- **選択肢B**: NodeKind に `ConditionalRepeat` や `TriangularMatrix` を追加して FillContent との対応を 1:1 に近づける。ただしこれは NodeKind の最小性に反する。

---

## 中程度の欠陥（修正可能だが放置すると問題になるもの）

### M-1. Canonical rendering が HashMap の順序非保証に依存して破綻する

**問題の説明:**

Canonical rendering は「StructureAST のノードを決定論的な順序で出力する」と述べているが、StructureAST のルート定義は:

```rust
pub struct StructureAst {
    pub root: NodeId,
    pub nodes: HashMap<NodeId, StructureNode>,
}
```

`HashMap` は Rust で反復順序が非決定的である（ランダム化ハッシュ）。`nodes()` を呼ぶたびにノードの列挙順序が変わりうる。

もちろん Sequence.children や Tuple.elements の `Vec<NodeId>` は順序を保持するが、ノードの実体を取得するためには `nodes.get(&id)` を繰り返す必要があり、HashMap 全体をイテレートする `nodes()` の実装では順序が壊れる。

**破綻する具体例:**

同一の AST を2回 serialize した場合、HashMap の内部順序の違いにより異なるテキストが出力される。テスト生成されたサンプルの一意性検証が失敗する。

**推奨される修正方向:**

- `HashMap<NodeId, StructureNode>` を `IndexMap<NodeId, StructureNode>`（挿入順保持）に変更する。
- または Arena アロケータ（`Vec<StructureNode>` + `NodeId` を index として使う）に変更する。Arena 方式は O(1) アクセスかつ順序保証で、AST に最適:
  ```rust
  pub struct StructureAst {
      pub root: NodeId,
      pub arena: Vec<StructureNode>,  // NodeId(n) → arena[n]
  }
  ```

---

### M-2. preview_action が Projection/Operation の責務境界を侵害する

**問題の説明:**

`ProjectionQueryAPI::preview_action(&self, action: &Action) -> Result<PreviewResult, OperationError>` は「Action を適用した場合の結果をプレビューする（dry-run）」と定義されている。しかしこれを実現するには:

1. Action の前提条件チェック（型互換性、slot の空き状況、依存関係）→ これは Operation の検証ロジック
2. 新ノード生成のシミュレーション → これは Operation の適用ロジック
3. 制約の影響範囲算出 → これは Operation の整合性チェックロジック

つまり preview_action を実装するには、Operation.apply のロジックをほぼ完全に Projection 側に複製する必要がある。これは「Projection は読むだけ」という設計原則に反する。

**破綻する具体例:**

Operation に新しい Action バリアントを追加した場合、preview_action も同時に更新しなければならない。しかし preview_action は ProjectionQueryAPI trait にあるため、Operation を変更するたびに Projection 側のコードも変更が必要になり、責務分離の利点（片方を変更しても他方に影響しない）が消失する。

**推奨される修正方向:**

- `preview_action` を ProjectionQueryAPI から除去し、Operation trait 側に移動する:
  ```rust
  pub trait Operation {
      fn apply(&mut self, action: Action) -> Result<ApplyResult, OperationError>;
      fn preview(&self, action: &Action) -> Result<PreviewResult, OperationError>;
  }
  ```
- `preview` は `&self`（不変参照）なので状態は変更せず、Operation 内部のロジックを再利用できる。

---

### M-3. 生成可能保証が制約の相互作用を考慮していない

**問題の説明:**

§5.3 の「生成可能保証」は3条件を挙げている:
1. 制約系が充足可能（Range の下界 ≤ 上界、等）
2. 依存グラフが非巡回
3. 各制約種別が生成器の対応範囲内

しかしこれは各制約を**個別に**見たときの充足可能性であり、**制約の組み合わせ**による非充足（unsat）を考慮していない。

**破綻する具体例:**

1. `Range(A_i, 1, 10)` + `Distinct(A)` + `LengthRelation(len(A) = N)` + `Range(N, 1, 100)` — N=11 以上のとき、1〜10 の範囲で 11 個以上の相異な値を取ることは不可能。個別の制約はすべて well-formed だが、組み合わせると unsat。
2. `Property(Graph, Connected)` + `Range(M, 0, N-1)` — M=0 なら辺が0本で連結不可能（N≥2 のとき）。Property と Range の相互作用が生成器を破綻させる。
3. `Sorted(A, Ascending)` + `Distinct(A)` は充足可能だが、`Sorted(A, Ascending)` + `Distinct(A)` + `Range(A_i, 1, 3)` + `LengthRelation(len(A), N)` + `Range(N, 1, 10)` は N>3 で unsat。

設計文書は「正常な競プロ問題ではサイクルは発生しない」と述べているが、制約の組み合わせ非充足は正常な問題でも起こりうる入力ミスであり、これを検出する仕組みが未定義。

**推奨される修正方向:**

- 生成可能保証の条件に「4. 制約の相互作用が充足可能」を追加し、以下の具体的検査を定義する:
  - `Distinct` + `Range` → 値域の幅 ≥ 配列長 の検証
  - `Property(Connected)` + `Range(M)` → M ≥ N-1 の検証
  - `Sorted` + `Distinct` + `Range` → 値域の幅 ≥ 配列長 の検証
- 検査不能な組み合わせは「生成可能保証の対象外」として明示し、ランタイムで上限付きリトライ + タイムアウトで対応する方針を定義する。

---

### M-4. Tuple.elements が Vec\<Reference\> — 変数の宣言位置と参照の区別が曖昧

**問題の説明:**

`Tuple { elements: Vec<Reference> }` は「同一行の変数組」を表現する。ヘッダ行 `N M K` は Tuple であり、elements は `[VariableRef(n1), VariableRef(n2), VariableRef(n3)]` となる。

しかしこれは N, M, K がすでに Scalar ノードとして別に存在することを前提としている。つまり Tuple は変数を**宣言**するのではなく**参照**するだけ。では N, M, K の Scalar ノードはどこに配置されるのか？

- Sequence の children に Scalar ノードを直接並べるのか？ → Tuple の存在意義がなくなる
- Scalar ノードは Tuple の「子」として木構造上に配置されるのか？ → だが Tuple.elements は `Vec<Reference>` であり `Vec<NodeId>` ではないため、親子関係を表現していない

**破綻する具体例:**

ヘッダ行 `N M` を構築するとき:
1. Scalar("N") を NodeId=1 で作成
2. Scalar("M") を NodeId=2 で作成
3. Tuple { elements: [VariableRef(1), VariableRef(2)] } を NodeId=3 で作成

この場合、Scalar(N) と Scalar(M) は木構造上どこの子か？ Tuple の子なら、children slot が必要だが Tuple には `elements: Vec<Reference>` しかない。Sequence の直接の子なら、Tuple と Scalar が同レベルに並ぶ奇妙な木構造になる。

**推奨される修正方向:**

- Tuple の elements を `Vec<NodeId>` に変更し、Tuple が子ノード（Scalar 等）を直接保持する構造にする:
  ```rust
  Tuple { elements: Vec<NodeId> }
  ```
  Tuple の各 element は子ノードとして Scalar/Hole を持つ。

---

### M-5. 可変構造クエリ（型コード分岐）が NodeKind で表現不能

**問題の説明:**

`FillContent::QueryList` は `variants: Vec<QueryVariant>` を持ち、各 QueryVariant は `type_code` と `param_types` を持つ。例えば:

- type=1: `1 x y`（2パラメータ）
- type=2: `2 x`（1パラメータ）

しかし `Repeat { count, body }` は body が固定の `Vec<NodeId>` であり、繰り返しごとに body の構造が変わるパターンを表現できない。

**破綻する具体例:**

ABC299-E のクエリや、多くの問題で見られるクエリ形式:
```
Q
c_1 x_1 [y_1]   ← c=1なら3要素、c=2なら2要素
c_2 x_2 [y_2]
...
```

これを Repeat + Tuple で表現しようとすると、Tuple の要素数が行ごとに異なるため、単一の Repeat body では不可能。サイト調査 I-4 パターンとして認識されているのに、NodeKind で表現できない。

**推奨される修正方向:**

- NodeKind に `TaggedUnion` を追加する:
  ```rust
  TaggedUnion {
      tag: Reference,          // 型コード変数
      variants: Vec<(Literal, Vec<NodeId>)>,  // (型コード値, body)
  }
  ```
- または Repeat の body を `Vec<NodeId>` から `RepeatBody` enum に拡張:
  ```rust
  enum RepeatBody {
      Fixed(Vec<NodeId>),
      Tagged { tag: Reference, branches: Vec<(Literal, Vec<NodeId>)> },
  }
  ```

---

## 軽微な懸念（改善の余地があるもの）

### L-1. Hole の hole_id と StructureNode.id の冗長

**問題の説明:**

`StructureNode { id: NodeId, kind: NodeKind }` の NodeKind::Hole は `Hole { hole_id: HoleId, expected_kind }` を持ち、`HoleId = NodeId` と定義されている。StructureNode.id と Hole.hole_id が同一値を持つことが期待されるが、型システムでこれを強制していない。

**破綻する具体例:**

`StructureNode { id: NodeId(5), kind: Hole { hole_id: NodeId(7), ... } }` が構築された場合、NodeId(5) と NodeId(7) のどちらでこの Hole を参照すべきか不明。ConstraintAST が NodeId(7) で制約を付けた場合、StructureAST.nodes.get(&NodeId(7)) では見つからない。

**推奨される修正方向:**

- `Hole` から `hole_id` を除去し、常に `StructureNode.id` を使う:
  ```rust
  Hole { expected_kind: Option<NodeKindHint> }
  ```

---

### L-2. ProjectedNodeKind がドメイン固有ヒューリスティックを Projection に持ち込む

**問題の説明:**

NodeKind は 8 バリアントだが、ProjectedNodeKind は 12 バリアント（Root, Section, Header, ScalarVar, ArrayVar, GridVar, **EdgeList, QueryList, MultiTestCase, TriangularBlock**, Constraint, Hole）を持つ。

EdgeList, QueryList, MultiTestCase, TriangularBlock は NodeKind に存在せず、Projection が「この Repeat + Tuple の組み合わせは EdgeList である」と推論する必要がある。これは Projection 層にドメイン固有の判定ロジックを持ち込む。

**破綻する具体例:**

`Repeat(M, Tuple(u, v))` が EdgeList なのか、単なる座標リストなのかを Projection は区別できない。Property("simple_graph") 制約の有無で判断する場合、Projection が ConstraintAST の意味解析を行うことになり、「薄い層」の原則に反する。

**推奨される修正方向:**

- NodeKind に semantic tag（`annotation: Option<SemanticTag>`）を追加し、構築時に EdgeList 等のタグを明示する。Projection はタグをそのまま参照するだけにする。
- または ProjectedNodeKind を NodeKind と 1:1 対応にし、ドメイン固有の分類は View 層に委ねる。

---

### L-3. Expression の評価意味論が未定義

**問題の説明:**

`Expression` 型は `Lit`, `Var`, `BinOp`, `Pow`, `FnCall` を持つが、これを実際にどう評価するかの定義がない。特に:

- `FnCall { name: "min", args: [Var(N), Lit(100)] }` の評価タイミングは？ 生成時に N が確定してから？
- `BinOp { op: Div, lhs: Var(N), rhs: Lit(0) }` のゼロ除算は？
- `Pow { base: Lit(10), exp: Lit(18) }` は i64 でオーバーフローする（10^18 は i64 範囲内だが、10^19 は溢れる）。

**破綻する具体例:**

`Range { target: Ref(M), lower: Lit(0), upper: BinOp(Div, BinOp(Mul, Var(N), BinOp(Sub, Var(N), Lit(1))), Lit(2)) }` — つまり `M ≤ N(N-1)/2`。N=0 のときは `0 * -1 / 2 = 0` で問題ないが、整数除算の切り捨て方向や負数の扱いが未定義。

**推奨される修正方向:**

- Expression に評価関数の仕様を定義する:
  ```rust
  impl Expression {
      fn eval(&self, env: &HashMap<NodeId, i64>) -> Result<i64, EvalError>;
  }
  ```
- EvalError として `DivByZero`, `Overflow`, `UnresolvedVar` を定義する。
- 評価は常に i128 で行い、最終的に i64 に収まるかチェックする（競プロでは 10^18 が頻出）。

---

### L-4. AddConstraint が Hole への制約追加を拒否するが、ast-draft.md の理念と矛盾する

**問題の説明:**

projection-operation.md の §3.2 で `AddConstraint` を Hole に対して実行すると `InvalidOperation` エラーになると定義されている:

> "Cannot add constraint to a Hole. Fill the hole first."

しかし ast-draft.md は「hole は単なる空欄ではなく、制約付き未完成ノードである」と明言しており、domain-model.md も「Hole が何を受け入れるかは ConstraintAST 側で NodeId に紐づけて管理する」と述べている。

**破綻する具体例:**

AI Agent がまず制約を先に設定してから hole を埋める戦略を取りたい場合（例: 「ここには 1 ≤ X ≤ 10^5 の整数スカラーが入る」と先に制約を付け、後で FillHole する）、この操作順序が拒否される。結果として、FillHole → AddConstraint の順序が強制されるが、これは hole_candidates の導出に制約情報が必要であるという設計（CandidateKind は ConstraintAST から導出）と矛盾する。

**推奨される修正方向:**

- Hole への AddConstraint を許可する。ConstraintSet は NodeId でマッピングするため、Hole の NodeId に制約を紐づけることに技術的障害はない。
- FillHole 時に Hole に付いていた制約を新ノードに引き継ぐロジックを Operation に追加する。

---

## 修正提案（具体的な改善案）

### P-1. StructureAST の純構造化

NodeKind からすべての型・レンダリング情報を除去し、純粋な構造定義のみにする:

```rust
pub enum NodeKind {
    Scalar { name: Ident },
    Array { name: Ident, length: Reference },
    Matrix { name: Ident, rows: Reference, cols: Reference },
    Tuple { elements: Vec<NodeId> },   // Vec<Reference> → Vec<NodeId>
    Repeat { count: Reference, body: Vec<NodeId> },
    Section { header: Option<NodeId>, body: Vec<NodeId> },
    Sequence { children: Vec<NodeId> },
    Hole { expected_kind: Option<NodeKindHint> },  // hole_id 除去
}
```

型・区切り・ソート等はすべて ConstraintAST 側:
- `TypeDecl { target, expected }` — 既存
- `RenderHint { target, separator: Separator }` — 新規
- StructureAST と ConstraintAST の責務が明確に分離される

### P-2. ConstraintId の導入と ConstraintSet の再設計

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintId(pub u64);

pub struct ConstraintSet {
    pub arena: HashMap<ConstraintId, Constraint>,
    pub by_node: HashMap<NodeId, Vec<ConstraintId>>,
    pub global: Vec<ConstraintId>,
}
```

Action の変更:
```rust
RemoveConstraint { constraint_id: ConstraintId },
```

ApplyResult の拡張:
```rust
pub struct ApplyResult {
    pub created_nodes: Vec<NodeId>,
    pub removed_nodes: Vec<NodeId>,
    pub created_constraints: Vec<ConstraintId>,
    pub affected_constraints: Vec<ConstraintId>,
}
```

### P-3. FillContent を Builder 層に分離

Operation は低レベル Action（NodeKind 単位の操作）のみを受け付け、FillContent のような高レベル操作は Builder 層が Action 列に展開する:

```
[ユーザー / AI]
    ↓ 高レベル意図 (EdgeList を追加)
[Builder 層]
    ↓ Action 列に展開
    [FillHole(hole, Repeat), FillHole(body_hole, Tuple),
     AddSlotElement(tuple, u), AddSlotElement(tuple, v),
     AddConstraint(u, Range(1, N)), AddConstraint(v, Range(1, N)),
     AddConstraint(edges, Property(SimpleGraph))]
[Operation 層]
    ↓ 各 Action を検証・適用
[StructureAST + ConstraintAST]
```

これにより:
- Operation は NodeKind レベルの整合性のみに責任を持つ
- ドメイン知識は Builder 層に局所化される
- Builder 層はテスト可能で差し替え可能（グラフ用 Builder、幾何用 Builder 等）

### P-4. Arena ベースの StructureAST

```rust
pub struct StructureAst {
    pub root: NodeId,
    pub arena: Vec<Option<StructureNode>>,  // NodeId(n) → arena[n]
    pub next_id: u64,
}

impl StructureAst {
    pub fn get(&self, id: NodeId) -> Option<&StructureNode> {
        self.arena.get(id.0 as usize)?.as_ref()
    }
}
```

利点:
- O(1) アクセス（HashMap の定数倍を回避）
- 挿入順序が自然に保持される
- Canonical rendering の決定論的順序が保証される
- メモリ局所性が良い（キャッシュフレンドリー）

---

## 総評

Sprint 2 の設計は、ドメインの理解と先行研究の適用において高い水準にある。特に Hole 第一級の設計方針、ProjectionAPI と Operation の責務分離の意図、サイト調査に基づく NodeKind の最小化は適切である。

しかし、**設計文書が宣言する原則と、Rust 型定義が示す実装の間に乖離がある**。特に構造と制約の分離（S-1）と制約の識別子管理（S-2）は、実装フェーズで即座に問題を引き起こす重大欠陥であり、Sprint 3 で修正すべきである。

FillContent の肥大化（S-3）は設計上の方向性の問題であり、Builder 層の導入で解決可能だが、放置すると Operation が巨大な god object になるリスクがある。

中程度の欠陥（M-1〜M-5）はいずれも具体的な修正案があり、Sprint 3〜4 で順次対応可能である。
