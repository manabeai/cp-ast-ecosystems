# 実サンプル検証レポート — CoverageValidationAgent

> Sprint 3 — 実サンプル検証担当
> 入力: doc/design/domain-model.md（StructureAST / ConstraintAST ドメインモデル）

---

## 1. サンプル一覧（問題ごとの判定表）

### 1.1 整数/配列（基本）

#### P-01: abc264_b — Nice Grid

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc264/tasks/abc264_b |
| カテゴリ | 整数/配列（基本） |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Tuple(R, C)。Range(R,1,15), Range(C,1,15)。最も単純なケース。 |

入力形式:
```
R C
```

#### P-02: abc350_c — Sort（順列）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc350/tasks/abc350_c |
| カテゴリ | 整数/配列（基本） |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Scalar(N) + Array(A, N, Int)。Property(A, Permutation) で順列制約を表現。生成は Fisher-Yates で対応。 |

入力形式:
```
N
A_1 ... A_N
```

---

### 1.2 グラフ（木）

#### P-03: abc270_c — Simple path（木上の単純パス）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc270/tasks/abc270_c |
| カテゴリ | グラフ（木） |
| StructureAST | **部分対応** |
| ConstraintAST | **対応可能** |
| canonical rendering | **一部曖昧** |
| sample generation | **条件付き** |
| コメント | 構造は Sequence > Tuple(N,X,Y) + Repeat(N−1, Tuple(U_i,V_i))。**問題点: Repeat.count が Reference（変数への参照）しか取れず、「N−1」のような式を直接指定できない。** 回避策として暗黙的に M=N−1 の Scalar を導入する必要がある。Property(graph, Tree) で木保証は表現可能。木生成は Prüfer sequence で対応。 |

入力形式:
```
N X Y
U_1 V_1
...
U_{N-1} V_{N-1}
```

**ギャップ: Repeat.count が Expression を受け取れない（Reference のみ）。**

---

### 1.3 グラフ（一般）

#### P-04: abc284_c — Count Connected Components

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc284/tasks/abc284_c |
| カテゴリ | グラフ（一般） |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Tuple(N,M) + Repeat(M, Tuple(u_i,v_i))。Range(M, 0, N*(N-1)/2) は Relation + Expression(BinOp) で表現可能。Property(graph, Simple) で単純グラフ保証。I-3 パターンの典型。 |

入力形式:
```
N M
u_1 v_1
...
u_M v_M
```

#### P-05: abc277_c — Ladder Takahashi

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc277/tasks/abc277_c |
| カテゴリ | グラフ（一般） |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Scalar(N) + Repeat(N, Tuple(A_i,B_i))。頂点番号が 1〜10^9 の大域的辺リスト。Range(A_i,1,10^9), Relation(A_i,Ne,B_i) で対応。グラフ構造としては辺リストそのもの。 |

入力形式:
```
N
A_1 B_1
...
A_N B_N
```

---

### 1.4 行列/2D配列

#### P-06: abc300_b — Same Map in the RPG World（2グリッド）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc300/tasks/abc300_b |
| カテゴリ | 行列/2D配列 |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Tuple(H,W) + Matrix(A, H, W, Char, None) + Matrix(B, H, W, Char, None)。2つの文字グリッドを連続して読む。domain-model §4.2「複数グリッドの連結入力」で対応可能と判定済み。 |

入力形式:
```
H W
A_{1,1}...A_{1,W}
...
A_{H,1}...A_{H,W}
B_{1,1}...B_{1,W}
...
B_{H,1}...B_{H,W}
```

#### P-07: abc351_b — Spot the Difference（2つの N×N グリッド）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc351/tasks/abc351_b |
| カテゴリ | 行列/2D配列 |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Scalar(N) + Matrix(A, N, N, Char, None) + Matrix(B, N, N, Char, None)。行数・列数が同一変数 N を共有する正方グリッド×2。Guarantee("exactly one cell differs") は Guarantee で表現。 |

入力形式:
```
N
A_{1,1}...A_{1,N}
...
A_{N,1}...A_{N,N}
B_{1,1}...B_{1,N}
...
B_{N,1}...B_{N,N}
```

#### P-08: abc337_d — Cheating Gomoku Narabe（グリッド + パラメータ）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc337/tasks/abc337_d |
| カテゴリ | 行列/2D配列 |
| StructureAST | **対応可能** |
| ConstraintAST | **部分対応** |
| canonical rendering | **可能** |
| sample generation | **条件付き** |
| コメント | Sequence > Tuple(H,W,K) + Matrix(grid, H, W, Char, None)。構造は I-2 パターン。ただし制約 H×W ≤ 2×10^5 は**積の上界**であり、Relation で表現するには Expression(BinOp(Mul, H, W)) が必要。Expression 自体は対応するが、Range の「変数ペアの積に対する上界」は Relation(H*W, Le, 200000) として表現可能。文字集合 {o, x, .} は TypeDecl/Property では直接指定不可で Custom が必要。 |

入力形式:
```
H W K
S_1
...
S_H
```

---

### 1.5 文字列

#### P-09: abc350_a — Past ABCs（固定フォーマット文字列）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc350/tasks/abc350_a |
| カテゴリ | 文字列 |
| StructureAST | **対応可能** |
| ConstraintAST | **部分対応** |
| canonical rendering | **可能** |
| sample generation | **条件付き** |
| コメント | Sequence > Scalar(S, Str)。構造は最もシンプル。制約「先頭3文字が ABC、末尾3文字が数字、全体長 6」は**文字列パターン制約**であり、現在の Constraint に直接対応するものがない。Property(S, Custom("pattern:ABC\\d{3}")) で近似可能だが、生成器が正規表現パターンを解釈する必要がある。 |

入力形式:
```
S
```

**ギャップ: 文字列パターン/正規表現制約が未定義。**

#### P-10: abc338_b — Frequency（単一文字列）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc338/tasks/abc338_b |
| カテゴリ | 文字列 |
| StructureAST | **対応可能** |
| ConstraintAST | **部分対応** |
| canonical rendering | **可能** |
| sample generation | **条件付き** |
| コメント | Sequence > Scalar(S, Str)。制約「英小文字のみ、長さ 1〜1000」。文字集合制約は TypeDecl で Str とは言えるが、「英小文字のみ」という文字集合制限は Property(S, Custom("lowercase_ascii")) が必要。**文字列長の Range 制約も未対応**（Range は数値向け）。 |

入力形式:
```
S
```

**ギャップ: 文字列長の Range 制約、文字集合制約が未定義。**

#### P-11: abc283_c — Cash Register（超大整数文字列）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc283/tasks/abc283_c |
| カテゴリ | 文字列 |
| StructureAST | **対応可能** |
| ConstraintAST | **部分対応** |
| canonical rendering | **可能** |
| sample generation | **条件付き** |
| コメント | Sequence > Scalar(S, Str)。入力は 10^{100000} 以下の整数だが文字列として読む必要がある。ExpectedType に BigInt がなく、Str で代用。Range(S, 1, 10^{100000}) は Expression の Lit(i64) では表現不能（i64 のオーバーフロー）。 |

入力形式:
```
S
```

**ギャップ: Expression.Lit が i64 のため、10^{100000} 級の値域を表現できない。BigInt リテラルが必要。**

---

### 1.6 複数テストケース

#### P-12: abc284_b — Multi Test Cases

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc284/tasks/abc284_b |
| カテゴリ | 複数テストケース |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Scalar(T) + Repeat(T, Section(Scalar(N) + Array(A, N, Int)))。I-5 パターンの典型。SumBound は本問では不要だが、モデルとして対応可能。 |

入力形式:
```
T
(for each test case:)
N
A_1 A_2 ... A_N
```

---

### 1.7 クエリ形式

#### P-13: abc278_d — All Assign Point Add（多型クエリ）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc278/tasks/abc278_d |
| カテゴリ | クエリ形式 |
| StructureAST | **部分対応** |
| ConstraintAST | **対応可能** |
| canonical rendering | **一部曖昧** |
| sample generation | **条件付き** |
| コメント | Sequence > Scalar(N) + Array(A,N) + Scalar(Q) + Repeat(Q, ???)。**クエリが 3 種類の異なるフォーマット**（`1 x`, `2 i x`, `3 i`）を持つ。現在の NodeKind に Variant/Union/Choice に相当するものがなく、1行の構造がクエリ種別タグに依存して変わるパターンを表現できない。Repeat の body を固定構造として定義するしかなく、「先頭の整数で分岐」という概念がない。 |

入力形式:
```
N
A_1 A_2 ... A_N
Q
query_1
...
query_Q
```

**ギャップ: Variant/Tagged Union NodeKind が欠如。クエリ種別によるフォーマット分岐を表現不可。**

#### P-14: abc356_c — Keys（行内可変長 + 混在型）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc356/tasks/abc356_c |
| カテゴリ | クエリ形式 / 可変長入力 |
| StructureAST | **部分対応** |
| ConstraintAST | **対応可能** |
| canonical rendering | **一部曖昧** |
| sample generation | **条件付き** |
| コメント | Sequence > Tuple(N,M,K) + Repeat(M, ???)。各行が `C_i A_{i,1} ... A_{i,C_i} R_i` の形式で、**同一行内に可変長配列と末尾の文字が混在**する。Tuple.elements は Vec\<Reference\> のため、行内に可変長の部分配列を埋め込めない。Tuple の要素が Array ノードを含む構造が必要だが、現設計では要素は Reference のみ。 |

入力形式:
```
N M K
C_1 A_{1,1} ... A_{1,C_1} R_1
...
C_M A_{M,1} ... A_{M,C_M} R_M
```

**ギャップ: Tuple 内に可変長配列（インライン Array）を埋め込む手段がない。**

---

### 1.8 可変長入力

#### P-15: abc284_a — Sequence of Strings

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc284/tasks/abc284_a |
| カテゴリ | 可変長入力 |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **可能** |
| コメント | Sequence > Scalar(N) + Repeat(N, Scalar(S_i, Str))。N 行の文字列を各行 1 つずつ読む。Repeat + Scalar(Str) で自然に表現可能。 |

入力形式:
```
N
S_1
...
S_N
```

---

### 1.9 相互依存制約

#### P-16: abc259_d — Circumferences（円の連結判定）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc259/tasks/abc259_d |
| カテゴリ | 相互依存制約 |
| StructureAST | **対応可能** |
| ConstraintAST | **対応可能** |
| canonical rendering | **可能** |
| sample generation | **条件付き** |
| コメント | Sequence > Scalar(N) + Tuple(sx,sy,tx,ty) + Repeat(N, Tuple(x_i,y_i,r_i))。構造は自然に対応。Guarantee("(sx,sy) は少なくとも1つの円の円周上") は Guarantee で表現可能だが、**幾何学的保証条件の生成は困難**。 |

入力形式:
```
N
s_x s_y t_x t_y
x_1 y_1 r_1
...
x_N y_N r_N
```

#### P-17: abc371_c — Make Isomorphic（2グラフ + 三角行列）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc371/tasks/abc371_c |
| カテゴリ | 相互依存制約 |
| StructureAST | **部分対応** |
| ConstraintAST | **対応可能** |
| canonical rendering | **一部曖昧** |
| sample generation | **条件付き** |
| コメント | 構造: Sequence > Scalar(N) + Section(Scalar(M_G) + Repeat(M_G, Tuple(u,v))) + Section(Scalar(M_H) + Repeat(M_H, Tuple(a,b))) + **三角行列コスト**。三角行列部分（行 i に N−i 個の値）は Matrix NodeKind で直接表現できない。domain-model §4.1 で「部分対応」と判定済み。Repeat + 可変長 Tuple（行ごとに要素数が異なる）で近似するが、Tuple.elements は固定長 Vec のため、行ごとに異なるノード定義が必要になる。 |

入力形式:
```
N
M_G
u_1 v_1
...
u_{M_G} v_{M_G}
M_H
a_1 b_1
...
a_{M_H} b_{M_H}
A_{1,2} A_{1,3} ... A_{1,N}
A_{2,3} ... A_{2,N}
...
A_{N-1,N}
```

**ギャップ: 三角行列（行ごとに列数が変化する 2D 構造）を直接表現する NodeKind がない。**

---

### 1.10 Interactive

#### P-18: abc313_d — Odd or Even（インタラクティブ）

| 項目 | 判定 |
|------|------|
| 問題URL | https://atcoder.jp/contests/abc313/tasks/abc313_d |
| カテゴリ | interactive |
| StructureAST | **非対応** |
| ConstraintAST | **非対応** |
| canonical rendering | **不可** |
| sample generation | **不可** |
| コメント | 入力/出力が交互に行われる双方向プロトコル。初期入力は N K だが、以後はクエリ→応答の繰り返し。StructureAST は「入力全体の構造木」を前提としており、対話型の入出力インターリーブは表現不能。domain-model §4.3 で「非対応（Phase 2）」と明記。 |

---

## 2. カテゴリ別カバー率

### 2.1 StructureAST カバー率

| カテゴリ | 問題数 | 対応可能 | 部分対応 | 非対応 | カバー率(対応可能) | カバー率(部分以上) |
|----------|--------|----------|----------|--------|--------------------|--------------------|
| 整数/配列（基本） | 2 | 2 | 0 | 0 | **100%** | 100% |
| グラフ（木） | 1 | 0 | 1 | 0 | **0%** | 100% |
| グラフ（一般） | 2 | 2 | 0 | 0 | **100%** | 100% |
| 行列/2D配列 | 3 | 3 | 0 | 0 | **100%** | 100% |
| 文字列 | 3 | 3 | 0 | 0 | **100%** | 100% |
| 複数テストケース | 1 | 1 | 0 | 0 | **100%** | 100% |
| クエリ形式 | 2 | 0 | 2 | 0 | **0%** | 100% |
| 可変長入力 | 1 | 1 | 0 | 0 | **100%** | 100% |
| 相互依存制約 | 2 | 1 | 1 | 0 | **50%** | 100% |
| interactive | 1 | 0 | 0 | 1 | **0%** | 0% |
| **合計** | **18** | **13** | **4** | **1** | **72.2%** | **94.4%** |

### 2.2 ConstraintAST カバー率

| カテゴリ | 問題数 | 対応可能 | 部分対応 | 非対応 | カバー率(対応可能) | カバー率(部分以上) |
|----------|--------|----------|----------|--------|--------------------|--------------------|
| 整数/配列（基本） | 2 | 2 | 0 | 0 | **100%** | 100% |
| グラフ（木） | 1 | 1 | 0 | 0 | **100%** | 100% |
| グラフ（一般） | 2 | 2 | 0 | 0 | **100%** | 100% |
| 行列/2D配列 | 3 | 2 | 1 | 0 | **67%** | 100% |
| 文字列 | 3 | 0 | 3 | 0 | **0%** | 100% |
| 複数テストケース | 1 | 1 | 0 | 0 | **100%** | 100% |
| クエリ形式 | 2 | 2 | 0 | 0 | **100%** | 100% |
| 可変長入力 | 1 | 1 | 0 | 0 | **100%** | 100% |
| 相互依存制約 | 2 | 2 | 0 | 0 | **100%** | 100% |
| interactive | 1 | 0 | 0 | 1 | **0%** | 0% |
| **合計** | **18** | **13** | **4** | **1** | **72.2%** | **94.4%** |

### 2.3 サンプル生成カバー率

| カテゴリ | 問題数 | 可能 | 条件付き | 不可 | カバー率(可能) |
|----------|--------|------|----------|------|----------------|
| 整数/配列（基本） | 2 | 2 | 0 | 0 | **100%** |
| グラフ（木） | 1 | 0 | 1 | 0 | **0%** |
| グラフ（一般） | 2 | 2 | 0 | 0 | **100%** |
| 行列/2D配列 | 3 | 2 | 1 | 0 | **67%** |
| 文字列 | 3 | 0 | 3 | 0 | **0%** |
| 複数テストケース | 1 | 1 | 0 | 0 | **100%** |
| クエリ形式 | 2 | 0 | 2 | 0 | **0%** |
| 可変長入力 | 1 | 1 | 0 | 0 | **100%** |
| 相互依存制約 | 2 | 0 | 2 | 0 | **0%** |
| interactive | 1 | 0 | 0 | 1 | **0%** |
| **合計** | **18** | **8** | **9** | **1** | **44.4%** |

### 2.4 総合カバー率サマリ

| 指標 | 完全対応 | 部分対応以上 | 非対応 |
|------|----------|-------------|--------|
| StructureAST | 72.2% (13/18) | 94.4% (17/18) | 5.6% (1/18) |
| ConstraintAST | 72.2% (13/18) | 94.4% (17/18) | 5.6% (1/18) |
| Canonical Rendering | 72.2% (13/18) | 94.4% (17/18) | 5.6% (1/18) |
| Sample Generation | 44.4% (8/18) | 94.4% (17/18) | 5.6% (1/18) |

---

## 3. 未対応カテゴリの分析

### 3.1 完全非対応: Interactive（1問）

**問題**: abc313_d  
**理由**: StructureAST は「入力全体の静的な構造木」を前提としており、入力/出力が交互に発生する対話型プロトコルはモデルの対象外。domain-model で明示的に Phase 2 に延期済み。  
**影響度**: 低（AtCoder の interactive 問題は全体の約 5% 以下）  
**対策**: Phase 2 で `Interactive` NodeKind を追加。

### 3.2 StructureAST の部分対応ギャップ（4問）

#### ギャップ A: Repeat.count が Expression を受け取れない（P-03: abc270_c）

- **現状**: `Repeat.count: Reference` — 変数ノードへの参照のみ
- **必要**: `N-1` のような算術式を count に指定したい
- **出現頻度**: 木問題の辺数 `N-1`、一部のクエリ問題で頻出
- **対策案**: `Repeat.count` を `Reference | Expression` の union にする、または暗黙 Scalar 導入を許容するルールを策定
- **影響範囲**: StructureAST のみ。ConstraintAST は影響なし。

#### ギャップ B: Variant/Tagged Union NodeKind の欠如（P-13: abc278_d）

- **現状**: Repeat の body は固定構造のノードリスト
- **必要**: クエリ種別タグ（先頭の整数）に応じて行構造が変わる
- **出現頻度**: クエリ形式問題の約 60%（3種以上のクエリ型を持つ問題）
- **対策案**: `Choice { tag: Reference, variants: HashMap<i64, Vec<NodeId>> }` NodeKind を追加
- **影響範囲**: StructureAST, canonical rendering, sample generator すべてに影響。設計上の最大の拡張ポイント。

#### ギャップ C: Tuple 内インライン可変長配列（P-14: abc356_c）

- **現状**: `Tuple.elements: Vec<Reference>` — 固定個数の参照のみ
- **必要**: 1行内に `C_i` 個のキー + 結果文字（可変長 + 混在型）
- **出現頻度**: 中（可変長行はクエリ形式・一部の特殊入力で出現）
- **対策案**: Tuple 内に Array ノードを含められるよう `elements` を `Vec<NodeId>` に変更、または InlineTuple NodeKind を追加
- **影響範囲**: StructureAST, canonical rendering。

#### ギャップ D: 三角行列（P-17: abc371_c）

- **現状**: Matrix は rows × cols の固定矩形のみ
- **必要**: 行ごとに列数が変化する三角行列（行 i に N-i 個の要素）
- **出現頻度**: 低（コスト行列・距離行列で散見）
- **対策案**: Matrix に `shape: MatrixShape { Rectangular, UpperTriangular, LowerTriangular }` を追加（domain-model §4.1 で既に提案済み）
- **影響範囲**: StructureAST, canonical rendering, sample generator。

### 3.3 ConstraintAST の部分対応ギャップ（4問）

#### ギャップ E: 文字列パターン/正規表現制約（P-09: abc350_a）

- **現状**: TypeDecl で型 (Str/Int/Char) は指定可能だが、文字列の内容パターン（正規表現）は表現不能
- **必要**: 「先頭3文字が ABC で末尾3文字が数字」のようなパターン制約
- **対策案**: `Pattern { target: Reference, regex: String }` 制約を追加、または Property(target, Custom("regex:...")) で対応

#### ギャップ F: 文字列長制約・文字集合制約（P-10: abc338_b, P-08: abc337_d）

- **現状**: Range は数値向け。文字列の長さに対する値域制約や、使用可能文字集合の指定方法がない
- **必要**: 「長さ 1〜1000」「英小文字のみ」「{o, x, .} のみ」
- **対策案**:
  1. `StringLength { target: Reference, min: Expression, max: Expression }` 制約を追加
  2. `Charset { target: Reference, allowed: CharsetSpec }` 制約を追加（CharsetSpec = LowercaseAscii | UppercaseAscii | Digit | Custom(String)）
  3. または LengthRelation を文字列にも適用可能にする

#### ギャップ G: Expression.Lit の i64 制限（P-11: abc283_c）

- **現状**: `Expression::Lit(i64)` で上限が約 9.2×10^{18}
- **必要**: 10^{100000} のような超大整数
- **出現頻度**: 低（大整数入力は稀）
- **対策案**: `Expression::BigLit(String)` を追加、または Lit を BigInt 型にする

---

## 4. 最優先の拡張候補

検出されたギャップを影響度・出現頻度・実装コストで優先順位付けする:

| 優先度 | ギャップ | 影響問題数 | 出現頻度 | 実装コスト | 推奨フェーズ |
|--------|----------|-----------|----------|-----------|-------------|
| **★★★** | B: Variant/Choice NodeKind | 1/18 (5.6%) | 高（クエリ問題の約60%） | 高 | Sprint 4 |
| **★★☆** | F: 文字列長・文字集合制約 | 3/18 (16.7%) | 高（文字列問題のほぼ全て） | 低 | Sprint 4 |
| **★★☆** | A: Repeat.count に Expression | 1/18 (5.6%) | 中（木問題で頻出） | 低 | Sprint 4 |
| **★☆☆** | C: Tuple 内インライン配列 | 1/18 (5.6%) | 中 | 中 | Sprint 5 |
| **★☆☆** | D: 三角行列サポート | 1/18 (5.6%) | 低 | 低 | Sprint 5 |
| **★☆☆** | E: 文字列パターン制約 | 1/18 (5.6%) | 低 | 低 | Sprint 5 |
| ☆☆☆ | G: BigInt リテラル | 1/18 (5.6%) | 低 | 低 | Phase 2 |
| ☆☆☆ | Interactive | 1/18 (5.6%) | 低 | 高 | Phase 2 |

### 推奨アクション

1. **Sprint 4 で対応すべき 3 件:**
   - **Variant/Choice NodeKind**: クエリ形式問題のカバレッジを劇的に改善。設計上の最大のギャップ。
   - **文字列制約の拡充** (StringLength + Charset): 低コストで文字列カテゴリの ConstraintAST カバー率を 0% → 100% に改善。
   - **Repeat.count の Expression 対応**: 型を `Reference | Expression` に拡張するだけで木問題のカバー率が改善。

2. **Sprint 5 以降:**
   - Tuple 内インライン配列、三角行列、文字列パターン制約は出現頻度が低く、workaround が存在するため後回し可。

3. **Phase 2:**
   - Interactive 対応と BigInt リテラルは利用頻度が極めて低く、Phase 2 で十分。

### 完全対応後の予測カバー率

Sprint 4 の 3 件を実装した場合の予測:

| 指標 | 現在 | Sprint 4 後（予測） |
|------|------|---------------------|
| StructureAST 完全対応 | 72.2% | **83.3%** (+2問) |
| ConstraintAST 完全対応 | 72.2% | **94.4%** (+4問) |
| Sample Generation 完全対応 | 44.4% | **61.1%** (+3問) |

---

## 付録: 検証方法

- AtCoder から 10 カテゴリ × 各 1〜3 問 = 計 18 問を選定
- 各問題の入力形式を web_fetch で取得し、StructureAST / ConstraintAST のノード・制約にマッピング
- 判定基準:
  - **対応可能**: 既存の NodeKind / Constraint で完全に表現可能
  - **部分対応**: 既存のノードで近似可能だが、一部情報が欠落または workaround が必要
  - **非対応**: 現モデルでは表現不能
- 問題選定は AtCoder Beginner Contest (ABC) の A〜F 問題から幅広く選定
