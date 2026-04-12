# Sample Generation Feasibility Review

> Sprint 3 — SampleGenerationAgent によるレビュー成果物
> 入力: doc/design/domain-model.md（ConstraintAST ドメインモデル）

---

## 1. 生成可能な制約

各 Constraint 種別について、ConstraintAST からランダムな妥当入力を生成する具体的戦略を記述する。

### 1.1 Range

**戦略**: 依存グラフの順序に従い、先行変数を解決した後、`evaluate(lower) .. evaluate(upper)` の閉区間から一様ランダムにサンプリングする。

```
generate_range(target, lower, upper, env) -> Value:
  lo = evaluate(lower, env)   // env は既に生成済み変数の値を保持
  hi = evaluate(upper, env)
  if lo > hi: return Err(Unsat)
  return uniform_random(lo, hi)
```

- **式評価**: Expression の `Lit`, `BinOp`, `Pow`, `Var`, `FnCall` を再帰的に評価する。`Var(ref)` は env から解決する。
- **Power 表現**: `Pow { base: Lit(10), exp: Lit(9) }` → `10^9 = 1_000_000_000`。Rust の `i64` で `10^18` まで対応可能（`i64::MAX ≈ 9.2 × 10^18`）。
- **導出上界**: `N(N-1)/2` のような式は `BinOp(Div, BinOp(Mul, Var(N), BinOp(Sub, Var(N), Lit(1))), Lit(2))` として評価可能。先に N が生成されていれば問題ない。
- **対応状況**: ✅ 完全対応可能。最も基本的な制約であり、式評価器があれば直截に実装できる。

### 1.2 TypeDecl

**戦略**: 型に応じた生成器を選択する。

| ExpectedType | 生成方法 |
|-------------|---------|
| `Int` | Range 制約と組み合わせて整数を生成 |
| `Str` | 文字集合（後述の不足情報）+ 長さ制約から文字列を生成 |
| `Char` | 文字集合から 1 文字をサンプリング |

- **対応状況**: ⚠️ 部分対応。整数型は Range と組み合わせれば完全対応。文字列型は文字集合情報が現設計に不足している（後述の追加要件 3.1）。

### 1.3 LengthRelation

**戦略**: 依存グラフにより長さ変数が先に生成される。その値を用いて配列の要素数を確定する。

```
// 依存グラフ: N → A（LengthRelation: len(A) = N）
env["N"] = generate_range(N, ...)    // N を先に生成
A = Vec::with_capacity(env["N"])
for i in 0..env["N"]:
    A[i] = generate_range(A_i, ...)  // 各要素を Range 制約に従って生成
```

- **2D 配列**: Matrix の場合、`rows` と `cols` の両方が先に生成されている必要がある。依存グラフが `H → Matrix`, `W → Matrix` の辺を持つため、H, W の両方が生成された後に Matrix の要素を生成する。
- **対応状況**: ✅ 完全対応可能。依存グラフのトポロジカルソートが正しく動作すれば自然に解決する。

### 1.4 Relation

**戦略**: 関係式の依存方向に応じて生成範囲を動的に絞り込む。

| パターン | 戦略 |
|---------|------|
| `1 ≤ u_i < v_i ≤ N` | u を `[1, N-1]` から生成 → v を `[u+1, N]` から生成 |
| `M ≤ N(N-1)/2` | N を先に生成 → M の上界を `min(元の上界, N(N-1)/2)` に絞り込み |
| `X ≠ Y` | X を生成 → Y を X を除いた範囲から生成（リジェクション or 除外サンプリング） |

- **一般的な二項関係**: `lhs op rhs` の形式で、lhs/rhs の一方が既知なら他方の生成範囲を制限できる。
- **Relation 内の Expression が複雑な場合**: `BinOp` を含む式は逆関数の導出が困難。例: `A_i + B_i ≤ K` のとき A_i を生成した後 B_i の上界を `K - A_i` に制限する必要がある。現設計の Expression は一般的な式を許容するため、逆関数の自動導出は保証できない。
- **対応状況**: ⚠️ 単純な二項比較は対応可能。複合式を含む Relation は部分的対応（後述の生成困難セクション参照）。

### 1.5 Distinct

**戦略**: 制約の性質と要素数に応じて最適な生成アルゴリズムを選択する。

| 状況 | 戦略 |
|------|------|
| `DistinctUnit::Element` + 範囲十分 | 範囲 `[lo, hi]` から重複なしランダムサンプリング（Fisher-Yates の部分シャッフル or reservoir sampling） |
| `DistinctUnit::Element` + 順列 | `[1..N]` のシャッフル（O(N)） |
| `DistinctUnit::Tuple` | 生成済みタプル集合を保持し、リジェクションサンプリング。衝突率が高い場合は集合差分からの直接サンプリング |

- **対応状況**: ✅ 要素単位の相異は完全対応可能。タプル単位の相異は値域が十分大きければ対応可能だが、密な値域ではリジェクション率が高くなりうる。

### 1.6 Property

**戦略**: タグごとの専用生成器を実装する。

| PropertyTag | 生成戦略 | 計算量 |
|------------|---------|--------|
| `Permutation` | Fisher-Yates シャッフル | O(N) |
| `Binary` | 各要素を `{0, 1}` から一様ランダム | O(N) |
| `Odd` / `Even` | Range の範囲内で奇数/偶数のみをサンプリング: `lo + (lo%2 != target) + 2*rand(0, (hi-lo)/2)` | O(1) per element |
| `Simple` (グラフ) | 辺リスト生成時に自己ループ・多重辺を排除。集合管理によるリジェクション | O(M) expected |
| `Connected` | ランダムスパニングツリー（Prüfer 列 or ランダムウォーク）を生成 → 残り M-N+1 本をランダム追加 | O(N + M) |
| `Tree` | Prüfer 列からの変換、またはランダム親選択 | O(N) |
| `Custom(s)` | 未対応 → 警告を返す | — |

- **対応状況**: ⚠️ 定義済みタグは対応可能。`Custom` タグは生成不可能（設計上の限界として許容）。

### 1.7 SumBound

**戦略**: テストケース横断の総和制約を満たすように各ケースの対象変数を分配する。

```
generate_sum_bound(T, variable_name, upper_bound, per_case_range):
  remaining = upper_bound
  values = []
  for i in 0..T:
    max_for_this = min(per_case_range.upper, remaining - (T - i - 1) * per_case_range.lower)
    min_for_this = max(per_case_range.lower, remaining - (T - i - 1) * per_case_range.upper)  
    if min_for_this > max_for_this: return Err(Unsat)
    v = uniform_random(min_for_this, max_for_this)
    values.push(v)
    remaining -= v
  return values
```

- **対応状況**: ✅ 対応可能。ただし分配の一様性は保証しない（先頭ケースにバイアスがかかる）。一様分配が必要な場合は棒折り（stick-breaking）法等が必要。

### 1.8 Sorted

**戦略**: 要素を Range 制約内で生成した後、指定順序でソートする。

| SortOrder | 後処理 |
|-----------|--------|
| `Ascending` | 生成後 strict sort。重複があればリジェクション |
| `NonDecreasing` | 生成後ソート（重複許可） |
| `Descending` / `NonIncreasing` | 同上 + reverse |

- **厳密な昇順の場合**: 値域 `[lo, hi]` から N 個の strictly increasing な値を選ぶ必要がある。`hi - lo + 1 ≥ N` が必要条件。重複なしランダムサンプリング後にソートするのが最も単純。
- **対応状況**: ✅ 完全対応可能。

### 1.9 Guarantee

**戦略**: 生成器の責務外。生成後のバリデーションフェーズで検証する。

- `Guarantee` は「入力にはこの性質が成り立つ」という宣言であり、生成器が能動的に満たす必要はない。
- ただし、生成されたサンプルが Guarantee を満たすかどうかの事後検証は有用。`predicate` フィールドが `Some(expr)` の場合は式評価で検証可能。`None`（自然言語記述のみ）の場合は検証不能。
- **対応状況**: ✅ 生成には影響しない。事後検証は predicate 付きの場合のみ可能。

---

## 2. 生成が難しい制約

### 2.1 複合 Relation の逆関数導出

**問題**: `Relation { lhs: A_i + B_i, op: Le, rhs: Var(K) }` のような複合式では、A_i を生成した後に B_i の上界を `K - A_i` と導出する必要がある。一般の Expression に対する逆関数の自動導出は、以下の理由で困難:

- `BinOp(Mul, ...)` を含む式の逆関数は整数割り算の切り捨て問題が生じる
- `FnCall` を含む式は逆関数が定義できない場合がある（`abs`, `min`, `max`）
- 複数変数を含む式では、どの変数を「自由」にしてどの変数を「従属」にするかの選択が必要

**対処方針**:
1. **単純なケースを優先対応**: `Var op Lit`, `Var op Var`, `Var + Lit op Var` のような線形な関係は逆関数を導出可能。
2. **一般ケースはリジェクションサンプリング**: 逆関数が導出できない場合、全変数を先に生成し、Relation を満たさなければリトライ。リトライ上限（例: 1000回）を設定し、超過で Unsat を報告。
3. **将来的な拡張**: シンボリック逆関数ソルバの導入（線形整数計画法ベース）。

### 2.2 Distinct + 狭い値域

**問題**: `Distinct` 制約で値域が要素数に対して狭い場合（例: N=100, 値域=[1,105]）、リジェクション率が極めて高くなる。

**対処方針**:
1. **充足可能性の事前検査**: `hi - lo + 1 ≥ N` を生成前にチェック。
2. **集合ベースのサンプリング**: 値域から N 個をランダムに選択する方式に切り替え（`reservoir sampling` or `partial Fisher-Yates`）。これにより O(N) で衝突なく生成可能。
3. **タプル単位の Distinct**: 値域の組み合わせ数 `|V1| × |V2|` が要素数以上であることを検査。不足時は Unsat を報告。

### 2.3 Property の組み合わせ

**問題**: 複数の Property が同時に指定される場合（例: `Simple` + `Connected` のグラフ）、個々の Property 生成器を合成する方法が非自明。

**対処方針**:
1. **プリセット組み合わせ**: よくある組み合わせ（`Simple + Connected`, `Simple + Connected + Tree`）は専用生成器を用意。
2. **レイヤー合成**: まず最も制約が強い Property（例: `Tree`）で構造を生成し、残りの Property を満たすか検証する。
3. **現設計への影響**: Property タグは `Vec<Constraint>` として複数付与されるため、組み合わせの列挙は可能。ただし、どの組み合わせが有効かを判定するロジックは生成器側に必要。

### 2.4 SumBound と Per-case 制約の相互作用

**問題**: `SumBound(N, 3×10^5)` と各ケースの `Range(N, 1, 2×10^5)` が同時にある場合、ケース数 T と per-case の N の分配が相互に依存する。T=2, N の上界=2×10^5, 総和上界=3×10^5 のとき、両ケースで N=2×10^5 は不可能。

**対処方針**:
1. **分配時に per-case Range を考慮**: 1.7 節の `generate_sum_bound` で `per_case_range` に Range 制約を反映する（既に設計済み）。
2. **事前の充足可能性検査**: `T * range.lower ≤ sum_upper` かつ `range.upper ≥ range.lower` を検査。

### 2.5 Guarantee 制約の能動的充足

**問題**: `Guarantee { description: "A+B=C_i なる i が丁度 1 つ存在する" }` のような制約は、predicate が自然言語記述のみの場合、生成器が能動的に満たすことは不可能。

**対処方針**:
1. **predicate 付きの場合**: 式評価による事後リジェクション。ただし充足率が低い場合は事実上生成不可能。
2. **predicate なしの場合**: 生成器は Guarantee を無視し、生成結果に「Guarantee 未検証」の警告を付与する。
3. **根本的な限界**: Guarantee は一般には任意の述語であり、充足するサンプルの構成的生成は決定不能問題に帰着しうる。「best-effort + 警告」が現実的な落としどころ。

### 2.6 循環依存（Cycle Detection）

**問題**: ユーザの編集途中で `Range(A, 0, Ref(B))` + `Range(B, 0, Ref(A))` のような循環依存が生じうる。

**対処方針**:
1. **依存グラフ構築時にサイクル検出**: Kahn のアルゴリズム（BFS ベースのトポロジカルソート）で入次数 0 のノードが枯渇した時点でサイクルを検出。
2. **エラー報告**: サイクルに関与するノードの一覧を返す。
3. **Hole による遮断**: Hole ノードは依存を遮断するため、編集途中の一時的な循環は Hole の存在で回避される。この設計は妥当。

---

## 3. 設計上の追加要件

現設計の ConstraintAST ドメインモデルに対し、生成器の実装のために以下の情報が不足している。

### 3.1 文字列の文字集合（CharSet）

**不足**: TypeDecl で `ExpectedType::Str` または `ExpectedType::Char` が指定された場合、生成に使用する文字集合が不明。

**例**: 「S は英小文字からなる文字列」→ 文字集合は `{'a'..'z'}`。「S は英大文字と英小文字からなる」→ `{'a'..'z', 'A'..'Z'}`。

**提案**: Constraint に `CharSet` 制約を追加するか、TypeDecl を拡張する。

```rust
// 案1: TypeDecl を拡張
TypeDecl {
    target: Reference,
    expected: ExpectedType,
    charset: Option<CharSet>,  // 追加
}

// 案2: 新しい Constraint を追加
CharSetConstraint {
    target: Reference,
    charset: CharSet,
}

// CharSet の定義
enum CharSet {
    LowerAlpha,           // a-z
    UpperAlpha,           // A-Z
    Alpha,                // a-z, A-Z
    Digit,                // 0-9
    AlphaNumeric,         // a-z, A-Z, 0-9
    Custom(Vec<char>),    // 任意の文字集合
    Range(char, char),    // 範囲指定: 'a'..'z'
}
```

**優先度**: 高。文字列を含む問題は頻出であり、文字集合なしでは生成不可能。

### 3.2 文字列長の制約

**不足**: 文字列スカラー（`ExpectedType::Str`）に対する長さの制約が現設計では表現しにくい。配列の LengthRelation は配列専用であり、文字列の長さ制約に使えるかが不明確。

**提案**: LengthRelation を文字列にも適用可能と明示するか、Range 制約で `len(S)` を対象にできるようにする。

```rust
// 案: LengthRelation を一般化
LengthRelation {
    target: Reference,    // Array or Str Scalar
    length: Expression,   // Reference ではなく Expression にして柔軟性を確保
}
```

**優先度**: 高。

### 3.3 Expression の evaluate 関数の仕様

**不足**: ドメインモデルに `evaluate_constant()` への言及があるが（ast-draft.md 由来）、現設計の Expression 型に対する評価関数の仕様が明示されていない。

**必要な仕様**:
1. **評価環境（Env）**: `HashMap<NodeId, Value>` — 生成済み変数の値を保持
2. **部分評価**: 環境に未解決の変数がある場合の振る舞い（エラー vs. 式のまま返却）
3. **オーバーフロー処理**: `10^18 * 2` のような式で `i64` 溢れが起きた場合の処理
4. **FnCall の対応関数一覧**: `min`, `max`, `abs`, `len` 等、サポートする関数の明確なリスト

**優先度**: 高。生成器の根幹機能。

### 3.4 依存グラフの構築ルール（形式化）

**不足**: ドメインモデル Section 5.1 に依存グラフの構築規則が記載されているが、Constraint の各種別からどのようにエッジを抽出するかの形式的な定義がない。

**必要な形式化**:

```
extract_edges(constraint) -> Vec<(NodeId, NodeId)>:
  match constraint:
    Range { target, lower, upper }:
      for var in free_vars(lower) ∪ free_vars(upper):
        yield (var, target)        // target は var に依存
    
    LengthRelation { array, length }:
      yield (length, array)        // array は length に依存
    
    Relation { lhs, op, rhs }:
      // 依存方向の決定が必要 — どちらが先に生成されるか
      // 提案: lhs の変数を先行、rhs の変数を後行とする規約
      for var_l in free_vars(lhs):
        for var_r in free_vars(rhs):
          yield (var_l, var_r)
    
    SumBound { variable, upper }:
      for var in free_vars(upper):
        yield (var, variable)
    
    Distinct { elements, .. }:
      // 依存辺なし（要素は独立に生成後、事後検証）
    
    Sorted { elements, .. }:
      // 依存辺なし（要素は独立に生成後、ソート）
    
    Property { target, .. }:
      // 依存辺なし（生成戦略の選択に影響するが、変数間の依存は作らない）
    
    Guarantee { .. }:
      // 依存辺なし
```

**特に Relation の依存方向**: `u_i < v_i` で u → v（u を先に生成）とするか、v → u とするかは一意に決まらない。**現設計にはこの方向を決定する情報がない**。

**提案**: Relation に `generation_hint: Option<GenerationDirection>` を追加するか、依存グラフ構築時にヒューリスティック（lhs を先行とする）を適用するかの設計判断が必要。

**優先度**: 中。ヒューリスティックで多くのケースは対応可能だが、エッジケースで問題になる。

### 3.5 生成結果の出力形式

**不足**: 生成器が出力するサンプルケースのデータ構造が定義されていない。StructureAST + 生成値をどのような形式で保持し、どのようにテキスト出力に変換するか。

**提案**:

```rust
/// 生成されたサンプルケース
pub struct GeneratedSample {
    /// 各ノードに対する生成値
    pub values: HashMap<NodeId, GeneratedValue>,
    /// 生成不可能だったノード（Guarantee 未検証等）
    pub warnings: Vec<GenerationWarning>,
}

pub enum GeneratedValue {
    Int(i64),
    Str(String),
    Array(Vec<GeneratedValue>),
    Matrix(Vec<Vec<GeneratedValue>>),
}

pub enum GenerationWarning {
    GuaranteeNotVerified { description: String },
    PropertyNotSupported { tag: PropertyTag },
    RejectionLimitReached { constraint: String, attempts: u32 },
}
```

**優先度**: 中。生成器の API 設計に必要。

### 3.6 Repeat スコープ内の制約の適用範囲

**不足**: Repeat(count=M, body=\[Tuple(u_i, v_i)\]) のとき、`Range(u_i, 1, N)` という制約は「各イテレーション i について u_i ∈ [1, N]」を意味する。しかし、ConstraintSet は `HashMap<NodeId, Vec<Constraint>>` であり、イテレーション単位のスコープ情報がない。

**問題**: Tuple ノードの NodeId は1つしかないが、M 回のイテレーションで M 個の異なる値を生成する必要がある。制約が「各イテレーションに独立に適用される」ことを明示する機構が不足している。

**提案**: 制約の適用スコープを明示する注釈を追加するか、Repeat 内の制約は暗黙的に各イテレーションに適用されるという規約を文書化する。

**優先度**: 高。生成器が Repeat を正しく展開するために必須。

---

## 4. 生成可能保証の提案定義

### 4.1 用語の定義

| 用語 | 定義 |
|------|------|
| **生成環境** E | `HashMap<NodeId, Value>` — 生成済み変数の値を保持する環境 |
| **制約系** C | `ConstraintSet` — StructureAST に紐づく全制約の集合 |
| **依存グラフ** G(C) | 制約 C から抽出されるノード間の有向グラフ |
| **生成順序** σ | G(C) のトポロジカルソートの結果（存在すれば） |
| **生成器対応集合** S | 生成器が対応する Constraint 種別と PropertyTag の集合 |

### 4.2 充足可能性条件（SatisfiabilityCondition）

制約系 C が**静的に充足可能**であるとは、以下のすべてを満たすことをいう:

1. **DAG 条件**: G(C) が有向非巡回グラフ（DAG）である
2. **Range 非空条件**: すべての Range 制約について、定数のみで構成される上下界では `lower ≤ upper` が成立する。変数を含む場合は、後段で動的に検査する
3. **Distinct 充足条件**: すべての Distinct 制約について、対象要素の値域の大きさが要素数以上である（定数で確定する場合）
4. **Sorted 充足条件**: Sorted + Ascending の場合、値域の大きさ ≥ 要素数
5. **SumBound 充足条件**: `T * per_case_lower ≤ sum_upper`
6. **Hole 非存在条件**: StructureAST に Hole が残っていない

### 4.3 生成可能保証（Generation Guarantee）の定義

> **定義（条件付き生成可能保証）**:
>
> 制約系 C と生成器対応集合 S について、以下の条件がすべて成り立つとき、
> 生成器は C を充足するサンプルケースを**高確率で有限時間内に**生成できる:
>
> 1. C は静的に充足可能である（4.2 の条件をすべて満たす）
> 2. C に含まれるすべての Constraint の種別が S に含まれる
> 3. C に含まれるすべての Property タグが S に含まれる
> 4. Guarantee 制約は生成保証の対象外とする（事後検証のみ）
>
> **「高確率で」の意味**: リジェクションサンプリングを使用する制約（Relation, Distinct のリジェクション戦略）
> について、リトライ回数 R（デフォルト 1000）以内に成功する確率が 1 - ε（ε ≈ 0）であること。
> 厳密な確率保証は値域の大きさと制約の厳しさに依存する。

### 4.4 保証レベルの階層

生成器が返すべきステータスを以下の 3 レベルで定義する:

| レベル | 名称 | 意味 | 条件 |
|--------|------|------|------|
| **L1** | `Guaranteed` | 確実に生成可能 | Range のみ or Range + LengthRelation + TypeDecl(Int) で構成。リジェクション不要 |
| **L2** | `HighProbability` | 高確率で生成可能 | L1 + Distinct, Sorted, Relation(単純), Property(対応済み) を含む |
| **L3** | `BestEffort` | 生成を試みるが保証なし | Guarantee, Property(Custom), 複合 Relation を含む |

### 4.5 Unsat 判定と報告

生成器は以下のタイミングで Unsat を判定し、報告する:

| タイミング | 検査内容 | 報告形式 |
|-----------|---------|---------|
| **静的検査**（生成前） | DAG 条件, Range 非空（定数のみ）, Distinct 充足条件 | `Err(UnsatStatic { reason, nodes })` |
| **動的検査**（生成中） | Range 非空（変数評価後）, リジェクション上限超過 | `Err(UnsatDynamic { reason, env, node })` |
| **事後検証**（生成後） | Guarantee の predicate 評価 | `Warning(GuaranteeViolation { ... })` |

```rust
pub enum GenerationResult {
    /// 生成成功
    Ok {
        sample: GeneratedSample,
        level: GuaranteeLevel,
    },
    /// 静的に生成不可能
    UnsatStatic {
        reason: UnsatReason,
        involved_nodes: Vec<NodeId>,
    },
    /// 動的に生成不可能（生成途中で矛盾を検出）
    UnsatDynamic {
        reason: UnsatReason,
        partial_env: HashMap<NodeId, GeneratedValue>,
        failed_node: NodeId,
    },
}

pub enum UnsatReason {
    CyclicDependency { cycle: Vec<NodeId> },
    EmptyRange { target: NodeId, lower: i64, upper: i64 },
    InsufficientDomain { target: NodeId, required: usize, available: usize },
    RejectionLimitExceeded { target: NodeId, attempts: u32 },
    UnsupportedProperty { tag: PropertyTag },
    HolePresent { hole: NodeId },
}
```

---

## 5. 総合評価

### 5.1 設計の強み

- **依存グラフによるトポロジカル生成**（Section 5.1）は正しい方針であり、ほとんどの競プロ問題の入力生成をカバーできる
- **Constraint の種別が明確に分離**されており、生成器の戦略を種別ごとに切り替えやすい
- **Expression 型が十分に表現力がある**: `Pow`, `BinOp`, `FnCall` で `10^9`, `N(N-1)/2`, `min(N, K)` 等の典型的な式をカバー
- **Property タグが拡張可能**: `Custom(String)` で未知のタグも表現でき、生成不可能な場合の graceful degradation が可能

### 5.2 対処すべきギャップ

| # | ギャップ | 影響度 | 対処の容易さ |
|---|---------|--------|------------|
| 1 | 文字集合（CharSet）の欠如 | 高 | 容易（Constraint 追加） |
| 2 | 文字列長の制約表現 | 高 | 容易（LengthRelation 拡張） |
| 3 | Expression 評価関数の仕様不在 | 高 | 中（仕様策定が必要） |
| 4 | Repeat スコープの制約適用ルール | 高 | 中（規約の文書化 or 型の拡張） |
| 5 | Relation の依存方向 | 中 | 容易（ヒューリスティック or ヒント追加） |
| 6 | 生成結果の出力形式 | 中 | 容易（型定義追加） |
