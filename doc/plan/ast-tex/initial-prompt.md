1. 全体の流れと現在地
# cp-ast-ecosystems: 全体の流れと現在地

## 概要

本プロジェクトは、競技プログラミング問題の入力仕様を、自由テキストではなく **構造化された AST** として表現し、それを土台に複数の機能を演繹的に構築することを目的とする。

中核となる考え方は次の通りである。

- 入力仕様の構造は `AST core` に集約する
- 制約は AST に整合する形で保持する
- フロントエンドは自由入力ではなく GUI ベースの構造化エディタとする
- GUI が不正状態を作れないよう、バックエンドが「次に可能な操作」と「操作結果」を定義する
- AST と整合性さえ保証されれば、複数の表現や派生物をバックエンド側で演繹できる

---

## AST core を作る理由

AST core は単なる構文木ではなく、競プロ入力仕様の **意味構造** を保持する中核である。

ここで保持したいものは少なくとも以下である。

- 入力の構造
  - scalar
  - array
  - matrix
  - tuple
  - repeat
  - section
  - choice
  - hole
- 変数間や構造間の参照
- 制約
  - 型
  - 範囲
  - 長さ
  - 関係
  - distinct
  - sorted
  - charset
  - sum bound
  - guarantee
  - render hint など

重要なのは、問題文の見た目そのものではなく、**意味としての入力仕様** を保持することである。

---

## AST core を土台にして嬉しいこと

AST と整合性が保証されていれば、バックエンド側で次のものを **演繹的に** 構築できる。

- 制約の表記
- 入力の表記
- ランダムテスト生成機

つまり、個別に自由テキストや ad-hoc なロジックを持つのではなく、単一の AST から複数の派生物を安定して得られる。

この設計によって、

- 表示と意味のズレが減る
- 編集結果と生成結果の整合が取りやすい
- テスト生成器が構造と制約に基づいて一貫した入力を作れる
- 将来的に他の表現形式へも展開しやすい

という利点がある。

---

## フロントエンド方針

フロントエンドは自由テキスト入力ではなく、**GUI による構造化エディタ** とする。

狙いは以下である。

- ユーザーに不正な状態を直接書かせない
- 複雑な整合性ロジックをフロントエンドに持たせない
- AST の編集操作を明示的な状態遷移として扱う

この方針のために、バックエンド側で次の二つを定義する。

### Projection

現在の AST から見て、

- どのノードが存在しているか
- どのスロットに何を入れられるか
- hole に何を入れられるか
- 次にどの操作が可能か
- どこが未完成か

を返す読み取り用の像である。

Projection の役割は、フロントエンドが自力で複雑な推論をせずとも、UI を一意に構築できるようにすることにある。

### Operation

AST に対して特定の変更を加え、更新後 AST を返す操作である。

例えば次のようなものを含む。

- FillHole
- ReplaceNode
- AddConstraint
- RemoveConstraint
- AddSlotElement
- RemoveSlotElement
- IntroduceMultiTestCase

Operation の役割は、状態遷移を明示化し、GUI 編集を常に合法な範囲に閉じ込めることにある。

---

## Projection / Operation を置く嬉しさ

この設計の最大の利点は、**GUI 上で不正な状態をそもそも構築できなくする** ことである。

フロントエンドは、

- Projection が返した候補を表示する
- ユーザーが選んだ操作を Operation として送る
- 更新後 AST を受け取って再描画する

という流れで動ける。

つまり、

- フロントエンドが AST の整合性ロジックを持たなくてよい
- 可能な状態遷移は常にバックエンドが支配する
- UI は「何が可能か」を演繹するだけでよい

という構成になる。

これは構造化エディタとして非常に相性がよい。

---

## 現在地

ここまでの流れで、すでに次の認識が共有されている。

1. `AST core` はすでに存在する前提である
2. AST と制約を土台に、複数の派生物をバックエンド側で生成したい
3. フロントエンドは GUI ベースの構造化エディタであり、自由テキストではない
4. そのために `projection` と `operation` を定義している
5. まず最初に作りたい派生物は次の二つである
   - 制約の表記
   - 入力の表記

ここではまだ自然文生成全般を目指しているわけではない。

必要なのは、競プロ問題で一般的に見られるような、たとえば次のような表記である。

- 制約欄
  - `1 \le N \le 2 \times 10^5`
  - `1 \le D_i \le 10^9 \ (1 \le i \le N)`
- 入力欄
  - `N Q`
  - `D_1 D_2 \cdots D_N`
  - `T_1`
  - `T_2`
  - `\vdots`
  - `T_Q`

つまり現在の関心は、**AST から競プロらしい入力表記と制約表記を安定生成すること** にある。

---

## 現時点での設計判断

### 採用している考え方

- AST core を唯一の意味モデルとする
- Projection と Operation で GUI 編集を制御する
- フロントエンドは可能な遷移を表示するだけに寄せる
- バックエンドで表記や生成器を演繹する

### 今はまだ重く考えなくてよいもの

- 自然文の Human モード
- 長文 problem statement generator 全体
- render-ir の独立 crate
- フロントエンド独自の複雑なロジック

---

## 直近のターゲット

直近で作るべきものは次の二つである。

1. **制約の表記**
   - AST と制約から競プロ風の constraint 表記を生成する

2. **入力の表記**
   - AST の構造から競プロ風の input layout を生成する

この二つができれば、AST core を土台にした最初の大きな価値が可視化される。

---

## 今後の拡張

この土台が固まれば、将来的には同じ AST から次も導出可能になる。

- ランダムテスト生成機
- statement generator への接続
- TeX 出力
- Web UI プレビュー
- AI Agent の補助出力

ただし現在は、まず

- 制約の表記
- 入力の表記

に集中するのが妥当である。

---

## まとめ

本プロジェクトの現在地は次のように要約できる。

- AST core は入力仕様の意味構造を保持する中核である
- Projection と Operation により、GUI 上で不正状態を作れない構造化エディタを実現する
- AST と整合性が保証されれば、バックエンドで複数の派生物を演繹できる
- まず最初に作るべき派生物は、制約の表記と入力の表記である

この文脈では、自然文生成全般よりも、**競プロらしい安定した pretty printer / TeX renderer** を設計することが優先される。
2. TeX として生成するための方針
# cp-ast-ecosystems: TeX 生成方針

## 目的

`cp-ast-core` を土台として、競技プログラミング問題で一般的に使われる

- 入力の表記
- 制約の表記

を **TeX として安定生成** する。

ここでの目的は自然文生成ではない。

狙うのは、たとえば次のような競プロらしい表記である。

### 入力

- `N \ Q`
- `D_1 \ D_2 \ \cdots \ D_N`
- `T_1`
- `T_2`
- `\vdots`
- `T_Q`

### 制約

- `1 \le N \le 2 \times 10^5`
- `1 \le Q \le 2 \times 10^5`
- `1 \le D_i \le 10^9 \ (1 \le i \le N)`
- `1 \le T_j \le N \ (1 \le j \le Q)`

つまり、必要なのは **競プロ専用の安定した TeX pretty printer** である。

---

## 前提

- `cp-ast-core` はすでに存在する
- AST と制約の意味は core に閉じている
- TeX 側で意味を追加・変更しない
- TeX renderer は core の内容を表現形式に落とすだけとする
- 表記のための補助的な整形はしてよいが、意味を捏造してはいけない

---

## 生成対象

TeX として生成したいものは少なくとも次の二つである。

1. **入力表記**
2. **制約表記**

必要に応じて、将来的に

- 出力表記
- 注意書き
- hole を含む未完成状態の可視化

へ拡張可能とする。

---

## 方針の核心

TeX 生成は、自然文の「Human モード」を持つ必要はない。

必要なのは、

- AST に対して決定的に
- 競プロらしい
- スタイルの揃った
- 差分安定な

TeX 断片を出すことである。

したがって、ここで重視すべき性質は次の通りである。

- **決定性**
- **競プロ表記との親和性**
- **差分安定性**
- **core との意味整合**
- **フロントや Agent から扱いやすいこと**

---

## 生成器の責務

TeX renderer の責務は次に限定する。

### 含むもの

- 識別子の TeX 表現への変換
- 添字付き変数の表現
- `\cdots`, `\vdots` などの挿入
- `\le`, `\times` などの数式記号化
- 制約式の整形
- 入力レイアウトの整形
- itemize 形式などへの整形
- 必要なら section fragment としての整形

### 含まないもの

- AST 編集
- validation 本体
- 自然文の説明生成
- GUI projection
- PDF ビルド
- 問題文全体テンプレートの管理

---

## 生成単位

TeX renderer は最低限、次の単位で生成できるべきである。

### 1. 入力表記の生成

AST の構造から、入力形式を TeX 断片に変換する。

例:

```tex
\[
\begin{array}{l}
N \ Q \\
D_1 \ D_2 \ \cdots \ D_N \\
T_1 \\
T_2 \\
\vdots \\
T_Q
\end{array}
\]
2. 制約表記の生成

Constraint 群から、制約一覧を TeX 断片に変換する。

例:

\begin{itemize}
  \item $1 \le N \le 2 \times 10^5$
  \item $1 \le Q \le 2 \times 10^5$
  \item $1 \le D_i \le 10^9 \ (1 \le i \le N)$
  \item $1 \le T_j \le N \ (1 \le j \le Q)$
\end{itemize}
3. 全体断片の生成

必要なら入力と制約をまとめた fragment を返す。

例:

\paragraph{入力}
...

\paragraph{制約}
...
TeX 表記の基本規則
識別子
N は $N$
Q は $Q$
D_i は $D_i$
D_{i,j} は $D_{i,j}$

変数名は、できるだけ math mode 前提で統一する。

配列

Array(name = D, length = N) のような構造から、入力表記では

D_1 D_2 \cdots D_N

を生成する。

制約表記では

D_i
(1 \le i \le N)

のように、代表添字つきで出す。

繰り返し

Repeat(count = Q, body = T) のような構造から、入力表記では

T_1
T_2
\vdots
T_Q

のような縦展開を生成できるようにする。

範囲制約

Range(target, lower, upper) は可能なら

lower \le target \le upper

の形に正規化する。

例:

1 \le N \le 2 \times 10^5
1 \le D_i \le 10^9
添字の有効範囲

配列や繰り返し変数に対する制約では、必要に応じて

(1 \le i \le N)
(1 \le j \le Q)

のような添字範囲を後ろに付与する。

差分安定性の原則

TeX 生成では、読みやすさよりもまず 決定性 を重視する。

つまり、

同じ AST なら毎回同じ出力
順序が安定している
省略規則が固定
\cdots と \vdots の使い方が固定
制約の並び順が固定

であることが重要である。

これにより、

snapshot test
golden test
UI 差分確認
Agent の比較

がやりやすくなる。

hole の扱い

AST が未完成で hole を含む場合も、TeX renderer は壊れてはいけない。

hole は無視せず、明示的に見える形で出す。

例:

\texttt{<hole>}
\texttt{<array-hole>}
\boxed{\texttt{hole}}

どの記法を採用するかは後で決めればよいが、原則として

renderer が落ちない
未完成であることが見える
位置が分かる

ことを優先する。

API 方針

最低限必要なのは次のような API である。

Rust 側の概念 API
render_input_tex(document, options) -> TexOutput
render_constraints_tex(document, options) -> TexOutput
render_full_tex(document, options) -> TexOutput

TexOutput には少なくとも次を含める。

tex: String
warnings: Vec<TexWarning>
option の方向性

オプションは最初は小さく保つ。

例:

section_mode: Fragment | Standalone
include_holes: bool
style: AtCoderLike | GenericCP

ここでいう style は自然文スタイルではなく、競プロ表記の流儀 の違いに限定する。

入力表記と制約表記は分ける

設計上かなり重要なのは、

入力表記
制約表記

を別 renderer あるいは別 entry point として扱うことである。

理由は以下。

入力表記は AST の構造に強く依存する
制約表記は Constraint の正規化に強く依存する
テスト戦略も別になりやすい
UI 上でも別セクションとして使われる

したがって、最初から責務を分けておく方がよい。

WASM / フロントエンドとの接続方針

TeX renderer はフロントエンドに直接ロジックを持たせない。

バックエンドまたは wasm 側で TeX を生成し、フロントはそれを表示するだけに寄せる。

境界 API としては、たとえば次で十分である。

render_input_tex(request_json) -> response_json
render_constraints_tex(request_json) -> response_json
render_full_tex(request_json) -> response_json

これにより、

フロントは AST の細かい整形ロジックを持たない
競プロ表記の規則は Rust 側で一元管理できる
projection / operation と同じ思想で、表示規則もバックエンド支配にできる
テスト方針

TeX 生成は golden test と相性がよい。

最低限、次を用意する。

入力表記テスト
scalar のみ
tuple
array
repeat
array + repeat の組み合わせ
hole を含む場合
制約表記テスト
scalar の range
array element の range
複数制約の並び
index range 付き制約
unsupported な制約の warning
全体テスト
問題例に近い AST から、期待する TeX fragment が丸ごと出るか
今の位置づけ

現時点では、自然文全般の生成器を先に作る必要は薄い。

必要なのは、AST を土台とした最初の明確な価値として、

制約の表記
入力の表記

を TeX として生成することにある。

したがって優先順位としては、

制約 TeX renderer
入力 TeX renderer
必要ならそれらを束ねる full fragment renderer

の順が自然である。

まとめ

この段階で目指すべき TeX 生成器は、自然文生成器ではなく、次の性質を持つ競プロ専用 pretty printer である。

AST / Constraint に整合している
差分安定である
入力表記と制約表記を安定生成できる
hole を含む未完成状態でも壊れない
フロントエンドが複雑な整形ロジックを持たなくてよい

この方針により、cp-ast-core を土台とした最初の可視価値を、TeX 断片として明確に提供できる。
