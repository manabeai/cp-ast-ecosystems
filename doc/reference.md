# Hole / Livelits / 構造編集まわりの先行研究メモ

いま話している発想は、だいたいこの流れに乗っている。

**構造編集** → **projectional editor** → **typed holes** → **不完全プログラムの live 実行** → **GUI で hole を埋める livelits** → **例や合成で hole を埋める**

---

## 1. 構造編集の古典

### Syntax-Directed Editing of General Data Structures（1981）
最古典寄りの源流。  
「テキストを自由入力して後から直す」ではなく、**木構造を直接編集して syntactically correct な状態を保つ**という方向のかなり早い代表例。  
「GUIのためのAST」「不正状態を作らせない」にかなり近い祖先。

- Link: https://dl.acm.org/doi/pdf/10.1145/872730.806449

### CEPAGE: a full-screen structural editor
これも古い structural editor 系。  
構造化文書やプログラムを、普通のテキスト編集とは違う「構造編集」として扱う系譜にある。理論というより設計思想の祖先として見るとよい。

- Link: https://se.inf.ethz.ch/~meyer/publications/architext/cepage_cgl_en.pdf

---

## 2. Projectional Editing の系譜

### A General Architecture for Heterogeneous Language Engineering and Projectional Editor Support（2015）
projectional editor を、**abstract syntax を中心に concrete syntax や表示を投影する**ものとして整理している。  
「StructureAST / ConstraintAST を正本にして View は薄くする」という考え方にかなり直結する。

- Link: https://arxiv.org/pdf/1506.03398

### Efficient Development of Consistent Projectional Editors（grammar cells）
projectional editor を実用レベルで気持ちよく使うための編集体験の研究。  
「構造編集は正しいが使いにくい」問題に対して、テキストっぽい一貫した編集感をどう与えるかを扱う。GUI をただのラッパーで終わらせない観点で重要。

- Link: https://www.mathematik.uni-marburg.de/~seba/publications/grammar-cells.pdf

### Projectional Editors for JSON-Based DSLs（2023）
JSON Schema 的な構造から projectional editor を導く最近の流れ。  
「DSL の構造定義から editor を導出する」という意味で、**ASTから編集可能な像を導く**発想にかなり近い。

- Link: https://arxiv.org/abs/2307.11260

---

## 3. Typed Holes の本流

### GHC Typed Holes
実務上もっとも有名な typed holes。  
hole に対して「この位置で期待される型」と「その場で使える relevant bindings」を返す。  
「hole に何を入れればよいか」を返す感覚の有名な出発点。

- Link: https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/typed_holes.html

### Hazelnut: A Bidirectionally Typed Structure Editor Calculus（2017）
超重要。  
typed lambda calculus に **holes + cursor** を入れた structure editor の形式化で、**すべての編集状態が statically meaningful** になるように action semantics を定義している。  
「GUI操作から AST を安全に作る」に最も近い理論的中核。

- Link: https://arxiv.org/pdf/1607.04180

---

## 4. 不完全プログラムを live に扱う流れ

### Live Functional Programming with Typed Holes（2019）
Hazelnut の静的意味論を、**未完成プログラムを実行できる動的意味論**へ拡張したもの。  
空の hole や型不一致を含んでいても、そこで即停止せず、**holes のまわりを評価し続ける**。  
さらに hole closure を追跡し、埋めた後に fill-and-resume できる。  
「未完成状態を第一級に扱う」発想のど真ん中。

- Link: https://arxiv.org/abs/1805.00155

### Total Type Error Localization and Recovery with Holes（2024）
typed holes / incomplete programs 系の最近の発展。  
型エラーを含む状態でも downstream semantic services を継続するために、**回復可能な意味論**を与える。  
構造編集や live feedback を本気でやるなら、この方向も重要。

- Link: https://hazel.org/papers/marking-popl24.pdf

---

## 5. livelits（live literals）の直系

### Filling Typed Holes with Live GUIs（PLDI 2021）
これが **livelits** 本人。  
特定の型の hole を埋めるときに、**型に対応する GUI を surfacing して direct manipulation で hole を埋める**。  
「hole に対する GUI」「GUI 用の AST / projection」がまさにここに繋がる。

- Link: https://hazel.org/papers/livelits-pldi2021.pdf

特に、
- typed holes
- live programming
- direct manipulation
- projectional / structure editing 的発想

が交差する位置にあるので、今の案に一番刺さる文献の一つ。

---

## 6. hole を「埋める」側の源流

### Type-and-Example-Directed Program Synthesis（2015）
hole をどう埋めるかを、人間の直接編集ではなく**型 + 例**から合成する流れの代表。  
後の Smyth や Hazel Assistant 系に繋がる。GUI で hole を埋めるのでなく、**例と型から候補を絞る**という別の支流。

- Link: https://www.cis.upenn.edu/~stevez/papers/OZ15.pdf

### Example-Directed Synthesis: A Type-Theoretic Interpretation（2016）
「例」は何なのかを型理論的に整理した paper。  
hole 埋めや synthesis をやるとき、example を単なるテスト入力集合ではなく、仕様の一部としてどう見るかを支える。

- Link: https://www.cis.upenn.edu/~stevez/papers/FOWZ16.pdf

### Program Sketching with Live Bidirectional Evaluation（2020）
未完成プログラムに例を与え、**hole を live に埋める**流れのかなり重要な paper。  
具体的な assertion の評価から input-output examples を取り出し、それを使って hole completion を guide する。  
競プロ DSL でも、将来 sample case や制約から hole completion をやるなら直結する。

- Link: https://jlubin.net/assets/icfp20.pdf

---

## 7. live programming / example-centric の祖先

### Example-Centric Programming（2004）
プログラマは具体例で理解する、という立場から IDE が例を中心に支援すべきだという古典。  
typed holes そのものではないが、**未完成な構築を concrete example で支える**発想の祖先として重要。

- Link: https://www.subtext-lang.org/OOPSLA04.pdf

### Subtext: Uncovering the Simplicity of Programming（2005）
「紙に書いたコードの延長」ではなく、**表現と実行が一体化した新しい programming medium** を目指した系譜。  
構造編集・ live feedback・ visual / direct manipulation の思想的祖先として読む価値が高い。

- Link: https://www.subtext-lang.org/OOPSLA05.pdf

### Babylonian-style Programming（2019）
一般-purpose source code に live examples を統合する研究。  
typed holes より「例」の側に寄っているが、**コードと振る舞いの距離を縮める**という意味で livelits や Hazel 系と相性がいい。

- Link: https://arxiv.org/abs/1902.00549

---

## 8. direct manipulation と hybrid editor の流れ

### Sketch-n-Sketch: Output-Directed Programming for SVG（2019）
「出力を直接いじるとプログラムが更新される」系。  
typed holes の paper ではないが、**programmatic and direct manipulation together** の代表で、livelits の「GUIで hole を埋める」感覚と親和性が高い。

- Link: https://jlubin.net/assets/uist19.pdf

### Fusing Direct Manipulations into Functional Programs（2024）
direct manipulation を functional program へどう統合するかの最近の発展。  
将来「GUI編集」と「AST / DSL」を深く融合したいなら、この系譜も押さえておくとよい。

- Link: https://xingzhang-pku.github.io/pub/POPL24.pdf

---

# あなたのアイデアとの対応表

## 1. GUIのためのAST / structure-first
- Syntax-Directed Editing of General Data Structures
- Projectional Editor Support
- Hazelnut

## 2. hole を未完成状態として正規に持つ
- GHC Typed Holes
- Hazelnut
- Live Functional Programming with Typed Holes

## 3. hole に GUI を出す
- Filling Typed Holes with Live GUIs

## 4. hole を例や合成で埋める
- Type-and-Example-Directed Program Synthesis
- Program Sketching with Live Bidirectional Evaluation

## 5. View を薄い投影にする
- Projectional Editor Support
- Projectional Editors for JSON-Based DSLs

---

# まず読む順番

1. **Hazelnut**
2. **Live Functional Programming with Typed Holes**
3. **Filling Typed Holes with Live GUIs**
4. **Projectional Editors for JSON-Based DSLs**
5. **Program Sketching with Live Bidirectional Evaluation**
6. 余裕があれば **Syntax-Directed Editing of General Data Structures** と **Subtext**

この順だと、

**構造編集 → hole → live 実行 → GUI hole → 合成**

の流れで頭に入る。

---

# 系譜を一行で言うと

**syntax-directed editor** が祖先で、  
そこから **projectional editor** が育ち、  
そこに **typed holes** を入れて形式化したのが **Hazelnut**、  
それを live 実行に拡張したのが **Hazel / Live Functional Programming with Typed Holes**、  
さらに GUI で hole を埋める方向が **livelits**、  
例や合成で hole を埋める方向が **Smyth / type-and-example-directed synthesis**。