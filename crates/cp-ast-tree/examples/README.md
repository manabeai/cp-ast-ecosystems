# cp-ast-tree Examples

このディレクトリには、様々な AST パターンを可視化するサンプルプログラムが入っています。

## 実行方法

```bash
cargo run -p cp-ast-tree --example <name>
```

## 例一覧

| ファイル | 問題パターン | 使用 NodeKind |
|---|---|---|
| `graph` | グラフ問題 (N頂点M辺) | Scalar, Tuple, Repeat, Sequence |
| `array_sum` | 配列問題 (N個の整数) | Scalar, Array, Sequence |
| `matrix` | 行列問題 (H×W グリッド) | Scalar, Matrix, Tuple, Sequence |
| `query` | クエリ型問題 (3種のクエリ) | Scalar, Tuple, Choice, Repeat, Sequence |
| `multi_testcase` | 複数テストケース | Scalar, Array, Section, Repeat, Sequence |
| `strings` | 文字列問題 (N行の文字列) | Scalar, Repeat, Sequence |
| `holes` | 未完成 AST (Hole あり) | Scalar, Hole, Sequence |

## 実行例

```
$ cargo run -p cp-ast-tree --example graph

=== Structure ===
Sequence
├── Tuple
│   ├── Scalar(N)
│   └── Scalar(M)
└── Repeat(count=M, i)
    └── Tuple
        ├── Scalar(u)
        └── Scalar(v)

=== Combined ===
Sequence
├── Tuple
│   ├── Scalar(N)  [1 ≤ N ≤ 10^5, N is integer]
│   └── Scalar(M)  [1 ≤ M ≤ N]
└── Repeat(count=M, i)
    └── Tuple
        ├── Scalar(u)  [1 ≤ u ≤ N]
        └── Scalar(v)  [1 ≤ v ≤ N]
(global) The graph is simple and connected
```

## TreeOptions

各例は `TreeOptions::default()` を使っています。追加オプション:

| フィールド | 説明 |
|---|---|
| `show_node_ids` | `#N` 形式で NodeId を表示 |
| `show_constraint_ids` | `C0:` 形式で ConstraintId を表示 |

```rust
let opts = TreeOptions { show_node_ids: true, ..TreeOptions::default() };
```
