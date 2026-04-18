# Problem User Flows

> E2E テスト先行設計の Phase 1 成果物
>
> 各 MVP 問題について、空状態から完成までのユーザー操作順序を固定する。
> この文書が E2E テストの仕様根拠となる。

---

## 共通: 初期状態

エディタを開いた直後の状態:

- **Structure ペイン**: 空。1つの insertion hotspot（下方向）のみ表示
- **Constraint ペイン**: 空
- **右ペイン**: TeX 入力形式=空、TeX 制約=空、sample=空
- **EditorState**: 選択なし、draft なし

---

## 1. 基本配列 (ABC395-A 相当)

### 目標

**入力形式**:
```
N
A_1 A_2 ... A_N
```

**制約**:
- 1 <= N <= 10^6
- 1 <= A_i <= 10^9

### 操作順序

#### Step 1: scalar N を追加

1. Structure ペインの insertion hotspot（下方向）をクリック
2. ポップアップが開く。候補: [scalar, 横配列, 縦配列, tuple, repeat, ...]
3. `scalar` を選択
4. 型選択: `number` を選択（デフォルト）
5. 名前入力: `N` を入力
6. 確定ボタンをクリック

**結果**:
- Structure ペイン: `N` が 1 行目に表示される
- Structure ペイン: N の下に新しい insertion hotspot が出現
- Constraint ペイン: draft `? <= N <= ?` が自動生成される
- 右ペイン TeX 入力: `N` が表示される
- 右ペイン sample: 数値（例: `5`）が表示される

#### Step 2: 横配列 A を追加

1. N の下の insertion hotspot をクリック
2. ポップアップが開く
3. `横配列` を選択
4. 型選択: `number`
5. 名前入力: `A` を入力
6. 長さ選択: ドロップダウンから `N` を選択（既存変数を参照）
7. 確定

**結果**:
- Structure ペイン: 2 行目に `A_1 A_2 ... A_N` が表示される
- Constraint ペイン: draft `? <= A_i <= ?` が追加される（計 2 つの draft）
- 右ペイン TeX 入力: `N` と `A_1 \cdots A_N` が表示される
- 右ペイン sample: 2 行（N の値 + 配列値）が表示される

#### Step 3: N の range 制約を埋める

1. Constraint ペインの draft `? <= N <= ?` をクリック
2. 下限入力欄に `1` を入力
3. 上限入力欄に `10^6` を入力（数式入力）
4. 確定

**結果**:
- draft が消え、completed constraint `1 <= N <= 10^6` が表示される
- 右ペイン TeX 制約: `1 \le N \le 10^6` が表示される

#### Step 4: A_i の range 制約を埋める

1. Constraint ペインの draft `? <= A_i <= ?` をクリック
2. 下限: `1`
3. 上限: `10^9`
4. 確定

**結果**:
- completed constraint `1 <= A_i <= 10^9` が表示される
- 右ペイン TeX 制約: `1 \le A_i \le 10^9` が追加される

### 完成条件

- Structure ペイン:
  ```
  N
  A_1 A_2 ... A_N
  ```
- Constraint ペイン: draft なし、completed 2 つ
  - `1 <= N <= 10^6`
  - `1 <= A_i <= 10^9`
- 右ペイン TeX 入力形式: `N` / `A_1 \cdots A_N`
- 右ペイン TeX 制約: `1 \le N \le 10^6` / `1 \le A_i \le 10^9`
- 右ペイン sample: 2 行以上、有効な数値

---

## 2. グリッド (ABC390-C 相当)

### 目標

**入力形式**:
```
H W
S_1
S_2
...
S_H
```

**制約**:
- 1 <= H <= 500
- 1 <= W <= 500
- S_i は英小文字からなる長さ W の文字列

### 操作順序

#### Step 1: tuple [H, W] を追加

1. insertion hotspot をクリック
2. `tuple` を選択
3. 1 つ目: 型 `number`、名前 `H`
4. 2 つ目: 型 `number`、名前 `W`
5. 確定

**結果**:
- Structure ペイン: `H W` が 1 行に表示される
- Constraint ペイン: draft `? <= H <= ?` と `? <= W <= ?` が生える
- 右ペイン TeX 入力: `H \quad W`
- 右ペイン sample: `3 4`（仮値）

#### Step 2: 文字グリッドテンプレートを追加

1. H W の下の insertion hotspot をクリック
2. `文字グリッド` テンプレートを選択
3. 行数: ドロップダウンから `H` を選択
4. 列数（文字列長）: ドロップダウンから `W` を選択
5. 確定

**結果**:
- Structure ペイン:
  ```
  H W
  S_1
  ...
  S_H
  ```
- Constraint ペイン: 以下の draft が自動生成
  - `|S_i| = W`（自動、既に completed として生成してよい）
  - `charset(S) = ?`（draft）
- 右ペイン TeX 入力: グリッド形式が表示
- 右ペイン sample: H 行 x W 文字のグリッドが生成

#### Step 3: charset を設定

1. Constraint ペインの draft `charset(S) = ?` をクリック
2. プリセットから「英小文字」を選択
3. 確定

**結果**:
- completed: `S_i は英小文字からなる`

#### Step 4: H, W の range を埋める

1. draft `? <= H <= ?` → 下限 `1`, 上限 `500`
2. draft `? <= W <= ?` → 下限 `1`, 上限 `500`

### 完成条件

- Structure: `H W` / `S_1` / `...` / `S_H`
- Constraints: H range, W range, |S_i| = W, charset = 英小文字
- 右ペイン: TeX グリッド + sample grid（H行 x W文字）

---

## 3. 木入力 (典型 ABC-D 相当)

### 目標

**入力形式**:
```
N
u_1 v_1
u_2 v_2
...
u_{N-1} v_{N-1}
```

**制約**:
- 2 <= N <= 2 * 10^5
- 1 <= u_i, v_i <= N
- グラフは木である

### 操作順序

#### Step 1: scalar N を追加

1. insertion hotspot → `scalar` → `number` → `N` → 確定

**結果**:
- Structure: `N`
- Draft: `? <= N <= ?`

#### Step 2: 辺リストテンプレートを追加

1. N の下の hotspot をクリック
2. `辺リスト` テンプレートを選択
3. 本数（count）: 式入力で `N - 1` を入力
4. 確定

**結果**:
- Structure:
  ```
  N
  u_1 v_1
  ...
  u_{N-1} v_{N-1}
  ```
- Constraint ペイン: 以下の draft が自動生成
  - `? <= u_i <= ?`（draft、初期値として `1 <= u_i <= N` を提案）
  - `? <= v_i <= ?`（draft、初期値として `1 <= v_i <= N` を提案）
- 右ペイン: TeX + sample 更新

#### Step 3: 辺の range を確定

辺リストテンプレートが `1 <= u_i <= N` と `1 <= v_i <= N` を提案していれば、確認して確定。

**結果**:
- completed: `1 <= u_i <= N`, `1 <= v_i <= N`

#### Step 4: Tree property を追加

1. Constraint ペインの Property ショートカットボタンをクリック
2. 候補: [Tree, Simple, Connected, Distinct, Sorted, Permutation, ...]
3. `Tree` を選択

**結果**:
- completed constraint: `グラフは木である`

#### Step 5: N の range を埋める

1. draft `? <= N <= ?` → 下限 `2`, 上限 `2 * 10^5`

### 完成条件

- Structure: `N` / `u_1 v_1` / `...` / `u_{N-1} v_{N-1}`
- Constraints: N range, u_i range, v_i range, Tree property
- 右ペイン: TeX + sample tree（N-1 本の辺）

---

## 4. クエリ列 (ABC395-D 相当)

### 目標

**入力形式**:
```
N Q
1 a b
2 a b
3 a
```
（Q 行のクエリ。先頭の数字で 3 種のバリアント分岐）

**制約**:
- 1 <= N <= 2 * 10^5
- 1 <= Q <= 2 * 10^5
- 1 <= a, b <= N

### 操作順序

#### Step 1: tuple [N, Q] を追加

1. hotspot → `tuple` → `number, N` + `number, Q` → 確定

**結果**:
- Structure: `N Q`
- Draft: `? <= N <= ?`, `? <= Q <= ?`

#### Step 2: クエリ列テンプレートを追加

1. N Q の下の hotspot をクリック
2. `クエリ列` テンプレートを選択
3. count: ドロップダウンから `Q` を選択
4. 確定

**結果**:
- Structure: `N Q` の下にクエリ列ブロックが表示
- 初期状態: variant なし（variant 追加用の hotspot が表示される）

#### Step 3: variant 1 を追加 (tag=1, body=[a, b])

1. クエリ列ブロック内の「variant 追加」hotspot をクリック
2. tag 値: `1` を入力
3. body の構造:
   - `scalar` → `number` → `a`
   - 右に追加 → `scalar` → `number` → `b`
4. 確定

**結果**:
- Structure にバリアント `1 a b` が表示される

#### Step 4: variant 2 を追加 (tag=2, body=[a, b])

1. 「variant 追加」hotspot をクリック
2. tag: `2`, body: `a, b`（variant 1 と同構造）

#### Step 5: variant 3 を追加 (tag=3, body=[a])

1. 「variant 追加」hotspot をクリック
2. tag: `3`, body: `a`（1 変数のみ）

#### Step 6: 各変数の制約を埋める

1. N range: `1` 〜 `2 * 10^5`
2. Q range: `1` 〜 `2 * 10^5`
3. a range: `1` 〜 `N`
4. b range: `1` 〜 `N`

### 完成条件

- Structure: `N Q` / クエリ列（3 variant）
- Constraints: N, Q, a, b の range
- 右ペイン: TeX クエリ形式 + sample（variant 混在）

---

## 5. 複数テストケース (典型マルチケース)

### 目標

**入力形式**:
```
T
N
A_1 A_2 ... A_N
（T ケース分）
```

**制約**:
- 1 <= T <= 10^5
- 1 <= N <= 2 * 10^5
- sum(N) <= 2 * 10^5
- 1 <= A_i <= 10^9

### 操作順序

#### Step 1: scalar T を追加

1. hotspot → `scalar` → `number` → `T` → 確定

**結果**:
- Structure: `T`
- Draft: `? <= T <= ?`

#### Step 2: 複数テストケーステンプレートを適用

1. T の下の hotspot をクリック
2. `複数テストケース` テンプレートを選択
3. テストケース数: `T` を選択
4. 確定

**結果**:
- Structure: `T` の下にリピートブロックが表示
- ブロック内に insertion hotspot が表示
- 右ペイン: `T` と空のケース表示

#### Step 3: ケース内に scalar N を追加

1. リピートブロック内の hotspot → `scalar` → `number` → `N` → 確定

**結果**:
- Structure: ブロック内に `N` が追加
- Draft: `? <= N <= ?`

#### Step 4: ケース内に横配列 A を追加

1. N の下の hotspot → `横配列` → `number` → `A` → 長さ `N` → 確定

**結果**:
- Structure: ブロック内に `A_1 A_2 ... A_N` が追加
- Draft: `? <= A_i <= ?`

#### Step 5: SumBound を追加

1. Constraint ペインの SumBound ショートカットをクリック
2. 対象変数: `N`
3. 上界: `2 * 10^5`
4. 確定

**結果**:
- completed: `sum(N) <= 2 * 10^5`

#### Step 6: 残りの制約を埋める

1. T range: `1` 〜 `10^5`
2. N range: `1` 〜 `2 * 10^5`
3. A_i range: `1` 〜 `10^9`

### 完成条件

- Structure: `T` / repeat block { `N` / `A_1...A_N` }
- Constraints: T range, N range, A_i range, SumBound
- 右ペイン: TeX（複数ケース表記）+ sample（T ケース分）

---

## 補足: 全カテゴリ共通の検証項目

### Structure 側

- [ ] 初期状態で insertion hotspot が見える
- [ ] hotspot クリックでポップアップに正しい候補が出る
- [ ] ノード追加後に入力形式として自然に見える（ツリーではない）
- [ ] 追加後に新しい hotspot が出現する

### Constraint 側

- [ ] Structure 操作に応じて draft constraint が自動生成される
- [ ] draft はクリックで値を埋められる
- [ ] 値を埋めた draft は completed constraint に昇格する
- [ ] completed と draft は視覚的に区別される

### 右ペイン

- [ ] 各操作後に TeX 入力形式がリアルタイム更新される
- [ ] 各操作後に TeX 制約がリアルタイム更新される
- [ ] 各操作後に sample ケースが更新される
- [ ] sample は制約に合致する有効な値を含む

### 数式編集

- [ ] 表示された数式の個別要素をクリックできる
- [ ] クリック後に編集 UI が開く
- [ ] 編集中は AST を壊さない（未完成式は EditorState のみ）
- [ ] 確定後に完成式として AST に反映される
