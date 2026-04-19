# Competitive Programming AST Editor
## 設計推進方針（E2E 先行）

---

## 0. この文書の目的

この文書は、Competitive Programming AST Editor の設計を **E2E テスト先行** で進めるための実務文書である。

このプロジェクトでは、抽象設計を先に肥大化させるのではなく、**実際の競技プログラミング問題を題材に、ユーザーの操作順序を言語化し、それを E2E テストとして固定すること** を最優先とする。

本方針の狙いは次の通り。

1. 実際の問題に対して使える編集体験を最初に固定する  
2. UI / AST / wasm 境界の設計を、実フローに引っ張られて自然に収束させる  
3. 「理論的には可能だが、UI として不自然」な設計を早期に排除する  
4. 基本ケースを壊さない回帰基盤を先に作る  

---

## 1. なぜ E2E 先行か

このエディタは、通常のフォームアプリではない。

- Structure ペインは競プロの入力形式を模した projection
- Constraint ペインは draft と completed constraint を持つ
- 数式はレンダリングされたものを直接触って編集する
- AST は正本だが、UI は direct manipulation を前提にする
- Rust/wasm と Preact の責務境界がある

このため、先に型設計や API 設計を詰めすぎると、**実際のユーザー操作とズレた設計ができやすい**。

一方、実問題に対して

- どこをクリックして
- 何がポップアップし
- 何が draft として生え
- 何が最終的に確定するか

を E2E テストとして先に固定すると、設計の曖昧さが減る。

したがって、**最初に完成させるべき成果物は設計書そのものではなく、ユーザー操作順序を表した E2E シナリオ群** である。

---

## 2. 設計推進の最上位原則

### 2.1 実問題から始める
`atcoder-coverage-extended.md` 相当のカバレッジ文書を見て、各問題を「どう編集するか」に落とすことを最初に行う。  
手元確認できた対応文書では、36問がカテゴリ別に整理されており、単一値・配列・グリッド・木・一般グラフ・クエリ列・複数テストケース・distinct/sorted・式付き境界などが含まれている。:contentReference[oaicite:2]{index=2}

### 2.2 1問ごとに「ユーザー操作順序」を言語化する
各問題について、最初の空状態から

- どの insertion hotspot を押すか
- 何を選ぶか
- どの draft constraint が出るか
- どの数式を直接クリックするか
- 最終的に右ペインに何が出るか

を文章で固定する。

### 2.3 その操作順序をそのまま E2E テストにする
文章化したフローは、そのまま Playwright 等の E2E テストケースに落とす。  
この段階では実装が未完成でもよい。  
まず「どうあるべきか」のテストを完成させる。

### 2.4 実装はテストを通すために後から追う
UI 実装、Projection API、Operation API、wasm export は、E2E テストが要求する契約を満たすように実装する。

---

## 3. 前提となる UI / 状態配置の契約

この文書で前提とする契約は以下。

### 3.1 AST
- AST は 1 種類
- Structure の未完成は AST の Hole
- Constraint は完成済みのみ AST に入る
- Expression は完成済みのみ AST に入る

### 3.2 EditorState
- Expression の未完成は PendingExprAction
- Constraint の未完成は DraftConstraint
- 選択状態、ポップアップ状態、モーダル状態もここに置く

### 3.3 Structure ペイン
- 競プロの入力形式を模した projection
- 挿入位置は方向付き insertion hotspot として表示
- popup / wizard でノード作成を始める

### 3.4 Constraint ペイン
- DraftConstraint と completed constraint を同時に表示してよい
- Structure 操作に応じて draft が自動生成されてよい

### 3.5 右ペイン
右ペインには常に次を出す。

1. TeX でレンダリングされた制約
2. TeX でレンダリングされた入力形式
3. 実際のサンプルケース

### 3.6 数式編集
- 表示済みの数式を直接クリックして編集する
- `N / ?` のような未完成式を AST に入れない
- 操作中の未完成は UI state にだけ存在する

---

## 4. 設計推進のフェーズ

---

## Phase 0. 題材問題の固定

### 目的
設計対象を抽象論ではなく、実際の問題群に固定する。

### 入力
- `atcoder-coverage-extended.md` 相当文書
- 手元では `phase2-real-problem-coverage.md` が代替参照先。36問が列挙されている。:contentReference[oaicite:3]{index=3}

### やること
1. 問題をカテゴリごとに整理する
2. MVP で必ず扱う問題群を決める
3. 中級ケースとして追う問題群を決める

### 最低限含めるカテゴリ
- 単一値
- 配列
- グリッド
- 木 / 辺リスト
- クエリ列
- 複数テストケース
- distinct / permutation / sorted
- 式付き境界
- section が必要な問題

### 期待成果物
- `coverage-scenario-index.md`
  - 問題名
  - カテゴリ
  - 優先度
  - MVP対象かどうか

---

## Phase 1. 問題ごとのユーザー操作順序を文章化する

### 目的
各問題に対して、空状態から完成までのユーザー操作フローを固定する。

### 1問ごとに書くべき内容
1. 目標となる入力形式
2. 目標となる制約
3. 初期状態
4. ユーザーのクリック順序
5. 各ステップで開くポップアップ
6. 自動で生える draft constraint
7. 右ペインの期待表示
8. 最終的な完成条件

### 書き方の例
```md
#### ABC350-C Sort

目標:
N
A_1 ... A_N

制約:
2 <= N <= 2*10^5
A は 1..N の順列

操作:
1. 初期 insertion hotspot をクリック
2. scalar → number → 名前 N
3. draft `? <= N <= ?` が生える
4. 右ペインに `N` と sample が表示される
5. N の下の hotspot をクリック
6. 横配列 → number → 名前 A → 長さ N
7. draft `? <= A_i <= ?` が生える
8. Property ショートカットから Permutation を追加
...
期待成果物
problem-user-flows.md
完了条件

対象問題すべてについて、空状態から完成までのフローが文章になっていること。

Phase 2. 文章化したフローを E2E テストに落とす
目的

設計を仕様ではなく、実行可能な期待値として固定する。

方針
文章フロー 1 本につき、最低 1 本の E2E シナリオを書く
必要なら 1 問を複数シナリオに分割してよい
最初は失敗していてよい
重要なのは「どう動くべきか」がテストになっていること
E2E テストで必ず見るべきもの
Structure 側
hotspot が見える
popup に正しい候補が出る
追加後に入力形式っぽい見た目になる
Constraint 側
適切な draft が自動で生える
draft をクリックして埋められる
completed constraint に昇格する
右ペイン
TeX の入力形式が更新される
TeX の制約が更新される
sample ケースが更新される
数式編集
表示された数式を直接クリックできる
操作途中に AST を壊さず編集できる
確定後に完成式として反映される
期待成果物
web/tests/e2e/*.spec.ts
web/tests/e2e/README.md
完了条件

MVP対象問題について、E2E テストファイルが揃っていること。

Phase 3. E2E テストを通すために UI / wasm / core を実装する
目的

既に固定されたユーザー体験を実装として満たす。

実装順序
testability のための selector / data-testid を整備
Structure ペインの hotspot / popup を実装
Constraint draft 自動生成を実装
右ペインの TeX / sample 表示を実装
direct math editing を実装
必要な wasm API を実装
テストを通す
原則
テストを変えて実装に合わせない
実装をテストの要求に合わせる
ただしテストの前提が誤っていれば、ユーザーフロー文書に戻って修正する
Phase 4. カバレッジ不足を実問題から再抽出する
目的

MVP 実装で漏れた設計要素を、実問題に基づいて追加する。

注目する不足

現時点で重要なのは以下。
実問題カバレッジ文書でも、グリッド、Choice、Property、Distinct、行内可変長、下三角構造などが明示的な論点になっている。

特に優先して見る:

グリッドテンプレート
セクション表現
query branch (Choice)
Distinct / Sorted / Property / SumBound
行内可変長
複数グリッド連結
期待成果物
coverage-gaps-after-e2e.md
5. E2E テストを最優先にする理由
5.1 操作自然性を先に固定できる

設計だけ先に詰めると、木構造として美しいが UI として不自然なものができやすい。

5.2 右ペインの役割が明確になる

右ペインに

TeX 制約
TeX 入力形式
sample
を出す契約は、E2E がないと形骸化しやすい。
5.3 direct manipulation の契約が曖昧にならない

「数式を直接触る」と言っても、テストがないと実際にはただのフォーム編集に戻りやすい。

5.4 wasm 境界の過不足が分かる

E2E を先に書くことで、

今どの projection が必要か
何を click 時に取ればよいか
何を右ペイン更新のトリガーにするか
が見える。
6. 具体的に最初に書くべき E2E セット
6.1 基本配列

対象:

N
A_1 ... A_N
1 <= N <= 10^6
1 <= A_i <= 10^9

見ること:

scalar 作成
横配列作成
長さに既存変数を選ぶ
range draft 自動生成
右ペイン更新
6.2 グリッド

対象:

H W
S_1 ... S_H

見ること:

同一行 tuple
grid template
|S_i| = W
charset draft
TeX 形式のグリッド表示
sample grid 生成
6.3 木入力

対象:

N
u_1 v_1 ... u_{N-1} v_{N-1}

見ること:

edge list template
N - 1 count expression
u_i != v_i
graph property = Tree
sample tree 生成
6.4 クエリ列

対象:

Q
1 x / 2 i x / 3 i

見ること:

query list template
variant 追加
tag ベースの branch
right pane の TeX 表示
6.5 複数テストケース

対象:

T
各ケースに N と A

見ること:

multiple testcase template
repeat over cases
sum bound draft 追加余地
7. Subagent の分担
7.1 Coverage Flow Agent

役割:

atcoder-coverage-extended.md 相当文書を読み
各問題をユーザーフローに変換する

成果物:

problem-user-flows.md
7.2 E2E Author Agent

役割:

ユーザーフローから Playwright テストを書く

成果物:

web/tests/e2e/*.spec.ts
7.3 UI Contract Agent

役割:

各フローから必要な UI 契約を抽出する
hotspot / popup / right-pane / direct-math-editing の仕様を固定する

成果物:

ui-contract-from-e2e.md
7.4 wasm Boundary Agent

役割:

E2E テストが要求する read/write API を洗う
projection / action / operation の不足を整理する

成果物:

wasm-api-needed-for-e2e.md
7.5 Critical Reviewer Agent

役割:

ユーザーフローが不自然な箇所
テストが brittle な箇所
実問題で頻出だがテストされていないケース
を指摘する

成果物:

e2e-review-notes.md
8. 実装より前に完成していなければならないもの

次が完成するまで、UI 実装を本格化させてはならない。

対象問題一覧
各問題のユーザー操作順序
E2E テストケース一覧
MVP E2E テスト群
右ペインの期待表示仕様

つまり、最初に完成させるのはテストとフロー文書 である。

9. 受け入れ条件

この設計推進方針が守られたと言えるのは、次を満たしたとき。

必須
実問題ベースのユーザーフロー文書がある
MVP 問題群の E2E テストが先に書かれている
右ペインに TeX 制約・TeX 入力形式・sample を出す契約が固定されている
Structure 操作 → draft constraint 自動生成の流れがテストされている
数式 direct manipulation がテストされている
望ましい
カバレッジ文書から自動的にテスト計画へ落ちる形になっている
中級ケース（グリッド / 木 / query / section）も最初からテスト対象に入っている
10. 最終結論

このプロジェクトでは、設計の最初の完成物は AST 定義でも UI 実装でもない。

最初に完成させるべきなのは、実問題を起点にしたユーザー操作順序の文書と、その操作順序を固定する E2E テスト群である。

この順序で進めることで、

UI が実問題に引っ張られて自然に決まる
AST / EditorState / wasm 境界の責務が現実的に定まる
後から設計がふくらんでも基本ケースを壊しにくい

したがって今後の優先順位は次で固定する。

atcoder-coverage-extended.md 相当文書を基に問題を洗う
各問題のユーザー操作順序を書く
E2E テストを先に完成させる
そのテストを通すように UI / wasm / core を実装する
