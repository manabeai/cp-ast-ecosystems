# 競技プログラミング問題記述DSL — 設計仕様書

## 概要

本プロジェクトは、競技プログラミング問題の形式仕様（入力方式・制約・出力方式）を構造化ASTとして表現・編集・表示・生成するためのDSLライブラリをRustで実装する。

中心となる4モジュール:
- **StructureAST**: 問題仕様の構造的正本（形）
- **ConstraintAST**: 構造に課される許容条件
- **ProjectionAPI**: 外部に対して編集可能な像を返す導出層
- **Operation**: 編集要求を安全に状態遷移に変換する実行層

## 設計原則

1. **Structure-first**: テキストではなく構造が正本
2. **Hole は第一級**: 未完成状態は例外ではなくコアの構成要素
3. **型と制約の分離**: 型で守るものと制約で守るものを分ける
4. **View は薄い投影**: GUI/CLI/AI は同じコアからの投影
5. **生成可能な制約**: 制約は禁止条件の集合ではなく構築可能性の仕様

## 実行計画: Parallel Sprint Model

### 依存関係

```
Sprint 1 (調査)  ──→  Sprint 2 (設計)  ──→  Sprint 3 (検証)
                                                     │
                                                     ↓
                                               Sprint 4 (統合+環境)
                                                     │
                                                     ↓
                                               Sprint 5 (Rust実装)
```

Sprint内のSubAgentは並列実行。Sprint間は前段の出力に依存。

---

## Sprint 1: 調査フェーズ

### SiteSurveyAgent

**目的**: AtCoderを最優先に、競プロサイトの問題記述の共通構造と差異を観察する。

**サンプリング方針**:
- AtCoder ABC (A-F): 6問程度（易→難の幅）
- AtCoder ARC/AGC: 2-3問（非典型パターン）
- 重点カテゴリ: 配列、グラフ、行列、複数テストケース、クエリ形式、interactive、可変長入力、相互依存制約
- `web_fetch` で問題ページを巡回し、Input/Output/Constraints/Sample の構成を分析

**出力**:
- 共通パターン一覧
- サイト間・問題間の差異
- 例外表現（表現揺れ、特殊形）
- DSLで最小限押さえるべき構造（最小共通核）

### ResearchAlignmentAgent

**目的**: `doc/reference.md` の先行研究と現設計 (`ast-draft.md`) を照合する。

**分析対象概念**:
- StructureAST / ConstraintAST
- ProjectionAPI / Operation
- hole / canonical rendering / sample generation
- structure-first / editable projection

**照合先の研究領域**:
- structure editor / projectional editing (Hazelnut, grammar cells)
- typed holes / incomplete programs (GHC, Hazel)
- live functional programming / livelits
- type/example-directed synthesis
- DSL / metamodel / schema-driven editor

**出力**:
- 現設計と先行研究の対応表
- 見落としている既存知見
- 再発明に見える箇所
- 新規性が主張できそうな箇所
- 論文引用候補
- 「どこまでなら言ってよくて、どこからは言いすぎか」の境界

---

## Sprint 2: 設計フェーズ

### DomainModelAgent

**目的**: Sprint 1 の知見を踏まえ、StructureAST / ConstraintAST のドメインモデルを確定する。

**入力**: 既存ドラフト (`ast-draft.md`, `modules.md`) + Sprint 1 出力

**出力**:
- **最小ノード集合**: Scalar, Array, Matrix, MultiTestCase, Query, InputBlock, OutputBlock, Graph 等
- **最小制約集合**: Range, Length, Element, Dependency, Contextual, TypeConstraint
- **Node/Slot/Hole/Reference の整理**: 各概念の定義と関係
- **canonical rendering に必要な情報**: 順序規則、表示名正規化、展開規則
- **sample generator に必要な情報**: 生成順序、依存グラフ、制約の充足可能性判定
- **Rust 型定義ドラフト**: enum/struct レベルでの型設計

### ProjectionOperationAgent

**目的**: ProjectionAPI と Operation の責務境界を詰める。

**出力**:
- **Projection が返すべき情報**:
  - 表示対象ノード一覧、各ノードのラベル
  - slot 名、hole の期待型、hole に対する候補種別
  - 実行可能な操作一覧、編集不可理由
- **Action の分類**:
  - FillHole: hole を埋める
  - ReplaceNode: ノード差し替え
  - AddConstraint / RemoveConstraint: 制約操作
  - IntroduceMultiTestCase: 複数テストケース構造の導入
  - AddSlotElement: slot に要素を追加
- **apply 時の失敗理由の型**: `Result<NewState, OperationError>` の OperationError 設計
- **View を薄く保つ条件**: Projection は read-only 導出、状態変更は Operation 経由のみ
- **AI Agent が触れる API 面**: CLI/テストコードと同じ Operation API

---

## Sprint 3: 検証フェーズ

### CriticalReviewAgent

**目的**: Sprint 2 の設計を破壊的にレビューする。

**必ず疑う観点**:
- 構造と制約の分離が曖昧ではないか
- canonical rendering は本当に一意か
- 生成可能保証を言いすぎていないか
- query形式や複数ケースで破綻しないか
- 実サイトの変種に耐えない抽象化ではないか
- ProjectionAPI と Operation の責務が混ざっていないか

**出力**: 重大欠陥 / 中程度の欠陥 / 軽微な懸念 / 修正提案

### CoverageValidationAgent

**目的**: AtCoder の実問題でカバレッジを検証する。

**サンプル数**: 15-20問（カテゴリ均等）

**カテゴリ**:
- 整数/配列 (基本)
- グラフ (木、一般グラフ)
- 行列 / 2D配列
- 文字列
- 複数テストケース
- クエリ形式
- 可変長入力
- 相互依存制約
- interactive

**各問題の判定項目**:
| 項目 | 判定 |
|------|------|
| StructureAST | 対応可能 / 部分対応 / 非対応 |
| ConstraintAST | 対応可能 / 部分対応 / 非対応 |
| canonical rendering | 可能 / 一部曖昧 / 不可 |
| sample generation | 可能 / 条件付き / 不可 |

**出力**: 判定表 + カテゴリ別カバー率 + 最優先の未対応カテゴリ

### SampleGenerationAgent

**目的**: ConstraintAST からランダム sample case を生成するために必要な制約情報を整理する。

**検証項目**:
- 生成順序が導出できるか（依存グラフの DAG 性）
- 長さ依存や配列依存を解決できるか
- 範囲制約を扱えるか
- 複数テストケースを扱えるか
- unsat の場合の扱いが定義されているか
- 生成可能保証の定義が妥当か

**出力**: 生成可能な制約 / 生成が難しい制約 / 設計上の追加要件

### Sprint 3 の修正ループ

Sprint 3 で重大欠陥が発見された場合、Sprint 2 の設計を修正するループを **最大1回** 実行する。修正後は CriticalReviewAgent のみ再実行して確認する。

---

## Sprint 4: 統合 + 環境構築

### 最終設計文書

Sprint 1-3 の全出力を統合し、initial-prompt.md で指定された 13 章構成の最終設計文書を作成する:

1. Executive Summary
2. 観察した競プロサイトの構造的特徴
3. 設計目標と非目標
4. 4モジュールの責務分離
5. StructureAST 設計
6. ConstraintAST 設計
7. ProjectionAPI 設計
8. Operation 設計
9. canonical rendering 方針
10. ランダム sample case 生成方針
11. 実サイトサンプルに対するカバレッジ検証結果
12. 破綻例・未対応例・将来拡張
13. 実装着手順

保存先: `doc/design/final-design.md`

### Rust開発環境構築

`execution.md` の要件に従い:

- `cargo init --lib` でRustプロジェクト初期化（workspace構成）
- **GitHub Actions CI**:
  - `cargo build`
  - `cargo test`
  - `cargo clippy -- -D warnings`
  - `cargo fmt --check`
- **Pre-commit hooks**: clippy + fmt チェック
- **Copilot hooks**: ファイル変更時の自動検証
- `AGENTS.md` に開発規約を記載

---

## Sprint 5: Rust最小コア実装

### StructureAST crate

```rust
// 主要な型（最終的な型名は Sprint 2 で確定）
pub struct NodeId(u64);

pub enum NodeKind {
    Scalar,
    Array,
    Matrix,
    MultiTestCase,
    Query,
    InputBlock,
    OutputBlock,
    // Sprint 2 の結果で追加される可能性あり
}

pub struct Slot {
    pub name: String,
    pub child: SlotValue,
}

pub enum SlotValue {
    Filled(StructureNode),
    Hole(HoleInfo),
}

pub struct HoleInfo {
    pub id: NodeId,
    // 期待される情報は ConstraintAST 側で持つ
}

pub struct StructureNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub slots: Vec<Slot>,
}
```

### ConstraintAST crate

```rust
pub enum ExpectedType {
    Int,
    String,
    Array(Box<ExpectedType>),
    // ...
}

pub struct RangeConstraint {
    pub target: NodeId,
    pub min: Expression,
    pub max: Expression,
}

pub struct LengthConstraint {
    pub array: NodeId,
    pub length: NodeId, // 別の変数への参照
}

pub struct ElementConstraint {
    pub array: NodeId,
    pub element_constraint: Box<Constraint>,
}

pub enum Constraint {
    Range(RangeConstraint),
    Length(LengthConstraint),
    Element(ElementConstraint),
    Dependency(DependencyConstraint),
    // ...
}

pub struct ConstraintSet {
    pub constraints: Vec<Constraint>,
}
```

### テスト方針

- **典型問題表現テスト**: AtCoder ABC 級の問題（整数N + 配列A + 制約）をASTで表現し、構造が正しく保持されることを検証
- **canonical rendering 決定性テスト**: 同じ意味のASTから同じ文字列表現が得られることを検証
- **hole テスト**: 未完成状態を含むASTが安全に保持・操作できることを検証

### Sprint 5 で実装しないもの

- ProjectionAPI / Operation（Sprint 6 以降）
- Parser
- Sample generator の実装
- GUI / CLI フロントエンド
- 複雑なスコープ解析

---

## 品質基準

initial-prompt.md の品質基準に従い、以下を満たさない設計は却下する:

- [ ] 実サイト上の複数問題に当てられていない
- [ ] 抽象概念だけで Node / Constraint / Action が曖昧
- [ ] canonical rendering の決定規則がない
- [ ] sample generator の対象範囲が曖昧
- [ ] 批判的観点が不足している
- [ ] 検証データがない
- [ ] GUI都合の概念が core に混入している
- [ ] parser 前提に戻っている

## ユーザー承認ポイント

各 Sprint 完了時にユーザー承認を得る:
1. Sprint 1 完了後: 調査結果のレビュー
2. Sprint 2 完了後: 設計内容のレビュー
3. Sprint 3 完了後: 検証結果と修正方針のレビュー
4. Sprint 4 完了後: 最終設計文書と環境構築のレビュー
5. Sprint 5 完了後: 最小コア実装のレビュー
