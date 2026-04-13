# cp-ast-ecosystems: 実装進捗と残タスク

## 実装済み

### Phase 1: AST Core（完了）

`crates/cp-ast-core/` に以下のモジュールが実装済み。全 178 テスト通過。

| モジュール | 内容 | 状態 |
|-----------|------|------|
| `structure/` | Arena ベースの StructureAst、NodeId、9 種の NodeKind、Reference、Ident | ✅ 完了 |
| `constraint/` | ConstraintSet、12 種の Constraint、Expression（算術式）、ExpectedType | ✅ 完了 |
| `operation/` | AstEngine、FillHole、ReplaceNode、Add/RemoveConstraint、Add/RemoveSlotElement、IntroduceMultiTestCase、Preview | ✅ 完了 |
| `projection/` | ProjectionAPI trait、ノード列挙・Hole 検出・操作候補 | ✅ 完了 |
| `render/` | 入力形式テキスト、制約テキストのプレーンテキスト生成 | ✅ 完了 |
| `sample/` | 依存グラフ解析、制約ベースランダム生成、テキスト出力 | ✅ 完了 |

### TeX Renderer（完了）

| タスク | 内容 | コミット |
|-------|------|---------|
| T-01 | モジュール骨格（mod.rs, 型定義, スタブ） | `f166f03` |
| T-02 | TeX ヘルパー（expression_to_tex, reference_to_tex, ident_to_tex, IndexAllocator） | `662a387` |
| T-03 | 制約 TeX（12 Constraint 全バリアント対応、配列添字自動付与、カテゴリ順ソート） | `cdfd703` |
| T-04 | 入力形式 TeX（9 NodeKind 全バリアント対応、Repeat 展開、Matrix 多行） | `aa3da40` |
| T-05 | 統合テスト（Fragment/Standalone、グラフ問題 e2e、Hole 抑制） | `a86c9d4` |

`render_tex/` は 36 テストで golden test カバレッジ確保済み。

### Gap Resolution Phase（完了）

Gaps A, B, H, D を解消。8 タスク (T-01〜T-08) 完了、テスト 204 通過。

| タスク | 内容 | コミット |
|-------|------|---------|
| T-01 | NodeKind マイグレーション（Repeat.count/Array.length を Expression 化、index_var 追加） | 完了 |
| T-02 | resolve_expression_as_int 完全実装（BinOp, Pow, FnCall, loop var 解決） | 完了 |
| T-03 | ループ変数サポート（generate_repeat に loop_vars 管理追加） | 完了 |
| T-04 | Choice in Repeat のスナップショット＋出力修正 | 完了 |
| T-05 | プレーンテキスト描画改善（render_expression, Choice 表示） | 完了 |
| T-06 | TeX 描画改善（expression_to_tex, Choice の cases 環境） | 完了 |
| T-07 | E2E 統合テスト（Graph, 三角行列, Query 問題）＋ Tuple-in-Repeat 修正 | 完了 |
| T-08 | ドキュメント更新 | 完了 |

**解消したギャップ:**
- Gap A (P0): `Repeat.count` が `Expression` を受け付け（N-1 辺のグラフ問題等）
- Gap B (P1): Choice in Repeat の検証＋描画改善（クエリ型問題）
- Gap H (P2): Repeat にループ変数追加（イテレーション番号を式中で参照可能）
- Gap D (P2): 三角行列を Repeat+Array+ループ変数で表現（Matrix 変更不要）

---

## 未実装・今後のタスク

### 近い将来（Phase 2 候補）

core 型の拡張と既存モジュールの強化。設計済みだが未着手。

| 項目 | 概要 | 優先度 |
|------|------|--------|
| Tuple 内 inline Array | Tuple の要素に直接 Array を持てるように（`(A_i, B_i)` パターン） | 中 |
| 文字列長・文字種制約の強化 | StringLength / CharSet の表現力向上 | 中 |

### 中期（インフラ・接続）

| 項目 | 概要 |
|------|------|
| WASM バインディング | cp-ast-core を wasm-pack でビルド、JSON API で公開 |
| フロントエンド構造化エディタ | Projection/Operation を使った GUI エディタ（Web） |
| TeX プレビュー | WASM 経由で TeX 断片をブラウザ上でリアルタイム表示 |
| Sample 生成の高度化 | 境界値テスト、コーナーケース自動生成 |

### 長期（拡張）

| 項目 | 概要 |
|------|------|
| Statement generator | 問題文テンプレートへの AST 接続 |
| PDF ビルドパイプライン | TeX → PDF の自動化 |
| AI Agent 補助 | AST を入力として問題文ドラフト生成 |
| 出力表記の TeX | 出力形式・注意書きの TeX 生成 |
| render-ir 中間表現 | 複数出力形式が増えた場合の共通 IR（現時点では YAGNI） |

---

## 現在のコード規模

```
crates/cp-ast-core/src/
├── structure/     (6 files)  — AST ノード・参照・型
├── constraint/    (7 files)  — 制約・式・型
├── operation/     (8 files)  — 編集操作・エンジン
├── projection/    (4 files)  — UI 向け読み取り像
├── render/        (3 files)  — プレーンテキスト生成
├── render_tex/    (4 files)  — TeX 生成
├── sample/        (4 files)  — テストケース生成
└── lib.rs

テスト: 204 passing (unit + integration + e2e)
```

## まとめ

**完了**: AST Core の全基盤（構造・制約・操作・投影・テキスト描画・TeX 描画・サンプル生成）＋ Gap Resolution（Expression 化、ループ変数、Choice 描画改善）

**次のステップ**: フロントエンド接続（WASM + 構造化エディタ）、またはさらなる core 型拡張（Tuple 内 inline Array、文字列制約強化）。
