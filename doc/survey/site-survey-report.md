# AtCoder 問題構造サーベイレポート

> Sprint 1 — SiteSurveyAgent による競プロサイト問題記述の共通構造と差異の観察結果

---

## 1. サンプリング結果一覧

| # | 問題 | カテゴリ | 概要 |
|---|------|----------|------|
| 1 | [ABC300-A](https://atcoder.jp/contests/abc300/tasks/abc300_a) N-choice question | 基本（整数+配列） | 整数A,Bの和がどの選択肢に一致するか。1行目にN,A,B、2行目にN個の選択肢 |
| 2 | [ABC350-C](https://atcoder.jp/contests/abc350/tasks/abc350_c) Sort | 基本（配列） | 順列のソート操作列を出力。出力行数が可変 |
| 3 | [ABC284-C](https://atcoder.jp/contests/abc284/tasks/abc284_c) Count Connected Components | グラフ（一般） | 単純無向グラフの連結成分数。N,M + 辺リスト |
| 4 | [ABC244-E](https://atcoder.jp/contests/abc244/tasks/abc244_e) King Bombee | グラフ（DP） | グラフ上のパス数え上げ。1行目に6変数、以降M辺 |
| 5 | [ABC300-B](https://atcoder.jp/contests/abc300/tasks/abc300_b) Same Map in the RPG World | 行列/2D配列 | 2つのH×Wグリッドの循環シフト一致判定 |
| 6 | [ABC300-C](https://atcoder.jp/contests/abc300/tasks/abc300_c) Cross | 行列/2D配列 | H×Wグリッド上のバツ印パターン検出 |
| 7 | [AGC062-A](https://atcoder.jp/contests/agc062/tasks/agc062_a) Right Side Character | 複数テストケース | T個のテストケース、各ケースはN+文字列S |
| 8 | [ABC217-D](https://atcoder.jp/contests/abc217/tasks/abc217_d) Cutting Woods | クエリ形式 | L,Q + Q個のクエリ (c_i, x_i) |
| 9 | [ABC249-B](https://atcoder.jp/contests/abc249/tasks/abc249_b) Perfect String | 文字列 | 単一文字列の判定。入力は文字列1行のみ |
| 10 | [ABC299-E](https://atcoder.jp/contests/abc299/tasks/abc299_e) Nearest Black Vertex | 可変長・複合入力 | グラフ + 条件リスト（2セクション構成） |
| 11 | [ABC371-C](https://atcoder.jp/contests/abc371/tasks/abc371_c) Make Isomorphic | 可変長・相互依存制約 | 2グラフ + 三角行列コスト。入力構造が複雑 |
| 12 | [ABC313-D](https://atcoder.jp/contests/abc313/tasks/abc313_d) Odd or Even | 対話型(interactive) | インタラクティブ。`?` で質問、`!` で回答 |
| 13 | [ABC300-E](https://atcoder.jp/contests/abc300/tasks/abc300_e) Dice Product 3 | 単一整数入力 | 入力はNのみ。確率をmod出力 |
| 14 | [ARC154-A](https://atcoder.jp/contests/arc154/tasks/arc154_a) Swap Digit | 文字列（数値） | N + 2つのN桁整数（文字列として入力） |

---

## 2. 共通パターン

### 2.1 Input Format の定型構造

#### パターン I-1: ヘッダ行 + データ行（最頻出）

最も基本的な構造。1行目に次元パラメータ、続く行にデータ。

```
N [M] [その他パラメータ...]
データ行1
データ行2
...
```

**例:** abc284_c (グラフ)
```
N M
u_1 v_1
u_2 v_2
⋮
u_M v_M
```

**例:** abc350_c (配列)
```
N
A_1 A_2 … A_N
```

**例:** abc300_a (ヘッダに複数値 + 1行データ)
```
N A B
C_1 C_2 … C_N
```

**観察:** ヘッダ行のパラメータ数は可変（1〜6個）。データ行数はヘッダのパラメータ（N, M, Q 等）に依存する。

#### パターン I-2: グリッド入力

H×W のグリッドを行ごとに読む。文字グリッドは空白区切り**なし**。

```
H W
C[1][1]C[1][2]…C[1][W]
C[2][1]C[2][2]…C[2][W]
⋮
C[H][1]C[H][2]…C[H][W]
```

**例:** abc300_b, abc300_c

**観察:**
- 文字グリッドの各行は連結（空白なし）: `..#.#.`
- abc300_b では2つのグリッドが縦に連結される（A のH行 → B のH行）

#### パターン I-3: グラフ入力（辺リスト）

```
N M [追加パラメータ...]
u_1 v_1 [w_1]
u_2 v_2 [w_2]
⋮
u_M v_M [w_M]
```

**例:** abc284_c (`N M` + 辺リスト), abc244_e (`N M K S T X` + 辺リスト)

**観察:**
- 辺に重みがある場合は `u_i v_i w_i` の3列
- 追加パラメータ（S, T, X, K 等）はヘッダ行に同居する場合が多い

#### パターン I-4: クエリ形式

```
[初期パラメータ...]
Q
c_1 x_1 [y_1 ...]
c_2 x_2 [y_2 ...]
⋮
c_Q x_Q [y_Q ...]
```

**例:** abc217_d
```
L Q
c_1 x_1
c_2 x_2
⋮
c_Q x_Q
```

**観察:** クエリの種類コード `c_i` が1列目に来て、残りがそのクエリのパラメータ。

#### パターン I-5: 複数テストケース

```
T
case_1
case_2
⋮
case_T
```

**例:** agc062_a
```
T
N
S
(上記が T 回繰り返し)
```

**観察:**
- 各ケースの入力構造は他のパターンと同じ
- 総和制約が付くことが多い: 「N の総和は 3×10^5 以下」

#### パターン I-6: 複合セクション入力

入力が複数の独立したセクションで構成される。

**例:** abc299_e（グラフ + 条件リスト）
```
N M
u_1 v_1          ← セクション1: グラフ
⋮
u_M v_M
K                 ← セクション2: 条件
p_1 d_1
⋮
p_K d_K
```

**例:** abc371_c（2グラフ + コスト行列）
```
N
M_G              ← セクション1: グラフG
u_1 v_1
⋮
M_H              ← セクション2: グラフH
a_1 b_1
⋮
A_{1,2} … A_{1,N}  ← セクション3: 三角行列
A_{2,3} … A_{2,N}
⋮
A_{N-1,N}
```

#### パターン I-7: インタラクティブ（双方向I/O）

```
初期情報（N K 等）を読む

ループ:
  質問: ? x_1 x_2 … x_K   → stdout
  応答: T                  ← stdin

回答: ! A_1 A_2 … A_N     → stdout
```

**例:** abc313_d

**観察:** `?` プレフィックスで質問、`!` プレフィックスで最終回答。flush が必須。

### 2.2 Constraints の定型構造

#### パターン C-1: 単純な値域制約

```
1 ≤ N ≤ 300
1 ≤ A, B ≤ 1000
2 ≤ N ≤ 10^{18}
```

**表記パターン:**
- 不等式: `1 ≤ X ≤ 上界`
- 上界に `10^k` 形式が頻出
- 複数変数を併記: `1 ≤ A, B ≤ 1000`

#### パターン C-2: 型宣言

```
入力は全て整数
All values in the input are integers.
S は英大文字と英小文字からなる文字列である。
H, W は整数
```

**観察:** 日本語版と英語版でほぼ同一の表現。「入力は全て整数」が定型句。

#### パターン C-3: 導出・関数的制約

```
0 ≤ M ≤ N(N-1)/2
N-1 ≤ M ≤ min{N(N-1)/2, 2000}
1 ≤ |S| ≤ 100
```

**観察:** 変数間の関係式で上界/下界を定める。グラフのM は N の式で制約されることが多い。

#### パターン C-4: 変数間の関係制約

```
1 ≤ u_i < v_i ≤ N          (順序関係)
X ≠ S, X ≠ T               (不等関係)
(u_i, v_i) ≠ (u_j, v_j)    (タプルの相異)
C_i は相異なる               (全要素の相異)
1 ≤ p_1 < p_2 < … < p_K ≤ N (ソート済み)
```

**観察:** 順序関係（<）、不等（≠）、相異条件、ソート済み条件が頻出。

#### パターン C-5: 構造的制約（入力全体の性質）

```
(A_1,…,A_N) は (1,2,…,N) の並べ替え
与えられるグラフは単純かつ連結
K は奇数
A_i は 0 または 1
```

**観察:** グラフの性質（単純、連結、木）、数列の性質（順列、二値）がこのカテゴリ。

#### パターン C-6: 総和制約（複数テストケース用）

```
1 つの入力に含まれるテストケースについて、N の総和は 3×10^5 以下
```

**観察:** 複数テストケース問題でのみ出現。計算量保証用。

#### パターン C-7: 存在保証

```
A+B=C_i なる i が丁度 1 つ存在する
線 x_i はクエリを処理する時点で切られていないことが保証される
```

**観察:** 解の存在や入力の妥当性を保証する制約。

### 2.3 Output Format の定型構造

| パターン | 形式 | 例 |
|----------|------|-----|
| O-1: 単一値 | 整数1つ | abc300_a, abc284_c, abc300_e |
| O-2: Yes/No | `Yes` / `No` | abc300_b, abc249_b |
| O-3: 空白区切り列 | `S_1 S_2 … S_N` | abc300_c |
| O-4: クエリ応答（複数行） | 各クエリに1行ずつ | abc217_d |
| O-5: 条件分岐出力 | Yes なら追加行、No なら1行 | abc299_e (`Yes\nS` or `No`) |
| O-6: 可変行数操作列 | `K` 行目に操作数、以降 K 行 | abc350_c |
| O-7: mod 出力 | `答えを 998244353 で割ったあまり` | abc300_e, abc244_e, arc154_a |
| O-8: インタラクティブ | `?` で質問、`!` で最終回答 | abc313_d |

### 2.4 Sample Input/Output の構成

**共通構造:**
- 各問題に2〜4個のサンプルが付属
- 形式: `### 入力例 k` / `### 出力例 k` の対（日英併記の場合は `### Sample Input k` / `### Sample Output k` も）
- 最初のサンプルには丁寧な解説（ステップごとの説明）が付く
- 後続サンプルはエッジケース（空入力、最小ケース、最大ケース付近）を含む

---

## 3. 差異・表現揺れ

### 3.1 サイト内の表記揺れ

| 項目 | バリエーション | 例 |
|------|---------------|-----|
| 添字表記 | `C_i` vs `C[i][j]` vs `A_{i,j}` | abc300_a は `C_i`、abc300_c は `C[i][j]` |
| 省略記号 | `\dots` vs `\ldots` vs `\vdots` | 横: `\dots`/`\ldots`、縦: `\vdots` |
| 行区切り記述 | 明示的改行 vs `\vdots` | ほぼ全問で `\vdots` を使用 |
| グリッド添字記法 | `A_{i,j}` vs `C[i][j]` | 同一コンテスト内でも揺れ (abc300_b vs abc300_c) |
| 制約の型宣言位置 | 制約セクション内 vs 問題文内 | abc300_c: 制約に `H, W は整数`、abc284_c: 制約に `入力される値はすべて整数` |
| 数値上界の書き方 | `10^{18}` vs `10^9` vs `10 ^ 6` | スペースの有無に揺れ |

### 3.2 カテゴリ間の構造的差異

| 差異 | 詳細 |
|------|------|
| **ヘッダのパラメータ数** | 基本問題は1〜3個 (N, H W)、グラフ問題は2〜6個 (N M K S T X) |
| **データ行の区切り** | 数値は空白区切り、文字グリッドは無区切り（連結文字列） |
| **入力セクション数** | 基本問題は1セクション、グラフ+条件問題は2〜3セクション |
| **出力形式の多様性** | 基本問題は単一値が多い、構成問題は可変行数出力 |
| **制約の複雑さ** | 基本問題は単純値域のみ、グラフ問題は変数間制約が多い |

### 3.3 日英間の対応

- 問題文は日英併記（同一ページに両方掲載）
- 入力形式 (Input Format) と制約 (Constraints) は日英で**構造的に同一**
- 表現の違い: 「入力は全て整数」↔「All values in the input are integers.」

---

## 4. 例外表現（特殊な入力形式・非典型的な制約記述）

### 4.1 三角行列入力 — abc371_c

コスト行列 `A_{i,j}` が上三角形式で、行ごとに列数が減少する:

```
A_{1,2} A_{1,3} … A_{1,N}     ← N-1 要素
A_{2,3} … A_{2,N}             ← N-2 要素
⋮
A_{N-1,N}                     ← 1 要素
```

**特殊性:** 行数が `N-1` で固定だが、各行の要素数が `N-i` と可変。変数 i に依存した行幅。

### 4.2 複数グリッドの連結入力 — abc300_b

2つの H×W グリッドが区切りなく連結される:

```
H W
(グリッドA: H行)
(グリッドB: H行)
```

**特殊性:** セクション区切りの明示的マーカーがない。パーサは `H` の値を使って分割する必要がある。

### 4.3 インタラクティブプロトコル — abc313_d

通常の「入力→処理→出力」ではなく、双方向通信:

```
読む: N K
繰り返し:
  書く: ? x_1 x_2 … x_K
  読む: T
書く: ! A_1 A_2 … A_N
```

**特殊性:**
- 入出力が交互に発生
- `?` / `!` プレフィックスによるメッセージ型の区別
- flush が必須
- 適応的ジャッジ（応答が過去の質問に矛盾しない範囲で変わる）
- エラー応答 `-1` のハンドリング

### 4.4 可変行数の出力 — abc350_c

出力行数が入力依存で決まる:

```
K            ← 操作回数（0〜N-1）
i_1 j_1     ← K行の操作
⋮
i_K j_K
```

**特殊性:** 出力の1行目が後続行数を決定。「複数の正解が存在する」旨が明記される。

### 4.5 複合セクション + 可変長セクション — abc299_e

```
N M           ← セクション1のヘッダ
(M行の辺)     ← セクション1: グラフ
K             ← セクション2のヘッダ（独立した行）
(K行の条件)   ← セクション2: 条件
```

**特殊性:** セクション間の境界は「ヘッダ行がデータ行と異なる列数」で暗黙的に区切られる。M=0 や K=0 の場合、データ行がゼロ行になる。

### 4.6 mod 出力と確率の有理数表現 — abc300_e

**特殊性:** 出力値自体は整数だが、その意味が「有理数 P/Q の mod 逆元 R」という数学的定義。問題文に mod の説明が折りたたみで詳述される。

### 4.7 単一変数入力 — abc300_e

```
N
```

**特殊性:** 入力がスカラー1個のみ。ヘッダ行とデータ行の区別がない。

---

## 5. 最小共通核の提案 — DSL で最低限押さえるべき構造

以上のサーベイから、競プロ問題の入力記述を AST で表現するための最小共通核を提案する。

### 5.1 入力構造 (StructureAST) の最小要素

```
InputSpec          := Section+                    -- 1つ以上のセクション
Section            := Header? DataBlock*          -- ヘッダ(任意) + データブロック列
Header             := Variable+                   -- 空白区切りの変数群 (N, M, K 等)
DataBlock          := FixedLines                  -- 固定N行
                    | RepeatLines(count_var)       -- 変数依存の行数 (M行, Q行 等)
                    | Grid(rows_var, cols_var)     -- グリッド (H×W)
                    | TriangularBlock(size_var)    -- 三角行列
Line               := Element+                    -- 空白区切りの要素列
                    | CharSeq(length_var)          -- 無区切り文字列
Element            := Variable | Literal
```

### 5.2 変数・型システム

```
Variable := {
  name: Ident,         -- 変数名 (N, M, A_i, C[i][j] 等)
  index: Index?,       -- 添字 (i, {i,j} 等)  ※なければスカラー
  typ: Type,           -- Int | Str | Char
}

Index := SingleIndex(var)
       | MultiIndex(var, var)           -- 2次元添字

Type := Int | Str | Char | Mod(modulus)  -- Mod は mod 出力用
```

### 5.3 制約式 (ConstraintAST) の最小要素

```
Constraint := Range(var, lower, upper)           -- 1 ≤ N ≤ 10^5
            | TypeDecl(var, type)                 -- N は整数 / S は文字列
            | Relation(var, op, var_or_expr)      -- X ≠ S / u_i < v_i
            | Distinct(var_set)                   -- C_i は相異なる
            | Sorted(var_sequence, order)         -- p_1 < p_2 < … < p_K
            | Property(var, property_tag)         -- グラフは連結 / A は順列
            | SumBound(var, over_cases, upper)    -- Nの総和 ≤ 3×10^5
            | Derived(var, expr)                  -- M ≤ N(N-1)/2
            | Guarantee(predicate)               -- 解が存在する等の保証
```

### 5.4 出力構造

```
OutputSpec := SingleValue(type)                  -- 単一値
            | YesNo                               -- Yes / No
            | ConditionalOutput(cond, then, else) -- Yes + 追加行 / No
            | MultiLine(count_expr)               -- 複数行（行数固定 or 可変）
            | SpaceSep(var_list)                  -- 空白区切り
            | Interactive(protocol)               -- インタラクティブ
            | ModOutput(modulus)                  -- mod 998244353 等
```

### 5.5 メタ構造

```
Problem := {
  input: InputSpec,
  output: OutputSpec,
  constraints: Constraint[],
  samples: Sample[],
  interactive: bool,
  multi_testcase: Option<{count_var, sum_bound?}>,
}

Sample := {
  input: RawText,
  output: RawText,
  explanation: Option<RawText>,
}
```

### 5.6 カバレッジ検証

上記の最小核が各カテゴリをカバーする対応表:

| カテゴリ | 必要な構造要素 |
|----------|---------------|
| 基本（整数+配列） | Header + RepeatLines / FixedLines |
| グラフ | Header(N,M) + RepeatLines(M) + Line(u,v) |
| 行列/2D | Header(H,W) + Grid(H,W) + CharSeq |
| 複数テストケース | multi_testcase + SumBound |
| クエリ | Header(L,Q) + RepeatLines(Q) + Line(c,x) |
| 文字列 | FixedLines + CharSeq |
| 可変長 | Section × 複数 + TriangularBlock |
| 相互依存制約 | Relation + Distinct + Sorted |
| インタラクティブ | Interactive(protocol) |

### 5.7 優先度の高い実装順序

1. **Header + RepeatLines + Line** — 全問題の 90% 以上をカバー
2. **Grid + CharSeq** — 2Dグリッド問題に必須
3. **Range + TypeDecl + Relation** — 制約の 80% をカバー
4. **Section 分割** — 複合入力問題に必要
5. **multi_testcase / Interactive** — 特殊形式への対応
6. **TriangularBlock / ConditionalOutput** — レアケース対応

---

## 付録: 各問題の詳細分析

### A. ABC300-A: N-choice question

**Input Format:**
```
N A B
C_1 C_2 … C_N
```
- 行1: 3つのスカラー (N, A, B) を空白区切り
- 行2: N個の整数を空白区切り（1行に並列）

**Constraints:**
- 単純な値域: `1 ≤ N ≤ 300`, `1 ≤ A,B ≤ 1000`, `1 ≤ C_i ≤ 2000`
- 型宣言: 「入力は全て整数」
- 相異条件: 「C_i は相異なる」
- 存在保証: 「A+B=C_i なる i が丁度 1 つ存在する」

**Output:** 単一整数

---

### B. ABC350-C: Sort

**Input Format:**
```
N
A_1 … A_N
```
- 行1: スカラー N
- 行2: N個の整数（空白区切り、1行）

**Constraints:**
- `2 ≤ N ≤ 2×10^5`
- 構造的制約: 「(A_1,…,A_N) は (1,2,…,N) の並べ替え」

**Output:** 可変行数。1行目に操作数 K、以降 K 行に操作ペア。複数解許容。

---

### C. ABC284-C: Count Connected Components

**Input Format:**
```
N M
u_1 v_1
u_2 v_2
⋮
u_M v_M
```
- 行1: ヘッダ (N, M)
- 行2〜M+1: 辺リスト（各行2要素）
- M=0 の場合、辺の行がゼロ行（入力例2）

**Constraints:**
- `1 ≤ N ≤ 100`, `0 ≤ M ≤ N(N-1)/2`
- `1 ≤ u_i, v_i ≤ N`
- 「入力で与えられるグラフは単純」
- 「入力される値はすべて整数」

**Output:** 単一整数

---

### D. ABC244-E: King Bombee

**Input Format:**
```
N M K S T X
U_1 V_1
U_2 V_2
⋮
U_M V_M
```
- 行1: 6つのスカラー（ヘッダに多数のパラメータ）
- 行2〜M+1: 辺リスト

**Constraints:**
- 値域: `2≤N≤2000`, `1≤M≤2000`, `1≤K≤2000`
- 変数間制約: `X≠S`, `X≠T`, `1≤U_i<V_i≤N`
- タプル相異: `i≠j ならば (U_i,V_i)≠(U_j,V_j)`

**Output:** mod 998244353 で整数1つ

---

### E. ABC300-B: Same Map in the RPG World

**Input Format:**
```
H W
A_{1,1}A_{1,2}…A_{1,W}
⋮
A_{H,1}A_{H,2}…A_{H,W}
B_{1,1}B_{1,2}…B_{1,W}
⋮
B_{H,1}B_{H,2}…B_{H,W}
```
- 行1: ヘッダ (H, W)
- 行2〜H+1: グリッドA（各行W文字、無区切り）
- 行H+2〜2H+1: グリッドB（同形式）

**Constraints:**
- `2 ≤ H, W ≤ 30`
- 文字種制約: `A_{i,j} は '#' または '.'`

**Output:** `Yes` / `No`

---

### F. ABC300-C: Cross

**Input Format:**
```
H W
C[1][1]C[1][2]…C[1][W]
⋮
C[H][1]C[H][2]…C[H][W]
```
- グリッド入力（abc300_bと同形式、1グリッドのみ）
- 添字記法が `C[i][j]`（abc300_bの `A_{i,j}` とは異なる）

**Constraints:**
- `3 ≤ H, W ≤ 100`（下界が2ではなく3）
- 構造的制約: 「異なるバツ印を構成するマス同士は頂点を共有しない」

**Output:** 空白区切りの N 個の整数 (`S_1 S_2 … S_N`)

---

### G. AGC062-A: Right Side Character

**Input Format:**
```
T
N
S
(N, S が T 回繰り返し)
```
- 行1: テストケース数 T
- 各ケース: 2行 (N と S)

**Constraints:**
- `1 ≤ T ≤ 10^5`, `2 ≤ N ≤ 3×10^5`
- 文字種: 「S は 'A','B' のみからなる長さ N の文字列」
- **総和制約:** 「N の総和は 3×10^5 以下」

**Output:** T行、各行に1文字

---

### H. ABC217-D: Cutting Woods

**Input Format:**
```
L Q
c_1 x_1
c_2 x_2
⋮
c_Q x_Q
```
- 行1: ヘッダ (L, Q)
- 行2〜Q+1: クエリ列（種類コード + パラメータ）

**Constraints:**
- `1 ≤ L ≤ 10^9`, `1 ≤ Q ≤ 2×10^5`
- クエリ種類: `c_i = 1, 2`
- **条件付き保証:** 「線 x_i はクエリを処理する時点で切られていないことが保証される」

**Output:** c_i=2 のクエリの数だけ行を出力

---

### I. ABC249-B: Perfect String

**Input Format:**
```
S
```
- 入力は文字列1行のみ

**Constraints:**
- `1 ≤ |S| ≤ 100`
- 文字種: 「S は英大文字と英小文字からなる文字列」

**Output:** `Yes` / `No`

---

### J. ABC299-E: Nearest Black Vertex

**Input Format:**
```
N M
u_1 v_1
⋮
u_M v_M
K
p_1 d_1
⋮
p_K d_K
```
- 2セクション構成: グラフ (N,M + 辺) + 条件 (K + 条件行)
- K が単独行として出現（セクション区切り）

**Constraints:**
- 導出制約: `N-1 ≤ M ≤ min{N(N-1)/2, 2000}`
- ソート済み: `1 ≤ p_1 < p_2 < … < p_K ≤ N`
- ゼロ許容: `0 ≤ K ≤ N`, `0 ≤ d_i ≤ N`

**Output:** 条件分岐 — `No` or `Yes\n(01文字列)`

---

### K. ABC371-C: Make Isomorphic

**Input Format:**
```
N
M_G
u_1 v_1
⋮
u_{M_G} v_{M_G}
M_H
a_1 b_1
⋮
a_{M_H} b_{M_H}
A_{1,2} A_{1,3} … A_{1,N}
A_{2,3} … A_{2,N}
⋮
A_{N-1,N}
```
- 3セクション: グラフG + グラフH + 三角行列
- 三角行列: 行 i の要素数は N-i（可変幅）

**Constraints:**
- `1 ≤ N ≤ 8`（小さい上界）
- 辺制約: `1 ≤ u_i < v_i ≤ N` かつ辺の相異
- コスト制約: `1 ≤ A_{i,j} ≤ 10^6`

**Output:** 単一整数

---

### L. ABC313-D: Odd or Even (Interactive)

**Input/Output Protocol:**
```
入力: N K
質問: ? x_1 x_2 … x_K  → stdout
応答: T (0 or 1)        ← stdin
回答: ! A_1 A_2 … A_N   → stdout
```

**Constraints:**
- `1 ≤ K < N ≤ 1000`
- `K は奇数`
- `A_i は 0 または 1`

**特殊事項:**
- 質問回数上限: N 回
- 適応的ジャッジ
- エラー時 `-1` 応答
- flush 必須

---

### M. ABC300-E: Dice Product 3

**Input Format:**
```
N
```
- 入力はスカラー1個のみ

**Constraints:**
- `2 ≤ N ≤ 10^{18}` (非常に大きい上界)
- `N は整数`

**Output:** mod 998244353 で整数1つ。確率を有理数→mod逆元で表現。

---

### N. ARC154-A: Swap Digit

**Input Format:**
```
N
A
B
```
- 行1: 桁数 N
- 行2, 3: N桁の正整数（文字列として入力）

**Constraints:**
- `1 ≤ N ≤ 200000`
- 「A,B は先頭の桁が 0 でない N 桁の正整数」（暗黙に文字列長 = N）

**Output:** mod 998244353 で整数1つ
