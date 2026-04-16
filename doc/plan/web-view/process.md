# Competitive Programming AST Editor 設計推進文書

## 0. この文書の目的

この文書は、Competitive Programming AST Editor の設計を **AI Agent / Subagent 駆動で前進させるための進め方** を定義するものである。

この文書の役割は次の3つである。

1. **何を誰が検討するかを固定すること**
2. **各 Subagent の成果物をどう統合するかを定めること**
3. **設計が抽象論で終わらず、実装可能な粒度まで落ちるようにすること**

全体設計の根幹は `main.md` にある。
UI 設計思想および AST / EditorState / Projection / Operation の契約は `editor-ui-design-philosophy` を参照すること。

本書は、それらを前提とした **設計推進プロトコル** である。

---

## 1. 今回の設計対象

今回の対象は、以下である。

* Preact ベースの Editor UI
* Rust 製 AST core を wasm にビルドして接続するための境界設計
* GUI で構築した AST から sample を出力するまでの接続
* 基本ケースを確実に扱い、中程度に複雑な競プロ入力まで高い割合でカバーする操作体系

今回のゴールは **フル実装** ではない。
ただし、成果物は「そのまま実装タスクに分解できる」必要がある。

---

## 2. この設計推進で守るべき大原則

### 2.1 AST を正本とする

設計中も、最終的な正本が AST であるという前提を崩さない。

* UI は AST の投影
* 操作は Operation API を通じて AST を更新
* sample generation は AST から導く

---

### 2.2 未完成状態の配置契約を守る

このプロジェクトでは未完成状態を次のように分配する。

* **Structure の未完成** → AST の Hole
* **Expression の未完成** → EditorState / PendingExprAction
* **Constraint の未完成** → EditorState / DraftConstraint

Subagent はこの前提を破る設計を提案してよいが、その場合は必ず

* なぜ現行契約では不十分か
* 代替案の利点
* 代替案の欠点

を明示すること。

---

### 2.3 基本ケースを優先する

* 単一整数
* 複数整数
* 配列
* 長さ依存配列
* グリッド
* 辺リスト
* 複数ケース
* クエリ列

これらは迷わず構築できなければならない。

複雑なケースへの一般化は重要だが、基本ケースの操作体験を犠牲にしてはならない。

---

### 2.4 競プロ問題ベースで評価する

理論上きれいでも、実問題で詰む設計は採用しない。

したがって設計検討は必ず

* 実際の競プロ問題
* 想定ユーザー操作列
* 必要 UI / API / AST 要素

に落として検証する。

---

## 3. Subagent 駆動で進める理由

この設計は次の理由で難しい。

* AST / UI / wasm / sample generation が密接に絡む
* 一見自然な UI が AST 契約を壊しやすい
* 抽象設計だけでは実問題カバレッジが読めない
* 実装容易性と一般性のトレードオフが激しい

そのため、単一 Agent が一気にまとめるより、

* 領域ごとの専門性
* 実問題ベースの批判
* 統合時の相互牽制

を前提にした Subagent-driven 設計を採る。

---

## 4. Subagent 構成

最低限、以下の 6 Subagent を立てる。

---

### 4.1 Domain Model Agent

#### 役割

ドメインモデルと意味論境界を定める。

#### 主担当

* AST
* Structure / Constraint / Expression の責務分離
* Hole の意味
* EditorState との境界
* Projection / Operation API の最小骨格
* stable id / source ref の必要性

#### 必須成果物

* 型設計案
* AST 不変条件一覧
* EditorState に逃がすもの一覧
* Projection / Operation の責務表
* wasm に公開するべきコア API の素案

#### 禁止

* UI 遷移だけで契約を上書きすること
* 使い勝手だけを理由に AST を汚すこと

---

### 4.2 GUI Interaction Agent

#### 役割

ユーザー操作順序から UI を逆算する。

#### 主担当

* 主要画面構成
* コンポーネント責務
* クリック遷移
* hole / slot / candidate 表示
* pending action の UI 体験
* constraint 作成フロー

#### 必須成果物

* 主要画面案
* 主要操作フロー
* 1操作ごとの UI state 変化
* Preact コンポーネント境界案
* どの情報が render 時に必要か / click 時に遅延取得でよいかの整理

#### 禁止

* AST 側で持つべき意味論を UI state に押し込むこと

---

### 4.3 Real Problem Coverage Agent  **最重要**

#### 役割

実際の競プロ問題を使って設計の穴を暴く。

#### 主担当

* 競プロ問題 30 問程度の収集
* 問題ごとの入力構造の分類
* 想定ユーザー操作列の作成
* 現設計でのカバー可否判定
* 足りない API / UI / AST 要素の抽出

#### 最低限含めるべき問題カテゴリ

* 単一値
* 複数値
* 配列
* 二次元配列 / グリッド
* 木
* 一般グラフ
* 辺リスト
* クエリ列
* 複数テストケース
* 総和制約
* distinct / sorted
* `N/2`, `2N`, `max(1, N-1)` を含む境界
* choice / variant を含む入力
* section / repeat が必要な入力

#### 必須成果物

* 問題一覧
* 入力特徴
* 想定操作列
* その操作に必要な UI / API / AST 要素
* カバー可能 / 部分可能 / 不可 の判定
* 頻度感つき不足一覧

#### 禁止

* 理論設計を鵜呑みにして通すこと
* 「頑張ればできる」で曖昧に済ませること

---

### 4.4 wasm / Frontend Boundary Agent

#### 役割

Rust/wasm と Preact の境界を実装可能な形で定義する。

#### 主担当

* wasm export surface
* Preact が持つ state と wasm が持つ state の分離
* serialization / identity / stable id
* Projection / Operation 呼び出し方式
* 差分更新や再描画効率の懸念

#### 必須成果物

* wasm exported API 案
* Preact 側責務一覧
* 更新単位の提案
* パフォーマンス上のリスク一覧

#### 禁止

* 便利だからという理由で Preact に意味論を再実装させること

---

### 4.5 Sample Generation Agent

#### 役割

Editor 設計が sample generation に耐えるかを確認する。

#### 主担当

* sample generation に必要な最小情報
* AST から generator に渡すべき内容
* editor 上で補うべきメタ情報の有無
* generator の保証レベルと UI の期待値の接続

#### 必須成果物

* generator に必要な最小情報一覧
* editor で不足しうる情報一覧
* sample 出力ボタンまでの接続フロー
* 将来的な retry / repair / constructive 戦略への見通し

#### 禁止

* sample generation のためだけに UI 契約を壊すこと

---

### 4.6 Critical Reviewer Agent

#### 役割

他 Agent の案を批判的に壊しにいく。

#### 主担当

* 過剰設計の指摘
* 一般性の嘘の指摘
* 実問題での破綻点の指摘
* UI / AST / wasm 境界の矛盾検出
* 後回しにすべき論点の切り分け

#### 必須成果物

* リスク一覧
* 過剰設計 / 過少設計の指摘
* MVP で切るべきもの
* 代替案

#### 禁止

* ふんわりした感想で終わること
* 他 Agent の言い換えに留まること

---

## 5. Subagent の入出力契約

各 Subagent は必ず次の形式で出力する。

### 5.1 必須セクション

1. **前提**
2. **自分の担当範囲における主要判断**
3. **具体例**
4. **現行方針に対する支持 / 反対 / 留保**
5. **不足点**
6. **他 Agent に渡すべき論点**

---

### 5.2 具体例の必須条件

必ず最低 3 個以上の具体例を含めること。

例:

* `N: int`
* `A: int[N]`
* `A: int[N/2]`
* `1 <= A[i] <= N`
* `u_i != v_i`
* Tree 入力
* Query type に応じた Choice

---

### 5.3 曖昧語禁止

以下のような曖昧な書き方は禁止する。

* だいたい対応できる
* 工夫すればいける
* ある程度可能
* 多分十分

代わりに、

* 完全対応
* 部分対応
* 非対応
* MVP では後回し

のように明示すること。

---

## 6. 設計推進フェーズ

設計は次の順で進める。

---

### Phase 1. 前提固定

#### 目的

根幹思想を共有し、議論の土台を固定する。

#### 使う文書

* `main.md`
* `editor-ui-design-philosophy`

#### 出力

* 今回の設計で固定する前提一覧
* 未確定事項一覧
* 論点一覧

#### 完了条件

各 Subagent が同じ前提を見ている状態になること。

---

### Phase 2. 各 Subagent の独立検討

#### 目的

役割ごとに論点を掘る。

#### 出力

Subagent ごとのレポート。

#### 完了条件

各 Agent が以下を明示していること。

* 自分の結論
* 現行契約との整合性
* 不足点
* 他 Agent に投げるべき問い

---

### Phase 3. 実問題ベース検証

#### 目的

理論設計を実問題で殴る。

#### 主担当

Real Problem Coverage Agent

#### 出力

* 問題一覧
* 想定操作列
* 必要 API / UI / AST 要素
* 破綻箇所一覧

#### 完了条件

最低 30 問前後で検証し、頻出ケースに対する対応状況が分かること。

---

### Phase 4. 相互批判

#### 目的

各 Subagent の盲点を他 Agent が指摘する。

#### 進め方

* GUI Interaction Agent は Domain Model Agent の案の使いにくさを指摘する
* Domain Model Agent は GUI Agent の案の意味論的危険を指摘する
* Coverage Agent は両者の理想論を問題ベースで壊す
* wasm Agent は責務漏れを指摘する
* Sample Generation Agent は生成器接続の不足を指摘する
* Critical Reviewer Agent は全体を壊す

#### 完了条件

各主要設計判断について、最低 1 回は批判的検証を受けていること。

---

### Phase 5. 統合設計

#### 目的

最終アーキテクチャを収束させる。

#### 出力

* AST 契約
* EditorState 契約
* Projection 契約
* Operation 契約
* Preact / wasm 境界契約
* 主要操作フロー
* MVP スコープ

#### 判断基準

多数決ではなく、次の優先順位で判断する。

1. 基本ケースの操作自然性
2. 実問題カバレッジ
3. 実装容易性
4. 意味論の明快さ
5. 将来拡張性

---

### Phase 6. 実装計画化

#### 目的

設計をそのまま実装タスクに分解できる形にする。

#### 出力

* wasm core 実装タスク
* Preact UI 実装タスク
* Projection / Operation 実装タスク
* sample generation 接続タスク
* 実問題テストタスク

#### 完了条件

各タスクに担当・依存関係・完了条件が付いていること。

---

## 7. Subagent 連携ルール

### 7.1 独立検討 → 相互参照

各 Agent は最初に独立に考えてよい。
しかし最終レポートでは必ず他 Agent の案を参照し、整合性・齟齬・不足を明記すること。

---

### 7.2 理論設計だけで通さない

特に次の Agent は理論案を現実で壊す役目を持つ。

* Real Problem Coverage Agent
* Critical Reviewer Agent

この 2 Agent の指摘は強く扱うこと。

---

### 7.3 設計判断ごとに採用理由を言語化する

「なんとなく良さそう」で採用しない。

各主要判断について、

* 採用案
* 棄却案
* 採用理由
* 棄却理由

を残すこと。

例:

* Expression partial AST を採用しない理由
* ConstraintHole を採用しない理由
* render/action/global projection 分離を採用する理由

---

## 8. 主要論点と担当割り当て

以下の論点を最低限扱う。

| 論点                                      | 主担当               | 必ず確認する Agent      |
| --------------------------------------- | ----------------- | ----------------- |
| AST は 1 種類でよいか                          | Domain Model      | Critical Reviewer |
| Structure Hole の役割                      | Domain Model      | GUI Interaction   |
| Expr partial を持たない設計で十分か                | GUI Interaction   | Critical Reviewer |
| Constraint draft を EditorState に逃がしてよいか | GUI Interaction   | Domain Model      |
| 式スロット設計                                 | GUI Interaction   | Domain Model      |
| projection 分離方針                         | wasm Boundary     | GUI Interaction   |
| click 時候補列挙で十分か                         | GUI Interaction   | wasm Boundary     |
| 実問題30件でどこまでカバーできるか                      | Coverage          | 全員                |
| sample generation につながるか                | Sample Generation | Domain Model      |
| MVP で何を切るか                              | Critical Reviewer | 全員                |

---

## 9. 主要成果物

最終的に必要な成果物は以下。

### 9.1 設計成果物

* 統合設計書
* UI 詳細設計書
* wasm 境界仕様
* sample generation 接続仕様
* 実問題カバレッジ報告

### 9.2 実装準備成果物

* タスク分解表
* API 一覧
* コンポーネント一覧
* 画面ごとの責務一覧
* テスト観点一覧

---

## 10. 実問題カバレッジ検証プロトコル

### 10.1 問題収集方針

問題は 30 問程度を目安に集める。

偏りを避け、以下のカテゴリを満たすこと。

* 基本入力
* 配列
* グリッド
* グラフ
* 木
* 複数ケース
* クエリ列
* 総和制約
* sorted / distinct
* 式付き境界
* choice / section / repeat 必須ケース

---

### 10.2 各問題で必ずやること

1. 入力特徴を要約
2. 想定ユーザー操作列を書く
3. 各操作で必要な UI 要素を書く
4. 各操作で必要な API を書く
5. 現行設計で対応可能か判定
6. 不足があれば頻度感つきで報告

---

### 10.3 判定基準

* **完全対応**: 迷いなく自然に構築可能
* **部分対応**: workaround ありで構築可能
* **非対応**: 現設計では表現不能または UI が破綻

---

## 11. 統合時の意思決定ルール

### 11.1 優先順位

意見が割れたら次の順で判断する。

1. 基本ケースが速く正確に作れるか
2. 実問題で頻出か
3. 意味論を壊さないか
4. 実装コストが現実的か
5. 将来拡張可能か

---

### 11.2 追加機能の採用条件

新しい UI / AST / API を追加してよいのは次のいずれかを満たす場合だけ。

* 頻出ケースの操作を大きく改善する
* 実問題検証で高頻度の不足として現れた
* sample generation 接続に必須
* 既存契約では明確に破綻する

これを満たさないものは Phase 2 以降へ送る。

---

## 12. MVP の定義

MVP では以下を目指す。

### 必須

* Structure 編集

  * Scalar
  * Array
  * Tuple
  * Sequence
  * Repeat
  * Section
  * Choice
  * Hole 表示と fill
* Constraint 編集

  * TypeDecl
  * Range
  * LengthRelation
  * Relation
* 式スロット編集

  * 参照
  * 定数
  * `+ - * /`
  * `min / max`
* diagnostics
* AST から sample 出力

### 後回し候補

* 高度な自由入力式エディタ
* 大規模 drag & drop
* collaborative editing
* 高度な undo/redo
* interactive 問題サポート

---

## 13. AI Agent が最終的に出すべき統合文書の形

最終的な統合成果物は、以下の順でまとめる。

1. Executive Summary
2. 前提
3. 各 Subagent の結論
4. 統合アーキテクチャ
5. UI / Interaction 設計
6. 実問題カバレッジ評価
7. リスクと代替案
8. MVP と後回し項目
9. 実装計画
10. 次に着手すべきタスク

---

## 14. AI Agent への明示的指示

* 抽象論だけで終わらせないこと
* 必ず具体例を入れること
* 実問題で壊すことを恐れないこと
* AST / UI / wasm / sample generation の接続を曖昧にしないこと
* きれいな設計より、使える設計を優先すること
* ただし使いやすさを理由に意味論を壊さないこと

この文書は、設計そのものではなく、**設計を成功させるための手順と責任分担を固定するための文書**である。
