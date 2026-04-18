# E2E テスト設計計画（Phase 0 → 1 → 2）

## 概要

`doc/view/proccess.md` に従い、E2E テスト先行で競技プログラミング AST Editor の設計を固定する。
実問題を起点に、ユーザー操作順序を文書化し、それを Playwright E2E テストとして実装可能な設計に落とす。

**スコープ**: Phase 0（題材問題の固定）→ Phase 1（ユーザー操作順序の文章化）→ Phase 2（E2E テスト設計）

**現在の状態**:
- Rust core: Structure AST, Constraint AST, Operation (Action), Projection API, Render, TeX Render, Sample 生成が実装済み
- WASM 層: 読み取り専用 API のみ公開（render系 + preset）。Operation/Projection は未公開
- Web frontend: Preact + Vite。3ペイン Viewer（read-only）+ Preview ページ。編集機能なし
- E2E テスト: **ゼロ**。Playwright 未導入

---

## Phase 0: 題材問題の固定

### 目的
E2E テストの対象問題を実問題ベースで固定する。

### 対象カテゴリと代表問題

`doc/survey/atcoder-coverage-extended.md` の 31 問 + 標準的な競プロ入力パターンから選定。

#### MVP 対象（5カテゴリ、必須）

| # | カテゴリ | 代表問題 | 入力形式 | 制約例 |
|---|---------|---------|---------|-------|
| 1 | **基本配列** | ABC395-A 相当 | `N` / `A_1 ... A_N` | `1 ≤ N ≤ 10^6`, `1 ≤ A_i ≤ 10^9` |
| 2 | **グリッド** | ABC390-C 相当 | `H W` / `S_1` ... `S_H` | `1 ≤ H,W ≤ 500`, `S_i は英小文字` |
| 3 | **木入力** | 典型 ABC-D 相当 | `N` / `u_1 v_1` ... `u_{N-1} v_{N-1}` | `2 ≤ N ≤ 2×10^5`, `1 ≤ u_i,v_i ≤ N`, Tree |
| 4 | **クエリ列** | ABC395-D 相当 | `N Q` / 各クエリ `1 a b`/`2 a b`/`3 a` | `1 ≤ N,Q ≤ 2×10^5` |
| 5 | **複数テストケース** | 典型マルチケース | `T` / 各ケース `N` / `A_1...A_N` | `1 ≤ T ≤ 10^5`, `ΣN ≤ 2×10^5` |

#### 中級ケース（Phase 4 での追加対象）

| カテゴリ | 問題例 | ギャップ |
|---------|-------|---------|
| 複数独立配列 | ABC360-C | — |
| Repeat 内 Tuple | ABC390-E | — |
| 同一行2文字列 | ABC360-B | — |
| 下三角行列 | ABC370-B | Gap H |
| セクション | — | 要設計 |
| distinct/sorted | — | Constraint 操作 |

### 成果物
- `doc/view/coverage-scenario-index.md`

---

## Phase 1: ユーザー操作順序の文章化

### 目的
各 MVP 問題について、空状態から完成までの操作フローを固定する。

### 1問ごとに記述する内容（proccess.md 準拠）

1. 目標となる入力形式
2. 目標となる制約
3. 初期状態
4. ユーザーのクリック順序
5. 各ステップで開くポップアップ
6. 自動で生える draft constraint
7. 右ペインの期待表示
8. 最終的な完成条件

### カテゴリ別フロー概要

#### 1. 基本配列 (`N` / `A_1...A_N`)

```
初期状態: 空の Structure ペイン + 1つの insertion hotspot

Step 1: 初期 hotspot をクリック
  → ポップアップ: [scalar, 横配列, 縦配列, tuple, repeat, ...]
Step 2: scalar を選択 → number → 名前「N」を入力
  → Structure ペインに「N」が表示
  → draft constraint: `? ≤ N ≤ ?` が Constraint ペインに生える
  → 右ペイン: TeX入力「N」、sample「5」(仮値)
Step 3: N の下の hotspot をクリック
  → ポップアップ表示
Step 4: 横配列を選択 → number → 名前「A」→ 長さ「N」を選択
  → Structure ペインに「A_1 A_2 ... A_N」が表示
  → draft constraint: `? ≤ A_i ≤ ?` が生える
  → 右ペイン: TeX入力更新、sample 更新
Step 5: draft `? ≤ N ≤ ?` をクリック → 下限「1」上限「10^6」を入力
  → completed constraint に昇格
  → 右ペイン: TeX制約に「1 \le N \le 10^6」
Step 6: draft `? ≤ A_i ≤ ?` をクリック → 下限「1」上限「10^9」を入力
  → completed constraint に昇格

完成条件:
- Structure: N / A_1 ... A_N
- Constraints: 1 ≤ N ≤ 10^6, 1 ≤ A_i ≤ 10^9
- 右ペイン TeX/sample が正しく表示
```

#### 2. グリッド (`H W` / `S_1...S_H`)

```
Step 1: hotspot → tuple → [H, W] (number, number)
  → draft: ? ≤ H ≤ ?, ? ≤ W ≤ ?
Step 2: 下の hotspot → 「文字グリッド」テンプレート
  → 行数 = H, 長さ = W を選択
  → draft: |S_i| = W (自動), charset(S) = ?
Step 3: 制約を埋める: H,W の range, charset を英小文字に設定

完成条件:
- Structure: H W / S_1 / ... / S_H
- Constraints: range + |S_i| = W + charset
- 右ペイン: TeX グリッド表示 + sample grid 生成
```

#### 3. 木入力 (`N` / `u_1 v_1...u_{N-1} v_{N-1}`)

```
Step 1: hotspot → scalar → number → N
  → draft: ? ≤ N ≤ ?
Step 2: 下の hotspot → 「辺リスト」テンプレート
  → 本数 = N - 1 (式入力)
  → draft: 1 ≤ u_i ≤ N, 1 ≤ v_i ≤ N
Step 3: Property ショートカットから Tree を選択
  → constraint: graph property = Tree

完成条件:
- Structure: N / u_1 v_1 / ... / u_{N-1} v_{N-1}
- Constraints: range + tree property
- 右ペイン: TeX + sample tree
```

#### 4. クエリ列 (`N Q` / variant分岐)

```
Step 1: hotspot → tuple → [N, Q]
  → draft: range for N, Q
Step 2: 下の hotspot → 「クエリ列」テンプレート
  → count = Q
Step 3: variant を3つ追加:
  - variant 1: tag=1, body=[a, b]
  - variant 2: tag=2, body=[a, b]
  - variant 3: tag=3, body=[a]
Step 4: 各変数の制約を埋める

完成条件:
- Structure: N Q / query lines with Choice
- Constraints: range for all vars
- 右ペイン: TeX query表示
```

#### 5. 複数テストケース (`T` / 各ケース)

```
Step 1: hotspot → scalar → number → T (テストケース数)
Step 2: 「複数テストケース」テンプレートを適用
  → repeat over T cases
Step 3: ケース内で scalar N + 横配列 A を作成
Step 4: SumBound 追加: ΣN ≤ 2×10^5

完成条件:
- Structure: T / (N / A_1...A_N) × T
- Constraints: T range + N range + A range + sum bound
```

### 成果物
- `doc/view/problem-user-flows.md`

---

## Phase 2: E2E テスト設計

### 目的
Phase 1 のフローを Playwright E2E テストとして固定する。

### テスト基盤設計

#### ディレクトリ構成

```
web/
├── tests/
│   └── e2e/
│       ├── playwright.config.ts
│       ├── README.md
│       ├── fixtures/
│       │   ├── editor-page.ts      # Page Object Model
│       │   └── helpers.ts          # 共通ヘルパー
│       ├── basic-array.spec.ts     # カテゴリ 1
│       ├── grid.spec.ts            # カテゴリ 2
│       ├── tree.spec.ts            # カテゴリ 3
│       ├── query.spec.ts           # カテゴリ 4
│       └── multi-testcase.spec.ts  # カテゴリ 5
├── package.json                    # playwright devDependency 追加
└── ...
```

#### data-testid 設計

E2E テストが依存するセレクタの契約。UI 実装時にこれらの testid を付与する。

```
# Structure ペイン
data-testid="structure-pane"
data-testid="insertion-hotspot"          # 汎用 hotspot
data-testid="insertion-hotspot-below"    # 下に追加
data-testid="insertion-hotspot-right"    # 右に追加
data-testid="insertion-hotspot-inside"   # ブロック内に追加
data-testid="structure-line-{index}"     # 入力形式の各行
data-testid="structure-node-{nodeId}"    # 個別ノード

# ポップアップ / ウィザード
data-testid="node-popup"                 # ノード種別選択ポップアップ
data-testid="popup-option-scalar"
data-testid="popup-option-array"
data-testid="popup-option-grid-template"
data-testid="popup-option-edge-list"
data-testid="popup-option-query-list"
data-testid="popup-option-multi-testcase"
data-testid="popup-option-tuple"
data-testid="popup-option-repeat"
data-testid="name-input"                 # 変数名入力
data-testid="type-select"               # 型選択
data-testid="length-select"             # 長さ参照先選択
data-testid="confirm-button"            # 確定ボタン

# Constraint ペイン
data-testid="constraint-pane"
data-testid="draft-constraint-{index}"   # draft 制約
data-testid="completed-constraint-{index}" # 完成制約
data-testid="constraint-lower-input"     # Range 下限入力
data-testid="constraint-upper-input"     # Range 上限入力
data-testid="constraint-confirm"         # 制約確定
data-testid="property-shortcut"          # Property ショートカットボタン
data-testid="property-option-{name}"     # 個別 property 選択肢

# 右ペイン (Preview)
data-testid="preview-pane"
data-testid="tex-input-format"           # TeX 入力形式表示
data-testid="tex-constraints"            # TeX 制約表示
data-testid="sample-output"              # サンプルケース表示

# 数式編集
data-testid="math-editable-{id}"         # クリック可能な数式要素
data-testid="math-editor-input"          # 数式編集入力欄
data-testid="math-editor-confirm"        # 数式確定
```

#### Page Object Model (EditorPage)

```typescript
// web/tests/e2e/fixtures/editor-page.ts
import { Page, Locator } from '@playwright/test';

export class EditorPage {
  readonly page: Page;

  // Panes
  readonly structurePane: Locator;
  readonly constraintPane: Locator;
  readonly previewPane: Locator;

  // Structure operations
  readonly insertionHotspots: Locator;
  readonly nodePopup: Locator;

  constructor(page: Page) {
    this.page = page;
    this.structurePane = page.getByTestId('structure-pane');
    this.constraintPane = page.getByTestId('constraint-pane');
    this.previewPane = page.getByTestId('preview-pane');
    this.insertionHotspots = page.getByTestId(/^insertion-hotspot/);
    this.nodePopup = page.getByTestId('node-popup');
  }

  async goto() {
    await this.page.goto('/');
    // editor ページに遷移（将来的にルーティングが変わる可能性あり）
  }

  // --- Structure 操作 ---

  async clickHotspot(type: 'below' | 'right' | 'inside' = 'below') {
    await this.page.getByTestId(`insertion-hotspot-${type}`).first().click();
  }

  async selectPopupOption(option: string) {
    await this.nodePopup.waitFor({ state: 'visible' });
    await this.page.getByTestId(`popup-option-${option}`).click();
  }

  async inputName(name: string) {
    await this.page.getByTestId('name-input').fill(name);
  }

  async selectType(type: string) {
    await this.page.getByTestId('type-select').selectOption(type);
  }

  async selectLength(varName: string) {
    await this.page.getByTestId('length-select').selectOption(varName);
  }

  async confirm() {
    await this.page.getByTestId('confirm-button').click();
  }

  async addScalar(name: string, type: string = 'number') {
    await this.clickHotspot('below');
    await this.selectPopupOption('scalar');
    await this.selectType(type);
    await this.inputName(name);
    await this.confirm();
  }

  async addArray(name: string, lengthVar: string, type: string = 'number') {
    await this.clickHotspot('below');
    await this.selectPopupOption('array');
    await this.selectType(type);
    await this.inputName(name);
    await this.selectLength(lengthVar);
    await this.confirm();
  }

  // --- Constraint 操作 ---

  async getDraftConstraints(): Promise<Locator[]> {
    return this.constraintPane.getByTestId(/^draft-constraint/).all();
  }

  async fillDraftRange(index: number, lower: string, upper: string) {
    const draft = this.page.getByTestId(`draft-constraint-${index}`);
    await draft.click();
    await this.page.getByTestId('constraint-lower-input').fill(lower);
    await this.page.getByTestId('constraint-upper-input').fill(upper);
    await this.page.getByTestId('constraint-confirm').click();
  }

  async addProperty(propertyName: string) {
    await this.page.getByTestId('property-shortcut').click();
    await this.page.getByTestId(`property-option-${propertyName}`).click();
  }

  // --- 右ペイン検証 ---

  async getTexInputFormat(): Promise<string> {
    return this.page.getByTestId('tex-input-format').textContent() ?? '';
  }

  async getTexConstraints(): Promise<string> {
    return this.page.getByTestId('tex-constraints').textContent() ?? '';
  }

  async getSampleOutput(): Promise<string> {
    return this.page.getByTestId('sample-output').textContent() ?? '';
  }

  // --- 数式編集 ---

  async clickMathElement(id: string) {
    await this.page.getByTestId(`math-editable-${id}`).click();
  }

  async editMathValue(value: string) {
    await this.page.getByTestId('math-editor-input').fill(value);
    await this.page.getByTestId('math-editor-confirm').click();
  }
}
```

### テストシナリオ詳細

#### 1. basic-array.spec.ts

```typescript
import { test, expect } from '@playwright/test';
import { EditorPage } from './fixtures/editor-page';

test.describe('基本配列: N + A_1...A_N', () => {

  test('空状態から scalar N を追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    // 初期状態: hotspot が1つ見える
    await expect(editor.insertionHotspots.first()).toBeVisible();

    // scalar N を追加
    await editor.addScalar('N');

    // Structure ペインに N が表示
    await expect(editor.structurePane).toContainText('N');

    // draft constraint が自動生成
    const drafts = await editor.getDraftConstraints();
    expect(drafts.length).toBeGreaterThanOrEqual(1);

    // 右ペインに TeX 入力形式が表示
    const texInput = await editor.getTexInputFormat();
    expect(texInput).toContain('N');

    // sample が表示される
    const sample = await editor.getSampleOutput();
    expect(sample.length).toBeGreaterThan(0);
  });

  test('scalar N の後に横配列 A を追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    await editor.addScalar('N');
    await editor.addArray('A', 'N');

    // Structure ペインに配列が表示
    await expect(editor.structurePane).toContainText('A');

    // 配列用の draft constraint が追加で生成
    const drafts = await editor.getDraftConstraints();
    expect(drafts.length).toBeGreaterThanOrEqual(2); // N の range + A の range
  });

  test('draft constraint を埋めて completed に昇格する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    await editor.addScalar('N');
    await editor.addArray('A', 'N');

    // N の draft range を埋める
    await editor.fillDraftRange(0, '1', '10^6');

    // completed constraint が表示
    const completed = editor.page.getByTestId(/^completed-constraint/);
    await expect(completed.first()).toBeVisible();

    // 右ペインの TeX 制約に反映
    const texConstraints = await editor.getTexConstraints();
    expect(texConstraints).toContain('N');
  });

  test('完成状態: Structure + Constraints + 右ペイン全て正しい', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    // Structure 構築
    await editor.addScalar('N');
    await editor.addArray('A', 'N');

    // 全制約を埋める
    await editor.fillDraftRange(0, '1', '10^6');
    await editor.fillDraftRange(1, '1', '10^9');

    // 右ペイン検証
    const texInput = await editor.getTexInputFormat();
    expect(texInput).toContain('N');
    expect(texInput).toContain('A');

    const texConstraints = await editor.getTexConstraints();
    expect(texConstraints).toContain('N');
    expect(texConstraints).toContain('A');

    const sample = await editor.getSampleOutput();
    expect(sample.trim().split('\n').length).toBeGreaterThanOrEqual(2); // N行 + A行
  });
});
```

#### 2. grid.spec.ts

```typescript
test.describe('グリッド: H W / S_1...S_H', () => {

  test('tuple [H, W] を作成する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    await editor.clickHotspot('below');
    await editor.selectPopupOption('tuple');
    // H, W を同一行に追加するウィザード
    // (具体的な UI は実装時に確定)
    await editor.inputName('H');
    await editor.confirm();
    // W を追加
    await editor.inputName('W');
    await editor.confirm();

    await expect(editor.structurePane).toContainText('H');
    await expect(editor.structurePane).toContainText('W');
  });

  test('文字グリッドテンプレートを追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    // tuple 作成後
    await editor.clickHotspot('below');
    await editor.selectPopupOption('tuple');
    await editor.inputName('H');
    await editor.confirm();
    await editor.inputName('W');
    await editor.confirm();

    // グリッドテンプレート
    await editor.clickHotspot('below');
    await editor.selectPopupOption('grid-template');
    await editor.selectLength('H'); // rows
    await editor.selectLength('W'); // cols

    // draft: |S_i| = W, charset(S) = ? が自動生成
    const drafts = await editor.getDraftConstraints();
    expect(drafts.length).toBeGreaterThanOrEqual(3); // H range, W range, charset
  });

  test('完成状態: グリッド表示と sample 生成', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    // 構造構築 + 制約埋め (省略: 上記ステップの組み合わせ)
    // ...

    // 右ペイン: TeX にグリッド表示
    const texInput = await editor.getTexInputFormat();
    expect(texInput).toContain('H');
    expect(texInput).toContain('W');
    expect(texInput).toContain('S');

    // sample: H行 × W文字のグリッド
    const sample = await editor.getSampleOutput();
    expect(sample.trim().split('\n').length).toBeGreaterThanOrEqual(3); // H W行 + H行のグリッド
  });
});
```

#### 3. tree.spec.ts

```typescript
test.describe('木入力: N / u_1 v_1...u_{N-1} v_{N-1}', () => {

  test('辺リストテンプレートを追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    await editor.addScalar('N');

    // 辺リストテンプレート
    await editor.clickHotspot('below');
    await editor.selectPopupOption('edge-list');
    // count = N - 1 (式入力)
    // UI で N - 1 を入力する方法は実装時確定

    // draft: 1 ≤ u_i ≤ N, 1 ≤ v_i ≤ N が自動生成
    const drafts = await editor.getDraftConstraints();
    expect(drafts.length).toBeGreaterThanOrEqual(2); // N range + edge ranges
  });

  test('Tree property を追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    await editor.addScalar('N');
    await editor.clickHotspot('below');
    await editor.selectPopupOption('edge-list');
    // ...setup...

    await editor.addProperty('tree');

    // completed constraint に Tree が表示
    const completed = editor.page.getByTestId(/^completed-constraint/);
    const text = await completed.allTextContents();
    expect(text.some(t => t.toLowerCase().includes('tree'))).toBeTruthy();
  });
});
```

#### 4. query.spec.ts

```typescript
test.describe('クエリ列: N Q / variant分岐', () => {

  test('クエリ列テンプレートを追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    // N Q tuple
    await editor.clickHotspot('below');
    await editor.selectPopupOption('tuple');
    await editor.inputName('N');
    await editor.confirm();
    await editor.inputName('Q');
    await editor.confirm();

    // クエリ列
    await editor.clickHotspot('below');
    await editor.selectPopupOption('query-list');
    await editor.selectLength('Q'); // count = Q

    await expect(editor.structurePane).toContainText('Q');
  });

  test('variant を3つ追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    // setup tuple + query list...
    // variant 追加
    // tag=1, body=[a, b]
    // tag=2, body=[a, b]
    // tag=3, body=[a]

    // 右ペイン: TeX にクエリ形式が表示
    const texInput = await editor.getTexInputFormat();
    expect(texInput.length).toBeGreaterThan(0);
  });
});
```

#### 5. multi-testcase.spec.ts

```typescript
test.describe('複数テストケース: T / (N / A) × T', () => {

  test('複数テストケーステンプレートを適用する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    await editor.addScalar('T');
    await editor.clickHotspot('below');
    await editor.selectPopupOption('multi-testcase');
    // count = T, body に Scalar N + Array A を追加

    // Structure にマルチケース表示
    await expect(editor.structurePane).toContainText('T');
  });

  test('SumBound を追加する', async ({ page }) => {
    const editor = new EditorPage(page);
    await editor.goto();

    // setup...

    // SumBound: ΣN ≤ 2×10^5
    // (UI 操作は実装時確定)

    const texConstraints = await editor.getTexConstraints();
    expect(texConstraints.length).toBeGreaterThan(0);
  });
});
```

### 共通テスト（全カテゴリ横断）

```
// helpers.ts に定義する共通検証

1. 右ペイン三要素チェック:
   - TeX 入力形式が空でない
   - TeX 制約が空でない
   - sample が空でない

2. Structure → draft 自動生成チェック:
   - ノード追加後に draft が増える

3. draft → completed 昇格チェック:
   - range 入力後に draft が消え completed が増える
```

### Playwright 設定

```typescript
// web/tests/e2e/playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  },
});
```

### npm scripts 追加

```json
{
  "scripts": {
    "test:e2e": "playwright test --config tests/e2e/playwright.config.ts",
    "test:e2e:ui": "playwright test --config tests/e2e/playwright.config.ts --ui"
  },
  "devDependencies": {
    "@playwright/test": "^1.40.0"
  }
}
```

---

## 実装タスク一覧

### Phase 0 タスク
| ID | タスク | 説明 |
|----|--------|------|
| p0-coverage-index | カバレッジシナリオインデックス作成 | 5カテゴリの代表問題と優先度を doc/view/coverage-scenario-index.md に固定 |

### Phase 1 タスク
| ID | タスク | 説明 |
|----|--------|------|
| p1-flow-basic-array | 基本配列フロー文書化 | N + A_1...A_N の操作順序を文章化 |
| p1-flow-grid | グリッドフロー文書化 | H W + S_1...S_H の操作順序を文章化 |
| p1-flow-tree | 木入力フロー文書化 | N + 辺リスト の操作順序を文章化 |
| p1-flow-query | クエリ列フロー文書化 | N Q + variant分岐 の操作順序を文章化 |
| p1-flow-multi-tc | 複数テストケースフロー文書化 | T + repeat cases の操作順序を文章化 |

### Phase 2 タスク
| ID | タスク | 依存 |
|----|--------|------|
| p2-playwright-setup | Playwright 基盤構築 | — |
| p2-testid-contract | data-testid 契約文書 | — |
| p2-page-object | EditorPage Page Object 実装 | p2-playwright-setup |
| p2-spec-basic-array | basic-array.spec.ts 作成 | p1-flow-basic-array, p2-page-object |
| p2-spec-grid | grid.spec.ts 作成 | p1-flow-grid, p2-page-object |
| p2-spec-tree | tree.spec.ts 作成 | p1-flow-tree, p2-page-object |
| p2-spec-query | query.spec.ts 作成 | p1-flow-query, p2-page-object |
| p2-spec-multi-tc | multi-testcase.spec.ts 作成 | p1-flow-multi-tc, p2-page-object |
| p2-readme | E2E テスト README 作成 | p2 全 spec |
| p2-design-doc | 設計書を docs/superpowers/specs/ に保存 | Phase 2 完了 |

---

## 設計原則（proccess.md 準拠）

1. **テストを変えて実装に合わせない** — 実装をテストの要求に合わせる
2. **最初は失敗していてよい** — まず「どう動くべきか」を固定する
3. **右ペイン三要素は必須** — TeX入力形式 + TeX制約 + sample は常に検証する
4. **Structure 操作 → draft 自動生成** はテストで固定する
5. **数式 direct manipulation** もテスト対象に含める

## 注意事項

- 現在の UI は Viewer のみで編集機能がないため、E2E テストは全て fail から始まる
- WASM 層に Operation/Projection API の公開が Phase 3 で必要になる
- FillContent に Tuple/Repeat/Choice/EdgeList/QueryList が未実装（Phase 3 で追加）
- DraftConstraint の概念が Rust 側に存在しない（EditorState として TypeScript 側で管理する設計）
