# cp-ast-ecosystems AST境界層設計書

## 1. 目的

本設計書は、`cp-ast-core` の AST を **Rust -> JS -> Rust** のプロセスを経ても、意味・構造・ID・順序を削ぎ落とさずに受け渡しするための境界層を設計するものである。

今回のスコープは **AST 自体の受け渡し** に限定する。以下は意図的に対象外とする。

- projection
- action
- tex render
- sample generation
- フロントエンドでの編集体験
- 差分同期
- パフォーマンス最適化

この段階では、まず **完全性と損失のなさ** を最優先にする。

---

## 2. 現在の前提

- `cp-ast-core` には `constraint`, `operation`, `projection`, `render`, `render_tex`, `sample`, `structure` が存在している。
- AST の中核は `AstEngine` であり、これは `StructureAst` と `ConstraintSet` を所有する。
- `StructureAst` は arena-based な構造木であり、`NodeId` により参照され、挿入順序が決定的 canonical rendering のために保持される。
- `ConstraintSet` も arena-based であり、`ConstraintId` による参照、per-node 制約と global 制約、`next_id` を持つ。

つまり現在の内部表現は、単なる木のネストではなく、以下を持つ。

- arena
- stable id
- tombstone を含む sparse な状態
- 次に払い出す ID
- per-node / global constraint index

この性質を落とすと、Rust -> JS -> Rust の往復で元の状態に戻らなくなる。

---

## 3. 解くべき問題

フロントエンドとバックエンドのあいだで AST をやり取りするとき、単に「見た目が似ている JSON」にするだけでは不十分である。

特に失ってはいけないものは次である。

### 3.1 ID の安定性

- `NodeId`
- `ConstraintId`
- `next_id`

### 3.2 arena 構造

- live node だけでなく、削除済みスロットの存在
- index と id の対応

### 3.3 順序

- 挿入順序
- child 配列順序
- constraint の格納順

### 3.4 参照

- `Reference::VariableRef`
- `Reference::IndexedRef`
- expression 内の参照
- structure / constraint 間の参照関係

### 3.5 enum の意味

- `NodeKind`
- `Constraint`
- `Expression`
- `Reference`
- `Literal`

これらのどれかが欠けると、JS を経由しただけで AST が意味的に別物になる。

---

## 4. 設計方針

### 4.1 full snapshot を採用する

AST の受け渡しは **差分** ではなく、まずは **full snapshot** とする。

理由:

- 実装が単純
- デバッグしやすい
- 保存形式と統一しやすい
- 将来の undo/redo と相性がよい
- AST のドメインサイズ上、現時点では十分現実的

この時点で patch / delta protocol は採用しない。

### 4.2 内部型をそのまま外に晒さない

境界では Rust の内部型をそのまま serde で外に出すのではなく、**外部公開用 DTO** を定義する。

つまり、以下の往復を正規ルートとする。

```text
internal AstEngine
  <-> AstDocumentDto
  <-> JSON
  <-> JS object
  <-> JSON
  <-> AstDocumentDto
  <-> internal AstEngine
```

理由:

- 内部 refactor の自由度を守る
- JSON 契約を安定させる
- Rust / JS / 保存形式の境界を共通化できる

### 4.3 lossless roundtrip を最優先する

境界 DTO は「UI が使いやすい形」ではなく、まず **完全復元可能な形** を優先する。

この層の責務は prettify ではない。責務は **保存と復元** である。

### 4.4 JSON を第一の交換形式とする

交換形式はまず JSON とする。

理由:

- JS と自然に接続できる
- wasm / http / local storage / file 保存を共通化できる
- デバッグしやすい
- AI への入出力とも相性がよい

## 5. 境界で渡す単位

境界で渡す単位は `AstEngine` 相当全体とする。つまり構造だけではなく、制約も含む。

```text
AstDocument
  = StructureAst
  + ConstraintSet
```

これにより「構造だけ渡って制約が落ちる」という事故を防ぐ。

## 6. DTO 設計の基本原則

### 6.1 すべて versioned envelope に包む

```json
{
  "schema_version": 1,
  "document": { "...": "..." }
}
```

`schema_version` は必須にする。

### 6.2 ID は decimal string で表す

内部では `NodeId` / `ConstraintId` / `next_id` は `u64` ベースである。JSON を経由して JS に渡す場合、`number` は 53bit 制限があるため、安全のため ID と counter は decimal string として表現する。

例:

```json
{
  "id": "12",
  "next_id": "57"
}
```

これにより JS を経由しても桁落ちしない。

### 6.3 enum は discriminated union にする

すべての enum は文字列タグ付きの object にする。

悪い例:

```json
["Scalar", "N"]
```

良い例:

```json
{
  "kind": "Scalar",
  "name": "N"
}
```

この方針を以下すべてに適用する。

- `NodeKind`
- `Constraint`
- `Expression`
- `Reference`
- `Literal`

### 6.4 arena をそのまま表現する

`StructureAst` は `Vec<Option<StructureNode>>` と `next_id` を持つ arena である。完全復元のため、DTO でも arena と tombstone を保持する。

`ConstraintSet` も同様に arena と `next_id` を持つため、これも tombstone を含めて表現する。

## 7. 推奨 DTO 形

### 7.1 top-level

```json
{
  "schema_version": 1,
  "document": {
    "structure": { "...": "..." },
    "constraints": { "...": "..." }
  }
}
```

### 7.2 StructureAst DTO

```json
{
  "root": "0",
  "next_id": "12",
  "arena": [
    { "id": "0", "kind": { "kind": "Sequence", "children": ["1", "2"] } },
    { "id": "1", "kind": { "kind": "Scalar", "name": "N" } },
    null,
    { "id": "3", "kind": { "kind": "Array", "name": "A", "length": { "...": "..." } } }
  ]
}
```

原則:

- arena index と id は一致すること
- `null` は tombstone を意味する
- `next_id` は必ず持つ
- child / reference はすべて `NodeId` string を使う

### 7.3 ConstraintSet DTO

```json
{
  "next_id": "8",
  "arena": [
    { "id": "0", "constraint": { "...": "..." } },
    null,
    { "id": "2", "constraint": { "...": "..." } }
  ],
  "by_node": [
    { "node_id": "1", "constraints": ["0", "2"] }
  ],
  "global": ["5"]
}
```

原則:

- constraint arena も tombstone を保持する
- `by_node` の index は object で保持する
- global 制約は別途保持する
- `ConstraintId` も decimal string

### 7.4 StructureNode DTO

内部 `StructureNode` の fully lossless な復元に必要なフィールドをすべて持たせる。

少なくとも:

```json
{
  "id": "3",
  "kind": { "...": "..." }
}
```

もし今後 metadata が増えるなら、ここに追加する。

### 7.5 NodeKind DTO

例:

```json
{ "kind": "Scalar", "name": "N" }
{ "kind": "Array", "name": "A", "length": { "...": "..." } }
{ "kind": "Tuple", "elements": ["1", "2"] }
{ "kind": "Sequence", "children": ["3", "4"] }
{ "kind": "Repeat", "count": { "...": "..." }, "index_var": "i", "body": ["6"] }
{ "kind": "Choice", "tag": { "...": "..." }, "variants": ["..."] }
{ "kind": "Hole", "expected_kind": "AnyArray" }
```

### 7.6 Expression DTO

例:

```json
{ "kind": "Lit", "value": 10 }
{ "kind": "Var", "reference": { "...": "..." } }
{ "kind": "BinOp", "op": "Mul", "lhs": { "...": "..." }, "rhs": { "...": "..." } }
{ "kind": "Pow", "base": { "...": "..." }, "exp": { "...": "..." } }
{ "kind": "FnCall", "name": "min", "args": ["..."] }
```

### 7.7 Reference DTO

例:

```json
{ "kind": "VariableRef", "node_id": "1" }
{ "kind": "IndexedRef", "target": "2", "indices": ["i"] }
{ "kind": "Unresolved", "name": "i" }
```

### 7.8 Constraint DTO

constraint も同様に discriminated union にする。

```json
{
  "kind": "Range",
  "target": { "...": "..." },
  "lower": { "...": "..." },
  "upper": { "...": "..." }
}
```

`TypeDecl`, `Distinct`, `Sorted`, `CharSet`, `StringLength`, `Relation`, `SumBound`, `Guarantee`, `RenderHint` などすべて同様に表現する。

## 8. strict roundtrip invariant

この境界層では、次の不変条件を最重要とする。

### invariant 1

`Rust internal -> DTO -> JSON -> JS parse -> JSON stringify -> DTO -> Rust internal` を行っても、内部 AST は意味的に同一であること。

### invariant 2

ID が変わらないこと。

### invariant 3

arena の tombstone が消えないこと。

### invariant 4

`next_id` が変わらないこと。

### invariant 5

child 順序、constraint 順序が変わらないこと。

### invariant 6

未解決参照や hole も失われないこと。

## 9. 復元ポリシー

復元時は、DTO を読み込んで新規に builder 的に組み立てるのではなく、arena をそのまま復元する。

つまり、以下の方針を取る。

- `next_id` をそのまま入れる
- arena の `null` を tombstone として保持する
- `root` をそのまま設定する
- `by_node` / `global` をそのまま復元する

理由:

builder 的再構築は「同じ意味」には戻せても、「同じ内部状態」には戻らない恐れがあるため。

今回の要件は意味保持だけでなく **Rust -> JS -> Rust で削ぎ落とされないこと** なので、内部状態もできるだけ保持する。

## 10. フロントエンドの責務

この設計でフロントエンドが持つのは `AstDocumentDto` である。

ただしフロントエンドは、以下をしてよい。

- AST を state として保持する
- JSON として保存する
- バックエンドにそのまま返す

一方で、以下をしてはいけない。

- 内部整合性を勝手に修復する
- tombstone を勝手に詰める
- `next_id` を再採番する
- enum を簡略化して保存する

フロントは transport / state holder として AST を扱う。この層では AST の意味論を再実装しない。

## 11. 推奨 crate / module

この境界層は `cp-ast-json` のような専用層に分離するのが望ましい。

責務:

- DTO 定義
- internal <-> DTO 変換
- JSON schema version 管理
- roundtrip テスト

ここに UI 専用型や projection を混ぜない。

## 12. テスト戦略

### 12.1 roundtrip test

- `internal -> dto -> json -> dto -> internal`
- 元と復元後が一致するか

### 12.2 tombstone preservation test

- node / constraint を remove 済みの arena を roundtrip しても `null` が失われないか

### 12.3 id preservation test

- `NodeId`, `ConstraintId`, `next_id` が変わらないか

### 12.4 ordering test

- child 順序
- variant 順序
- constraint 順序

が変わらないか

### 12.5 unresolved / hole preservation test

- `Reference::Unresolved`
- `NodeKind::Hole`

がそのまま戻るか

### 12.6 JS bridge test

1. Rust で JSON を作る
2. JS で parse / stringify する
3. Rust で再読込する
4. 完全一致するか確認する

## 13. 非目標

今回は以下をしない。

- projection をこの DTO に混ぜる
- action をこの DTO に混ぜる
- 画面描画向けに prettify する
- 差分同期を導入する
- patch protocol を導入する
- partial update protocol を導入する
- パフォーマンス最適化のために内部 AST を JS で直接 mutate する

## 14. 最終判断

この段階では、AST 境界層として最も重要なのは **完全 roundtrip 性** である。

したがって次を採用する。

- full snapshot
- JSON ベース
- versioned envelope
- decimal string による ID 表現
- discriminated union 形式
- arena / tombstone / `next_id` の保持
- internal 型と外部 DTO の分離

この方針により、`cp-ast-core` の AST は Rust -> JS -> Rust の境界を跨いでも意味・構造・ID・順序を失わずに保持できる。

## 15. AI への依頼事項

以下の観点で詳細設計を進めること。

- 現在の `StructureAst` と `ConstraintSet` を完全復元できる DTO を定義すること
- Rust internal -> DTO -> Rust internal の lossless roundtrip を保証すること
- JS の 53bit 制限を回避する ID 表現を採用すること
- arena / tombstone / `next_id` を落とさないこと
- projection / action / UI 都合を混ぜないこと
- 将来の schema version migration の余地を残すこと
- golden test と roundtrip test の両方を設計すること
- `ast-core` は wasm として動かすこと
