# サブプロジェクトA: AtCoder調査 + E2Eテスト拡充 実装計画

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** AtCoder ABC 直近50問の入力形式を調査し、既存5カテゴリ（22テスト）でカバーしきれない汎用パターンを特定、必要なE2Eテストを追加する。

**Architecture:** サブエージェントがAtCoder問題ページから入力形式を抽出→パターン分類→既存テストとのギャップ分析→新specファイルとして追加。既存22テスト・POMメソッドは一切変更しない。

**Tech Stack:** Playwright (E2E), TypeScript, web_fetch (AtCoder scraping)

---

## ファイル構成

### 既存ファイル（変更禁止）
- `web/tests/e2e/basic-array.spec.ts` — 5テスト
- `web/tests/e2e/grid.spec.ts` — 4テスト
- `web/tests/e2e/tree.spec.ts` — 4テスト
- `web/tests/e2e/query.spec.ts` — 4テスト
- `web/tests/e2e/multi-testcase.spec.ts` — 5テスト
- `web/tests/e2e/fixtures/editor-page.ts` — 既存メソッド変更禁止（新メソッド追加のみ可）
- `web/tests/e2e/fixtures/helpers.ts` — 変更禁止

### 変更候補ファイル
- `web/tests/e2e/fixtures/editor-page.ts` — 新メソッド追加（末尾にのみ追加）
- `web/tests/e2e/README.md` — 新testid追記（既存削除なし）

### 新規作成候補ファイル（調査結果に依存）
- `web/tests/e2e/<new-pattern>.spec.ts` — 新パターンごとに1ファイル

---

## Task 1: AtCoder ABC 問題収集 + パターン分類

**目的:** 直近のABC問題から約50問の入力形式を取得し、パターンに分類する。

**対象:** ABC400〜ABC410 の全問題（A〜G、各回7問 × 11回 = 77問からサンプリング）

- [ ] **Step 1: 問題リストを作成してSQLに格納**

AtCoder の問題ページURLパターン: `https://atcoder.jp/contests/abc{NNN}/tasks/abc{NNN}_{a-g}`

以下のSQLテーブルを作成し、対象問題を登録する:

```sql
CREATE TABLE atcoder_problems (
  id TEXT PRIMARY KEY,          -- e.g., 'abc400_a'
  contest TEXT NOT NULL,        -- e.g., 'abc400'
  problem_letter TEXT NOT NULL, -- e.g., 'a'
  url TEXT NOT NULL,
  input_format TEXT,            -- 取得した入力形式テキスト
  constraints TEXT,             -- 取得した制約テキスト
  pattern TEXT,                 -- 分類結果
  notes TEXT,                   -- 備考
  status TEXT DEFAULT 'pending' -- pending / fetched / classified / error
);
```

ABC400〜ABC410 の A〜G 問題（存在しないものはスキップ）を INSERT する。

- [ ] **Step 2: 各問題の入力形式を取得**

`web_fetch` ツールで各問題ページにアクセスし、「入力」セクションと「制約」セクションを抽出する。

各問題について:
1. `https://atcoder.jp/contests/{contest}/tasks/{id}` を取得
2. `## 入力` または `Input` セクション（preformatted block）を抽出
3. `## 制約` または `Constraints` セクションを抽出
4. SQLの `input_format` と `constraints` カラムを UPDATE
5. `status` を `fetched` に更新

並列実行可能: 独立した問題を複数エージェントで同時取得してよい。

- [ ] **Step 3: パターン分類**

取得した各問題の入力形式を以下のカテゴリに分類し、SQL `pattern` カラムを更新:

| パターン名 | 判定基準 |
|---|---|
| `scalar-array` | N + A_1...A_N 形式 |
| `grid` | H W + S_1...S_H 形式（文字グリッド） |
| `tree` | N + 辺リスト (u_i v_i × N-1) |
| `query` | N Q + クエリ型分岐 |
| `multi-testcase` | T + ケース繰り返し |
| `scalar-only` | 単一スカラー or 複数スカラーのみ |
| `multi-array` | 複数の独立した配列 (A_1..A_N, B_1..B_N) |
| `weighted-edge` | 重み付き辺 (u_i v_i w_i × M) |
| `coordinate` | 座標ペア (x_i y_i × N) |
| `string-array` | 文字列配列 (S_1...S_N) |
| `graph-general` | 一般グラフ (N M + u_i v_i × M) |
| `matrix-numeric` | 数値行列 (H×W の数値グリッド) |
| `composite` | 上記の組み合わせ |
| `other` | 上記に該当しない |

分類後に `status` を `classified` に更新。

- [ ] **Step 4: 分類結果のサマリーを出力**

```sql
SELECT pattern, COUNT(*) as cnt FROM atcoder_problems
WHERE status = 'classified'
GROUP BY pattern ORDER BY cnt DESC;
```

結果をテキストで出力し、新パターン候補を特定する。

---

## Task 2: ギャップ分析

**目的:** 分類結果と既存E2Eテストを比較し、新規テストが必要なパターンを特定する。

- [ ] **Step 1: 既存カバレッジとの差分を確認**

既存テストがカバーしているパターン:
- `scalar-array` → basic-array.spec.ts ✓
- `grid` → grid.spec.ts ✓
- `tree` → tree.spec.ts ✓
- `query` → query.spec.ts ✓
- `multi-testcase` → multi-testcase.spec.ts ✓

カバーされていないパターンを抽出:

```sql
SELECT pattern, COUNT(*) as cnt, GROUP_CONCAT(id) as problems
FROM atcoder_problems
WHERE status = 'classified'
  AND pattern NOT IN ('scalar-array', 'grid', 'tree', 'query', 'multi-testcase', 'scalar-only')
GROUP BY pattern
HAVING cnt >= 3
ORDER BY cnt DESC;
```

- [ ] **Step 2: 新パターンの汎用性評価**

各新パターン候補について判定:
1. 3問以上で出現するか → Yes なら採用候補
2. 既存のNodeKind/API で表現可能か → No なら「専用API必要」として却下
3. 既存パターンの変種として `editor-page.ts` の既存メソッドで表現できるか → Yes なら新メソッド不要

判定結果をSQLに記録:

```sql
CREATE TABLE gap_analysis (
  pattern TEXT PRIMARY KEY,
  occurrence_count INTEGER,
  representative_problems TEXT,
  needs_new_spec INTEGER,      -- 1=yes, 0=no
  needs_new_pom_method INTEGER, -- 1=yes, 0=no
  rejection_reason TEXT,        -- NULL if accepted
  notes TEXT
);
```

- [ ] **Step 3: 新テスト仕様を決定**

`needs_new_spec = 1` のパターンについて、テスト仕様を作成する:
- spec ファイル名
- テストスイート名
- 必要なテスト数（最低2: 構造構築 + 完成状態）
- 使用する EditorPage メソッド（既存 or 新規）
- 対応する data-testid

結果をSQLに記録:

```sql
CREATE TABLE new_test_specs (
  id TEXT PRIMARY KEY,           -- e.g., 'weighted-edge'
  spec_filename TEXT NOT NULL,   -- e.g., 'weighted-edge.spec.ts'
  suite_name TEXT NOT NULL,      -- テストスイート名
  test_count INTEGER NOT NULL,
  new_pom_methods TEXT,          -- JSON array of new method names
  new_testids TEXT,              -- JSON array of new testids
  representative_problem TEXT,   -- 代表問題ID
  input_format_example TEXT,     -- 入力形式の例
  status TEXT DEFAULT 'pending'  -- pending / implemented / verified
);
```

---

## Task 3: 新E2Eテスト実装

**目的:** ギャップ分析で「必要」と判定されたパターンのE2Eテストを作成する。

**注意:** Task 2の結果に完全に依存する。Task 2で新テストが不要と判定された場合、このタスクはスキップ。

以下は、よく見つかると予想されるパターンのテンプレート。実際の調査結果に基づいて調整する。

- [ ] **Step 1: 新POMメソッドを追加（必要な場合のみ）**

`web/tests/e2e/fixtures/editor-page.ts` の末尾（`editMathValue` メソッドの後、クラス閉じ括弧の前）に新メソッドを追加。

例: 重み付き辺リスト用に `addEdgeListWeighted` が必要な場合:

```typescript
  /**
   * High-level helper: add a weighted edge list template.
   * Uses the edge-list template with an additional weight variable.
   */
  async addWeightedEdgeList(
    countVar: string,
    countOp: string,
    countOperand: string,
    weightName: string,
    weightType: string = 'number',
  ): Promise<void> {
    await this.clickHotspot('below');
    await this.selectPopupOption('weighted-edge-list');
    await this.buildCountExpression(countVar, countOp, countOperand);
    await this.inputName(weightName);
    await this.selectType(weightType);
    await this.confirm();
  }
```

**重要:** 既存メソッドは一切変更しない。新メソッドを末尾に追加するのみ。

- [ ] **Step 2: 新specファイルを作成**

各新パターンにつき1つのspecファイルを作成。

テンプレート構造（実際の内容はTask 2の結果に基づく）:

```typescript
/**
 * E2E Tests: [パターン名] ([入力形式の例])
 *
 * 対象: [代表問題]
 * ユーザーフロー: Task 2 ギャップ分析結果に基づく
 */
import { test, expect } from '@playwright/test';
import { EditorPage } from './fixtures/editor-page';
import {
  expectStructureContains,
  expectSampleLines,
  expectRightPanePopulated,
} from './fixtures/helpers';

test.describe('[パターン名]: [入力形式]', () => {
  let editor: EditorPage;

  test.beforeEach(async ({ page }) => {
    editor = new EditorPage(page);
    await editor.goto();
  });

  test('[構造構築テスト]', async () => {
    // Structure を構築
    // ...
    // 検証: Structure ペインに期待される変数が表示
    // 検証: draft constraint が適切な数だけ自動生成
  });

  test('[完成状態テスト]', async () => {
    // Structure を構築
    // 全制約を埋める
    // 検証: draft が全て消えている
    // 検証: completed が正しい数
    // 検証: 右ペイン三要素が揃っている
    await expectRightPanePopulated(editor);
    await expectSampleLines(editor, 2);
  });
});
```

- [ ] **Step 3: README.md に新testid を追記**

`web/tests/e2e/README.md` の適切なセクション末尾に新しいtestidを追記。既存行は削除しない。

- [ ] **Step 4: Playwright テスト検出を検証**

Run: `cd web && npx playwright test --config tests/e2e/playwright.config.ts --list`

Expected: 22 + N テスト（N は新規テスト数）が全て検出されること。

- [ ] **Step 5: コミット**

```bash
cd /home/mana/programs/cp-ast-ecosystems
git add web/tests/e2e/
git commit -m "feat(e2e): AtCoder調査に基づく新パターンE2Eテスト追加

パターン: [追加したパターン名リスト]
調査範囲: ABC400-410 (約50問)
新テスト: N件追加（既存22件は変更なし）

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 4: 最終検証

- [ ] **Step 1: 既存テストが変更されていないことを確認**

```bash
cd /home/mana/programs/cp-ast-ecosystems
git diff HEAD~1 -- web/tests/e2e/basic-array.spec.ts web/tests/e2e/grid.spec.ts web/tests/e2e/tree.spec.ts web/tests/e2e/query.spec.ts web/tests/e2e/multi-testcase.spec.ts
```

Expected: 差分が空であること。

- [ ] **Step 2: 全テストが検出可能であることを最終確認**

```bash
cd /home/mana/programs/cp-ast-ecosystems/web
npx playwright test --config tests/e2e/playwright.config.ts --list
```

Expected: 全テスト（既存22 + 新規N）が一覧表示されること。

- [ ] **Step 3: 調査結果のサマリーをユーザーに報告**

以下を含むサマリーを出力:
1. 調査した問題数と分類結果
2. 新パターンとして採用したもの / 却下したもの
3. 追加したE2Eテストの一覧
4. 後続サブプロジェクト B・C に影響する知見
