# AST TeX Renderer — Design Specification

## Goal

Build a TeX renderer module within `cp-ast-core` that produces deterministic, diff-stable competitive-programming-style TeX fragments from the existing AST and constraint types. Two primary outputs: constraint notation and input format notation.

## Non-Goals

- Natural language generation
- Platform-specific styling (AtCoder vs Codeforces)
- Intermediate representation (render-IR)
- Core type modifications (no new types in `structure/` or `constraint/`)
- PDF compilation or full document templates
- WASM/JSON boundary API (deferred)

---

## Architecture

The TeX renderer is a new module `render_tex/` within `cp-ast-core`, parallel to the existing `render/` module (plain text). It reads from `&AstEngine` immutably and produces `TexOutput` containing the TeX string and any warnings.

```
AstEngine (structure + constraints)
       │
       ├── render/          ← existing plain text renderer (untouched)
       │
       └── render_tex/      ← NEW: TeX renderer
            ├── mod.rs            — public API, types (TexOutput, TexOptions, TexWarning)
            ├── tex_helpers.rs    — expression→TeX, reference→TeX, ident→TeX, IndexAllocator
            ├── constraint_tex.rs — constraint notation TeX generation
            └── input_tex.rs      — input format TeX generation
```

No changes to existing modules. The TeX renderer depends only on `structure::*`, `constraint::*`, and `operation::AstEngine` (for read access).

---

## Public API

```rust
// crates/cp-ast-core/src/render_tex/mod.rs

/// Render constraint notation as TeX.
#[must_use]
pub fn render_constraints_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput;

/// Render input format as TeX.
#[must_use]
pub fn render_input_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput;

/// Render both input and constraint notation as a combined TeX fragment.
#[must_use]
pub fn render_full_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput;
```

### Types

```rust
/// TeX generation output.
#[derive(Debug, Clone)]
pub struct TexOutput {
    /// The generated TeX string.
    pub tex: String,
    /// Warnings encountered during generation.
    pub warnings: Vec<TexWarning>,
}

/// Options for TeX generation.
#[derive(Debug, Clone)]
pub struct TexOptions {
    /// Whether to include section headers (\paragraph{} wrappers).
    pub section_mode: SectionMode,
    /// Whether to render Hole nodes (if false, holes are silently skipped).
    pub include_holes: bool,
}

/// Section mode for TeX output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionMode {
    /// TeX fragments only (e.g., \begin{itemize}...\end{itemize}).
    Fragment,
    /// With section headers (e.g., \paragraph{入力}...\paragraph{制約}...).
    Standalone,
}

/// Warning emitted during TeX generation.
#[derive(Debug, Clone, PartialEq)]
pub enum TexWarning {
    /// A Hole node was encountered in the AST.
    HoleEncountered { node_id: NodeId },
    /// A constraint type is not supported for TeX rendering.
    UnsupportedConstraint { description: String },
    /// A Reference could not be resolved to a named node.
    UnresolvedReference { name: String },
}
```

`TexOptions` implements `Default`:
```rust
impl Default for TexOptions {
    fn default() -> Self {
        Self {
            section_mode: SectionMode::Fragment,
            include_holes: true,
        }
    }
}
```

---

## Constraint TeX Rules

### Rendering Order

Constraints are grouped and ordered by type (same as existing `render/constraint_text.rs`):

1. Range
2. ~~TypeDecl~~ (skipped — implicit in competitive programming)
3. LengthRelation
4. Relation
5. Distinct
6. Property (text output)
7. Sorted
8. SumBound
9. Guarantee, CharSet, StringLength
10. RenderHint (skipped)

### Constraint → TeX Mapping

| Constraint | TeX Output |
|---|---|
| `Range { target: N, lower: 1, upper: 2×10^5 }` | `$1 \le N \le 2 \times 10^{5}$` |
| `Range { target: A (array element), lower: 1, upper: 10^9 }` | `$1 \le A_i \le 10^{9} \ (1 \le i \le N)$` |
| `TypeDecl` | *(skipped — not displayed)* |
| `LengthRelation { target: S, length: N }` | `$\|S\| = N$` |
| `Relation { lhs, op, rhs }` | `$\text{lhs} \text{op} \text{rhs}$` with index range if applicable |
| `Distinct { elements: A }` | `$A_i \neq A_j \ (i \neq j)$` |
| `SumBound { variable: N, upper: 2×10^5 }` | `$\sum N \le 2 \times 10^{5}$` |
| `Sorted { elements: A, order: Ascending }` | `$A_1 \le A_2 \le \cdots \le A_N$` |
| `Guarantee { description }` | description text as-is |
| `CharSet { target: S, charset: LowerAlpha }` | `$S$ は英小文字からなる` |
| `StringLength { target: S, min: 1, max: N }` | `$1 \le \|S\| \le N$` |
| `Property { target, tag: Simple }` | descriptive text (e.g., "与えられるグラフは単純") |
| `RenderHint` | *(skipped)* |

### Array Element Index Range

When a constraint targets a Reference that resolves to an Array or a variable inside a Repeat body, the TeX renderer automatically appends an index range annotation:

- The index variable is determined by the `IndexAllocator` (see TeX Helpers)
- Format: `\ (1 \le {index} \le {count})`
- Example: `$1 \le D_i \le 10^{9} \ (1 \le i \le N)$`

### Overall Constraint Layout

```tex
\begin{itemize}
  \item $1 \le N \le 2 \times 10^{5}$
  \item $1 \le Q \le 2 \times 10^{5}$
  \item $1 \le D_i \le 10^{9} \ (1 \le i \le N)$
  \item $1 \le T_j \le N \ (1 \le j \le Q)$
\end{itemize}
```

---

## Input Format TeX Rules

### NodeKind → TeX Mapping

| NodeKind | TeX Line(s) | Example |
|---|---|---|
| `Scalar { name: "N" }` | `N` | Single variable on a line |
| `Tuple { [N, M] }` | `N \ M` | Space-separated on one line |
| `Array { name: "A", length: N }` | `A_1 \ A_2 \ \cdots \ A_N` | Ellipsis pattern |
| `Matrix { name: "C", rows: H, cols: W }` | Multi-row: `C_{1,1} \ C_{1,2} \ \cdots \ C_{1,W}` / `C_{2,1} \ \cdots \ C_{2,W}` / `\vdots` / `C_{H,1} \ \cdots \ C_{H,W}` | |
| `Repeat { count: Q, body: [Scalar T] }` | `T_1` / `T_2` / `\vdots` / `T_Q` | Vertical expansion |
| `Repeat { count: M, body: [Tuple(u,v)] }` | `u_1 \ v_1` / `u_2 \ v_2` / `\vdots` / `u_M \ v_M` | Indexed vertical expansion |
| `Section` | Render body recursively | Transparent container |
| `Sequence` | Render children recursively | Transparent container |
| `Hole` | `\texttt{<hole>}` | Visible placeholder |
| `Choice { tag, variants }` | `\texttt{(choice)}` (placeholder until Phase 2 T-06) | Deferred |

### Repeat Expansion Pattern

For `Repeat { count: C, body }`:

1. **Single Scalar body**: Vertical with subscripts
   ```
   T_1
   T_2
   \vdots
   T_C
   ```

2. **Single Tuple body**: Indexed vertical
   ```
   u_1 \ v_1
   u_2 \ v_2
   \vdots
   u_C \ v_C
   ```

3. **Multiple body elements**: Each body element gets its own vertical expansion block.

### Overall Input Format Layout

```tex
\[
\begin{array}{l}
N \ Q \\
A_1 \ A_2 \ \cdots \ A_N \\
u_1 \ v_1 \\
u_2 \ v_2 \\
\vdots \\
u_M \ v_M
\end{array}
\]
```

- Line separator: `\\`
- Element separator within a line: `\ ` (backslash-space)
- Wrapped in `\[ \begin{array}{l} ... \end{array} \]`

---

## TeX Helpers (`tex_helpers.rs`)

### Expression → TeX (`expression_to_tex`)

| Expression | TeX Output |
|---|---|
| `Lit(1)` | `1` |
| `Lit(200000)` | `2 \times 10^{5}` (auto-decomposition) |
| `Var(Reference::VariableRef(N))` | `N` |
| `BinOp(Mul, Lit(2), Pow(Lit(10), Lit(5)))` | `2 \times 10^{5}` |
| `BinOp(Add, Var(N), Lit(1))` | `N + 1` |
| `BinOp(Sub, Var(N), Lit(1))` | `N - 1` |
| `Pow(Lit(10), Lit(9))` | `10^{9}` |
| `FnCall("min", [N, M])` | `\min(N, M)` |

#### Large Number Auto-Decomposition

For `Lit(n)` where n is large:

1. Check if `n` is a power of 10: `10^k` → `10^{k}`
2. Check if `n = a × 10^k` where `a < 10` and `k ≥ 2`: → `a \times 10^{k}`
3. Otherwise: render as plain number

Note: Composite expressions like `10^9 + 7` should be constructed as `BinOp(Add, Pow(10,9), Lit(7))` in the AST, not as `Lit(1000000007)`. The auto-decomposition only handles pure `a × 10^k` forms from `Lit(n)`.

### Reference → TeX (`reference_to_tex`)

| Reference | TeX Output |
|---|---|
| `VariableRef(node)` where node is `Scalar { name }` | `{name}` |
| `VariableRef(node)` where node is `Array { name, .. }` | `{name}` |
| `IndexedRef { target, indices: [i] }` | `{name}_{i}` |
| `IndexedRef { target, indices: [i, j] }` | `{name}_{i,j}` |
| `Unresolved(ident)` | `{ident}` + warning |

### Ident → TeX (`ident_to_tex`)

| Ident | TeX Output |
|---|---|
| `"N"` (single uppercase) | `N` |
| `"x"` (single lowercase) | `x` |
| `"ans"` (multi-char) | `\mathrm{ans}` |
| `"ABC"` (multi-char uppercase) | `\mathrm{ABC}` |

Rule: single character → as-is (math italic); multiple characters → `\mathrm{}`.

### Index Allocator (`IndexAllocator`)

```rust
struct IndexAllocator {
    next: char,  // starts at 'i'
}

impl IndexAllocator {
    fn new() -> Self { Self { next: 'i' } }
    fn allocate(&mut self) -> char {
        let c = self.next;
        self.next = (self.next as u8 + 1) as char;  // i, j, k, l, ...
        c
    }
}
```

- Each Array or Repeat allocates an index variable
- Variables within the same Repeat share the same index
- Allocation order is determined by AST traversal order (deterministic)
- Used in both input format (subscripts) and constraint notation (index ranges)

---

## Full Fragment (`render_full_tex`)

When `section_mode == Standalone`:

```tex
\paragraph{入力}
\[
\begin{array}{l}
...
\end{array}
\]

\paragraph{制約}
\begin{itemize}
  \item ...
\end{itemize}
```

When `section_mode == Fragment`:

Returns input TeX + blank line + constraint TeX concatenated.

---

## Error Handling

The TeX renderer never panics. All error conditions produce output + warnings:

| Condition | Output | Warning |
|---|---|---|
| Hole node | `\texttt{<hole>}` | `HoleEncountered { node_id }` |
| Unresolved Reference | `\texttt{?name}` | `UnresolvedReference { name }` |
| Missing node (deleted) | `\texttt{<?>}` | `UnresolvedReference` |
| Unsupported Constraint | Text description as-is | `UnsupportedConstraint` |

---

## Determinism and Diff-Stability

The TeX renderer guarantees:

1. **Same AST → same output**: No randomness, no timestamp, no external state
2. **Stable ordering**: Constraints ordered by type category, then by insertion order within category
3. **Stable index allocation**: Determined by AST structure traversal order
4. **Fixed abbreviation rules**: `\cdots` for horizontal, `\vdots` for vertical — always
5. **Fixed number formatting**: Large number decomposition rules are deterministic

---

## Interaction with Existing Code

### No changes to existing modules

- `render/` — untouched
- `structure/` — untouched
- `constraint/` — untouched
- `operation/` — untouched (read-only access via `&AstEngine`)
- `sample/` — untouched

### New files only

- `src/render_tex/mod.rs`
- `src/render_tex/tex_helpers.rs`
- `src/render_tex/constraint_tex.rs`
- `src/render_tex/input_tex.rs`
- `src/lib.rs` — add `pub mod render_tex;`
- `tests/render_tex_basic.rs` — golden tests

---

## Test Strategy

All tests use exact string comparison (golden tests).

### Constraint TeX Tests

1. **Scalar range**: `1 \le N \le 10^{5}` (basic Range)
2. **Array element range + index**: `1 \le A_i \le 10^{9} \ (1 \le i \le N)`
3. **Multiple constraints ordering**: Ranges before Distinct before Guarantee
4. **SumBound**: `\sum N \le 2 \times 10^{5}`
5. **Sorted**: `A_1 \le A_2 \le \cdots \le A_N`
6. **Distinct**: `A_i \neq A_j \ (i \neq j)`
7. **TypeDecl skipped**: Not in output
8. **RenderHint skipped**: Not in output
9. **Expression decomposition**: `2 \times 10^{5}`, `10^{9} + 7`
10. **StringLength**: `1 \le \|S\| \le N`
11. **Unsupported warning**: Warning emitted for unknown constraints

### Input Format TeX Tests

1. **Single scalar**: `N`
2. **Tuple**: `N \ M`
3. **Array**: `A_1 \ A_2 \ \cdots \ A_N`
4. **Repeat with scalar body**: Vertical `T_1`, `T_2`, `\vdots`, `T_Q`
5. **Repeat with tuple body**: `u_1 \ v_1`, `\vdots`, `u_M \ v_M`
6. **Matrix**: Multi-row with `\vdots`
7. **Hole**: `\texttt{<hole>}` + warning
8. **Combined N + Array + Repeat**: Full input layout in `\begin{array}`

### End-to-End Test

- Build ABC284-C equivalent via operations
- Render TeX and compare against expected full fragment

---

## Implementation Order

1. **`tex_helpers.rs`** — Foundation: expression_to_tex, reference_to_tex, ident_to_tex, IndexAllocator
2. **`constraint_tex.rs`** — Constraint TeX rendering (uses helpers)
3. **`input_tex.rs`** — Input format TeX rendering (uses helpers + IndexAllocator)
4. **`mod.rs`** — Public API, types, render_full_tex
5. **Tests** — Golden tests for each component + e2e

---

## Open Questions

1. **Choice rendering detail**: The exact TeX format for Choice nodes (query-type branching) is deferred — the current plain text renderer also uses a placeholder `(choice)`. This will be addressed when Choice improvements land (Phase 2 T-06 or later).

2. **Guarantee/Property text language**: Currently planned as Japanese text (e.g., "与えられるグラフは単純"). Whether to support English or make this configurable is deferred.

3. **Matrix detail rendering**: For large matrices, the exact row abbreviation pattern (show first row, last row, vdots) needs validation against real competitive programming problems.
