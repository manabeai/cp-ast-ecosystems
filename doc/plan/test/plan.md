1. もっとも基本的な配列入力
作りたい仕様
N
A_1 A_2 ... A_N

制約:

1 <= N <= 2 * 10^5
1 <= A[i] <= 10^9
A は整数配列

もっと UI 上の完成イメージに寄せると:

N: int
A: int[N]

constraints:
- 1 <= N <= 2*10^5
- 1 <= A[i] <= 10^9
ユーザー操作フロー
Step 1: 変数 N を追加

ユーザーは Structure ペインで + 変数 を押す。

種類: Scalar
名前: N

この時点では型未設定でもよい。

Step 2: N に型を付ける

Detail ペインで N を選び、型を int にする。

この操作で AST 的には

Scalar(name=N)
TypeDecl(target=N, Int)

がそろう。

Step 3: 配列 A を追加

ユーザーは + 配列 を押す。

名前: A
要素型: int

この時点では長さ未設定でもよい。

UI には

A: int[?]

のように見えていてよい。

Step 4: 配列長に N を入れる

A の長さスロットをクリックし、候補から N を選ぶ。

このとき式編集は自由入力ではなく、

候補: N
定数
式テンプレート

のような UI でよい。

確定後:

A: int[N]

になる。

Step 5: N の Range 制約を追加

Constraint ペインで + Range。

target: N
lower: 1
upper: 2*10^5

これは draft constraint として組み立てて、完成時に AST に追加する。

Step 6: A[i] の Range 制約を追加

再び + Range。

target: A[i]
lower: 1
upper: 10^9

ここで target は PlaceExpr 的な対象選択 UI が必要。

つまり

A
A[i]
N

みたいな候補から選べる必要がある。

この例で確認できること

このケースで最低限必要なのは:

Scalar 追加
Array 追加
Type 設定
length slot 編集
Range constraint draft
A[i] の target 指定

つまりこれは MVP の最重要ケース。

2. 木入力 / 辺リスト入力
作りたい仕様
N
u_1 v_1
u_2 v_2
...
u_{N-1} v_{N-1}

制約:

2 <= N <= 2 * 10^5
1 <= u[i], v[i] <= N
u[i] != v[i]
与えられるグラフは木

UI 的な完成イメージ:

N: int
edges: repeat(N - 1) {
  u: int
  v: int
}

constraints:
- 2 <= N <= 2*10^5
- 1 <= u[i] <= N
- 1 <= v[i] <= N
- u[i] != v[i]
- edges has property Tree
ユーザー操作フロー
Step 1: N を追加

配列入力のときと同じ。

Scalar N
TypeDecl int
Step 2: 繰り返しブロックを追加

Structure で + Repeat を押す。

count slot が現れる
body は空 or Hole つき

ここで body にまだ何も入っていなくてよい。

Step 3: Repeat の count を N - 1 にする

count slot をクリック。

最初に N を選ぶ。
次に - x を選ぶ。
右辺に 1 を選ぶ。

ここで重要なのは、途中で AST に N - ? を入れなくてよいこと。
UI の pending action で

target = N
op = -
rhs 未確定

を持ち、1 が選ばれた時点で完成式 N - 1 を入れる。

Step 4: body に u, v の Tuple を追加

Repeat body の Hole をクリックして候補を出す。

候補から

Tuple
2つの Scalar を持つテンプレート

などを選べるとよい。

その後、body の中に

u
v

を追加する。

Step 5: u, v に型を付ける

両方 int。

Step 6: N の制約を追加

Range:

target: N
lower: 2
upper: 2*10^5
Step 7: u[i], v[i] の制約を追加

それぞれ Range:

1 <= u[i] <= N
1 <= v[i] <= N

ここでは Repeat 内のノードに対する indexed target の扱いが必要。

Step 8: u[i] != v[i] を追加

Constraint ペインで Relation を追加。

lhs: u[i]
op: !=
rhs: v[i]
Step 9: Tree property を追加

Constraint ペインで Property を追加。

target: edge list / repeat block
property: Tree

ここは高レベル制約ショートカットが欲しい。

この例で確認できること

このケースで必要なのは:

Repeat
count 式編集
body Hole
Tuple / 複数 field 追加
indexed target
property constraint

つまり 構造 Hole の価値 がかなり出る。

3. クエリ列 / Choice 入力
作りたい仕様
Q
query_1
query_2
...
query_Q

各 query は type に応じて形が違う。たとえば:

1 x
2 i x
3 i

制約:

1 <= Q <= 2 * 10^5
query type は 1, 2, 3 のいずれか
i は有効な index
x は整数

UI 的には:

Q: int
queries: repeat(Q) {
  choice(tag=t) {
    1 => (t, x)
    2 => (t, i, x)
    3 => (t, i)
  }
}
ユーザー操作フロー
Step 1: Q を追加
Scalar Q
TypeDecl int
Step 2: Repeat を追加
count = Q
Step 3: Repeat body に Choice を追加

body Hole をクリックし、候補から Choice を選ぶ。

この時点で Choice には

tag slot
variant list

が見える。

Step 4: tag 変数 t を追加

Choice の tag に使う変数として t を追加する。
もしくは Choice テンプレートが自動で t を作ってもよい。

型は int。

Step 5: variant を 3 つ追加

ユーザーは + variant を 3 回押す。

variant 1: literal 1
variant 2: literal 2
variant 3: literal 3
Step 6: 各 variant の body を埋める
variant 1

body に Tuple を追加:

t
x
variant 2

body に Tuple を追加:

t
i
x
variant 3

body に Tuple を追加:

t
i

ここでも body は Hole ベースで埋めていくのが自然。

Step 7: 型を付ける
t: int
i: int
x: int
Step 8: 制約を足す
1 <= Q <= 2*10^5
t in {1,2,3}
i の範囲
x の範囲

必要なら variant ごとの制約や RenderHint を追加。

この例で確認できること

このケースで必要なのは:

Repeat
Choice
variant 追加
tag 管理
body Hole
variant ごとの異なる構造
制約 target の文脈依存

これは 中程度に複雑なケース の代表。

3例を通して見えること
例1で必要
Scalar
Array
length slot
Range
例2で必要
Repeat
count expr
Tuple
Property(Tree)
indexed target
例3で必要
Choice
variant body
tag
複数の構造テンプレート