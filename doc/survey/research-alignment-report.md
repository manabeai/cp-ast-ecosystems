# 現設計と先行研究の照合レポート

> 対象: `doc/plan/ast-draft.md`（StructureAST / ConstraintsAST 設計ドラフト）
> 照合元: `doc/reference.md`（先行研究メモ）+ 関連文献
> 評価視点: 研究として通すために、何が先行研究の再発明で、何が新規性として主張できるか

---

## 1. 現設計と先行研究の対応表

| # | 設計概念 | 対応する先行研究 | 差異・備考 |
|---|---------|----------------|-----------|
| 1 | **Structure-first（AST が正本、テキストは投影）** | Projectional editing 全般（Voelter 2015; grammar cells; JSON-based DSL editors 2023）; MPS (JetBrains) | 本設計が述べている内容は projectional editing の定義そのもの。MPS は 2003 年から同じ思想で実用化されている。差異はドメイン（競プロ仕様記述）への適用。 |
| 2 | **StructureAST（構造の骨格表現）** | Hazelnut の zippered AST with holes (Omar et al. 2017); projectional editor の abstract syntax tree | Hazelnut は lambda calculus 上で形式化。本設計は型付き lambda calculus ではなく DSL スキーマ。ノード種別が汎用言語ではなく競プロドメイン固有（配列・値域・テストケース等）である点が差異。 |
| 3 | **ConstraintsAST（許容範囲の宣言的表現）** | 部分的に GHC typed holes の expected type + relevant bindings; 部分的に attribute grammars の synthesized/inherited attributes; 部分的に SMT-based constraint systems | **最も対応関係が曖昧な概念。** Hazelnut は hole の期待型を双方向型検査で導出するが、slot role・能力制約・文脈依存候補の合成までは扱わない。本設計が言う「制約の合成 = 型制約 ∩ 構造制約 ∩ 文脈制約 ∩ 能力制約」は attribute grammar の constraint propagation や refinement type に近い。だが ConstraintsAST という名前で一つの AST として分離する設計は先行研究に直接の対応がない。 |
| 4 | **Hole（第一級の未完成ノード）** | GHC typed holes; Hazelnut holes; Hazel の hole semantics (Omar et al. 2019) | ほぼ再発明。Hazelnut が「すべての編集状態が statically meaningful」であることを形式的に証明済み。本設計の hole の哲学は Hazelnut の定理の非形式的な再記述に近い。 |
| 5 | **Slot-centric design（名前付きスロット）** | MPS の cell model; grammar cells (Voelter et al.); Hazelnut の cursor-relative editing | MPS では各ノードが名前付き cell を持ち、エディタがそれに基づいて UI を生成する。grammar cells は slot 単位でテキスト風の編集体験を与える。slot 概念自体は新しくない。 |
| 6 | **Projection / View（構造と制約からの投影）** | Projectional editing の定義そのもの（Voelter 2015; MPS; Spoofax） | View を「独自の意味論を持たない薄い層」とする記述は projectional editing の標準的定義と一致。 |
| 7 | **Canonical rendering（AST → 決定的な外部表記）** | Pretty-printing の研究（Wadler 2003 "A prettier printer"）; projectional editor の notation definition | canonical rendering 自体は古典的。ただし「競プロサイト風制約欄への決定的再構成」という具体的応用先は先行研究にない。 |
| 8 | **Sample generation（AST → 制約充足するランダムテストケース）** | QuickCheck (Claessen & Hughes 2000) の property-based testing; SmallCheck; Feat (functional enumeration of algebraic types); constraint-based test generation (SMT solvers) | 制約からのランダム生成は property-based testing の中核。本設計の「ConstraintsAST からサンプル生成」は QuickCheck の Arbitrary + property の構造に概念的に近い。差異は「仕様 AST から直接生成」というパスの統合。 |
| 9 | **Editable-core first（GUI 専用ではなくコアが編集可能）** | Hazelnut の action semantics; MPS の editing model | Hazelnut は AST 操作を action として形式化し、特定の UI に依存しない。MPS も内部モデルと editor を分離する。本設計の方針は先行研究の標準的アーキテクチャ。 |
| 10 | **NodeId（安定識別子による差分・履歴）** | Roslyn (C# compiler) の SyntaxNode identity; tree-sitter の edit-friendly tree; immutable persistent data structures (Okasaki) | 編集可能 AST に安定 ID を付与するのは構造エディタの標準技法。 |
| 11 | **型システムと制約システムの分担** | Refinement types (Liquid Haskell); dependent types; bidirectional type checking (Hazelnut) | 「型で排除できるものは型で、文脈依存なものは制約で」という分担は、refinement types が形式化している領域。ただし本設計は Rust の型システム（phantom types）と別レイヤの制約 AST の組み合わせであり、refinement types とは異なるアーキテクチャ。 |
| 12 | **競プロ問題の形式仕様 DSL** | Polygon (Codeforces) のテストケース生成; testlib.h; Rime (AtCoder 周辺ツール); problem format standards (ICPC CMS 等) | 既存の競プロツールは手続き的生成器（C++ ジェネレータ）が主流。宣言的 AST としての仕様記述は先行例が少ない。ただし「競プロ問題を構造化する」試みは実務ツールとして散在している。 |

---

## 2. 見落としている既存知見（取り込むべきもの）

### 2.1 JetBrains MPS の設計知見

本設計は projectional editing の概念に強く依拠しているが、reference.md に **MPS (Meta Programming System)** への言及がない。MPS は 2003 年から開発されている最も成熟した projectional editor であり、本設計が述べている概念（AST が正本、named slots/cells、projection としての view、type system integration）のほぼすべてを 20 年以上にわたって実装・改良してきた。

**取り込むべき知見:**
- Cell model: ノードごとに複数の cell (editor cell, inspector cell) を持つ設計
- Constraints language: MPS には constraints aspect があり、本設計の ConstraintsAST と比較対象になる
- Type system aspect: MPS の型システムは推論規則を宣言的に記述する
- Generator aspect: AST → テキストの変換規則を宣言的に定義

**評価への影響:** MPS を知らずに「AST を正本にして view は投影」と主張すると、査読者から「MPS でやればいいのでは？」と指摘される可能性が高い。MPS との差異を明確にすべき。

### 2.2 Attribute Grammars

ConstraintsAST の「制約をスロットや hole に紐づけ、文脈依存の情報を伝搬する」という設計は、attribute grammar の inherited / synthesized attributes と本質的に同じ問題を解いている。Reference grammars (Knuth 1968) および現代的な実装 (Silver, JastAdd) との関係を整理すべき。

**特に:** 「候補集合 = 型制約 ∩ 構造制約 ∩ 文脈制約 ∩ 能力制約」という合成は、attribute grammar の attribute evaluation の特殊ケースとして理論的に位置づけられる。

### 2.3 Schema-Driven Editing の文脈

reference.md に「Projectional Editors for JSON-Based DSLs (2023)」は挙がっているが、より広い schema-driven editing の文脈が不足している:
- XML Schema → Eclipse editor 生成（EMF/GMF）
- JSON Schema → form 生成（react-jsonschema-form 等）
- Protocol Buffers / GraphQL schema → editor 支援

本設計の「StructureAST のスキーマから編集可能な UI を導出する」は、このパターンの特殊ケースである。

### 2.4 Property-Based Testing / Constraint-Based Generation

sample generation の部分について、以下の文献系譜が完全に欠落している:
- **QuickCheck** (Claessen & Hughes 2000): ランダムテスト生成の原点
- **SmallCheck** (Runciman et al. 2008): 網羅的小規模テスト
- **Feat** (Duregård et al. 2012): 代数的データ型の functional enumeration
- **Luck** (Lampropoulos et al. 2017): 制約付きランダム生成の言語

「制約を満たすランダムデータを生成する」は property-based testing コミュニティが 20 年以上取り組んでいる問題であり、本設計の sample generation はこの系譜に正確に位置づけられるべき。

### 2.5 Refinement Types / Liquid Types

ConstraintsAST が「値域制約」「比較可能性」「文脈依存の能力制約」を表現するという設計は、refinement types (Rondon et al. 2008, Liquid Haskell) や dependent types が形式化してきた領域と重なる。「型で守るもの」と「ConstraintsAST で守るもの」の分担は、base type と refinement の関係に対応する。

### 2.6 Bidirectional Type Checking の詳細

Hazelnut が依拠する bidirectional type checking (Pierce & Turner 2000) の技法は、hole の期待型導出において中心的な役割を果たす。本設計が「hole の位置に要求される型」を導出すると述べている部分は、bidirectional type checking の synthesis/checking mode の切り替えに対応する。形式化を考えるなら避けて通れない。

---

## 3. 再発明に見える箇所

以下は、本設計が独自に述べているが、先行研究で既に同等以上の設計・形式化が存在する箇所である。

### 3.1 「不正状態は事後検出より事前排除を優先する」

**先行研究:** Hazelnut (Omar et al. 2017) の中核定理そのもの。Hazelnut は「すべての edit state が well-typed である」ことを証明している。本設計の記述は Hazelnut の Theorem 1 (Sensibility) の非形式的な言い換えに近い。

**問題:** 本設計はこの性質を「方針」として述べているが、Hazelnut は action semantics を定義し、型保存を形式的に証明している。方針として掲げるだけでは不十分であり、Hazelnut を引用した上で、本設計がどの程度の保証を与えるのか（形式的証明まで行うのか、テストベースか、型システムによる部分的保証か）を明示すべき。

### 3.2 「未完成状態は第一級の存在である」

**先行研究:** Hazelnut holes + Live Functional Programming with Typed Holes (Omar et al. 2019)。hole を「例外ではなく第一級の要素」として扱うことは、これらの研究の出発点である。

**問題:** 本設計の hole の記述は Hazelnut/Hazel の hole semantics をほぼそのまま再記述している。引用なしにこれを述べると、再発明の印象を与える。

### 3.3 「View は projection であり正本ではない」

**先行研究:** Projectional editing の定義そのもの (Voelter 2015; MPS)。

**問題:** これは projectional editing の文字通りの定義である。独自の設計判断として記述するのではなく、「本設計は projectional editing のアーキテクチャを採用する」と明示的に述べるべき。

### 3.4 Slot-centric design

**先行研究:** MPS の cell model; grammar cells (Voelter et al.)。

**問題:** slot/cell 概念は MPS が 20 年前から実装している。grammar cells 論文は slot 単位の編集体験を研究している。名前付きスロットは構造エディタの標準設計。

### 3.5 NodeId と安定識別子

**先行研究:** Roslyn (Microsoft, 2014) の immutable syntax tree with identity; persistent data structures 全般。

**問題:** 編集可能な AST にノード ID を付与するのは標準技法であり、設計上の判断として特筆すべき点ではない。

### 3.6 「構造は文字列より先にある」

**先行研究:** Structure editing (1981〜); projectional editing の全系譜。

**問題:** これは structure editing / projectional editing の基本テーゼであり、40 年以上の歴史がある。

---

## 4. 新規性が主張できそうな箇所

厳密に評価すると、個々の概念レベルでの新規性は限定的である。しかし以下の組み合わせ・応用には新規性を主張できる余地がある。

### 4.1 StructureAST / ConstraintsAST の二層分離（条件付き）

**主張可能な点:** 構造（何があるか）と制約（何が許されるか）を別の AST として明示的に分離する設計は、Hazelnut（型情報は syntax に埋め込み）や MPS（constraints aspect は type system aspect と並列だが AST としては一体）とは異なるアーキテクチャ上の選択である。

**条件:** この分離がもたらす具体的な利点（拡張性、候補列挙の柔軟性、制約の合成可能性）を、MPS の constraints aspect や attribute grammar との比較で実証する必要がある。「分けた方がきれい」では研究にならない。

**正直な評価:** 新規性は「設計判断」のレベルであり、「理論的貢献」ではない。

### 4.2 制約の合成による候補導出

**主張可能な点:** 「候補集合 = 型制約 ∩ 構造制約 ∩ 文脈制約 ∩ 能力制約」という合成可能な候補フィルタリングは、IDE の code completion とは異なる宣言的な枠組みを提供する可能性がある。

**条件:** これが attribute grammar の attribute evaluation や refinement types の subtyping 以上の何かであることを示す必要がある。現時点では構想段階であり、具体的な制約言語の設計も形式意味論もない。

### 4.3 競プロ仕様記述への structure editor 概念の適用

**主張可能な点:** Hazelnut/livelits/projectional editing の概念を、汎用プログラミング言語ではなく**競技プログラミング問題の形式仕様記述**というドメインに適用するのは、先行研究にない組み合わせである。

**これが最も主張しやすい新規性。** ただし以下に注意:
- 「ドメインが新しい」だけでは、研究としてのインパクトが限定的
- 競プロドメインが structure editor 概念の応用先として**なぜ興味深いか**を論じる必要がある（ドラフトの「評価に適している」という議論はこの方向で正しい）

### 4.4 単一 AST からの三方向投影

**主張可能な点:** 同一の AST から (1) 人間可読な仕様テキスト（canonical rendering）、(2) 制約充足するランダムテストケース（sample generation）、(3) 編集可能な GUI を導出するという三方向の投影は、個々には既存だが組み合わせとしてはまとまった先行研究がない。

**条件:** (1) は pretty-printing、(2) は property-based testing、(3) は projectional editing であり、それぞれ独立には既存。「統合すること自体の価値」を示す必要がある。

### 4.5 生成可能性を意識した制約設計

**主張可能な点:** ConstraintsAST が「禁止判定」だけでなく「生成」にも使えるように設計するという方針は、通常の型システムや制約系が検査（checking）に寄っているのに対し、生成（generation）を同じ記述から導出する点で差別化できる可能性がある。

**条件:** Luck (Lampropoulos et al. 2017) が「制約から直接サンプルを生成する言語」を既に提案している。QuickCheck の Arbitrary も型からの生成器である。本設計がこれらと何が違うかを明確にすべき。

---

## 5. 論文引用候補一覧

### 必須引用（本設計の直接の先行研究）

| # | 文献 | 関連概念 | 引用理由 |
|---|------|---------|---------|
| 1 | Omar et al., "Hazelnut: A Bidirectionally Typed Structure Editor Calculus," POPL 2017 | hole, structure editing, action semantics, type safety of edit states | 本設計の hole / structure-first / 不正状態排除の理論的基盤。引用なしには成立しない。 |
| 2 | Omar et al., "Live Functional Programming with Typed Holes," ICFP 2019 | live evaluation of incomplete programs, hole closures | 未完成状態の動的意味論。将来的な live evaluation に繋がる。 |
| 3 | Omar et al., "Filling Typed Holes with Live GUIs," PLDI 2021 | livelits, GUI-based hole filling, typed GUI generation | hole に対する GUI 表面化。本設計の GUI 編集構想の直系。 |
| 4 | Voelter et al., "A General Architecture for Heterogeneous Language Engineering and Projectional Editor Support," 2015 | projectional editing architecture, abstract syntax as source of truth | structure-first / view-as-projection の標準的定義。 |
| 5 | Voelter et al., "Efficient Development of Consistent Projectional Editors using Grammar Cells," SLE 2016 | grammar cells, slot-centric editing, text-like editing in structure editors | slot-centric design の先行研究。使いやすい構造編集。 |

### 強く推奨する引用

| # | 文献 | 関連概念 | 引用理由 |
|---|------|---------|---------|
| 6 | Berger et al., "Projectional Editors for JSON-Based DSLs," 2023 | JSON schema → projectional editor, DSL editor generation | 本設計に最も近い recent work。スキーマから editor を導出する。 |
| 7 | Claessen & Hughes, "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs," ICFP 2000 | property-based random test generation | sample generation の系譜の出発点。 |
| 8 | Osera & Zdancewic, "Type-and-Example-Directed Program Synthesis," PLDI 2015 | type-directed synthesis, hole filling via examples | hole completion の合成的アプローチ。 |
| 9 | Lubin et al., "Program Sketching with Live Bidirectional Evaluation," ICFP 2020 | live sketch evaluation, example-driven hole filling | 制約からの hole 補完。 |
| 10 | GHC Typed Holes (GHC User's Guide) | typed holes, expected type, relevant bindings | 実務上最も普及した typed holes 実装。 |

### 引用すべき背景文献

| # | 文献 | 関連概念 | 引用理由 |
|---|------|---------|---------|
| 11 | Knuth, "Semantics of Context-Free Languages," 1968 | attribute grammars | ConstraintsAST の理論的背景として。 |
| 12 | Rondon et al., "Liquid Types," PLDI 2008 | refinement types, value-level constraints in types | 型制約と値制約の分担の理論的背景。 |
| 13 | Lampropoulos et al., "Luck: A Probabilistic Language for Testing," 2017 | constraint-based random generation | 制約からの生成の直接の先行研究。 |
| 14 | Voelter, "Language Workbenches: The Killer-App for DSLs," 2013 / MPS documentation | language workbench, projectional editing in practice | 最も成熟した projectional editor 実装。 |
| 15 | Omar et al., "Total Type Error Localization and Recovery with Holes," POPL 2024 | error recovery via holes, marking | 型エラーからの回復。本設計の robustness に関連。 |
| 16 | Edwards, "Example-Centric Programming," OOPSLA 2004 | example as primary artifact | example-driven 開発の祖先。 |
| 17 | Wadler, "A Prettier Printer," 2003 | pretty-printing algebra | canonical rendering の理論的背景。 |
| 18 | Reps & Teitelbaum, "The Synthesizer Generator," 1984 | syntax-directed editor generation, attribute grammars | 構造エディタ生成の古典。ConstraintsAST の propagation と関連。 |

---

## 6. 「どこまでなら言ってよくて、どこからは言いすぎか」の境界

### ✅ 言ってよいこと

1. **「Hazelnut / livelits / projectional editing の概念を、競技プログラミング問題の形式仕様記述ドメインに適用する」**
   - ドメイン適用としての新規性は正当に主張できる。
   - ただし「適用した」だけでは contribution としては弱い。適用によって見えた知見（ドメイン固有の課題、汎用理論の限界）を示すべき。

2. **「StructureAST と ConstraintsAST を明示的に分離する設計を採用し、その利点を評価する」**
   - 設計判断としてのトレードオフ分析は contribution になり得る。
   - 「分離することで候補列挙が柔軟になった」等の具体的な利点を実証すれば主張可能。

3. **「単一の AST から canonical rendering と sample generation の両方を導出可能にする」**
   - 統合の有用性を実証すれば主張可能。
   - 「pretty-printing と test generation をそれぞれ別々に実装するのではなく、共通の AST から導出することの利点」を示す。

4. **「競プロ問題を、structure editor 概念の応用先として評価に適したドメインとして位置づける」**
   - ドメインの選択根拠として正当。

### ⚠️ 言い方に注意が必要なこと

5. **「hole を第一級に扱う」**
   - 言ってよいが、**必ず Hazelnut を引用し、本設計が Hazelnut の理論の応用であることを明示すべき**。
   - ❌ 「本設計では hole を第一級に扱うという新しいアプローチを提案する」
   - ✅ 「Hazelnut (Omar et al. 2017) に倣い、hole を第一級の構造要素として扱う。本設計では、これを競プロ仕様記述のドメインに適用し、...」

6. **「制約からの候補導出」**
   - IDE の code completion、attribute grammar、refinement types との関係を整理した上で述べるべき。
   - ❌ 「制約を合成して候補を導出する新しい仕組みを提案する」
   - ✅ 「attribute grammar の constraint propagation や IDE の code completion と類似の候補導出を、ConstraintsAST の宣言的記述から行う。本設計では特に...」

7. **「生成可能保証」**
   - property-based testing (QuickCheck) と constraint-based generation (Luck) を引用した上で、本設計の生成器がこれらとどう異なるか（あるいは同じか）を正直に述べるべき。
   - 条件付き保証であることはドラフトで正直に書かれており、この姿勢は維持すべき。

### ❌ 言いすぎになること

8. **「AST を正本にして view は投影にするという新しいアーキテクチャ」**
   - これは projectional editing の定義であり、40 年以上の歴史がある。新しいとは言えない。

9. **「不正状態を構築不能にするという新しい設計原則」**
   - Hazelnut の Sensibility 定理の再発明。本設計はこの性質の形式的証明を持たないため、Hazelnut より弱い主張しかできない。

10. **「typed holes の概念を構造エディタに導入する」**
    - Hazelnut が 2017 年に完了している仕事。

11. **「構造と制約の分離は先行研究にない新しいアーキテクチャ」**
    - MPS の aspects (structure, constraints, type system, editor) は同様の分離を行っている。分離自体は新しくない。分離の**仕方**や**粒度**に差異がある可能性はあるが、それを主張するなら MPS との具体的な比較が必要。

12. **「宣言的制約から生成器を自動導出する」**
    - 現時点で実装がないので、できると断言するのは言いすぎ。「目指す」は可。

### 📊 新規性の強さの整理

| 主張 | 新規性レベル | 条件 |
|------|------------|------|
| 競プロ仕様記述への構造エディタ概念の適用 | **中** | ドメイン固有の知見を示せれば |
| StructureAST / ConstraintsAST 分離 | **低〜中** | MPS との比較で差異を実証すれば |
| 単一 AST → rendering + generation + editing | **中** | 統合の利点を実証すれば |
| 制約の合成的候補導出 | **低** | attribute grammar / refinement types との差異不明 |
| hole の第一級扱い | **なし** | Hazelnut の応用 |
| structure-first / view-as-projection | **なし** | projectional editing の定義 |
| 不正状態の事前排除 | **なし** | Hazelnut の定理 |

---

## 付録: 設計ドラフトへの具体的指摘

### A. 引用の欠如

設計ドラフトは先行研究への明示的な引用をほぼ含んでいない。reference.md に先行研究メモがあるが、ドラフト本文中で「この概念は X に由来する」という記述がない。研究論文として書く際には、各概念の出典を明示すべき。

### B. 形式化の不在

Hazelnut が action semantics の形式化と型保存の証明を持つのに対し、本設計は自然言語による方針記述に留まっている。研究として通すには、少なくとも:
- StructureAST / ConstraintsAST の形式的構文
- hole の well-formedness 条件
- 操作（action）の型保存性

のいずれかについて形式的定義が必要。すべてを Hazelnut 級に形式化する必要はないが、「何が保証されるか」を曖昧にしたままでは査読に耐えない。

### C. ConstraintsAST の具体性不足

ConstraintsAST は本設計で最も独自性がある可能性のある概念だが、具体的な制約言語の構文・意味論が定義されていない。「能力制約」「文脈依存候補」などの概念が列挙されているが、これらがどのように表現・評価・合成されるかが不明。

### D. 競プロドメイン特有の貢献の掘り下げ不足

競プロ問題の構造（配列・値域・依存関係・テストケース構造）が、structure editor / typed holes の理論にどのような新しい課題を提起するかの分析が薄い。例えば:
- 変数間の依存関係（配列長が別変数に依存）は、Hazelnut の lambda calculus 上の hole とは異なる制約伝搬を要求する
- 値域制約は refinement types 的であり、通常の typed holes の型推論とは異なる推論が必要
- sample generation は hole filling とは逆方向の問題（仕様→具体値 vs. 具体値→仕様）

これらの分析を深めることが、ドメイン適用としての貢献を強めるために重要。
