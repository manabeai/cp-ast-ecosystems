# 4モジュール設計の全体像

本ライブラリは、競プロ問題記述DSLを対象とする構造編集コアである。
中心となるのは次の4モジュール。

| モジュール | 役割 | 関心事 |
|-----------|------|-------|
| StructureAST | 形 | 問題仕様の構造的正本 |
| ConstraintAST | 許容条件 | その構造に課される許容条件の正本 |
| ProjectionAPI | 可視化層 | 外部に対して編集可能な像を返す導出層 |
| Operation | 更新層 | 編集要求を妥当性を保ちながら状態遷移に変換 |

それぞれを分離する理由は、構造・制約・見せ方・更新を混ぜると責務が崩れ、GUI依存や場当たり的実装に流れやすいからである。

---

## 1. StructureAST

### 役割

StructureAST は、問題仕様の構造そのものを表す正本である。
ここでは「何がどこにあるか」を表現する。

たとえば競プロDSLでは、次のようなものを保持する。

- スカラー変数
- 配列
- 行列
- テストケース構造
- 入力順序
- スロット付きノード
- hole の位置
- NodeId

### 責務

StructureAST の責務は次に限る。

- ノード種別の定義
- 親子関係の保持
- slot の意味的区別
- hole を含む未完成構造の保持
- ノード識別子の安定管理
- 部分木としての差し替え可能性の維持

### 持つべきもの

- NodeId
- NodeKind
- Slot
- Hole
- 構造木本体
- 変数参照の構造的リンク

### 持つべきでないもの

- UI表示情報
- 色やレイアウト
- 候補列挙ロジック
- 制約判定ロジック
- 編集可能かどうかの判断
- エラー文言生成の主処理

### 一言まとめ

**StructureAST は「今どんな形になっているか」だけを持つ層。**

---

## 2. ConstraintAST

### 役割

ConstraintAST は、StructureAST 上の各位置に対して
何が許されるかを表す。

StructureAST が「形」なら、ConstraintAST は「妥当性の境界」である。

### 責務

ConstraintAST の責務は次の通り。

- 型制約の表現
- 値域制約の表現
- 長さ制約の表現
- 要素制約の表現
- 依存制約の表現
- 文脈依存制約の表現
- hole や slot に対する期待条件の表現
- 将来の候補列挙や生成器の基盤提供

### 競プロDSLでの例

- `1 <= N <= 2e5`
- `len(A) = N`
- `0 <= A[i] <= 10^9`
- `T 個のテストケース`
- `graph is a tree`
- `all elements are distinct`

### 持つべきもの

- expected type
- allowed node kinds
- range constraint
- length relation
- element predicate
- contextual predicates
- capability constraints

### 持つべきでないもの

- AST本体の形
- UIイベント
- 実際の編集適用
- 表示用レイアウト
- 操作履歴

### 一言まとめ

**ConstraintAST は「その位置に何を置いてよいか」を表す層。**

---

## 3. ProjectionAPI

### 役割

ProjectionAPI は、StructureAST と ConstraintAST から
外部クライアントが使える編集可能な像を導出する。

ここでいう外部クライアントとは、

- WebUI
- CLI
- AI Agent
- テストコード
- 将来の別フロントエンド

を含む。

ProjectionAPI は正本ではない。
正本は常に StructureAST / ConstraintAST であり、Projection はそこから導出される。

### 責務

ProjectionAPI の責務は次の通り。

- 現在構造の外部向け表現を返す
- hole / slot の位置を明示する
- 各位置で可能な操作候補を返す
- 各 hole に入れられる候補カテゴリを返す
- 欠けている情報を返す
- 必要なら説明用メタデータを返す

### Projection が返すべき情報の例

- 表示対象ノード一覧
- 各ノードのラベル
- slot 名
- hole の期待型
- hole に対する候補種別
- 実行可能な操作一覧
- 編集不可理由

### 持つべきでないもの

- ASTの更新責務
- 不変条件そのものの保持
- 制約の正本
- UI固有の描画詳細
- DOM依存情報

### 一言まとめ

**ProjectionAPI は「今この状態を外部からどう編集できるか」を見せる窓口。**

---

## 4. Operation

### 役割

Operation は、編集操作を受け取り、
StructureAST / ConstraintAST に対して安全に状態遷移を適用する層である。

この層があることで、

- GUIは判断しない
- Projectionは説明するだけ
- 実際の更新責任は Operation が持つ

という分担が成立する。

### 責務

Operation の責務は次の通り。

- 編集操作の型定義
- 操作適用
- 妥当性検査
- 制約違反検出
- 新状態の生成
- 失敗理由の返却
- 部分更新の適用
- 必要なら履歴や差分生成の基盤提供

### 操作の例

- hole を埋める
- ノードを差し替える
- slot に要素を追加する
- 配列定義を導入する
- 制約を追加する
- 制約を削除する
- テストケース構造を追加する

### Operation が依存するもの

- StructureAST
- ConstraintAST
- 必要に応じて Projection の入力仕様

### 持つべきでないもの

- UI描画
- 表示専用メタデータ
- 文字列パース
- 表示フォーマット整形

### 一言まとめ

**Operation は「何が起きるか」ではなく「編集を本当に反映する」実行層。**

---

## 4モジュールの関係

### 全体の流れ

```
StructureAST   ConstraintAST
      \            /
       \          /
        \        /
       ProjectionAPI
             |
        外部クライアント
             |
         Action要求
             |
          Operation
             |
   新しい StructureAST / ConstraintAST
```

もう少し正確にいうと、

1. StructureAST が現在の形を持つ
2. ConstraintAST が許容範囲を持つ
3. ProjectionAPI がその状態を編集可能な像として外部に見せる
4. 外部は Action を選ぶ
5. Operation がそれを適用し、新しい状態を返す

### 依存方向

依存方向はできるだけ次のように保つ。

```
StructureAST   ConstraintAST
        ↑          ↑
        └── Operation ──┐
                         │
                  ProjectionAPI
```

実装上の感覚としては、

- ProjectionAPI は StructureAST と ConstraintAST を読む
- Operation は StructureAST と ConstraintAST を更新する
- StructureAST と ConstraintAST はできるだけ相互依存を薄くする

のがよい。

---

## 各モジュールの責務を一行で

| モジュール | 責務 |
|-----------|------|
| **StructureAST** | 構造を保持する。 |
| **ConstraintAST** | 構造に対する許容条件を保持する。 |
| **ProjectionAPI** | 構造と制約から編集可能な外部向け像を導出する。 |
| **Operation** | 編集要求を受けて安全に新しい状態へ遷移させる。 |

---

## この4分割の利点

### 1. GUI依存にならない

GUIは Projection と Operation を使うだけになる。

### 2. AI Agent からも同じコアを使える

AI は Projection で状況を知り、Operation で変更する。

### 3. 研究として主張が整理しやすい

```
StructureAST:    構造
ConstraintAST:   制約
Projection:      編集可能性の導出
Operation:       妥当性を保つ更新
```

という綺麗な分解になる。

### 4. Parser を後回しにできる

Parser は後で「文字列から StructureAST / ConstraintAST を作る入口」として追加すればよい。

---

## 実装推奨優先順位

実装は次の順で進めるのがよい。

1. **StructureAST**
2. **ConstraintAST**
3. **Operation**
4. **ProjectionAPI**

理由は、Projection は Operation や制約導出の形が少し見えてからの方が安定するから。
ただし UI を早く試したいなら、最小 Projection を先に作ってもよい。

---

## まとめ

このライブラリは、次の4モジュールで構成する。

- **StructureAST**: 問題仕様の構造的正本
- **ConstraintAST**: その構造に課される許容条件の正本
- **ProjectionAPI**: 外部に対して編集可能な像を返す導出層
- **Operation**: 編集要求を妥当性を保ちながら状態遷移に変換する実行層

この分割により、**構造・制約・可視化・更新が分離され、GUI・AI・テスト・将来の parser をすべて同じコアの上に載せられる。**
