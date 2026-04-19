# 競技プログラミング AST Editor  
## UI設計ガイドライン（実装用）

---

## 0. この文書の目的

この文書は、Competitive Programming AST Editor の UI を設計・実装するための **具体的な設計指針** である。

単なる思想メモではなく、

- AI Agent にそのまま渡せる
- 実装の判断基準になる
- E2E テストの基準になる

ことを目的とする。

---

## 1. 最重要設計思想

---

### 1.1 Structure ペインは「ASTビュー」ではない

Structure ペインはツリー構造を見せる UI ではない。

ユーザーが見るのは **競プロの入力形式そのもの** である。

例：

```text
N
A_1 ... A_N
H W
S_1
...
S_H
N M
u_1 v_1
...
u_M v_M

ユーザーは「木構造を編集している」のではなく
入力フォーマットを直接組み立てている感覚を持つべき。

1.2 編集は「挿入位置をクリック」から始まる

ユーザーは自由入力ではなく、

右に追加
下に追加
ブロック内に追加
variant を追加

といった 方向付き挿入ポイント（insertion hotspot） をクリックして操作する。

内部では Hole でもよいが、UI 上では

❌ ?
ではなく
✅ 「ここに追加できる」ボックス

として見せる。

1.3 数式は「表示されたものを直接触る」

ユーザーは入力欄に値を打つのではなく、

1 <= A_i <= 10^9

の

1
A_i
10^9

を直接クリックして編集する。

つまり UI は

文字列ではなく
クリック可能なレンダリング木

である必要がある。

1.4 Structure 操作で制約が自動生成される

構造を作ると、制約の下書き（draft）が自動で生える。

例：

scalar N
? <= N <= ?
array A
? <= A_i <= ?
文字列
|S| = ?
charset(S) = ?
辺リスト
1 <= u_i <= N
1 <= v_i <= N
graph property = ?

これは「補助機能」ではなく、基本設計である。

2. 状態管理の分離
2.1 AST（正本）

AST は唯一の意味的な真実。

含む：

Structure
Constraint（完成済みのみ）
Expression（完成済みのみ）

含まない：

選択状態
draft constraint
未完成式
UI状態
2.2 EditorState

UI用の一時状態。

含む：

選択中ノード
DraftConstraint
PendingExprAction
ポップアップ状態
エラー状態
2.3 未完成状態の扱い
種類	どこに置くか
Structure未完成	AST (Hole)
Expression未完成	EditorState
Constraint未完成	DraftConstraint
3. Structure ペイン設計
3.1 入力形式として表示する

木ではなく、行ベースの入力として見せる。

3.2 挿入ポイントは方向付き

例：

同じ行に追加
下に追加
ブロック内に追加
3.3 ノード追加はポップアップ

ユーザーが hotspot を押すと候補が出る。

最低限：

scalar
横配列
縦配列
tuple
repeat
3.4 UIテンプレートは重要

以下は最初から意識して入れる：

文字グリッド
数値グリッド
辺リスト
クエリ列
複数テストケース
セクション
4. Constraint ペイン設計
4.1 Draft と完成を分ける
完成 → AST
draft → UIのみ

draft は

点線
?
未完成表示

などで区別する。

4.2 構造的制約はショートカットで追加できる

最低限：

Distinct
Sorted
Permutation
Graph(Simple)
Tree
Connected
SumBound
5. Expression 編集
5.1 数式はクリック編集

例：

N → /2
A_i → 別変数
<= → >=
5.2 未完成式は AST に入れない

❌ N / ? を AST に入れる
✅ PendingAction で管理

6. 右ペイン仕様（重要）

右側には常に3つを表示する：

6.1 TeX制約
1 \le N \le 10^6
6.2 TeX入力形式
N
A_1 \cdots A_N
6.3 サンプルケース
5
1 3 2 5 4

これは必須。
「おまけ」ではない。

7. 具体的ユーザーフロー
7.1 基本配列
目標
N
A_1 ... A_N
手順
初期のクリックボックスを押す
scalar → number → N
draft: ? <= N <= ?
下のボックスを押す
横配列 → number → A
長さ → N
draft: ? <= A_i <= ?
制約を埋める
7.2 グリッド
目標
H W
S_1
...
S_H
手順
H, W を同一行に追加
下に「文字グリッド」テンプレート
行数 = H
長さ = W
draft:
|S_i| = ?
charset
制約を埋める
7.3 木
目標
N
u_1 v_1
...
u_{N-1} v_{N-1}
手順
N
下に「辺リスト」テンプレート
本数 = N - 1
draft:
範囲
u_i != v_i
graph property
Tree を選択
7.4 クエリ
目標
Q
1 x
2 i x
3 i
手順
Q
下に「クエリ列」
count = Q
variant を3つ追加
各構造を埋める
制約を追加
8. やってはいけないこと
未完成式をASTに入れる
draft constraint をASTに混ぜる
StructureをツリーUIにする
数式をただの文字列にする
グリッド・セクション・Choiceを無視する
9. 最終まとめ

このEditorは

ASTベース
入力形式ベースUI
direct manipulation
draft制約
サンプル生成

を組み合わせた

構造編集型の競プロ入力エディタ

として設計する。
