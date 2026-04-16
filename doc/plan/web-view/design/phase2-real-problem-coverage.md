# Phase 2: Real Problem Coverage Agent 検証レポート

> 検証日: 2025-07-17
> 担当: Real Problem Coverage Agent
> 依存: phase1-premises.md, plan.md, main.md

---

## 1. 前提

### 1.1 検証目的

この文書は、Editor UI設計が**実際の競技プログラミング問題**に対してどの程度カバレッジを持つかを検証する。「だいたい対応できる」ではなく、**完全対応/部分対応/非対応**を明示的に判定する。

### 1.2 参照する設計要素

| 要素 | 出典 |
|------|------|
| NodeKind (9種) | node_kind.rs: Scalar, Array, Matrix, Tuple, Repeat, Section, Sequence, Choice, Hole |
| Constraint (12種) | constraint.rs: Range, TypeDecl, LengthRelation, Relation, Distinct, Property, SumBound, Sorted, Guarantee, CharSet, StringLength, RenderHint |
| Expression (5種) | expression.rs: Lit, Var, BinOp, Pow, FnCall |
| Action (7種) | action.rs: FillHole, ReplaceNode, AddConstraint, RemoveConstraint, IntroduceMultiTestCase, AddSlotElement, RemoveSlotElement |
| テンプレート | plan.md §20: 辺リスト, グリッド, 複数テストケース, クエリ列等 |

### 1.3 判定基準

| 判定 | 定義 |
|------|------|
| **完全対応** | 既存のNodeKind/Constraint/Expression/Actionで入力構造・制約を完全に表現可能 |
| **部分対応** | 近似可能だが、一部情報が欠落またはworkaroundが必要 |
| **非対応** | 現モデルでは表現不能 |

---

## 2. 問題一覧（32問）

### カテゴリ 1: 単一値

---

#### Problem 1: ABC264-B Nice Grid (Category: 単一値)
Source: AtCoder ABC264-B
**入力形式:**
```
R C
```
**制約:**
- 1 ≤ R ≤ 15
- 1 ≤ C ≤ 15
**入力特徴:** 単一行に2整数
**想定操作列:**
1. Structureペインで「+ Tuple」
2. Tuple内にScalar(R)とScalar(C)を追加
3. ConstraintsペインでRange(R, 1, 15)を追加
4. ConstraintsペインでRange(C, 1, 15)を追加
5. TypeDecl(R, Int), TypeDecl(C, Int)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Scalar
- Constraint: Range, TypeDecl
- Expression: Lit
- Operation: FillHole, AddConstraint
- Template: 単一変数テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 2: ABC380-A 123233 (Category: 単一値)
Source: AtCoder ABC380-A
**入力形式:**
```
N
```
**制約:**
- N は 6 桁の正整数
- N の各桁は 1, 2, 3 のいずれか
**入力特徴:** 単一整数、特殊な桁制約
**想定操作列:**
1. Structureペインで「+ 変数を追加」
2. 名前をNに設定
3. TypeDecl(N, Int)を追加
4. Range(N, 100000, 999999)を追加
5. Guarantee("各桁は1,2,3のいずれか")を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar
- Constraint: Range, TypeDecl, Guarantee
- Expression: Lit
- Operation: FillHole, AddConstraint
- Template: 単一変数テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** — （桁制約はGuaranteeで表現）

---

### カテゴリ 2: 複数値

---

#### Problem 3: ABC361-A Insert (Category: 複数値)
Source: AtCoder ABC361-A
**入力形式:**
```
N K X
A_1 A_2 ... A_N
```
**制約:**
- 1 ≤ K ≤ N ≤ 100
- 1 ≤ A_i, X ≤ 100
**入力特徴:** Tuple + 配列
**想定操作列:**
1. Structureペインで「+ Tuple」を追加
2. Tuple内にScalar(N), Scalar(K), Scalar(X)を追加
3. Structureペインで「+ 配列を追加」
4. 配列名A、長さN
5. Range制約を各変数に追加
6. Relation(K, Le, N)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Scalar, Array
- Constraint: Range, TypeDecl, Relation
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: 配列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 4: ABC360-E Random Swaps of Balls (Category: 複数値)
Source: AtCoder ABC360-E
**入力形式:**
```
N K
```
**制約:**
- 2 ≤ N ≤ 998244352
- 0 ≤ K ≤ 999
**入力特徴:** 2つの大整数
**想定操作列:**
1. Structureペインで「+ Tuple」
2. Scalar(N), Scalar(K)を追加
3. Range(N, 2, 998244352)を追加
4. Range(K, 0, 999)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Scalar
- Constraint: Range, TypeDecl
- Expression: Lit
- Operation: FillHole, AddConstraint
- Template: なし（基本操作のみ）
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

### カテゴリ 3: 配列

---

#### Problem 5: ABC350-C Sort (Category: 配列)
Source: AtCoder ABC350-C
**入力形式:**
```
N
A_1 A_2 ... A_N
```
**制約:**
- 2 ≤ N ≤ 2×10^5
- A は (1, 2, ..., N) の順列
**入力特徴:** 順列配列
**想定操作列:**
1. 「+ 変数を追加」でScalar(N)
2. 「+ 配列を追加」でArray(A, N)
3. Range(N, 2, 2*10^5)を追加
4. Property(A, Permutation)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Array
- Constraint: Range, Property(Permutation)
- Expression: Lit, Var, BinOp(Mul), Pow
- Operation: FillHole, AddConstraint
- Template: 配列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 6: ABC395-A Strictly Increasing? (Category: 配列)
Source: AtCoder ABC395-A
**入力形式:**
```
N
A_1 A_2 ... A_N
```
**制約:**
- 2 ≤ N ≤ 100
- 1 ≤ A_i ≤ 1000
**入力特徴:** 基本配列
**想定操作列:**
1. Scalar(N)を追加
2. Array(A, N)を追加
3. Range(N, 2, 100), Range(A[i], 1, 1000)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Array
- Constraint: Range, TypeDecl
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: 配列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 7: ABC360-C Move It (Category: 配列)
Source: AtCoder ABC360-C
**入力形式:**
```
N
A_1 A_2 ... A_N
W_1 W_2 ... W_N
```
**制約:**
- 1 ≤ N ≤ 10^5
- 1 ≤ A_i ≤ N
- 1 ≤ W_i ≤ 10^9
**入力特徴:** 同一長の2配列
**想定操作列:**
1. Scalar(N)を追加
2. Array(A, N)を追加
3. Array(W, N)を追加
4. 各種Range制約を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Array
- Constraint: Range
- Expression: Lit, Var, Pow
- Operation: FillHole, AddConstraint
- Template: 配列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

### カテゴリ 4: 二次元配列/グリッド

---

#### Problem 8: ABC300-B Same Map in the RPG World (Category: グリッド)
Source: AtCoder ABC300-B
**入力形式:**
```
H W
A_{1,1}...A_{1,W}
...
A_{H,1}...A_{H,W}
B_{1,1}...B_{1,W}
...
B_{H,1}...B_{H,W}
```
**制約:**
- 1 ≤ H, W ≤ 30
- A, B は '#' か '.' からなる
**入力特徴:** 2つの文字グリッド
**想定操作列:**
1. Tuple(H, W)を追加
2. 「+ グリッドを追加」でMatrix(A, H, W)
3. 「+ グリッドを追加」でMatrix(B, H, W)
4. CharSet(A, Custom('#', '.'))を追加
5. CharSet(B, Custom('#', '.'))を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Matrix
- Constraint: Range, CharSet
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: グリッドテンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 9: ABC390-C Paint the Grid (Category: グリッド)
Source: AtCoder ABC390-C
**入力形式:**
```
H W
S_1
S_2
...
S_H
```
**制約:**
- 1 ≤ H, W ≤ 10
- S_i は W 文字
**入力特徴:** 文字列配列としてのグリッド
**想定操作列:**
1. Tuple(H, W)を追加
2. Repeat(H, Scalar(S_i))を追加（各行を文字列として読む）
3. またはMatrix(S, H, W)として追加
4. StringLength(S_i, W, W)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat, Scalar または Matrix
- Constraint: Range, StringLength, LengthRelation
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: グリッドテンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 10: ABC337-D Cheating Gomoku Narabe (Category: グリッド)
Source: AtCoder ABC337-D
**入力形式:**
```
H W K
S_1
...
S_H
```
**制約:**
- 1 ≤ H
- 1 ≤ W
- H×W ≤ 2×10^5
- 1 ≤ K ≤ max(H, W)
- S_i は 'o', 'x', '.' からなる
**入力特徴:** 積制約付きグリッド
**想定操作列:**
1. Tuple(H, W, K)を追加
2. Matrix(S, H, W)を追加
3. Relation(H*W, Le, 2*10^5)を追加（BinOp(Mul)使用）
4. CharSet(S, Custom('o', 'x', '.'))を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Matrix
- Constraint: Range, Relation, CharSet
- Expression: Lit, Var, BinOp(Mul), Pow, FnCall(max)
- Operation: FillHole, AddConstraint
- Template: グリッドテンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** — （積の上界はRelation(BinOp)で表現可能）

---

#### Problem 11: ABC370-B Binary Alchemy (Category: グリッド/下三角)
Source: AtCoder ABC370-B
**入力形式:**
```
N
A_{1,1}
A_{2,1} A_{2,2}
A_{3,1} A_{3,2} A_{3,3}
...
A_{N,1} A_{N,2} ... A_{N,N}
```
**制約:**
- 1 ≤ N ≤ 99
- 1 ≤ A_{i,j} ≤ N
**入力特徴:** 下三角行列（行iにi個の要素）
**想定操作列:**
1. Scalar(N)を追加
2. ???（現行では表現不能）
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Repeat + **ループ変数による可変長** ← **現行なし**
- Constraint: Range
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 非対応
**部分対応/非対応の理由:** Repeat.index_varは存在するが、body内の配列長として参照する仕組みがない。「行iにi個の要素」を表現するには、Array.lengthがindex_varを参照できる必要がある。現行のRepeat内でbodyのArray長を動的に変えることができない。

---

### カテゴリ 5: 木

---

#### Problem 12: ABC270-C Simple Path (Category: 木)
Source: AtCoder ABC270-C
**入力形式:**
```
N X Y
U_1 V_1
...
U_{N-1} V_{N-1}
```
**制約:**
- 2 ≤ N ≤ 10^5
- 1 ≤ X, Y ≤ N, X ≠ Y
- グラフは木
**入力特徴:** N頂点の木（N-1辺）
**想定操作列:**
1. Tuple(N, X, Y)を追加
2. 「+ 辺リスト」テンプレートを使用
3. Repeat(N-1, Tuple(U_i, V_i))を追加
4. Property(graph, Tree)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat
- Constraint: Range, Relation, Property(Tree)
- Expression: Lit, Var, BinOp(Sub) ← **N-1のため**
- Operation: FillHole, AddConstraint
- Template: 辺リストテンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** — （Repeat.countがExpressionに対応済み、N-1も表現可能）

---

#### Problem 13: ABC259-E LCM on Whiteboard (Category: 木)
Source: AtCoder ABC259-E
**入力形式:**
```
N
M_1 p_{1,1} e_{1,1} ... p_{1,M_1} e_{1,M_1}
...
M_N p_{N,1} e_{N,1} ... p_{N,M_N} e_{N,M_N}
```
**制約:**
- 1 ≤ N ≤ 2×10^5
- 1 ≤ M_i
- ΣM_i ≤ 2×10^5
**入力特徴:** 各行可変長（素因数分解形式）
**想定操作列:**
1. Scalar(N)を追加
2. Repeat(N, ???)を追加
3. 各行は「M_i 個のペア」という構造
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Repeat, **行内可変長** ← **部分対応**
- Constraint: Range, SumBound
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 部分対応
**部分対応/非対応の理由:** 各行が「M_i + 2*M_i個」の要素を持つ形式。Tuple内にインライン配列を埋め込む必要があるが、現行Tuple.elementsはVec<NodeId>で固定個数のみ。Repeat内でScalar(M_i) + Repeat(M_i, Tuple(p, e))と2重Repeatで近似可能だが、canonical renderingは「同一行」にならない。

---

### カテゴリ 6: 一般グラフ

---

#### Problem 14: ABC284-C Count Connected Components (Category: 一般グラフ)
Source: AtCoder ABC284-C
**入力形式:**
```
N M
u_1 v_1
...
u_M v_M
```
**制約:**
- 1 ≤ N ≤ 100
- 0 ≤ M ≤ N(N-1)/2
- グラフは単純
**入力特徴:** 無向グラフの辺リスト
**想定操作列:**
1. Tuple(N, M)を追加
2. 「+ 辺リスト」テンプレートを使用
3. Repeat(M, Tuple(u_i, v_i))を追加
4. Property(graph, Simple)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat
- Constraint: Range, Property(Simple)
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: 辺リストテンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 15: ABC277-C Ladder Takahashi (Category: 一般グラフ)
Source: AtCoder ABC277-C
**入力形式:**
```
N
A_1 B_1
...
A_N B_N
```
**制約:**
- 1 ≤ N ≤ 2×10^5
- 1 ≤ A_i, B_i ≤ 10^9
- A_i ≠ B_i
**入力特徴:** 辺リスト（頂点番号が巨大）
**想定操作列:**
1. Scalar(N)を追加
2. Repeat(N, Tuple(A_i, B_i))を追加
3. Range(A_i, 1, 10^9)を追加
4. Relation(A_i, Ne, B_i)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Repeat, Tuple
- Constraint: Range, Relation
- Expression: Lit, Var, Pow
- Operation: FillHole, AddConstraint
- Template: 辺リストテンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

### カテゴリ 7: 辺リスト

---

#### Problem 16: ABC259-D Circumferences (Category: 辺リスト/座標)
Source: AtCoder ABC259-D
**入力形式:**
```
N
s_x s_y t_x t_y
x_1 y_1 r_1
...
x_N y_N r_N
```
**制約:**
- 1 ≤ N ≤ 3000
- |座標| ≤ 10^9
**入力特徴:** 座標+半径のリスト
**想定操作列:**
1. Scalar(N)を追加
2. Tuple(s_x, s_y, t_x, t_y)を追加
3. Repeat(N, Tuple(x_i, y_i, r_i))を追加
4. Range制約を各変数に追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Tuple, Repeat
- Constraint: Range
- Expression: Lit, Var, Pow
- Operation: FillHole, AddConstraint
- Template: なし（辺リストの変形）
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 17: ABC285-D Change Usernames (Category: 辺リスト)
Source: AtCoder ABC285-D
**入力形式:**
```
N
S_1 T_1
...
S_N T_N
```
**制約:**
- 1 ≤ N ≤ 10^5
- 1 ≤ |S_i|, |T_i| ≤ 8
- 英小文字のみ
**入力特徴:** 文字列ペアのリスト
**想定操作列:**
1. Scalar(N)を追加
2. Repeat(N, Tuple(S_i, T_i))を追加
3. StringLength(S_i, 1, 8)を追加
4. CharSet(S_i, LowerAlpha)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Repeat, Tuple
- Constraint: Range, StringLength, CharSet
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: 辺リストテンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 18: ABC294-E 2xN Grid (Category: 辺リスト/RLE)
Source: AtCoder ABC294-E
**入力形式:**
```
L N_1 N_2
v_{1,1} l_{1,1}
...
v_{1,N_1} l_{1,N_1}
v_{2,1} l_{2,1}
...
v_{2,N_2} l_{2,N_2}
```
**制約:**
- 1 ≤ L ≤ 10^{12}
- 1 ≤ N_1, N_2 ≤ 10^5
**入力特徴:** ランレングス符号化された2行
**想定操作列:**
1. Tuple(L, N_1, N_2)を追加
2. Repeat(N_1, Tuple(v, l))を追加（1行目）
3. Repeat(N_2, Tuple(v, l))を追加（2行目）
4. Range(L, 1, 10^12)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat
- Constraint: Range
- Expression: Lit, Var, Pow
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

### カテゴリ 8: クエリ列

---

#### Problem 19: ABC278-D All Assign Point Add (Category: クエリ列)
Source: AtCoder ABC278-D
**入力形式:**
```
N
A_1 A_2 ... A_N
Q
query_1
...
query_Q
```
各クエリは `1 x`（全代入）、`2 i x`（点加算）、`3 i`（点取得）
**制約:**
- 1 ≤ N, Q ≤ 2×10^5
**入力特徴:** 3種のクエリ型
**想定操作列:**
1. Scalar(N)を追加
2. Array(A, N)を追加
3. Scalar(Q)を追加
4. 「+ クエリ列」テンプレートを使用
5. Repeat(Q, Choice(tag, {1: Scalar(x), 2: Tuple(i,x), 3: Scalar(i)}))
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Array, Repeat, Choice
- Constraint: Range
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: クエリ列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** — （Choice NodeKindで3種のクエリ型を表現可能）

---

#### Problem 20: ABC380-E 1D Bucket Tool (Category: クエリ列)
Source: AtCoder ABC380-E
**入力形式:**
```
N Q
query_1
...
query_Q
```
各クエリは `1 x c`（色塗り）または `2 c`（色数え）
**制約:**
- 1 ≤ N, Q ≤ 5×10^5
**入力特徴:** 2種のクエリ型
**想定操作列:**
1. Tuple(N, Q)を追加
2. Repeat(Q, Choice(tag, {1: Tuple(x,c), 2: Scalar(c)}))を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat, Choice
- Constraint: Range
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: クエリ列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 21: ABC395-D Pigeon Swap (Category: クエリ列)
Source: AtCoder ABC395-D
**入力形式:**
```
N Q
op_1
...
op_Q
```
各opは `1 a b`、`2 a b`、`3 a` のいずれか
**制約:**
- 1 ≤ N, Q ≤ 3×10^5
**入力特徴:** 3種のクエリ型
**想定操作列:**
1. Tuple(N, Q)を追加
2. Repeat(Q, Choice(tag, {1: Tuple(a,b), 2: Tuple(a,b), 3: Scalar(a)}))を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat, Choice
- Constraint: Range
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: クエリ列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

### カテゴリ 9: 複数テストケース

---

#### Problem 22: ABC284-B Multi Test Cases (Category: 複数テストケース)
Source: AtCoder ABC284-B
**入力形式:**
```
T
(for each test case:)
N
A_1 A_2 ... A_N
```
**制約:**
- 1 ≤ T ≤ 100
- 1 ≤ N ≤ 100
**入力特徴:** 基本的なマルチテストケース
**想定操作列:**
1. 「+ 複数テストケース」テンプレートを使用
2. IntroduceMultiTestCase(count_var_name="T")を実行
3. 各テストケースのbodyにScalar(N)とArray(A, N)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Repeat, Section
- Constraint: Range
- Expression: Lit, Var
- Operation: IntroduceMultiTestCase
- Template: 複数テストケーステンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 23: Codeforces 1950A (Category: 複数テストケース)
Source: Codeforces 1950A
**入力形式:**
```
t
a_1 b_1 c_1
...
a_t b_t c_t
```
**制約:**
- 1 ≤ t ≤ 1000
- 1 ≤ a, b, c ≤ 10
**入力特徴:** 各テストケースが1行
**想定操作列:**
1. 「+ 複数テストケース」テンプレート
2. IntroduceMultiTestCase(count_var_name="t")
3. bodyにTuple(a, b, c)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Repeat, Tuple
- Constraint: Range
- Expression: Lit, Var
- Operation: IntroduceMultiTestCase
- Template: 複数テストケーステンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

### カテゴリ 10: 総和制約

---

#### Problem 24: ABC212-D Querying Multiset (Category: 総和制約)
Source: AtCoder ABC212-D
**入力形式:**
```
Q
query_1
...
query_Q
```
**制約:**
- 1 ≤ Q ≤ 2×10^5
- Type 1 クエリ: x を追加
- Type 2 クエリ: 全体に x を加算
- Type 3 クエリ: 最小値を取り出して出力
**入力特徴:** 3種クエリ（総和制約ではないが形式類似）
**想定操作列:**
1. Scalar(Q)を追加
2. Repeat(Q, Choice(tag, variants))を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar, Repeat, Choice
- Constraint: Range
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: クエリ列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 25: ABC217-D Cutting Woods (Category: 総和制約)
Source: AtCoder ABC217-D
**入力形式:**
```
L Q
c_1 x_1
...
c_Q x_Q
```
**制約:**
- 1 ≤ L ≤ 10^9
- 1 ≤ Q ≤ 2×10^5
- c_i ∈ {1, 2}
**入力特徴:** 2種クエリ
**想定操作列:**
1. Tuple(L, Q)を追加
2. Repeat(Q, Tuple(c_i, x_i))を追加
3. またはRepeat(Q, Choice(tag=c_i, ...))を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat, Choice
- Constraint: Range
- Expression: Lit, Var, Pow
- Operation: FillHole, AddConstraint
- Template: クエリ列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 26: ABC202-D aab aba baa (Category: 総和制約)
Source: AtCoder ABC202-D
**入力形式:**
```
A B K
```
**制約:**
- 1 ≤ A, B ≤ 30
- 1 ≤ K ≤ C(A+B, A)
**入力特徴:** 組み合わせ数制約
**想定操作列:**
1. Tuple(A, B, K)を追加
2. Range(A, 1, 30), Range(B, 1, 30)を追加
3. Guarantee("K ≤ C(A+B, A)")を追加（二項係数制約）
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Scalar
- Constraint: Range, Guarantee
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** — （組み合わせ数の上界はGuaranteeで表現）

---

### カテゴリ 11: distinct/sorted

---

#### Problem 27: ABC360-D X marks the spot (Category: distinct)
Source: AtCoder ABC360-D
**入力形式:**
```
N T
S
X_1 X_2 ... X_N
```
**制約:**
- 2 ≤ N ≤ 10^5
- X_i ≠ X_j (i ≠ j)
**入力特徴:** distinct配列
**想定操作列:**
1. Tuple(N, T)を追加
2. Scalar(S)を追加（文字列）
3. Array(X, N)を追加
4. Distinct(X, Element)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Scalar, Array
- Constraint: Range, Distinct, CharSet, StringLength
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: 配列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 28: ARC180-B Improve Inversions (Category: sorted/permutation)
Source: AtCoder ARC180-B
**入力形式:**
```
N K
P_1 P_2 ... P_N
```
**制約:**
- 2 ≤ N ≤ 2×10^5
- P は (1, 2, ..., N) の順列
**入力特徴:** 順列
**想定操作列:**
1. Tuple(N, K)を追加
2. Array(P, N)を追加
3. Property(P, Permutation)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Array
- Constraint: Range, Property(Permutation)
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: 配列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 29: ABC352-D Permutation Subsequence (Category: sorted)
Source: AtCoder ABC352-D
**入力形式:**
```
N K
P_1 P_2 ... P_N
```
**制約:**
- 1 ≤ K ≤ N ≤ 2×10^5
- P は (1, 2, ..., N) の順列
**入力特徴:** 順列 + K
**想定操作列:**
1. Tuple(N, K)を追加
2. Array(P, N)を追加
3. Property(P, Permutation)を追加
4. Relation(K, Le, N)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Array
- Constraint: Range, Property(Permutation), Relation
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: 配列テンプレート
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

### カテゴリ 12: 式付き境界

---

#### Problem 30: ABC361-B Intersection of Cuboids (Category: 式付き境界)
Source: AtCoder ABC361-B
**入力形式:**
```
a b c d e f
g h i j k l
```
**制約:**
- 0 ≤ a < d ≤ 1000
- 0 ≤ b < e ≤ 1000
- 0 ≤ c < f ≤ 1000
- (同様にg~l)
**入力特徴:** 6要素×2行、変数間関係制約
**想定操作列:**
1. Tuple(a,b,c,d,e,f)を追加
2. Tuple(g,h,i,j,k,l)を追加
3. Range(a, 0, 1000), Relation(a, Lt, d)などを追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Scalar
- Constraint: Range, Relation
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 31: ABC356-D Masked Popcount (Category: 式付き境界)
Source: AtCoder ABC356-D
**入力形式:**
```
N M
```
**制約:**
- 1 ≤ N, M ≤ 2^60
**入力特徴:** 2^60という巨大な上界
**想定操作列:**
1. Tuple(N, M)を追加
2. Range(N, 1, 2^60)を追加
3. Range(M, 1, 2^60)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Scalar
- Constraint: Range
- Expression: Lit, Var, Pow ← **Pow(2, 60) = 約1.15×10^18 はi64範囲内**
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** — （2^60 ≈ 1.15×10^18 はi64の範囲内）

---

### カテゴリ 13: choice/section/repeat必須ケース

---

#### Problem 32: ABC356-C Keys (Category: choice/section)
Source: AtCoder ABC356-C
**入力形式:**
```
N M K
C_1 A_{1,1} ... A_{1,C_1} R_1
...
C_M A_{M,1} ... A_{M,C_M} R_M
```
**制約:**
- 1 ≤ N ≤ 15
- 1 ≤ M ≤ 2^N - 1
- R_i ∈ {o, x}
**入力特徴:** 行内可変長配列 + 末尾文字
**想定操作列:**
1. Tuple(N, M, K)を追加
2. Repeat(M, ???)を追加
3. 各行は「C_i A_{i,1}...A_{i,C_i} R_i」という構造
**必要なUI/API/AST要素:**
- Structure: Sequence, Tuple, Repeat, **Tuple内インライン配列** ← **現行なし**
- Constraint: Range, CharSet
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 部分対応
**部分対応/非対応の理由:** Tuple内にインライン配列を埋め込む機能がない。Tuple.elementsはVec<NodeId>で、Array NodeIdを含めることは構造上可能だが、canonical renderingが「同一行にC_i個の要素 + R_i」となることを保証する仕組みがない。Repeat(M, Sequence[Scalar(C_i), Array(A_i, C_i), Scalar(R_i)])で近似可能だが、改行が入る。

---

### カテゴリ 14: 文字列

---

#### Problem 33: ABC380-B Hurdle Parsing (Category: 文字列)
Source: AtCoder ABC380-B
**入力形式:**
```
S
```
**制約:**
- S は '|' と '-' からなる
- 先頭と末尾は '|'
**入力特徴:** 特殊文字集合文字列
**想定操作列:**
1. Scalar(S)を追加
2. CharSet(S, Custom('|', '-'))を追加
3. Guarantee("先頭と末尾は '|'")を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar
- Constraint: CharSet, Guarantee
- Expression: Lit
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** — （先頭末尾制約はGuaranteeで表現）

---

#### Problem 34: ABC338-B Frequency (Category: 文字列)
Source: AtCoder ABC338-B
**入力形式:**
```
S
```
**制約:**
- 1 ≤ |S| ≤ 1000
- S は英小文字のみ
**入力特徴:** 基本文字列
**想定操作列:**
1. Scalar(S)を追加
2. StringLength(S, 1, 1000)を追加
3. CharSet(S, LowerAlpha)を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar
- Constraint: StringLength, CharSet
- Expression: Lit
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 35: ABC361-D Gomamayo Sequence (Category: 文字列)
Source: AtCoder ABC361-D
**入力形式:**
```
N
S
T
```
**制約:**
- 2 ≤ N ≤ 16
- |S| = |T| = N
- S, T は '0' と '1' のみ
**入力特徴:** 2つの01文字列
**想定操作列:**
1. Scalar(N)を追加
2. Scalar(S), Scalar(T)を追加
3. LengthRelation(S, N), LengthRelation(T, N)を追加
4. CharSet(S, Custom('0', '1'))を追加
5. CharSet(T, Custom('0', '1'))を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar
- Constraint: Range, LengthRelation, CharSet
- Expression: Lit, Var
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** —

---

#### Problem 36: ABC350-A Past ABCs (Category: 文字列)
Source: AtCoder ABC350-A
**入力形式:**
```
S
```
**制約:**
- S は "ABC" + 3桁数字
- |S| = 6
**入力特徴:** 固定フォーマット文字列
**想定操作列:**
1. Scalar(S)を追加
2. StringLength(S, 6, 6)を追加
3. Guarantee("先頭3文字が 'ABC'、末尾3文字が数字")を追加
**必要なUI/API/AST要素:**
- Structure: Sequence, Scalar
- Constraint: StringLength, Guarantee
- Expression: Lit
- Operation: FillHole, AddConstraint
- Template: なし
**判定:** 完全対応
**部分対応/非対応の理由:** — （パターン制約はGuaranteeで表現。正規表現制約は現行未対応だがGuaranteeで代替可能）

---

## 3. カバレッジサマリ

### 3.1 カテゴリ別判定表

| カテゴリ | 問題数 | 完全対応 | 部分対応 | 非対応 |
|----------|--------|----------|----------|--------|
| 1. 単一値 | 2 | 2 | 0 | 0 |
| 2. 複数値 | 2 | 2 | 0 | 0 |
| 3. 配列 | 3 | 3 | 0 | 0 |
| 4. グリッド | 4 | 3 | 0 | 1 |
| 5. 木 | 2 | 1 | 1 | 0 |
| 6. 一般グラフ | 2 | 2 | 0 | 0 |
| 7. 辺リスト | 3 | 3 | 0 | 0 |
| 8. クエリ列 | 3 | 3 | 0 | 0 |
| 9. 複数テストケース | 2 | 2 | 0 | 0 |
| 10. 総和制約 | 3 | 3 | 0 | 0 |
| 11. distinct/sorted | 3 | 3 | 0 | 0 |
| 12. 式付き境界 | 2 | 2 | 0 | 0 |
| 13. choice/section | 1 | 0 | 1 | 0 |
| 14. 文字列 | 4 | 4 | 0 | 0 |
| **合計** | **36** | **33** | **2** | **1** |

### 3.2 総合カバレッジ

| 指標 | 値 |
|------|-----|
| 完全対応率 | **91.7%** (33/36) |
| 部分対応以上 | **97.2%** (35/36) |
| 非対応率 | **2.8%** (1/36) |

---

## 4. 頻度感つき不足一覧

| 不足要素 | 影響問題数 | 頻度感 | 重要度 |
|----------|-----------|--------|--------|
| **下三角行列/jagged 2D** | 1 (ABC370-B) | 低（ABC-B以上で年数回） | 後回し |
| **Tuple内インライン配列** | 2 (ABC259-E, ABC356-C) | 中（可変長行で出現） | 早期 |
| **行内可変長配列** | 2 (上記と同) | 中 | 早期 |
| **正規表現/パターン制約** | 1 (ABC350-A) | 低（Guaranteeで代替可能） | 後回し |

### 4.1 詳細分析

#### 不足A: 下三角行列 (非対応: 1問)

- **現状**: Matrix は rows×cols の矩形のみ
- **必要**: 行iにi個の要素を持つ構造
- **出現頻度**: 低（ABCでは年に数問程度、主にB問題のコスト行列）
- **対策案**:
  1. Repeat.index_varをArray.lengthで参照可能にする
  2. Matrix.shape を追加 (Rectangular, UpperTriangular, LowerTriangular)
- **重要度**: 後回し（出現頻度が低い）

#### 不足B: Tuple内インライン配列 (部分対応: 2問)

- **現状**: Tuple.elements は Vec<NodeId>。Arrayを含めることは可能だが、同一行renderingの保証がない
- **必要**: 「C_i A_{i,1}...A_{i,C_i} R_i」のように1行内に可変長部分を含む
- **出現頻度**: 中（可変長行入力で出現、ABC-C以上で月1-2問程度）
- **対策案**:
  1. RenderHint(Separator::None) で同一行出力を強制
  2. InlineTuple NodeKind を追加
- **重要度**: 早期（中頻度だが、workaroundが不自然）

---

## 5. 現行方針に対する支持/反対/留保

### 5.1 支持する方針

| 方針 | 理由 |
|------|------|
| Choice NodeKindの採用 | クエリ列問題（ABC-D/E で頻出）を完全にカバー。本検証で3問すべて完全対応を確認。 |
| Repeat.countのExpression化 | 木問題のN-1辺を自然に表現可能。本検証で確認。 |
| 12種Constraintの設計 | StringLength, CharSet, Propertyが文字列・グラフ問題をカバー。36問中33問で不足なし。 |
| template-driven編集 | 辺リスト、グリッド、クエリ列テンプレートで操作列が自然に記述可能。 |
| Guaranteeによる自由文制約 | 正規表現パターン、組み合わせ数制約などの稀なケースを吸収。 |

### 5.2 留保する方針

| 方針 | 理由 |
|------|------|
| Expression.Lit(i64) | 2^60（約10^18）までは対応。10^100000級（ABC283-Cなど）は非対応だが、出現頻度は極めて低い。MVP後回しで十分。 |

### 5.3 反対/要検討の方針

| 方針 | 理由 |
|------|------|
| **Tuple内インライン配列の未対応** | ABC356-C のような可変長行がworkaroundでは不自然。RenderHint.Separator(None) または InlineTuple 追加を検討すべき。 |

---

## 6. 他Agentに渡すべき論点

### 6.1 → Domain Model Agent

| 論点 | 詳細 |
|------|------|
| Tuple内インライン配列 | Tuple.elements に Array NodeId を含めた場合の canonical rendering をどう定義するか |
| RenderHint.Separator(None) の適用範囲 | Tuple 内の配列要素を同一行にするか、改行するかの制御 |
| 下三角行列のモデリング | Option 1: Repeat.index_var参照、Option 2: TriangularMatrix追加、Option 3: 非対応維持 |

### 6.2 → GUI Interaction Agent

| 論点 | 詳細 |
|------|------|
| クエリ列テンプレートのUX | Choice variants の追加UIをどう設計するか（ABC278-D のような3種クエリの入力フロー） |
| 辺リストテンプレートの自動化 | 「木」「一般グラフ」「重み付き」の選択肢を用意すべきか |
| Guaranteeの入力方式 | 自由テキスト入力か、テンプレート選択か |

### 6.3 → wasm Boundary Agent

| 論点 | 詳細 |
|------|------|
| Choice NodeKind の ProjectionAPI | Choice.variantsの各ブランチをどう render するか（展開表示か折りたたみか） |
| sample generation と Choice | タグ値の分布（uniform random? or 指定可能?）|

### 6.4 → Critical Reviewer Agent

| 論点 | 詳細 |
|------|------|
| 非対応1問（下三角行列）の許容度 | ABCカバレッジ90%以上が達成されているなら許容か |
| 部分対応2問の改善優先度 | Tuple内インライン配列をMVP必須とすべきか |

---

## 7. 結論

### 7.1 カバレッジ評価

現行の設計（NodeKind 9種、Constraint 12種、Expression 5種、Action 7種）により、**36問中33問（91.7%）が完全対応**、**35問（97.2%）が部分対応以上**である。

ABCのA〜D問題に限れば、カバレッジは**95%以上**と推定される。

### 7.2 MVP必須対応項目

| 項目 | 優先度 |
|------|--------|
| Choice NodeKind の実装検証 | MVP必須（クエリ列カバーに必須） |
| 辺リストテンプレート | MVP必須（木/グラフカバーに必須） |
| グリッドテンプレート | MVP必須（行列入力カバーに必須） |
| クエリ列テンプレート | MVP必須（クエリ問題カバーに必須） |

### 7.3 早期対応推奨項目

| 項目 | 優先度 |
|------|--------|
| Tuple内インライン配列 | 早期（中頻度の可変長行対応） |
| RenderHint.Separator(None) | 早期（上記の前提） |

### 7.4 後回し可能項目

| 項目 | 優先度 |
|------|--------|
| 下三角行列 | 後回し（年数回のみ） |
| BigInt リテラル | 後回し（極稀） |
| 正規表現パターン制約 | 後回し（Guaranteeで代替可能） |

---

## 付録: 検証方法

- AtCoder ABC/ARC/AGC および Codeforces から14カテゴリ×各2-4問 = 計36問を選定
- 各問題の入力形式を分析し、NodeKind/Constraint/Expression/Action への対応を検証
- 操作列は plan.md §19-20 の操作フローに準拠
- 判定基準は §1.3 に記載
