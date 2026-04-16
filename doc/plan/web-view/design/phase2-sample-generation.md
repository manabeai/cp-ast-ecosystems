# Phase 2: Sample Generation Agent Report

本文書は、Editor UI設計における**Sample Generation連携**の判断と提案を記録する。

**担当領域**: Generator最小必要情報、AST→Generator情報フロー、Editor不足メタ情報、保証レベルとUI期待値整合、retry/repair/constructive戦略展望

---

## 1. 前提

### 1.1 固定前提の確認

以下のphase1-premises.md前提を継承・確認した:

| # | 前提 | 本Agentの解釈 |
|---|------|---------------|
| A-2 | ASTが唯一の正本（sample generationの入力） | **完全支持**。Generatorへの入力は`AstEngine`のみ |
| A-7 | Holeは第一級市民（未完成でもシステムは壊れない） | **完全支持**。Hole存在時は生成不可とするが、Generatorはパニックしない |
| D-1 | 基本ケース優先（scalar, array, grid, edge list, multi-testcase, query） | **完全支持**。これらすべてで生成可能を保証する |
| U-9 | sample generationのeditor連携詳細が未確定 | **本文書で確定する** |

### 1.2 現行Sample Generator実装の棚卸し

**crates/cp-ast-core/src/sample/** を精査した結果:

| モジュール | 実装状態 | 機能 |
|-----------|----------|------|
| `dependency.rs` | 完全実装 | `DependencyGraph::build()`, `topological_sort()`, サイクル検出 |
| `generator.rs` | 完全実装 | 全NodeKind対応、Constraint駆動値生成、Expression評価 |
| `output.rs` | 完全実装 | 競プロ形式テキスト出力（空白/改行制御） |

**現行対応Constraint**:
- Range: ✅ 式評価で動的範囲計算
- TypeDecl: ✅ Int/Str/Char対応
- LengthRelation: ✅ 依存グラフ経由で自動解決
- CharSet: ✅ 7種の文字集合対応（LowerAlpha, UpperAlpha, Alpha, Digit, AlphaNumeric, Custom, Range）
- StringLength: ✅ 文字列長制約対応
- Distinct: ✅ Fisher-Yates / リジェクションサンプリング
- Sorted: ✅ 生成後ソート
- Property: ✅ Permutation, Tree, Simple, Binary対応
- Relation: ⚠️ 生成順序制御なし（ヒューリスティック未実装）
- SumBound: ⚠️ 未実装（マルチテストケース用）
- Guarantee: ✅ 無視（生成責務外）

### 1.3 現行wasm API

```rust
// crates/cp-ast-wasm/src/lib.rs:123-128
pub fn generate_sample(document_json: &str, seed: u32) -> Result<String, JsError>
```

- 入力: AstEngine JSON + seed
- 出力: 競プロ形式テキスト or エラー
- Seed: u32（JS Number互換）

---

## 2. 主要判断

### 2.1 Generator最小必要情報リスト

Generatorが**絶対に必要とする**情報:

| 情報 | 提供元 | 必須度 | 備考 |
|------|--------|--------|------|
| Structure全体 | `AstEngine.structure` | **必須** | ノード間参照の解決に必要 |
| Constraint全体 | `AstEngine.constraints` | **必須** | 値生成範囲・型の決定に必要 |
| Seed | API引数 | **必須** | 決定的生成の保証 |
| GenerationConfig | オプション | 任意 | max_retries, max_repeat_count |

**現行実装の確認**:
```rust
// generator.rs:732-734
pub fn generate(engine: &AstEngine, seed: u64) -> Result<GeneratedSample, GenerationError> {
    generate_with_config(engine, seed, GenerationConfig::default())
}
```

**判断**: 現行APIは必要十分。EditorはAstEngineを渡すのみで良い。

### 2.2 情報フロー

```
AST (Structure + Constraints)
    │
    ├─── ConstraintSet.for_node(id) ──► 各ノードの制約取得
    │
    ▼
DependencyGraph::build(engine)
    │
    ├─── 構造参照（Array.length, Matrix.rows/cols, Repeat.count）
    ├─── 親子関係（Repeat body, Choice variants）
    │
    ▼
topological_sort() → Vec<NodeId>
    │ サイクル検出 → CycleError
    ▼
GenerationContext
    │
    ├─── evaluate(Expression) → i64
    ├─── resolve_range() → (lo, hi)
    ├─── generate_node_inner() ───┐
    │                              │
    │    ┌────────────────────────┘
    │    │
    │    ├── Scalar: Range制約評価 → uniform_random(lo, hi)
    │    ├── Array: 長さ式評価 → 各要素生成 → Distinct/Sorted適用
    │    ├── Matrix: rows/cols参照解決 → グリッド生成
    │    ├── Repeat: count式評価 → body N回実行 → repeat_instances
    │    ├── Choice: ランダムvariant選択 → tag値設定 → 選択variant生成
    │    └── Hole: スキップ
    │
    ▼
GeneratedSample
    │
    ├── values: HashMap<NodeId, SampleValue>
    ├── repeat_instances: HashMap<NodeId, Vec<HashMap<NodeId, SampleValue>>>
    │
    ▼
sample_to_text(engine, sample) → String
```

### 2.3 Editor文脈における不足情報

| # | 情報 | 現行状態 | Editorが補完すべきか | 対処方針 |
|---|------|----------|---------------------|----------|
| M-1 | Relation生成順序ヒント | 未実装（lhs先行ヒューリスティックなし） | No | Generator内部で決定 |
| M-2 | SumBound総和分配 | 未実装 | No | Generator拡張で対応 |
| M-3 | Property組み合わせ戦略 | Tree+Simple等の組み合わせ未対応 | No | Generator拡張で対応 |
| M-4 | デフォルト Range | 未指定時 (1, 100) | Yes（警告表示） | Editor側でWarning |
| M-5 | デフォルト CharSet | 未指定時 LowerAlpha | No | 合理的デフォルト |
| M-6 | デフォルト StringLength | 未指定時 (1, 10) | Yes（警告表示） | Editor側でWarning |

**判断**: EditorはGenerator実装詳細を知る必要がない。ただしデフォルト値適用時の警告表示は有用。

### 2.4 保証レベル

doc/review/sample-generation.mdの定義を踏襲・具体化:

| レベル | 名称 | 条件 | UIでの表示 | 生成失敗確率 |
|--------|------|------|------------|-------------|
| **L1** | `Guaranteed` | Range + LengthRelation + TypeDecl(Int) のみ | ✅ 生成確実 | 0% |
| **L2** | `HighProbability` | L1 + Distinct + Sorted + Property(組込) | ⚠️ 高確率生成 | < 0.1%（リジェクション上限内） |
| **L3** | `BestEffort` | Guarantee含む / Custom Property / 複合Relation | ℹ️ ベストエフォート | 〜数%（制約依存） |
| **E0** | `Ungeneratable` | Holeあり / サイクル検出 / 充足不能 | ❌ 生成不可 | 100% |

**現行GenerationErrorのマッピング**:
```rust
enum GenerationError {
    CycleDetected(_)      → E0
    UnresolvedReference(_) → E0（依存グラフ異常）
    TypeMismatch{..}      → E0（制約矛盾）
    RangeEmpty{..}        → E0（範囲矛盾）
    RetryExhausted{..}    → L2失敗（UI側で再試行提案）
    InvalidExpression(_)   → E0
    InvalidStructure(_)    → E0
}
```

### 2.5 UI接続

#### Generate ボタン有効化条件

| 条件 | 判定元 | 優先度 |
|------|--------|--------|
| Holeが存在しない | Structure走査 | **必須** |
| サイクルが存在しない | DependencyGraph::topological_sort() | **必須** |
| Range制約の上下界が静的検査で矛盾しない | Constraint静的解析 | 推奨 |
| すべてのProperty tagがGenerator対応済み | 対応Propertyリスト | 推奨 |

**wasm API拡張提案**:
```rust
/// 生成可能性の事前検査
/// Returns: { generatable: bool, level: L1|L2|L3, warnings: Vec<String>, errors: Vec<String> }
pub fn check_generatability(document_json: &str) -> Result<String, JsError>
```

#### 生成ブロック診断

Editorが「Generate」ボタン押下前に表示すべき診断:

| 診断 | レベル | メッセージ例 |
|------|--------|-------------|
| Hole存在 | Error | "Structure に未定義 Hole があります" |
| サイクル検出 | Error | "依存関係にサイクルがあります: N → M → N" |
| 範囲矛盾（静的） | Error | "Range(X, 10, 5) の下界が上界を超えています" |
| Range未設定（スカラー） | Warning | "N に Range 制約がありません（デフォルト 1-100 を使用）" |
| TypeDecl未設定 | Warning | "A に型宣言がありません（Int として扱います）" |
| Property(Custom) | Warning | "Custom Property 'bipartite' は生成非対応です" |

#### 生成失敗時の表示

| エラー種別 | UIメッセージ | 対処提案 |
|-----------|-------------|----------|
| CycleDetected | "循環依存が検出されました" | 関与ノードをハイライト |
| RangeEmpty | "制約 Range(X, lo, hi) で lo > hi となりました" | 制約編集リンク |
| RetryExhausted | "Distinct 制約の充足に失敗しました（100回リトライ）" | 値域拡大 or 要素数削減を提案 |
| InvalidStructure | "Choice ノードに variant がありません" | ノード編集リンク |

### 2.6 インクリメンタル生成

**現行設計**: 生成は毎回フルパス（依存グラフ構築→トポロジカルソート→全ノード生成）

**小規模編集後の効率化可能性**:

| 編集種別 | 再生成スコープ | 実現難易度 |
|----------|--------------|------------|
| Constraint変更（Range上下界） | 対象ノード+依存先のみ | 中 |
| ノード名変更 | 再生成不要（Output層のみ） | 易 |
| ノード追加（新規スカラー） | 新規ノードのみ | 中 |
| ノード削除 | フル再生成（依存グラフ再構築） | 難 |
| 構造変更（配列→スカラー等） | フル再生成 | 難 |

**MVP判断**: インクリメンタル生成は**MVP後回し**。理由:
1. 典型的な問題サイズ（N≤10^5）でも生成は O(ms) で完了
2. 依存グラフの差分更新は実装複雑度が高い
3. seed維持による再現性確保が困難になる

---

## 3. 具体例

### 3.1 Simple: N (1-100), A[N] (1-1000)

**AST構造**:
```
Sequence[
  Scalar(name="N"),
  Array(name="A", length=Var(N))
]
```

**Constraints**:
```
Range(N, 1, 100)
Range(A, 1, 1000)
TypeDecl(N, Int)
TypeDecl(A, Int)
LengthRelation(A, N)
```

**依存グラフ**:
```
N ← 依存なし
A ← N（length参照）
```

**生成フロー**:
1. `topological_sort()` → `[N, A]`
2. `generate_scalar(N)`:
   - `resolve_range()` → `(1, 100)`
   - `rng.gen_range(1..=100)` → 例: 42
   - `values[N] = Int(42)`
3. `generate_array(A)`:
   - `resolve_expression_as_int(Var(N))` → 42
   - `resolve_range()` → `(1, 1000)`
   - 42要素を `gen_range(1..=1000)` で生成
   - `values[A] = Array([...])`

**出力例**:
```
42
859 421 15 ... (42個)
```

**保証レベル**: L1（Guaranteed）

### 3.2 Medium: Tree with N nodes, weighted edges 1-10^9

**AST構造**:
```
Sequence[
  Scalar(name="N"),
  Repeat(
    count=Var(N)-1,
    index_var="i",
    body=[
      Tuple[
        Scalar(name="u"),
        Scalar(name="v"),
        Scalar(name="w")
      ]
    ]
  )
]
```

**Constraints**:
```
Range(N, 2, 10^5)
Range(u, 1, N)
Range(v, 1, N)
Range(w, 1, 10^9)
TypeDecl(N, Int)
TypeDecl(u, Int)
TypeDecl(v, Int)
TypeDecl(w, Int)
Property(辺リスト, Tree)
```

**依存グラフ**:
```
N ← 依存なし
Repeat ← N（count参照）
Tuple ← Repeat（body要素）
u, v, w ← Tuple（子要素）
```

**生成フロー**:
1. `generate_scalar(N)` → 例: 5
2. `generate_repeat()`:
   - count = `N - 1` = 4
   - Property(Tree)検出 → `generate_property_array(Tree, 4)` ではなく、Repeatなので**各イテレーションでu,v,w生成**
   
**現行実装の問題**: Property(Tree)はArray用であり、Repeatスコープでは未対応。

**対処方針**:
- Edge list形式のTree生成はRepeat外のArray（u[], v[], w[]構造）で表現すべき
- または、Repeat + Property(Tree)の組み合わせに対応する専用ロジック追加

**保証レベル**: L2（Tree Property対応済み）またはL3（Repeat内Tree未対応の場合）

### 3.3 Complex: Multi-testcase with sum bound

**AST構造**:
```
Section(header=Scalar("T"), body=[
  Repeat(
    count=Var(T),
    index_var="case",
    body=[
      Sequence[
        Scalar(name="N"),
        Array(name="A", length=Var(N))
      ]
    ]
  )
])
```

**Constraints**:
```
Range(T, 1, 10^5)
Range(N, 1, 2*10^5)
Range(A, 1, 10^9)
SumBound(N, 2*10^5)  // ΣN ≤ 2×10^5
```

**依存グラフ**:
```
T ← 依存なし
Repeat ← T
N ← Repeat（各イテレーション）
A ← N
```

**生成フロー（理想）**:
1. `generate_scalar(T)` → 例: 3
2. SumBound考慮の分配:
   - remaining = 2*10^5
   - 各ケースで N を `[1, min(2*10^5, remaining - (T-1))]` から選択
   - case1: N=50000, remaining=150000
   - case2: N=100000, remaining=50000
   - case3: N=50000, remaining=0
3. 各 A を N に応じて生成

**現行実装状態**: SumBound**未実装**

**対処方針**: Generator拡張が必要。Editor側で対処不可。

**保証レベル**: L3（BestEffort）またはE0（SumBound未実装の場合）

### 3.4 Edge case: Incomplete AST with Holes

**AST構造**:
```
Sequence[
  Scalar(name="N"),
  Hole(hint=AnyArray)  // 未完成
]
```

**生成フロー**:
1. `topological_sort()` → `[N, Hole]`
2. `generate_node_inner(Hole)` → **スキップ**（generator.rs:431）

**出力**:
```
42
```
（Hole部分は出力されない）

**保証レベル**: E0（Ungeneratable）

**UI表示**: "Structure に未定義 Hole があります" + Hole位置ハイライト

---

## 4. Editor不足情報一覧

| 情報 | 現在の提供状況 | Editorが補完すべきか | 優先度 | 理由 |
|------|---------------|---------------------|--------|------|
| 生成可能性事前検査 | API未提供 | **Yes**（wasm API追加） | 高 | Generateボタン有効化判定に必須 |
| 保証レベル表示 | API未提供 | **Yes**（wasm API追加） | 高 | ユーザー期待値管理に必要 |
| サイクル関与ノード | CycleError.involved | No（現行で十分） | - | ハイライト表示に使用 |
| デフォルト値適用警告 | 未実装 | **Yes**（Editor側で検出） | 中 | 暗黙のデフォルト回避 |
| SumBound対応 | 未実装 | No（Generator拡張） | 高 | Generator責務 |
| Relation生成順序 | 未実装 | No（Generator拡張） | 中 | Generator責務 |
| Property(Custom)警告 | 未実装 | **Yes**（wasm API拡張） | 中 | L3への降格を通知 |
| インクリメンタル生成 | 未実装 | No | 低 | MVP後回し |

---

## 5. 現行方針に対する支持/反対/留保

### 5.1 完全支持

| 方針 | 支持理由 |
|------|----------|
| A-2: ASTが唯一の正本 | GeneratorはAstEngineのみを入力とし、Editor独自状態への依存がない |
| A-7: Hole第一級市民 | Generator.generate_node_innerでHoleをスキップする実装が安全 |
| plan.md §22: Bottom PanelにSample出力 | 即時フィードバックに有用 |
| plan.md §22: Generate/Regenerateの分離 | Seed管理で同一仕様から複数サンプル生成可能 |

### 5.2 条件付き支持

| 方針 | 条件 |
|------|------|
| L1/L2/L3保証レベル | **wasm APIで保証レベルを返却する拡張が前提** |
| SumBound対応 | **Generator拡張がMVPスコープに含まれること** |

### 5.3 留保

| 方針 | 留保理由 |
|------|----------|
| インクリメンタル生成 | 実装複雑度と効果のトレードオフが不明。性能測定後に再評価 |
| Relation生成順序ヒント | Constraintスキーマ変更が必要。ヒューリスティックで多くのケースは対応可能 |

---

## 6. 不足点

### 6.1 wasm API拡張

**必要なAPI**:

```rust
/// 生成可能性事前検査
#[wasm_bindgen]
pub fn check_generatability(document_json: &str) -> Result<String, JsError>;
// 返却: { generatable: bool, level: "L1"|"L2"|"L3"|"E0", warnings: [...], errors: [...] }

/// 保証レベル付き生成
#[wasm_bindgen]
pub fn generate_sample_with_level(document_json: &str, seed: u32) -> Result<String, JsError>;
// 返却: { text: "...", level: "L1"|"L2"|"L3", warnings: [...] }
```

### 6.2 SumBound実装

SumBoundはマルチテストケース問題で頻出（AtCoder ABC 80%以上）。Generator拡張が**MVP必須**。

**実装方針**:
1. SumBound対象変数を特定
2. Repeat生成時に残り総和を追跡
3. 各イテレーションで `[per_case_lower, min(per_case_upper, remaining)]` から選択

### 6.3 診断API

Editorが生成前に表示する警告・エラーのためのAPI:

```rust
/// 診断情報取得（生成前検査）
#[wasm_bindgen]
pub fn get_generation_diagnostics(document_json: &str) -> Result<String, JsError>;
// 返却: [{ level: "error"|"warning"|"info", message: "...", node_id: ..., constraint_id: ... }]
```

---

## 7. 他Agentに渡すべき論点

### 7.1 → Domain Model Agent

- **SlotId設計**: SumBound対応にはRepeatスコープ内での制約適用ルールが必要。`SlotId`が各イテレーションを識別できるか確認が必要
- **Expression評価**: FnCall対応関数リスト（min, max, abs）の文書化

### 7.2 → GUI Interaction Agent

- **Bottom Panelレイアウト**: 保証レベル表示領域、警告リスト表示領域の確保
- **Generateボタン**: 有効/無効状態の視覚的区別、ツールチップで無効理由表示
- **生成失敗時**: エラーメッセージ + 関与ノード/制約へのナビゲーション

### 7.3 → wasm Boundary Agent

- **check_generatability API**: 静的検査の範囲と計算量
- **generate_sample_with_level API**: 保証レベル計算のタイミング（事前 vs 事後）
- **診断API**: 生成可能性診断とvalidation診断の統合/分離

### 7.4 → Critical Review Agent

- **SumBound MVP必須判断**: AtCoderカバレッジ観点でのSumBound対応優先度検証
- **L2失敗時のUI方針**: 再試行促進 vs エラー表示の判断基準
