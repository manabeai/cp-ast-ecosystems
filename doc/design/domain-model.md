# StructureAST / ConstraintAST ドメインモデル

> Sprint 2 — DomainModelAgent による設計成果物
> 入力: ast-draft.md, modules.md, site-survey-report.md, research-alignment-report.md

---

## 1. 最小ノード集合

サイト調査（14問題）で観察された入力パターン I-1〜I-7 を網羅するために必要な最小 NodeKind を以下に定義する。

### NodeKind 一覧

| # | NodeKind | 役割 | Slot | 対応する競プロ概念 | 具体例 |
|---|----------|------|------|--------------------|--------|
| 1 | **Scalar** | 単一変数の宣言 | `name: Ident`, `typ: ExpectedType` | N, M, Q, K 等のパラメータ変数 | `N`（整数スカラー）, `S`（文字列スカラー） |
| 2 | **Array** | 1次元列の宣言 | `name: Ident`, `length: Reference`, `element_type: ExpectedType` | A_1 A_2 … A_N | `A`（長さ N の整数配列）, `S`（文字列として入力される数列） |
| 3 | **Matrix** | 2次元配列の宣言 | `name: Ident`, `rows: Reference`, `cols: Reference`, `element_type: ExpectedType`, `separator: Separator` | H×W グリッド, コスト行列 | `C[i][j]`（文字グリッド, separator=None）, `A_{i,j}`（整数行列, separator=Space） |
| 4 | **Tuple** | 同一行で読む複数変数の組 | `elements: Vec<Reference>` | ヘッダ行 `N M K S T X`, 辺 `u_i v_i w_i` | `(N, A, B)`（abc300_a 1行目）, `(u_i, v_i)`（辺リストの1行） |
| 5 | **Repeat** | 変数依存の繰り返しブロック | `count: Reference`, `body: Vec<NodeId>` | M行の辺リスト, Q個のクエリ | M回の `(u_i, v_i)` 繰り返し, T回のテストケース |
| 6 | **Section** | 意味的に区切られた入力ブロック | `header: Option<NodeId>`, `body: Vec<NodeId>` | 複合セクション入力（グラフ部+条件部） | abc299_e のグラフセクション / 条件セクション |
| 7 | **Sequence** | 入力全体の順序付きルート | `children: Vec<NodeId>` | InputSpec 全体 | 問題全体の入力定義 |
| 8 | **Hole** | 未完成位置のプレースホルダ | `hole_id: HoleId`, `expected_kind: Option<NodeKindHint>` | 編集途中の空欄 | 「ここに配列定義が入る予定」 |

### 補足: NodeKind の設計判断

- **Scalar / Array / Matrix** を分離した理由: サイト調査で「スカラー」「1D配列」「2Dグリッド」が構造的に異なる読み方を要求することが確認された（空白区切り / 無区切り文字列 / 行列展開）。
- **Tuple** を独立させた理由: ヘッダ行の `N M K` のように「同一行で複数変数を読む」パターンが最頻出であり、これを Array とは区別する必要がある（Tuple の各要素は異なる変数名・異なる型を持つ）。
- **Repeat** は「M行の辺リスト」「T個のテストケース」「Q個のクエリ」をすべて統一的に表現する。body の内容が Tuple なら辺リスト、Section なら複数テストケースになる。
- **Section** は abc299_e（グラフ+条件）や abc371_c（2グラフ+コスト行列）のような複合入力を表現する。
- **Hole** は Hazelnut に倣い第一級ノードとして扱う。Hole 自体は制約情報を持たず、対応する制約は ConstraintAST 側で管理する。

---

## 2. 最小制約集合

サイト調査の制約パターン C-1〜C-7 を網羅するために必要な最小 Constraint 種別を以下に定義する。

### Constraint 一覧

| # | Constraint | 何を制約するか | 対象 NodeKind | 具体例 |
|---|-----------|---------------|---------------|--------|
| 1 | **Range** | 変数の値域（下界・上界） | Scalar, Array(要素), Matrix(要素) | `1 ≤ N ≤ 2×10^5`, `0 ≤ A_i ≤ 10^9` |
| 2 | **TypeDecl** | 変数の型宣言 | Scalar, Array, Matrix | `入力は全て整数`, `S は英大文字と英小文字からなる文字列` |
| 3 | **LengthRelation** | 配列長・繰り返し数と変数の関係 | Array, Matrix, Repeat | `len(A) = N`, `辺数 = M`, `行数 = H, 列数 = W` |
| 4 | **Relation** | 変数間の比較・不等式 | Scalar, Array(要素) | `1 ≤ u_i < v_i ≤ N`, `X ≠ S`, `M ≤ N(N-1)/2` |
| 5 | **Distinct** | 要素の相異条件 | Array, Tuple(要素集合) | `C_i は相異なる`, `(U_i,V_i) ≠ (U_j,V_j)` |
| 6 | **Property** | 構造全体の性質タグ | Array, グラフ関連ノード | `グラフは単純かつ連結`, `(A_1,…,A_N) は順列`, `K は奇数` |
| 7 | **SumBound** | テストケース横断の総和制約 | Repeat(テストケース), Scalar | `N の総和は 3×10^5 以下` |
| 8 | **Sorted** | 要素列のソート済み条件 | Array | `p_1 < p_2 < … < p_K` |
| 9 | **Guarantee** | 入力の存在保証・妥当性保証 | Sequence(全体) | `A+B=C_i なる i が丁度 1 つ存在する` |

### 補足: Constraint の設計判断

- **Range** が最も頻出であり、サイト調査 14問中 13問で出現。下界・上界は式（Expression）で表現し、`N(N-1)/2` のような導出上界にも対応する。
- **TypeDecl** は「入力は全て整数」のような包括的型宣言を表現する。個別変数に型が付く場合は Scalar/Array の `element_type` で表現し、TypeDecl は補助的な役割。
- **LengthRelation** は StructureAST の Repeat.count / Array.length と ConstraintAST を橋渡しする重要な制約。
- **Relation** は `<`, `≤`, `≠`, `=` を含む二項関係。C-3（導出制約）と C-4（変数間関係）を統合する。
- **Property** はタグベースで拡張可能にする（`Simple`, `Connected`, `Tree`, `Permutation`, `Binary` 等）。
- **SumBound** は複数テストケース問題でのみ出現するが、計算量保証として重要。
- **Guarantee** は生成には直接影響しないが、問題仕様の完全な表現のために必要。

---

## 3. Node / Slot / Hole / Reference の整理

### 3.1 Node

```
Node は StructureAST の構成単位である。
各 Node は一意の NodeId を持ち、NodeKind により種別が決まる。
Node は名前付き Slot を通じて子ノードや値を保持する。
```

**定義:**
- `Node = (NodeId, NodeKind, HashMap<SlotName, SlotValue>)`
- NodeKind が Slot の名前と型を決定する（スキーマとして機能する）

### 3.2 Slot

Slot は Node が持つ名前付きの接続点である。Slot には以下の3種類がある。

| Slot 種別 | 意味 | 例 |
|-----------|------|-----|
| **Single** | 単一の子ノードまたは値を保持 | Array.length → Reference(N), Scalar.typ → ExpectedType |
| **List** | 順序付きの子ノードリストを保持 | Sequence.children → Vec\<NodeId\>, Tuple.elements → Vec\<Reference\> |
| **Optional** | 子ノードまたは値を 0 or 1 個保持 | Section.header → Option\<NodeId\>, Hole.expected_kind → Option\<NodeKindHint\> |

**SlotValue の型:**

```
SlotValue ::=
  | Child(NodeId)              -- 単一子ノード
  | Children(Vec<NodeId>)      -- 子ノードリスト
  | Ref(Reference)             -- 変数参照
  | Refs(Vec<Reference>)       -- 変数参照リスト
  | Value(Literal)             -- リテラル値
  | Empty                      -- 未充足（Hole と異なり、Optional slot の空状態）
```

### 3.3 Hole

Hole は StructureAST 上の第一級ノードであり、「この位置にまだノードが配置されていない」ことを表す。

**Hole の設計方針（Hazelnut に倣う）:**

1. **Hole はノードである**: Hole は null や空欄ではなく、NodeId を持つ正規のノード。
2. **Hole は制約情報を持たない**: Hole が「何を受け入れるか」は ConstraintAST 側で NodeId に紐づけて管理する。StructureAST 側の Hole は `HoleId` と任意の `NodeKindHint`（UI ヒント用）のみを持つ。
3. **Hole は差し替え可能**: Hole を具体ノードに置き換える操作は Operation 層が担う。
4. **Hole は入れ子可能**: Hole を含むノード（例: body に Hole を含む Repeat）は合法な状態である。

**HoleId と NodeId の関係:**
- HoleId は NodeId の一種（`NodeId` の部分集合）として実装する。
- これにより ConstraintAST が NodeId で Hole を参照でき、特別な場合分けが不要になる。

### 3.4 Reference

Reference は StructureAST 内で「ある変数を参照する」ことを表す。

```
Reference ::=
  | VariableRef(NodeId)           -- 変数ノードへの参照（NodeId による）
  | IndexedRef(NodeId, Vec<Ident>) -- 添字付き参照: A[i], C[i][j]
  | Unresolved(Ident)             -- 未解決参照（Hole 的な状態の参照版）
```

**VariableRef と NodeId の関係:**

- VariableRef は NodeId を通じて変数を定義しているノードを指す。
- 例: Array ノード `A`（NodeId=n1）に対し、Repeat の count slot が `VariableRef(n1)` を持つ → 「この繰り返しは A の長さ分」
- IndexedRef は `A_i` や `C[i][j]` のように添字付きアクセスを表現する。添字変数 `i`, `j` は Repeat の暗黙ループ変数として解決される。
- Unresolved は編集途中で参照先がまだ決まっていない状態。Hole の参照版として扱う。

**Reference の解決方法:**
1. VariableRef / IndexedRef は NodeId によるルックアップで解決する（O(1)）。
2. NodeId はグローバルに一意であり、木構造を辿る必要はない。
3. Unresolved は名前解決フェーズで VariableRef に変換される。

---

## 4. Canonical Rendering に必要な情報

### 4.1 順序規則

Canonical rendering では、StructureAST のノードを決定論的な順序で出力する必要がある。

**順序の決定方法:**

1. **Sequence の children 順序がマスター**: Sequence ノードの children リストの順序が入力行の出現順序を決定する。これは StructureAST の構造自体が順序情報を保持することを意味する。
2. **Tuple 内の要素順序**: Tuple.elements の Vec 順序が同一行内の変数の出現順序を決定する。
3. **Repeat の body 順序**: body 内のノード順序が繰り返し内の各行の構造を決定する。
4. **Section の header → body 順序**: header が先、body が後。

**規則:**
- 制約（ConstraintAST）の表示順序: Range → TypeDecl → LengthRelation → Relation → Distinct → Property → Sorted → SumBound → Guarantee の優先度。同一種別内は対象変数名の辞書順。
- StructureAST のノード順序は木の深さ優先走査順に一致させる。

### 4.2 表示名正規化

変数名の canonical form を以下のように定める:

| パターン | 正規形 | 例 |
|----------|--------|-----|
| スカラー | 大文字英字 | `N`, `M`, `K`, `Q`, `T` |
| 配列要素 | 名前 + 下付き添字 | `A_i`, `C_i` |
| 行列要素 | 名前 + 二重添字 | `A_{i,j}`, `C[i][j]` |
| 辺の端点 | 小文字 + 添字 | `u_i`, `v_i`, `w_i` |
| 展開表記 | 名前 + 番号添字 | `A_1`, `A_2`, ..., `A_N` |

**正規化ルール:**
- 変数名は StructureAST の Scalar/Array/Matrix の `name` フィールドの値をそのまま使う。
- 添字変数は Repeat のネスト深さに応じて `i`, `j`, `k` を自動割り当てる。
- StructureAST 側に canonical name を持たせ、rendering は StructureAST の情報のみで完結する。

### 4.3 展開規則

配列や繰り返しの表示を決定論的に行う。

| 構造 | 展開パターン | 例 |
|------|-------------|-----|
| Array(長さ N) | `A_1 A_2 … A_N` | `C_1 C_2 … C_N` |
| Repeat(回数 M, body=Tuple(u,v)) | M行の `u_i v_i` | 辺リスト |
| Matrix(H×W, sep=None) | H行、各行 W 文字連結 | 文字グリッド |
| Matrix(H×W, sep=Space) | H行、各行 W 要素空白区切り | 整数行列 |

**省略記号の規則:**
- 水平展開: `A_1 A_2 … A_N`（要素数が変数依存の場合、`…` で省略）
- 垂直展開: 繰り返しブロックは `⋮` で省略
- 展開は常に「最初の 1〜2 要素 + 省略 + 最後の要素」の形式

---

## 5. Sample Generator に必要な情報

### 5.1 生成順序の導出方法

StructureAST + ConstraintAST から依存グラフを構築し、トポロジカルソートで生成順序を決定する。

**依存グラフの構築規則:**

1. **変数宣言依存**: Array.length が Scalar `N` を参照 → `N` → `A` の辺
2. **制約依存**: `Range(A_i, 0, Ref(K))` → `K` → `A` の辺
3. **構造依存**: Repeat.count が `M` を参照 → `M` → 辺リスト の辺
4. **Relation 依存**: `M ≤ N(N-1)/2` → `N` → `M` の辺（上界が N に依存）

**生成順序の例（abc284_c: グラフ問題）:**

```
N(スカラー) → M(N に依存した上界) → 辺リスト(M 行, 各辺の端点は 1..N)
```

**サイクル検出:**
- 依存グラフにサイクルがある場合は「生成不可能」として報告する。
- 正常な競プロ問題ではサイクルは発生しないが、ユーザの編集途中でサイクルが生じる可能性はある（Hole で遮断される）。

### 5.2 各制約種別の生成戦略

| Constraint | 生成戦略 | 備考 |
|-----------|---------|------|
| **Range** | 下界〜上界の一様ランダム | 最も基本。式評価が必要（`N(N-1)/2` 等） |
| **TypeDecl** | 型に応じた生成器を選択 | Int → 整数, Str → 文字列（文字集合は別途指定） |
| **LengthRelation** | 先に長さ変数を生成し、その値で配列長を決定 | 依存グラフの順序に従う |
| **Relation** | 先行変数を生成後、関係を満たす範囲に絞って生成 | `u_i < v_i` → u を先に生成し、v を u+1..N から選択 |
| **Distinct** | リジェクション or Fisher-Yates シャッフル | 順列の場合はシャッフル一発。相異配列は集合からのサンプリング |
| **Property** | タグに応じた専用生成器 | `Permutation` → シャッフル, `Connected` → ランダム木+辺追加, `Tree` → Prüfer sequence |
| **SumBound** | テストケース数を先に決め、各ケースの N を総和制約内で分配 | 貪欲分配 or ランダム分割 |
| **Sorted** | 生成後ソート or 順序統計量からサンプリング | 生成後ソートが最も単純 |
| **Guarantee** | 生成器の責務外（検証のみ） | 保証条件はバリデーションで確認 |

### 5.3 生成可能保証の定義

**条件付き保証（ast-draft.md の方針を継承）:**

> StructureAST + ConstraintAST が以下の条件を満たすとき、生成器は有限時間内に制約充足するサンプルケースを生成できる:
>
> 1. **制約系が充足可能**: Range の下界 ≤ 上界, LengthRelation の参照先が存在, 等
> 2. **依存グラフが非巡回**: トポロジカルソートが可能
> 3. **各制約種別が生成器の対応範囲内**: 未対応の Property タグなどがない

**生成不可能な場合の報告:**
- 依存グラフのサイクル → エラー: "circular dependency detected"
- Range の空区間 → エラー: "unsatisfiable range for variable X"
- 未対応の Property → 警告: "no generator for property tag Y"

---

## 6. Rust 型定義ドラフト

```rust
use std::collections::HashMap;

// ─── 識別子 ───

/// ノードのグローバル一意識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// Hole も NodeId で識別する（型エイリアスではなく同一型）
pub type HoleId = NodeId;

/// 変数名・添字名
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

// ─── StructureAST ───

/// StructureAST のルートノード
#[derive(Debug, Clone, PartialEq)]
pub struct StructureAst {
    pub root: NodeId,
    pub nodes: HashMap<NodeId, StructureNode>,
}

/// ノードの構造定義
#[derive(Debug, Clone, PartialEq)]
pub struct StructureNode {
    pub id: NodeId,
    pub kind: NodeKind,
}

/// ノード種別
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// 単一変数: N, M, S 等
    Scalar {
        name: Ident,
        typ: ExpectedType,
    },
    /// 1次元配列: A_1 ... A_N
    Array {
        name: Ident,
        length: Reference,
        element_type: ExpectedType,
    },
    /// 2次元配列 / グリッド: C[i][j], A_{i,j}
    Matrix {
        name: Ident,
        rows: Reference,
        cols: Reference,
        element_type: ExpectedType,
        separator: Separator,
    },
    /// 同一行の変数組: (N, M, K), (u_i, v_i)
    Tuple {
        elements: Vec<Reference>,
    },
    /// 変数依存の繰り返し: M行, T個のテストケース
    Repeat {
        count: Reference,
        body: Vec<NodeId>,
    },
    /// 意味的に区切られた入力ブロック
    Section {
        header: Option<NodeId>,
        body: Vec<NodeId>,
    },
    /// 入力全体の順序付きルート
    Sequence {
        children: Vec<NodeId>,
    },
    /// 未完成位置（第一級）
    Hole {
        hole_id: HoleId,
        expected_kind: Option<NodeKindHint>,
    },
}

/// 型の期待値
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedType {
    Int,
    Str,
    Char,
}

/// 行列の区切り文字指定
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Separator {
    Space,
    None,
}

/// Hole に対するノード種別ヒント（UI 向け）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKindHint {
    AnyScalar,
    AnyArray,
    AnyMatrix,
    AnyTuple,
    AnyRepeat,
    AnySection,
    Any,
}

// ─── Reference ───

/// 変数参照
#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
    /// 変数ノードへの直接参照
    VariableRef(NodeId),
    /// 添字付き参照: A[i], C[i][j]
    IndexedRef {
        target: NodeId,
        indices: Vec<Ident>,
    },
    /// 未解決参照（名前だけ持つ, Hole の参照版）
    Unresolved(Ident),
}

// ─── Slot / SlotValue ───

/// Slot 名
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlotName(pub String);

/// Slot に格納される値
#[derive(Debug, Clone, PartialEq)]
pub enum SlotValue {
    Child(NodeId),
    Children(Vec<NodeId>),
    Ref(Reference),
    Refs(Vec<Reference>),
    Value(Literal),
    Empty,
}

/// リテラル値
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    IntLit(i64),
    StrLit(String),
}

// ─── ConstraintAST ───

/// ConstraintAST のルート: NodeId → 制約集合 のマッピング
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintSet {
    /// ノードごとの制約リスト
    pub constraints: HashMap<NodeId, Vec<Constraint>>,
    /// 全体にかかる制約（特定ノードに紐づかないもの）
    pub global_constraints: Vec<Constraint>,
}

/// 制約種別
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// 値域制約: lower ≤ var ≤ upper
    Range {
        target: Reference,
        lower: Expression,
        upper: Expression,
    },
    /// 型宣言: 変数/入力全体の型
    TypeDecl {
        target: Reference,
        expected: ExpectedType,
    },
    /// 長さ関係: len(array) = var
    LengthRelation {
        array: Reference,
        length: Reference,
    },
    /// 変数間関係: lhs op rhs
    Relation {
        lhs: Expression,
        op: RelationOp,
        rhs: Expression,
    },
    /// 相異条件: elements がすべて異なる
    Distinct {
        elements: Reference,
        /// タプル単位の相異か要素単位か
        unit: DistinctUnit,
    },
    /// 構造的性質タグ
    Property {
        target: Reference,
        tag: PropertyTag,
    },
    /// 総和制約（複数テストケース用）
    SumBound {
        variable: Reference,
        upper: Expression,
    },
    /// ソート済み条件
    Sorted {
        elements: Reference,
        order: SortOrder,
    },
    /// 存在保証・妥当性保証
    Guarantee {
        description: String,
        predicate: Option<Expression>,
    },
}

/// 関係演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationOp {
    Lt,     // <
    Le,     // ≤
    Gt,     // >
    Ge,     // ≥
    Eq,     // =
    Ne,     // ≠
}

/// 相異の単位
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistinctUnit {
    Element,
    Tuple,
}

/// 構造的性質タグ（拡張可能）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyTag {
    Simple,         // グラフが単純
    Connected,      // グラフが連結
    Tree,           // グラフが木
    Permutation,    // 配列が順列
    Binary,         // 要素が 0 or 1
    Odd,            // 値が奇数
    Even,           // 値が偶数
    Custom(String), // 拡張用
}

/// ソート順
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,       // <  (strictly)
    NonDecreasing,   // ≤
    Descending,      // >
    NonIncreasing,   // ≥
}

/// 式（Expression）— 制約内の数式を表現
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// リテラル整数
    Lit(i64),
    /// 変数参照
    Var(Reference),
    /// 二項演算: lhs op rhs
    BinOp {
        op: ArithOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    /// べき乗: base^exp (10^9, 10^{18} 等)
    Pow {
        base: Box<Expression>,
        exp: Box<Expression>,
    },
    /// 関数適用: min, max, abs, len 等
    FnCall {
        name: Ident,
        args: Vec<Expression>,
    },
}

/// 算術演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}
```

---

## 非典型パターンへの対応可否

サイト調査で見つかった例外表現（セクション 4.1〜4.7）に対する本モデルの対応状況:

| # | 非典型パターン | 対応状況 | 対応方法 / 備考 |
|---|---------------|---------|----------------|
| 4.1 | **三角行列入力** (abc371_c) | **部分対応** | Matrix で `rows=N-1` は表現可能だが、「行ごとに列数が減少する」パターンは Matrix 単体では表現できない。将来的に `TriangularMatrix` NodeKind の追加、または Matrix に `shape: MatrixShape` slot を追加して対応する。暫定的には Repeat + 可変長 Tuple で近似可能。 |
| 4.2 | **複数グリッドの連結入力** (abc300_b) | **対応可能** | Section を2つ並べ、各 Section に Matrix ノードを配置することで表現できる。明示的セクション区切りがない点はパーサの問題であり、StructureAST レベルでは2つの Matrix として正しく表現される。 |
| 4.3 | **インタラクティブプロトコル** (abc313_d) | **非対応** | 双方向I/O・`?`/`!` プレフィクス・flush 要件は本モデルの対象外。入力/出力の交互実行は StructureAST の「入力仕様の構造木」というモデルに合わない。Sprint 1 の優先度判断（I-7 は最低優先度）に従い、Phase 2 以降で `Interactive` NodeKind を検討する。 |
| 4.4 | **可変行数の出力** (abc350_c) | **対応可能** | 出力構造は本モデルの主対象外だが、入力としての可変行数は Repeat(count=K) で表現可能。出力仕様のモデル化は別途 OutputSpec として設計する。 |
| 4.5 | **複合セクション + 可変長セクション** (abc299_e) | **対応可能** | Sequence > Section × 2 > (Section1: Tuple(N,M) + Repeat(M, Tuple(u,v))) + (Section2: Scalar(K) + Repeat(K, Tuple(p,d))) で正確に表現できる。 |
| 4.6 | **mod 出力** (abc300_e) | **対象外** | 出力形式の問題であり、入力構造/制約モデルの範囲外。OutputSpec で `ModOutput(modulus)` として対応予定（サーベイで定義済み）。 |
| 4.7 | **単一変数入力** (abc300_e) | **対応可能** | Sequence > Scalar(N) として自然に表現できる。最も単純なケース。 |

### 対応サマリ

- **対応可能**: 4.2, 4.4, 4.5, 4.7 — 現モデルで問題なく表現可能
- **部分対応**: 4.1 — 追加 NodeKind or MatrixShape の拡張で完全対応可能
- **非対応**: 4.3 — インタラクティブ問題は Phase 2 以降に延期
- **対象外**: 4.6 — 出力形式は別モデル（OutputSpec）の責務

---

## 付録: 設計判断の根拠

### A. サイト調査との対応表

| 入力パターン | 対応 NodeKind |
|-------------|--------------|
| I-1: ヘッダ行+データ行 | Sequence > Tuple(header) + Repeat(data) |
| I-2: グリッド入力 | Tuple(H,W) + Matrix(H,W) |
| I-3: グラフ入力 | Tuple(N,M) + Repeat(M, Tuple(u,v)) |
| I-4: クエリ形式 | Tuple(L,Q) + Repeat(Q, Tuple(c,x)) |
| I-5: 複数テストケース | Scalar(T) + Repeat(T, Section(...)) |
| I-6: 複合セクション | Sequence > Section × n |
| I-7: インタラクティブ | 非対応（Phase 2） |

### B. 先行研究との対応

| 設計概念 | 対応する先行研究 | 本モデルでの扱い |
|---------|----------------|-----------------|
| Hole 第一級 | Hazelnut (Omar et al. 2017) | NodeKind::Hole として実装。Hazelnut の応用であることを明示。 |
| StructureAST / ConstraintAST 分離 | MPS aspects, attribute grammars | MPS の constraints aspect に類似するが、独立した AST として分離する設計判断。 |
| 生成可能な制約 | QuickCheck, Luck (Lampropoulos 2017) | ConstraintAST を検査だけでなく生成にも使える設計。依存グラフによるトポロジカル生成。 |
| Canonical rendering | Wadler (2003) pretty-printing | StructureAST の構造順序から決定論的に再構成。 |
| NodeId 安定識別子 | Roslyn, persistent data structures | 標準技法として採用。 |
