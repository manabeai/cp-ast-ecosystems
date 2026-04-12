# CP-AST Ecosystems Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Design and implement a competitive programming problem specification DSL in Rust, validated against real AtCoder problems.

**Architecture:** 5-sprint parallel model. Sprint 1 (survey + research) produces design inputs. Sprint 2 (domain model + projection/operation design) produces type-level specifications. Sprint 3 (critical review + coverage + sample generation) validates the design. Sprint 4 (final document + Rust environment) sets up for implementation. Sprint 5 (TDD Rust minimal core) implements StructureAST + ConstraintAST.

**Tech Stack:** Rust (2021 edition), cargo workspace, GitHub Actions CI, clippy + rustfmt

**Spec:** `docs/superpowers/specs/2026-04-12-cp-ast-ecosystems-design.md`

**Reference docs:**
- `doc/plan/initial-prompt.md` — orchestration prompt with SubAgent definitions
- `doc/plan/ast-draft.md` — existing AST design draft
- `doc/plan/modules.md` — 4-module architecture
- `doc/plan/execution.md` — CI/hooks requirements
- `doc/reference.md` — academic references

---

## File Structure

### Design Documents (Sprint 1-3)

```
doc/
  survey/
    site-survey-report.md          # SiteSurveyAgent output
    research-alignment-report.md   # ResearchAlignmentAgent output
  design/
    domain-model.md                # DomainModelAgent output
    projection-operation.md        # ProjectionOperationAgent output
    final-design.md                # 13-chapter integrated design document
  review/
    critical-review.md             # CriticalReviewAgent output
    coverage-validation.md         # CoverageValidationAgent output
    sample-generation.md           # SampleGenerationAgent output
```

### Rust Project (Sprint 4-5)

```
Cargo.toml                         # workspace root
crates/
  cp-ast-core/
    Cargo.toml
    src/
      lib.rs                       # re-exports structure + constraint modules
      structure/
        mod.rs                     # module declarations + re-exports
        node_id.rs                 # NodeId newtype with auto-increment
        node_kind.rs               # NodeKind enum
        slot.rs                    # Slot, SlotValue
        structure_node.rs          # StructureNode tree type
      constraint/
        mod.rs                     # module declarations + re-exports
        expected_type.rs           # ExpectedType enum
        expression.rs              # Expression for constraint bounds
        constraint.rs              # Constraint variants
        constraint_set.rs          # ConstraintSet with composition
    tests/
      structure_basic.rs           # StructureNode creation and traversal
      hole_basic.rs                # Hole creation and detection
      constraint_basic.rs          # Constraint creation and validation
      typical_problem.rs           # Express ABC-level problem as AST
```

### CI/Hooks (Sprint 4)

```
.github/
  workflows/
    ci.yml                         # cargo build + test + clippy + fmt
.githooks/
  pre-commit                       # clippy + fmt check
AGENTS.md                          # development conventions
```

---

## Sprint 1: Survey Phase

### Task 1: SiteSurveyAgent — AtCoder Problem Structure Survey

**Files:**
- Create: `doc/survey/site-survey-report.md`

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch SiteSurveyAgent**

Dispatch a `general-purpose` agent with the following prompt. The agent should use `web_fetch` to survey AtCoder problem pages and produce a structured report.

**Agent prompt:**

```
あなたは競プロサイト観察担当（SiteSurveyAgent）です。

## 任務

AtCoderを最優先に、競プロサイトの問題記述の共通構造と差異を観察してください。

## サンプリング方針

以下のカテゴリから各1-2問をweb_fetchで取得・分析してください:

1. 基本（整数+配列）: ABC 典型A-C問題
2. グラフ（木/一般グラフ）: ABC D-E級
3. 行列/2D配列: ABC or ARC
4. 複数テストケース: ARC/AGC（T個のケース形式）
5. クエリ形式: ABC D-F級
6. 文字列: ABC B-D級
7. 可変長入力: 問題ごとに入力行数が変わるもの
8. 相互依存制約: 変数間の制約が複雑なもの
9. interactive: 対話型問題

AtCoder の問題ページURL形式: https://atcoder.jp/contests/{contest_id}/tasks/{task_id}
例: https://atcoder.jp/contests/abc300/tasks/abc300_a

各問題ページから Input/Output/Constraints/Sample を抽出してください。

## 分析観点

各問題について以下を記録:
- Input Format の構造（行ごとの要素、区切り、変数依存）
- Constraints の書き方（値域、長さ、型、依存関係）
- Output Format の構造
- Sample Input/Output の構成

## 出力フォーマット

以下の章立てで doc/survey/site-survey-report.md に保存:

1. サンプリング結果一覧（問題URL + カテゴリ + 概要）
2. 共通パターン（Input/Constraints/Output の定型構造）
3. 差異・表現揺れ（サイト内/カテゴリ間の違い）
4. 例外表現（特殊な入力形式、非典型的な制約記述）
5. DSLで最小限押さえるべき構造（最小共通核の提案）

robots.txt を尊重し、アクセス頻度に配慮すること。
```

- [ ] **Step 2: Review SiteSurveyAgent output**

Read `doc/survey/site-survey-report.md` and verify:
- At least 8 problems surveyed across categories
- Each problem has Input/Constraints/Output analysis
- 共通パターン section identifies at least 5 recurring structural patterns
- 例外表現 section includes at least 3 non-typical cases
- 最小共通核 section proposes a concrete minimal set

If incomplete, provide feedback and re-dispatch.

- [ ] **Step 3: Commit**

```bash
git add doc/survey/site-survey-report.md
git commit -m "docs: add AtCoder site survey report

Survey of 8+ AtCoder problems across categories (basic, graph,
matrix, multi-testcase, query, string, variable-length, interactive).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 2: ResearchAlignmentAgent — Prior Art Alignment (parallel with Task 1)

**Files:**
- Create: `doc/survey/research-alignment-report.md`
- Read: `doc/reference.md`, `doc/plan/ast-draft.md`

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch ResearchAlignmentAgent**

Dispatch a `general-purpose` agent with the following prompt:

```
あなたは ResearchAlignmentAgent です。

## 任務

以下の2つのファイルを読み、現設計を先行研究と照らし合わせて分析してください。

- doc/reference.md — 先行研究メモ（Hazelnut, livelits, projectional editor等）
- doc/plan/ast-draft.md — 現在の AST 設計ドラフト

## 分析対象概念

現設計から以下の概念を抽出し、先行研究との対応を整理:
- StructureAST / ConstraintAST
- ProjectionAPI / Operation
- hole / canonical rendering / sample generation
- structure-first / editable projection
- typed constraints / hole completion

## 照合先の研究領域

doc/reference.md に記載の文献を中心に:
- structure editor / projectional editing (Hazelnut, grammar cells)
- typed holes / incomplete programs (GHC, Hazel)
- live functional programming / livelits (PLDI 2021)
- type/example-directed synthesis
- DSL / metamodel / schema-driven editor
- JSON-based DSL projectional editors (2023)

## 必ず出力するもの

以下の章立てで doc/survey/research-alignment-report.md に保存:

1. 現設計と先行研究の対応表（表形式: 設計概念 / 対応する先行研究 / 差異）
2. 見落としている既存知見（取り込むべきもの）
3. 再発明に見える箇所（既に同等の設計が存在するもの）
4. 新規性が主張できそうな箇所
5. 論文引用候補一覧
6. 「どこまでなら言ってよくて、どこからは言いすぎか」の境界

好意的に読むのではなく、研究として通す観点で厳しく評価してください。
```

- [ ] **Step 2: Review ResearchAlignmentAgent output**

Read `doc/survey/research-alignment-report.md` and verify:
- 対応表 has at least 8 rows covering all major concepts
- 再発明 section identifies specific overlaps (not vague)
- 新規性 section is honest about what's genuinely new
- 引用候補 lists at least 10 papers with relevance notes

If incomplete, provide feedback and re-dispatch.

- [ ] **Step 3: Commit**

```bash
git add doc/survey/research-alignment-report.md
git commit -m "docs: add research alignment report

Maps current design concepts to prior art (Hazelnut, livelits,
projectional editing). Identifies novel vs. reinvented elements.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint 2: Design Phase

### Task 3: DomainModelAgent — StructureAST / ConstraintAST Domain Model

**Files:**
- Create: `doc/design/domain-model.md`
- Read: `doc/plan/ast-draft.md`, `doc/plan/modules.md`, `doc/survey/site-survey-report.md`, `doc/survey/research-alignment-report.md`

**Depends on:** Task 1, Task 2

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch DomainModelAgent**

Dispatch a `general-purpose` agent with the following prompt:

```
あなたは DomainModelAgent です。

## 任務

StructureAST / ConstraintAST のドメインモデルを確定してください。

## 入力

以下のファイルを読んでから設計に入ること:
- doc/plan/ast-draft.md — 既存AST設計ドラフト（出発点）
- doc/plan/modules.md — 4モジュール設計の全体像
- doc/survey/site-survey-report.md — AtCoderサイト調査結果
- doc/survey/research-alignment-report.md — 先行研究との対応

## 設計原則

- Structure-first: テキストではなく構造が正本
- Hole は第一級: 未完成状態はコアの構成要素
- 型と制約の分離: 型で守るものと制約で守るものを分ける
- 生成可能な制約: 制約は禁止条件ではなく構築可能性の仕様

## 出力

以下の章立てで doc/design/domain-model.md に保存:

### 1. 最小ノード集合
サイト調査結果を踏まえ、最小限必要な NodeKind を列挙。
各 NodeKind について: 名前 / 役割 / 持つ Slot / 対応する競プロ概念 / 具体例

### 2. 最小制約集合
サイト調査結果を踏まえ、最小限必要な Constraint 種別を列挙。
各制約について: 名前 / 何を制約するか / 対象の NodeKind / 具体例

### 3. Node/Slot/Hole/Reference の整理
各概念の厳密な定義。特に:
- Slot の種類（単一子 / リスト子 / オプション子）
- Hole の情報（制約は ConstraintAST 側で持つ）
- Reference の解決方法（NodeId による）
- VariableRef と NodeId の関係

### 4. Canonical Rendering に必要な情報
- 順序規則（ノードの表示順序の決定方法）
- 表示名正規化（変数名の canonical form）
- 展開規則（配列展開 A_1 A_2 ... A_N 等）

### 5. Sample Generator に必要な情報
- 生成順序の導出方法（依存グラフ）
- 各制約種別の生成戦略
- 生成可能保証の定義

### 6. Rust 型定義ドラフト
enum/struct レベルでの型設計を Rust コードブロックで提示。
NodeId, NodeKind, Slot, SlotValue, HoleInfo, StructureNode,
ExpectedType, Expression, Constraint variants, ConstraintSet。
各型に derive マクロを含めること。

サイト調査で見つかった「非典型パターン」に対して、提案するモデルが
どこまで対応できるか（対応可能 / 部分対応 / 非対応）も必ず明記すること。
```

- [ ] **Step 2: Review DomainModelAgent output**

Read `doc/design/domain-model.md` and verify:
- 最小ノード集合 has at least 6 NodeKind variants with concrete examples
- 最小制約集合 has at least 5 Constraint variants
- Rust 型定義ドラフト section has compilable-looking Rust code
- サイト調査の非典型パターンへの対応可否が明記されている

If incomplete, provide feedback and re-dispatch.

- [ ] **Step 3: Commit**

```bash
git add doc/design/domain-model.md
git commit -m "docs: add domain model design

Defines minimal node set, constraint set, and Rust type drafts
for StructureAST and ConstraintAST.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 4: ProjectionOperationAgent — API Surface Design (parallel with Task 3)

**Files:**
- Create: `doc/design/projection-operation.md`
- Read: `doc/plan/modules.md`, `doc/survey/site-survey-report.md`

**Depends on:** Task 1, Task 2

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch ProjectionOperationAgent**

Dispatch a `general-purpose` agent with the following prompt:

```
あなたは ProjectionOperationAgent です。

## 任務

ProjectionAPI と Operation の責務境界を詰め、API面を定義してください。

## 入力

以下のファイルを読むこと:
- doc/plan/modules.md — 4モジュール設計の全体像
- doc/survey/site-survey-report.md — AtCoderサイト調査結果

## 設計原則

- View は薄い投影: GUI/CLI/AI は同じコアからの投影
- Projection は read-only 導出、状態変更は Operation 経由のみ
- AI Agent / CLI / テストコードは同じ Operation API を使う

## 出力

以下の章立てで doc/design/projection-operation.md に保存:

### 1. ProjectionAPI が返すべき情報
型定義を Rust trait として提示:
- 表示対象ノード一覧、各ノードのラベル
- slot 名、hole の期待型、hole に対する候補種別
- 実行可能な操作一覧、編集不可理由
具体例: あるABC問題のASTに対して、Projectionが返す情報の具体例

### 2. Action の分類
enum として定義:
- FillHole: hole を埋める
- ReplaceNode: ノード差し替え
- AddConstraint / RemoveConstraint: 制約操作
- IntroduceMultiTestCase: 複数テストケース構造の導入
- AddSlotElement: slot に要素を追加
各 Action について: 入力パラメータ / 前提条件 / 成功時の状態変化 / 失敗理由

### 3. OperationError の設計
失敗理由を enum として定義:
- TypeMismatch
- NodeNotFound
- SlotOccupied
- ConstraintViolation
- InvalidOperation
各エラーについて具体例

### 4. View を薄く保つ条件
Projection が持つべきでないもの / Operation に委譲すべきもの の明確な線引き

### 5. AI Agent が触れる API 面
CLI/テストコードと同じ Operation API で十分か、
Agent 向けの追加 API が必要かを検討

各セクションに、具体的な競プロ問題での操作例を含めること。
```

- [ ] **Step 2: Review ProjectionOperationAgent output**

Read `doc/design/projection-operation.md` and verify:
- Rust trait/enum 定義が含まれている
- 各 Action に入力/前提/成功/失敗が定義されている
- OperationError が具体例付き
- 具体的な競プロ問題での操作例がある

If incomplete, provide feedback and re-dispatch.

- [ ] **Step 3: Commit**

```bash
git add doc/design/projection-operation.md
git commit -m "docs: add ProjectionAPI and Operation design

Defines Projection trait, Action enum, OperationError,
and API surface for GUI/CLI/AI consumers.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint 3: Validation Phase

### Task 5: CriticalReviewAgent — Destructive Design Review

**Files:**
- Create: `doc/review/critical-review.md`
- Read: `doc/design/domain-model.md`, `doc/design/projection-operation.md`

**Depends on:** Task 3, Task 4

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch CriticalReviewAgent**

Dispatch a `general-purpose` agent with the following prompt:

```
あなたは破壊的レビュー担当（CriticalReviewAgent）です。
提示された設計を好意的に読むのではなく、破綻させる視点で読んでください。

## 入力

以下の設計文書を読むこと:
- doc/design/domain-model.md — StructureAST/ConstraintAST ドメインモデル
- doc/design/projection-operation.md — ProjectionAPI/Operation 設計
- doc/plan/ast-draft.md — 元の設計ドラフト（比較用）

## 必ず疑う観点

1. 構造と制約の分離が曖昧ではないか
   - StructureAST に制約的な情報が混入していないか
   - ConstraintAST に構造的な情報が混入していないか
2. canonical rendering は本当に一意か
   - 同じ意味の異なる AST 構造から同じ rendering が出るか
   - ノード順序が rendering に影響しないか
3. 生成可能保証を言いすぎていないか
   - 「生成可能」の定義が曖昧でないか
   - unsat ケースの扱いが定義されているか
4. query形式や複数ケースで破綻しないか
   - クエリの種別が動的に決まるケース
   - テストケース間で共有される制約
5. 実サイトの変種に耐えない抽象化ではないか
   - 過剰に一般化していて使いにくくないか
   - 逆に特殊すぎて拡張できないか
6. ProjectionAPI と Operation の責務が混ざっていないか
   - Projection が state mutation していないか
   - Operation が表示ロジックを持っていないか

## 出力フォーマット

doc/review/critical-review.md に以下の構成で保存:

### 重大欠陥（設計の根幹に関わるもの）
### 中程度の欠陥（修正可能だが放置すると問題になるもの）
### 軽微な懸念（改善の余地があるもの）
### 修正提案（具体的な改善案）

各項目に:
- 問題の説明
- 破綻する具体例
- 推奨される修正方向
```

- [ ] **Step 2: Review CriticalReviewAgent output**

Read `doc/review/critical-review.md` and verify:
- At least 2 重大欠陥 or clear statement that none exist
- 破綻する具体例 が各項目に含まれている
- 修正提案 が具体的（「改善すべき」ではなく具体的な型/構造の変更案）

- [ ] **Step 3: Commit**

```bash
git add doc/review/critical-review.md
git commit -m "docs: add critical design review

Destructive review of domain model and projection/operation design.
Identifies defects with concrete failure examples.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 6: CoverageValidationAgent — Real Problem Coverage (parallel with Task 5)

**Files:**
- Create: `doc/review/coverage-validation.md`
- Read: `doc/design/domain-model.md`

**Depends on:** Task 3, Task 4

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch CoverageValidationAgent**

Dispatch a `general-purpose` agent with the following prompt:

```
あなたは実サンプル検証担当（CoverageValidationAgent）です。

## 任務

AtCoderから15-20問を収集し、現設計でどこまで表現できるかを判定してください。

## 入力

- doc/design/domain-model.md — 現在の StructureAST/ConstraintAST ドメインモデル

## サンプリング方針

以下のカテゴリから各2問ずつ web_fetch で取得:
1. 整数/配列（基本）— ABC A-C級
2. グラフ（木）— ABC D-E級
3. グラフ（一般）— ABC D-F級
4. 行列/2D配列
5. 文字列
6. 複数テストケース
7. クエリ形式
8. 可変長入力
9. 相互依存制約
10. interactive（あれば）

AtCoder 問題URL: https://atcoder.jp/contests/{contest_id}/tasks/{task_id}

## 各問題の判定

各問題について以下を表に記入:

| 項目 | 判定 |
|------|------|
| 問題URL | |
| カテゴリ | |
| StructureAST | 対応可能 / 部分対応 / 非対応 |
| ConstraintAST | 対応可能 / 部分対応 / 非対応 |
| canonical rendering | 可能 / 一部曖昧 / 不可 |
| sample generation | 可能 / 条件付き / 不可 |
| コメント | 具体的にどの部分が対応/非対応か |

「部分対応」「非対応」の場合は、具体的にどのノード/制約が足りないかを明記。

## 出力

doc/review/coverage-validation.md に以下の構成で保存:

1. サンプル一覧（問題ごとの判定表）
2. カテゴリ別カバー率（表形式）
3. 未対応カテゴリの分析
4. 最優先の拡張候補
```

- [ ] **Step 2: Review CoverageValidationAgent output**

Read `doc/review/coverage-validation.md` and verify:
- At least 15 problems in the judgment table
- Each problem has all 4 judgment items filled
- カバー率 section has concrete percentages
- 未対応カテゴリ identifies specific gaps

- [ ] **Step 3: Commit**

```bash
git add doc/review/coverage-validation.md
git commit -m "docs: add coverage validation report

15-20 AtCoder problems validated against current design.
Category-level coverage rates and gap analysis.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 7: SampleGenerationAgent — Generation Feasibility (parallel with Task 5, 6)

**Files:**
- Create: `doc/review/sample-generation.md`
- Read: `doc/design/domain-model.md`

**Depends on:** Task 3

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch SampleGenerationAgent**

Dispatch a `general-purpose` agent with the following prompt:

```
あなたは sample case 生成担当（SampleGenerationAgent）です。

## 任務

ConstraintAST からランダムに妥当な入力例を生成する観点で設計をレビューしてください。

## 入力

- doc/design/domain-model.md — 現在の ConstraintAST ドメインモデル

## 特に見る項目

1. 生成順序の導出可能性
   - 制約間の依存グラフが DAG になるか
   - トポロジカルソートで生成順序を決められるか
   - 循環依存がある場合の検出方法

2. 長さ依存・配列依存の解決
   - N を先に生成し、A[0..N] を後に生成できるか
   - 2D配列 (H×W) の生成順序

3. 範囲制約の処理
   - 整数範囲: rand in [min, max]
   - Power 表現 (10^9 等) の評価

4. 複数テストケースの処理
   - T 個のケースを独立に生成できるか
   - ケース間の制約（合計 N の上限等）

5. unsat 判定
   - min > max のような明らかな矛盾
   - 制約同士の矛盾（長さ制約 + 要素制約で充足不可能なケース）
   - unsat 時の挙動（エラー返却 vs. best-effort）

6. 生成可能保証の定義
   - 何を「生成可能」と定義するか
   - 保証できる範囲と保証できない範囲

## 出力

doc/review/sample-generation.md に以下の構成で保存:

### 生成可能な制約
- 各制約種別について生成戦略を具体的に記述

### 生成が難しい制約
- なぜ難しいか、どう対処するか

### 設計上の追加要件
- 生成器の実装に必要だが現設計に不足している情報

### 生成可能保証の提案定義
- 条件付き保証の厳密な定義案
```

- [ ] **Step 2: Review SampleGenerationAgent output**

Read `doc/review/sample-generation.md` and verify:
- 生成可能 section has concrete generation strategies per constraint type
- 生成困難 section identifies specific problem patterns
- 追加要件 section is actionable (not vague)
- 生成可能保証 has a precise definition proposal

- [ ] **Step 3: Commit**

```bash
git add doc/review/sample-generation.md
git commit -m "docs: add sample generation feasibility report

Analyzes generation strategies per constraint type,
identifies difficult patterns, proposes guarantee definition.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 8: Sprint 3 Synthesis — Design Revision

**Files:**
- Modify: `doc/design/domain-model.md`
- Modify: `doc/design/projection-operation.md`
- Read: `doc/review/critical-review.md`, `doc/review/coverage-validation.md`, `doc/review/sample-generation.md`

**Depends on:** Task 5, Task 6, Task 7

- [ ] **Step 1: Synthesize Sprint 3 findings**

Read all three review documents. Create a consolidated list of:
- 重大欠陥 from CriticalReviewAgent
- 未対応カテゴリ from CoverageValidationAgent
- 設計上の追加要件 from SampleGenerationAgent

For each finding, classify as: **must-fix before implementation** vs. **known limitation (document and defer)**.

- [ ] **Step 2: Apply must-fix changes to domain-model.md**

Edit `doc/design/domain-model.md` to address must-fix items. Typical changes:
- Add missing NodeKind variants identified by coverage gaps
- Refine Constraint types based on generation feasibility
- Clarify StructureAST/ConstraintAST boundary based on critical review

- [ ] **Step 3: Apply must-fix changes to projection-operation.md**

Edit `doc/design/projection-operation.md` to address must-fix items related to Projection/Operation.

- [ ] **Step 4: Re-dispatch CriticalReviewAgent (if heavy changes were made)**

Only if Step 2-3 resulted in significant structural changes. Re-run CriticalReviewAgent on the updated design with focus on the changed areas.

- [ ] **Step 5: Commit**

```bash
git add doc/design/domain-model.md doc/design/projection-operation.md
git commit -m "docs: revise design based on Sprint 3 validation

Address critical review findings, coverage gaps, and
sample generation requirements.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint 4: Integration + Environment

### Task 9: Final Design Document

**Files:**
- Create: `doc/design/final-design.md`
- Read: All Sprint 1-3 outputs

**Depends on:** Task 8

**Agent:** Dispatch as `general-purpose` sub-agent.

- [ ] **Step 1: Dispatch integration agent**

Dispatch a `general-purpose` agent with the following prompt:

```
あなたは最終設計文書統合担当です。

## 任務

Sprint 1-3 の全出力を統合し、13章構成の最終設計文書を作成してください。

## 入力ファイル

全て読むこと:
- doc/survey/site-survey-report.md
- doc/survey/research-alignment-report.md
- doc/design/domain-model.md
- doc/design/projection-operation.md
- doc/review/critical-review.md
- doc/review/coverage-validation.md
- doc/review/sample-generation.md
- doc/plan/initial-prompt.md（品質基準の確認用）

## 出力構成

doc/design/final-design.md に以下の 13 章構成で保存:

1. Executive Summary（1ページ以内）
2. 観察した競プロサイトの構造的特徴（site-survey-report から）
3. 設計目標と非目標（何を目指し、何を目指さないか）
4. 4モジュールの責務分離
5. StructureAST 設計（型定義 + 具体例）
6. ConstraintAST 設計（型定義 + 具体例）
7. ProjectionAPI 設計（trait 定義 + 使用例）
8. Operation 設計（Action enum + OperationError + 使用例）
9. canonical rendering 方針（規則 + 具体例）
10. ランダム sample case 生成方針（戦略 + 保証定義）
11. 実サイトサンプルに対するカバレッジ検証結果（coverage-validation から）
12. 破綻例・未対応例・将来拡張（critical-review + coverage から）
13. 実装着手順（Sprint 5 の具体的なタスク順序）

## 品質基準

以下を全て満たすこと:
- 各設計節に「この設計で表現できる問題の例」が最低1つ
- カバレッジ検証で 対応可能/部分対応/非対応 が分かれている
- 「非対応」が恥ではなく明記されている
- 「今すぐ実装すべき最小コア」が絞り込まれている
- 先行研究との関係が明確（再発明でないことの説明）
```

- [ ] **Step 2: Review final design document**

Read `doc/design/final-design.md` and verify against initial-prompt.md quality criteria:
- [ ] 実サイト上の複数問題に当てられている
- [ ] Node / Constraint / Action が具体的
- [ ] canonical rendering の決定規則がある
- [ ] sample generator の対象範囲が明確
- [ ] 批判的観点が反映されている
- [ ] 検証データがある
- [ ] GUI都合の概念が core に混入していない
- [ ] parser 前提に戻っていない

- [ ] **Step 3: Commit**

```bash
git add doc/design/final-design.md
git commit -m "docs: add final integrated design document

13-chapter design document covering all 4 modules,
validated against AtCoder problems with coverage data.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 10: Rust Project Initialization + CI (parallel with Task 9)

**Files:**
- Create: `Cargo.toml`
- Create: `crates/cp-ast-core/Cargo.toml`
- Create: `crates/cp-ast-core/src/lib.rs`
- Create: `.github/workflows/ci.yml`
- Create: `.githooks/pre-commit`
- Modify: `AGENTS.md`

**Depends on:** None (can start as soon as Sprint 3 is complete)

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT"

[workspace.lints.clippy]
all = "deny"
pedantic = "warn"
```

- [ ] **Step 2: Create cp-ast-core crate**

```toml
# crates/cp-ast-core/Cargo.toml
[package]
name = "cp-ast-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Core AST types for competitive programming problem specification DSL"

[lints]
workspace = true
```

```rust
// crates/cp-ast-core/src/lib.rs
pub mod structure;
pub mod constraint;
```

- [ ] **Step 3: Create placeholder modules**

```rust
// crates/cp-ast-core/src/structure/mod.rs
// Re-exports are added as types are implemented in Tasks 11-13.
pub mod node_id;
pub mod node_kind;
pub mod slot;
pub mod structure_node;
```

```rust
// crates/cp-ast-core/src/structure/node_id.rs
// Implemented in Task 11
```

```rust
// crates/cp-ast-core/src/structure/node_kind.rs
// Implemented in Task 12
```

```rust
// crates/cp-ast-core/src/structure/slot.rs
// Implemented in Task 13
```

```rust
// crates/cp-ast-core/src/structure/structure_node.rs
// Implemented in Task 13
```

```rust
// crates/cp-ast-core/src/constraint/mod.rs
// Re-exports are added as types are implemented in Tasks 14-15.
pub mod expected_type;
pub mod expression;
pub mod constraint;
pub mod constraint_set;
```

```rust
// crates/cp-ast-core/src/constraint/expected_type.rs
// Implemented in Task 14
```

```rust
// crates/cp-ast-core/src/constraint/expression.rs
// Implemented in Task 14
```

```rust
// crates/cp-ast-core/src/constraint/constraint.rs
// Implemented in Task 15
```

```rust
// crates/cp-ast-core/src/constraint/constraint_set.rs
// Implemented in Task 15
```

> **Important:** Each implementation task (11-15) must add `pub use` re-exports to the corresponding `mod.rs` after implementing the types. For example, after implementing `NodeId` in Task 11, add `pub use node_id::NodeId;` to `structure/mod.rs`.

- [ ] **Step 4: Verify project compiles**

Run: `cargo build`
Expected: Successful build with no errors (warnings OK at this stage)

- [ ] **Step 5: Create GitHub Actions CI**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [master, main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Format check
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      - name: Build
        run: cargo build --all-targets
      - name: Test
        run: cargo test --all-targets
```

- [ ] **Step 6: Create pre-commit hook**

```bash
#!/bin/sh
# .githooks/pre-commit

set -e

echo "Running cargo fmt check..."
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
    echo "Format check failed. Run 'cargo fmt' to fix."
    exit 1
fi

echo "Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "Clippy check failed. Fix warnings before committing."
    exit 1
fi

echo "Pre-commit checks passed."
```

Make executable: `chmod +x .githooks/pre-commit`
Configure git: `git config core.hooksPath .githooks`

- [ ] **Step 7: Update AGENTS.md**

```markdown
# Development Conventions

## Language
- Rust 2021 edition, minimum version 1.75

## Build & Test
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --all`
- Format check: `cargo fmt --all -- --check`

## Conventions
- All public types derive `Debug, Clone`
- Use `#[must_use]` on functions returning values
- Prefer `Result<T, E>` over panics
- No `unwrap()` in library code (tests OK)
- Follow clippy pedantic suggestions where reasonable

## Project Structure
- `crates/cp-ast-core/` — Core AST types (StructureAST + ConstraintAST)
- `doc/` — Design documents and references
- `docs/superpowers/` — Specs and plans

## Git
- Pre-commit hooks: `.githooks/pre-commit` (fmt + clippy)
- Configure: `git config core.hooksPath .githooks`
```

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml crates/ .github/ .githooks/ AGENTS.md
git commit -m "chore: initialize Rust workspace with CI and hooks

- Cargo workspace with cp-ast-core crate
- GitHub Actions CI (build, test, clippy, fmt)
- Pre-commit hooks (fmt + clippy)
- AGENTS.md with development conventions

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint 5: Rust Minimal Core Implementation

> **Note:** The exact types below are based on the spec draft. If Sprint 2 (Task 3) produced different type definitions in `doc/design/domain-model.md`, use those instead. The structure and test approach remain the same.

### Task 11: StructureAST — NodeId

**Files:**
- Modify: `crates/cp-ast-core/src/structure/node_id.rs`
- Create: `crates/cp-ast-core/tests/structure_basic.rs`

- [ ] **Step 1: Write failing test for NodeId**

```rust
// crates/cp-ast-core/tests/structure_basic.rs
use cp_ast_core::structure::NodeId;

#[test]
fn node_id_unique() {
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    assert_ne!(id1, id2);
}

#[test]
fn node_id_copy_equality() {
    let id = NodeId::new();
    let id_copy = id;
    assert_eq!(id, id_copy);
}

#[test]
fn node_id_debug_format() {
    let id = NodeId::new();
    let debug = format!("{id:?}");
    assert!(debug.contains("NodeId"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test structure_basic -- --nocapture`
Expected: Compilation error — `NodeId` has no `new()` method

- [ ] **Step 3: Implement NodeId**

```rust
// crates/cp-ast-core/src/structure/node_id.rs
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Stable, unique identifier for AST nodes.
///
/// Each `NodeId` is globally unique within a process lifetime.
/// Used for node identification, reference resolution, and diff comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    /// Create a new unique `NodeId`.
    #[must_use]
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the raw numeric value of this ID.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test structure_basic -- --nocapture`
Expected: All 3 tests pass

- [ ] **Step 5: Add re-export to structure/mod.rs**

Add to `crates/cp-ast-core/src/structure/mod.rs`:

```rust
pub use node_id::NodeId;
```

- [ ] **Step 6: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`
Expected: No errors

- [ ] **Step 7: Commit**

```bash
git add crates/cp-ast-core/src/structure/node_id.rs crates/cp-ast-core/src/structure/mod.rs crates/cp-ast-core/tests/structure_basic.rs
git commit -m "feat(structure): implement NodeId with auto-increment

Globally unique, Copy+Eq+Hash node identifier using atomic counter.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 12: StructureAST — NodeKind

**Files:**
- Modify: `crates/cp-ast-core/src/structure/node_kind.rs`
- Modify: `crates/cp-ast-core/tests/structure_basic.rs`

- [ ] **Step 1: Write failing test for NodeKind**

Append to `crates/cp-ast-core/tests/structure_basic.rs`:

```rust
use cp_ast_core::structure::NodeKind;

#[test]
fn node_kind_equality() {
    assert_eq!(NodeKind::Scalar, NodeKind::Scalar);
    assert_ne!(NodeKind::Scalar, NodeKind::Array);
}

#[test]
fn node_kind_clone() {
    let kind = NodeKind::Array;
    let cloned = kind.clone();
    assert_eq!(kind, cloned);
}

#[test]
fn node_kind_all_variants_exist() {
    // Verify all expected variants compile
    let _variants = [
        NodeKind::Scalar,
        NodeKind::Array,
        NodeKind::Matrix,
        NodeKind::MultiTestCase,
        NodeKind::Query,
        NodeKind::InputBlock,
        NodeKind::OutputBlock,
    ];
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test structure_basic -- --nocapture`
Expected: Compilation error — `NodeKind` not defined

- [ ] **Step 3: Implement NodeKind**

```rust
// crates/cp-ast-core/src/structure/node_kind.rs

/// The kind of structure node in a competitive programming problem specification.
///
/// Each variant represents a structural concept recognized by competitive
/// programming problem authors and readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// A single value (e.g., N, M, K)
    Scalar,
    /// A one-dimensional sequence (e.g., A_1, A_2, ..., A_N)
    Array,
    /// A two-dimensional grid (e.g., H×W matrix)
    Matrix,
    /// A wrapper for T test cases
    MultiTestCase,
    /// A query structure with type-dependent sub-formats
    Query,
    /// A block describing input format
    InputBlock,
    /// A block describing output format
    OutputBlock,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test structure_basic -- --nocapture`
Expected: All tests pass

- [ ] **Step 5: Add re-export to structure/mod.rs**

Add to `crates/cp-ast-core/src/structure/mod.rs`:

```rust
pub use node_kind::NodeKind;
```

- [ ] **Step 6: Commit**

```bash
git add crates/cp-ast-core/src/structure/node_kind.rs crates/cp-ast-core/src/structure/mod.rs crates/cp-ast-core/tests/structure_basic.rs
git commit -m "feat(structure): implement NodeKind enum

7 variants covering core competitive programming structural concepts.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 13: StructureAST — Slot, SlotValue, HoleInfo, StructureNode

**Files:**
- Modify: `crates/cp-ast-core/src/structure/slot.rs`
- Modify: `crates/cp-ast-core/src/structure/structure_node.rs`
- Modify: `crates/cp-ast-core/tests/structure_basic.rs`
- Create: `crates/cp-ast-core/tests/hole_basic.rs`

- [ ] **Step 1: Write failing tests for Slot and StructureNode**

Append to `crates/cp-ast-core/tests/structure_basic.rs`:

```rust
use cp_ast_core::structure::{Slot, SlotValue, StructureNode, HoleInfo};

#[test]
fn structure_node_with_filled_slots() {
    let child = StructureNode::new(NodeKind::Scalar)
        .with_name("N");

    let parent = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::filled("first_var", child));

    assert_eq!(parent.kind(), NodeKind::InputBlock);
    assert_eq!(parent.slots().len(), 1);
    assert_eq!(parent.slots()[0].name(), "first_var");
    assert!(parent.slots()[0].is_filled());
}

#[test]
fn structure_node_with_hole_slot() {
    let node = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::hole("missing_var"));

    assert_eq!(node.slots().len(), 1);
    assert!(node.slots()[0].is_hole());
}

#[test]
fn structure_node_name() {
    let node = StructureNode::new(NodeKind::Scalar).with_name("N");
    assert_eq!(node.name(), Some("N"));

    let unnamed = StructureNode::new(NodeKind::InputBlock);
    assert_eq!(unnamed.name(), None);
}
```

- [ ] **Step 2: Write failing tests for holes**

```rust
// crates/cp-ast-core/tests/hole_basic.rs
use cp_ast_core::structure::{NodeId, NodeKind, StructureNode, Slot, SlotValue, HoleInfo};

#[test]
fn hole_has_unique_id() {
    let slot1 = Slot::hole("a");
    let slot2 = Slot::hole("b");

    let id1 = match slot1.value() {
        SlotValue::Hole(ref h) => h.id(),
        _ => panic!("expected hole"),
    };
    let id2 = match slot2.value() {
        SlotValue::Hole(ref h) => h.id(),
        _ => panic!("expected hole"),
    };
    assert_ne!(id1, id2);
}

#[test]
fn collect_holes_from_tree() {
    let child = StructureNode::new(NodeKind::Scalar).with_name("N");
    let node = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::filled("defined", child))
        .with_slot(Slot::hole("undefined"));

    let holes: Vec<_> = node.slots().iter().filter(|s| s.is_hole()).collect();
    assert_eq!(holes.len(), 1);
    assert_eq!(holes[0].name(), "undefined");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -- --nocapture`
Expected: Compilation errors — `Slot`, `SlotValue`, `HoleInfo`, `StructureNode` not defined

- [ ] **Step 4: Implement Slot and SlotValue**

```rust
// crates/cp-ast-core/src/structure/slot.rs
use super::node_id::NodeId;
use super::structure_node::StructureNode;

/// Information about an unfilled position in the AST.
///
/// A hole represents a position that has not yet been filled.
/// The semantic expectations for this position (expected type, allowed
/// node kinds) are held by the ConstraintAST, not here.
#[derive(Debug, Clone)]
pub struct HoleInfo {
    id: NodeId,
}

impl HoleInfo {
    /// Create a new hole with a unique ID.
    #[must_use]
    pub fn new() -> Self {
        Self { id: NodeId::new() }
    }

    /// Returns the unique ID of this hole.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }
}

impl Default for HoleInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// The value held by a slot: either a filled node or a hole.
#[derive(Debug, Clone)]
pub enum SlotValue {
    /// A filled position containing a structure node.
    Filled(StructureNode),
    /// An unfilled position (hole).
    Hole(HoleInfo),
}

/// A named position within a structure node.
///
/// Slots give semantic meaning to parent-child relationships.
/// For example, an `Array` node might have slots "element_type" and "length".
#[derive(Debug, Clone)]
pub struct Slot {
    name: String,
    value: SlotValue,
}

impl Slot {
    /// Create a slot filled with a structure node.
    #[must_use]
    pub fn filled(name: &str, node: StructureNode) -> Self {
        Self {
            name: name.to_owned(),
            value: SlotValue::Filled(node),
        }
    }

    /// Create a slot with a hole (unfilled position).
    #[must_use]
    pub fn hole(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: SlotValue::Hole(HoleInfo::new()),
        }
    }

    /// Returns the name of this slot.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the slot's value.
    #[must_use]
    pub fn value(&self) -> &SlotValue {
        &self.value
    }

    /// Returns `true` if this slot is filled.
    #[must_use]
    pub fn is_filled(&self) -> bool {
        matches!(self.value, SlotValue::Filled(_))
    }

    /// Returns `true` if this slot is a hole.
    #[must_use]
    pub fn is_hole(&self) -> bool {
        matches!(self.value, SlotValue::Hole(_))
    }
}
```

- [ ] **Step 5: Implement StructureNode**

```rust
// crates/cp-ast-core/src/structure/structure_node.rs
use super::node_id::NodeId;
use super::node_kind::NodeKind;
use super::slot::Slot;

/// A node in the structure AST representing a part of a problem specification.
///
/// Each node has a unique ID, a kind, an optional name, and zero or more
/// named slots that hold child nodes or holes.
#[derive(Debug, Clone)]
pub struct StructureNode {
    id: NodeId,
    kind: NodeKind,
    name: Option<String>,
    slots: Vec<Slot>,
}

impl StructureNode {
    /// Create a new structure node of the given kind with a unique ID.
    #[must_use]
    pub fn new(kind: NodeKind) -> Self {
        Self {
            id: NodeId::new(),
            kind,
            name: None,
            slots: Vec::new(),
        }
    }

    /// Set the name of this node (builder pattern).
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Add a slot to this node (builder pattern).
    #[must_use]
    pub fn with_slot(mut self, slot: Slot) -> Self {
        self.slots.push(slot);
        self
    }

    /// Returns the unique ID of this node.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Returns the kind of this node.
    #[must_use]
    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns the name of this node, if set.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns a slice of this node's slots.
    #[must_use]
    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -- --nocapture`
Expected: All tests pass (structure_basic + hole_basic)

- [ ] **Step 7: Add re-exports to structure/mod.rs**

Update `crates/cp-ast-core/src/structure/mod.rs` to add:

```rust
pub use slot::{HoleInfo, Slot, SlotValue};
pub use structure_node::StructureNode;
```

- [ ] **Step 8: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`
Expected: No errors

- [ ] **Step 9: Commit**

```bash
git add crates/cp-ast-core/src/structure/
git add crates/cp-ast-core/tests/structure_basic.rs
git add crates/cp-ast-core/tests/hole_basic.rs
git commit -m "feat(structure): implement Slot, SlotValue, HoleInfo, StructureNode

Builder-pattern StructureNode with named slots supporting
filled nodes and holes. Holes carry unique IDs.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 14: ConstraintAST — ExpectedType and Expression

**Files:**
- Modify: `crates/cp-ast-core/src/constraint/expected_type.rs`
- Modify: `crates/cp-ast-core/src/constraint/expression.rs`
- Create: `crates/cp-ast-core/tests/constraint_basic.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/cp-ast-core/tests/constraint_basic.rs
use cp_ast_core::constraint::{ExpectedType, Expression};

#[test]
fn expected_type_equality() {
    assert_eq!(ExpectedType::Int, ExpectedType::Int);
    assert_ne!(ExpectedType::Int, ExpectedType::String);
}

#[test]
fn expected_type_array_of_int() {
    let arr = ExpectedType::Array(Box::new(ExpectedType::Int));
    assert_eq!(arr, ExpectedType::Array(Box::new(ExpectedType::Int)));
    assert_ne!(arr, ExpectedType::Array(Box::new(ExpectedType::String)));
}

#[test]
fn expression_literal() {
    let expr = Expression::Literal(42);
    assert_eq!(expr, Expression::Literal(42));
}

#[test]
fn expression_power() {
    // 10^9
    let expr = Expression::Power(10, 9);
    assert_eq!(expr, Expression::Power(10, 9));
}

#[test]
fn expression_mul() {
    // 2 * 10^5
    let expr = Expression::Mul(
        Box::new(Expression::Literal(2)),
        Box::new(Expression::Power(10, 5)),
    );
    assert!(matches!(expr, Expression::Mul(_, _)));
}

#[test]
fn expression_evaluate_literal() {
    let expr = Expression::Literal(42);
    assert_eq!(expr.evaluate_constant(), Some(42));
}

#[test]
fn expression_evaluate_power() {
    let expr = Expression::Power(10, 9);
    assert_eq!(expr.evaluate_constant(), Some(1_000_000_000));
}

#[test]
fn expression_evaluate_mul() {
    // 2 * 10^5 = 200_000
    let expr = Expression::Mul(
        Box::new(Expression::Literal(2)),
        Box::new(Expression::Power(10, 5)),
    );
    assert_eq!(expr.evaluate_constant(), Some(200_000));
}

#[test]
fn expression_evaluate_ref_returns_none() {
    use cp_ast_core::structure::NodeId;
    let expr = Expression::Ref(NodeId::new());
    assert_eq!(expr.evaluate_constant(), None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test constraint_basic -- --nocapture`
Expected: Compilation error — types not defined

- [ ] **Step 3: Implement ExpectedType**

```rust
// crates/cp-ast-core/src/constraint/expected_type.rs

/// The expected type for a position in the structure AST.
///
/// This represents what kind of value a hole or slot should accept.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedType {
    /// Integer value (e.g., N, M)
    Int,
    /// String value
    String,
    /// Floating-point value
    Float,
    /// Array of elements with a given type (e.g., A_1..A_N of Int)
    Array(Box<ExpectedType>),
    /// Tuple of heterogeneous types
    Tuple(Vec<ExpectedType>),
}
```

- [ ] **Step 4: Implement Expression**

```rust
// crates/cp-ast-core/src/constraint/expression.rs
use crate::structure::NodeId;

/// An expression used in constraint bounds.
///
/// Expressions represent values that appear in constraints like
/// `1 <= N <= 2 * 10^5`. They can be literal constants, references
/// to other nodes (variables), or arithmetic combinations.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal integer value (e.g., `1`, `42`)
    Literal(i64),
    /// A power expression (base^exp), e.g., `10^9`
    Power(i64, u32),
    /// A reference to another node's value (e.g., `N`)
    Ref(NodeId),
    /// Multiplication of two expressions (e.g., `2 * 10^5`)
    Mul(Box<Expression>, Box<Expression>),
}

impl Expression {
    /// Evaluate this expression to a constant value, if possible.
    ///
    /// Returns `None` if the expression contains node references
    /// that cannot be resolved without runtime context.
    #[must_use]
    pub fn evaluate_constant(&self) -> Option<i64> {
        match self {
            Self::Literal(v) => Some(*v),
            Self::Power(base, exp) => Some(base.pow(*exp)),
            Self::Ref(_) => None,
            Self::Mul(lhs, rhs) => {
                let l = lhs.evaluate_constant()?;
                let r = rhs.evaluate_constant()?;
                Some(l * r)
            }
        }
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test constraint_basic -- --nocapture`
Expected: All 9 tests pass

- [ ] **Step 6: Add re-exports to constraint/mod.rs**

Update `crates/cp-ast-core/src/constraint/mod.rs` to add:

```rust
pub use expected_type::ExpectedType;
pub use expression::Expression;
```

- [ ] **Step 7: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`
Expected: No errors

- [ ] **Step 8: Commit**

```bash
git add crates/cp-ast-core/src/constraint/expected_type.rs
git add crates/cp-ast-core/src/constraint/expression.rs
git add crates/cp-ast-core/src/constraint/mod.rs
git add crates/cp-ast-core/tests/constraint_basic.rs
git commit -m "feat(constraint): implement ExpectedType and Expression

ExpectedType enum (Int/String/Float/Array/Tuple).
Expression with Literal/Power/Ref/Mul and constant evaluation.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 15: ConstraintAST — Constraint Variants and ConstraintSet

**Files:**
- Modify: `crates/cp-ast-core/src/constraint/constraint.rs`
- Modify: `crates/cp-ast-core/src/constraint/constraint_set.rs`
- Modify: `crates/cp-ast-core/tests/constraint_basic.rs`

- [ ] **Step 1: Write failing tests for Constraint and ConstraintSet**

Append to `crates/cp-ast-core/tests/constraint_basic.rs`:

```rust
use cp_ast_core::constraint::{Constraint, ConstraintSet};
use cp_ast_core::structure::NodeId;

#[test]
fn range_constraint_creation() {
    let target = NodeId::new();
    let c = Constraint::range(target, Expression::Literal(1), Expression::Power(10, 5));

    assert!(matches!(c, Constraint::Range { .. }));
}

#[test]
fn length_constraint_creation() {
    let array_id = NodeId::new();
    let length_id = NodeId::new();
    let c = Constraint::length(array_id, length_id);

    assert!(matches!(c, Constraint::Length { .. }));
}

#[test]
fn element_constraint_creation() {
    let array_id = NodeId::new();
    let element_c = Constraint::range(NodeId::new(), Expression::Literal(0), Expression::Power(10, 9));
    let c = Constraint::element(array_id, element_c);

    assert!(matches!(c, Constraint::Element { .. }));
}

#[test]
fn type_constraint_creation() {
    let target = NodeId::new();
    let c = Constraint::expected_type(target, ExpectedType::Int);

    assert!(matches!(c, Constraint::Type { .. }));
}

#[test]
fn constraint_set_empty() {
    let set = ConstraintSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn constraint_set_add_and_query() {
    let n_id = NodeId::new();
    let a_id = NodeId::new();

    let mut set = ConstraintSet::new();
    set.add(Constraint::range(n_id, Expression::Literal(1), Expression::Power(10, 5)));
    set.add(Constraint::expected_type(n_id, ExpectedType::Int));
    set.add(Constraint::length(a_id, n_id));

    assert_eq!(set.len(), 3);

    let n_constraints: Vec<_> = set.for_target(n_id).collect();
    assert_eq!(n_constraints.len(), 2);

    let a_constraints: Vec<_> = set.for_target(a_id).collect();
    assert_eq!(a_constraints.len(), 1);
}

#[test]
fn constraint_set_no_constraints_for_unknown_node() {
    let set = ConstraintSet::new();
    let unknown = NodeId::new();
    let results: Vec<_> = set.for_target(unknown).collect();
    assert!(results.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test constraint_basic -- --nocapture`
Expected: Compilation error — `Constraint`, `ConstraintSet` not defined

- [ ] **Step 3: Implement Constraint**

```rust
// crates/cp-ast-core/src/constraint/constraint.rs
use crate::structure::NodeId;
use super::expected_type::ExpectedType;
use super::expression::Expression;

/// A constraint on a position in the structure AST.
///
/// Constraints define what is allowed at each position. They are used for
/// validation, candidate enumeration, and sample case generation.
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Value range constraint (e.g., `1 <= N <= 2*10^5`)
    Range {
        target: NodeId,
        min: Expression,
        max: Expression,
    },
    /// Array length tied to another variable (e.g., `len(A) = N`)
    Length {
        array: NodeId,
        length_ref: NodeId,
    },
    /// Per-element constraint on an array (e.g., `0 <= A[i] <= 10^9`)
    Element {
        array: NodeId,
        element_constraint: Box<Constraint>,
    },
    /// Expected type for a position (e.g., `N: Int`)
    Type {
        target: NodeId,
        expected: ExpectedType,
    },
}

impl Constraint {
    /// Create a range constraint.
    #[must_use]
    pub fn range(target: NodeId, min: Expression, max: Expression) -> Self {
        Self::Range { target, min, max }
    }

    /// Create a length constraint.
    #[must_use]
    pub fn length(array: NodeId, length_ref: NodeId) -> Self {
        Self::Length { array, length_ref }
    }

    /// Create an element constraint.
    #[must_use]
    pub fn element(array: NodeId, element_constraint: Constraint) -> Self {
        Self::Element {
            array,
            element_constraint: Box::new(element_constraint),
        }
    }

    /// Create a type constraint.
    #[must_use]
    pub fn expected_type(target: NodeId, expected: ExpectedType) -> Self {
        Self::Type { target, expected }
    }

    /// Returns the primary target NodeId of this constraint.
    #[must_use]
    pub fn target(&self) -> NodeId {
        match self {
            Self::Range { target, .. }
            | Self::Type { target, .. } => *target,
            Self::Length { array, .. }
            | Self::Element { array, .. } => *array,
        }
    }
}
```

- [ ] **Step 4: Implement ConstraintSet**

```rust
// crates/cp-ast-core/src/constraint/constraint_set.rs
use crate::structure::NodeId;
use super::constraint::Constraint;

/// A composable set of constraints on the structure AST.
///
/// Constraints can be queried by target node to determine what
/// is allowed at each position.
#[derive(Debug, Clone, Default)]
pub struct ConstraintSet {
    constraints: Vec<Constraint>,
}

impl ConstraintSet {
    /// Create an empty constraint set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a constraint to the set.
    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Returns the number of constraints in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    /// Returns `true` if the constraint set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    /// Returns an iterator over constraints targeting the given node.
    pub fn for_target(&self, target: NodeId) -> impl Iterator<Item = &Constraint> {
        self.constraints.iter().filter(move |c| c.target() == target)
    }

    /// Returns an iterator over all constraints.
    pub fn iter(&self) -> impl Iterator<Item = &Constraint> {
        self.constraints.iter()
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test constraint_basic -- --nocapture`
Expected: All tests pass

- [ ] **Step 6: Add re-exports to constraint/mod.rs**

Update `crates/cp-ast-core/src/constraint/mod.rs` to add:

```rust
pub use constraint::Constraint;
pub use constraint_set::ConstraintSet;
```

- [ ] **Step 7: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`
Expected: No errors

- [ ] **Step 8: Commit**

```bash
git add crates/cp-ast-core/src/constraint/constraint.rs
git add crates/cp-ast-core/src/constraint/constraint_set.rs
git add crates/cp-ast-core/src/constraint/mod.rs
git add crates/cp-ast-core/tests/constraint_basic.rs
git commit -m "feat(constraint): implement Constraint variants and ConstraintSet

Range, Length, Element, Type constraint variants.
ConstraintSet with add/query-by-target operations.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 16: Integration Test — Express Typical ABC Problem as AST

**Files:**
- Create: `crates/cp-ast-core/tests/typical_problem.rs`

- [ ] **Step 1: Write integration test**

This test expresses a typical AtCoder ABC problem:

```
Input:
N
A_1 A_2 ... A_N

Constraints:
1 <= N <= 2 * 10^5
0 <= A_i <= 10^9
```

```rust
// crates/cp-ast-core/tests/typical_problem.rs
//! Integration test: express a typical AtCoder ABC problem as AST.
//!
//! Problem: Given N and array A of length N, (typical ABC B-C level).
//! Input format:
//!   N
//!   A_1 A_2 ... A_N
//! Constraints:
//!   1 <= N <= 2 * 10^5
//!   0 <= A_i <= 10^9

use cp_ast_core::structure::{NodeId, NodeKind, StructureNode, Slot};
use cp_ast_core::constraint::{
    Constraint, ConstraintSet, ExpectedType, Expression,
};

#[test]
fn express_n_plus_array_problem() {
    // --- Build StructureAST ---

    // Scalar variable N
    let n_node = StructureNode::new(NodeKind::Scalar).with_name("N");
    let n_id = n_node.id();

    // Array variable A
    let a_node = StructureNode::new(NodeKind::Array).with_name("A");
    let a_id = a_node.id();

    // Input block containing N then A
    let input = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::filled("line_1", n_node))
        .with_slot(Slot::filled("line_2", a_node));

    // Verify structure
    assert_eq!(input.kind(), NodeKind::InputBlock);
    assert_eq!(input.slots().len(), 2);
    assert_eq!(input.slots()[0].name(), "line_1");
    assert_eq!(input.slots()[1].name(), "line_2");

    // --- Build ConstraintAST ---

    let mut constraints = ConstraintSet::new();

    // N: Int
    constraints.add(Constraint::expected_type(n_id, ExpectedType::Int));

    // 1 <= N <= 2 * 10^5
    constraints.add(Constraint::range(
        n_id,
        Expression::Literal(1),
        Expression::Mul(
            Box::new(Expression::Literal(2)),
            Box::new(Expression::Power(10, 5)),
        ),
    ));

    // A: Array(Int)
    constraints.add(Constraint::expected_type(
        a_id,
        ExpectedType::Array(Box::new(ExpectedType::Int)),
    ));

    // len(A) = N
    constraints.add(Constraint::length(a_id, n_id));

    // 0 <= A[i] <= 10^9
    constraints.add(Constraint::element(
        a_id,
        Constraint::range(
            a_id, // element constraint references the array
            Expression::Literal(0),
            Expression::Power(10, 9),
        ),
    ));

    // Verify constraints
    assert_eq!(constraints.len(), 5);

    // N has 2 constraints (type + range)
    let n_constraints: Vec<_> = constraints.for_target(n_id).collect();
    assert_eq!(n_constraints.len(), 2);

    // A has 3 constraints (type + length + element)
    let a_constraints: Vec<_> = constraints.for_target(a_id).collect();
    assert_eq!(a_constraints.len(), 3);

    // Verify range upper bound evaluates correctly
    let range_upper = Expression::Mul(
        Box::new(Expression::Literal(2)),
        Box::new(Expression::Power(10, 5)),
    );
    assert_eq!(range_upper.evaluate_constant(), Some(200_000));
}

#[test]
fn express_problem_with_hole() {
    // A problem being built incrementally:
    // InputBlock with N defined but second variable still a hole

    let n_node = StructureNode::new(NodeKind::Scalar).with_name("N");

    let input = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::filled("line_1", n_node))
        .with_slot(Slot::hole("line_2")); // Not yet defined

    assert_eq!(input.slots().len(), 2);
    assert!(input.slots()[0].is_filled());
    assert!(input.slots()[1].is_hole());
}
```

- [ ] **Step 2: Run integration test**

Run: `cargo test --test typical_problem -- --nocapture`
Expected: Both tests pass

- [ ] **Step 3: Run full test suite**

Run: `cargo test -- --nocapture`
Expected: All tests across all test files pass

- [ ] **Step 4: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add crates/cp-ast-core/tests/typical_problem.rs
git commit -m "test: integration test expressing ABC problem as AST

Demonstrates N + Array A problem with constraints:
1<=N<=2*10^5, 0<=A[i]<=10^9, len(A)=N.
Also tests incremental building with holes.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Summary

| Sprint | Tasks | Parallelism | Output |
|--------|-------|-------------|--------|
| 1 | Task 1-2 | Parallel | Survey + research reports |
| 2 | Task 3-4 | Parallel | Domain model + API design |
| 3 | Task 5-7 + 8 | 5-6-7 parallel, 8 sequential | Reviews + synthesis |
| 4 | Task 9-10 | Parallel | Final doc + Rust env |
| 5 | Task 11-16 | Sequential (TDD) | Rust minimal core |

**Total tasks:** 16
**Parallel groups:** Tasks 1-2, Tasks 3-4, Tasks 5-6-7, Tasks 9-10
**Sequential dependencies:** Sprint boundaries, TDD chain in Sprint 5
