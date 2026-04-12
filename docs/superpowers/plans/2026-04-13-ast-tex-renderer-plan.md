# AST TeX Renderer — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `render_tex` module to `cp-ast-core` that generates deterministic competitive-programming-style TeX fragments for constraint notation and input format notation.

**Architecture:** A new module `render_tex/` parallel to existing `render/`, containing: `mod.rs` (public API + types), `tex_helpers.rs` (expression/reference/ident→TeX conversion + `IndexAllocator`), `constraint_tex.rs` (constraint→TeX), `input_tex.rs` (input format→TeX). No changes to any existing module.

**Tech Stack:** Rust 2021, cargo clippy pedantic, golden tests with exact string comparison.

---

## File Structure

### New files
- `crates/cp-ast-core/src/render_tex/mod.rs` — Public API: `render_constraints_tex`, `render_input_tex`, `render_full_tex`; types: `TexOutput`, `TexOptions`, `SectionMode`, `TexWarning`
- `crates/cp-ast-core/src/render_tex/tex_helpers.rs` — `expression_to_tex`, `reference_to_tex`, `ident_to_tex`, `IndexAllocator`, `decompose_large_number`
- `crates/cp-ast-core/src/render_tex/constraint_tex.rs` — `render_constraints_tex_impl`: constraint→TeX for each `Constraint` variant
- `crates/cp-ast-core/src/render_tex/input_tex.rs` — `render_input_tex_impl`: recursive AST walk producing `\begin{array}` layout
- `crates/cp-ast-core/tests/render_tex_basic.rs` — Golden tests for all TeX rendering

### Modified files
- `crates/cp-ast-core/src/lib.rs:4` — Add `pub mod render_tex;` after existing `pub mod render;`

---

## Existing Code Reference

### Key imports used by existing tests (`tests/render_basic.rs`)
```rust
use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::{render_constraints, render_input};
use cp_ast_core::structure::{Ident, NodeKind, Reference};
```

### How ASTs are constructed in tests
```rust
let mut engine = AstEngine::new();
let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
// Set root children
if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
    root.set_kind(NodeKind::Sequence { children: vec![n_id] });
}
// Add constraint
engine.constraints.add(Some(n_id), Constraint::Range {
    target: Reference::VariableRef(n_id),
    lower: Expression::Lit(1),
    upper: Expression::Lit(100),
});
```

### Constraint iteration
```rust
for (_, constraint) in engine.constraints.iter() { ... }
```

### StructureAst traversal
```rust
engine.structure.get(node_id) -> Option<&StructureNode>
node.kind() -> &NodeKind
engine.structure.root() -> NodeId
```

### Build & test commands
```bash
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets
```

---

## Task T-01: Module skeleton and types

**Files:**
- Create: `crates/cp-ast-core/src/render_tex/mod.rs`
- Create: `crates/cp-ast-core/src/render_tex/tex_helpers.rs`
- Create: `crates/cp-ast-core/src/render_tex/constraint_tex.rs`
- Create: `crates/cp-ast-core/src/render_tex/input_tex.rs`
- Modify: `crates/cp-ast-core/src/lib.rs:4`
- Test: `crates/cp-ast-core/tests/render_tex_basic.rs`

- [ ] **Step 1: Write a failing test that imports the new module**

Create `crates/cp-ast-core/tests/render_tex_basic.rs`:

```rust
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render_tex::{
    render_constraints_tex, render_input_tex, render_full_tex,
    SectionMode, TexOptions, TexOutput, TexWarning,
};
use cp_ast_core::structure::{Ident, NodeKind};

#[test]
fn empty_engine_produces_empty_tex() {
    let engine = AstEngine::new();
    let options = TexOptions::default();

    let input_result = render_input_tex(&engine, &options);
    assert!(input_result.tex.is_empty() || input_result.tex.trim().is_empty());
    assert!(input_result.warnings.is_empty());

    let constraint_result = render_constraints_tex(&engine, &options);
    assert!(constraint_result.tex.is_empty());
    assert!(constraint_result.warnings.is_empty());
}

#[test]
fn default_options() {
    let options = TexOptions::default();
    assert_eq!(options.section_mode, SectionMode::Fragment);
    assert!(options.include_holes);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test render_tex_basic 2>&1 | tail -20`
Expected: Compilation error — `render_tex` module doesn't exist yet.

- [ ] **Step 3: Create module skeleton**

Add to `crates/cp-ast-core/src/lib.rs` after line 4 (`pub mod render;`):

```rust
pub mod render_tex;
```

Create `crates/cp-ast-core/src/render_tex/mod.rs`:

```rust
//! TeX rendering for competitive programming AST structures.
//!
//! Produces deterministic, diff-stable TeX fragments for:
//! - Constraint notation (itemize lists)
//! - Input format notation (array layouts)

pub mod constraint_tex;
pub mod input_tex;
pub mod tex_helpers;

use crate::operation::AstEngine;
use crate::structure::NodeId;

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
    /// Whether to include section headers (`\paragraph{}` wrappers).
    pub section_mode: SectionMode,
    /// Whether to render Hole nodes (if false, holes are silently skipped).
    pub include_holes: bool,
}

impl Default for TexOptions {
    fn default() -> Self {
        Self {
            section_mode: SectionMode::Fragment,
            include_holes: true,
        }
    }
}

/// Section mode for TeX output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionMode {
    /// TeX fragments only.
    Fragment,
    /// With section headers.
    Standalone,
}

/// Warning emitted during TeX generation.
#[derive(Debug, Clone, PartialEq)]
pub enum TexWarning {
    /// A Hole node was encountered in the AST.
    HoleEncountered {
        /// The ID of the Hole node.
        node_id: NodeId,
    },
    /// A constraint type is not supported for TeX rendering.
    UnsupportedConstraint {
        /// Description of the unsupported constraint.
        description: String,
    },
    /// A Reference could not be resolved to a named node.
    UnresolvedReference {
        /// The unresolvable reference name.
        name: String,
    },
}

/// Render constraint notation as TeX.
#[must_use]
pub fn render_constraints_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput {
    constraint_tex::render_constraints_tex_impl(engine, options)
}

/// Render input format as TeX.
#[must_use]
pub fn render_input_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput {
    input_tex::render_input_tex_impl(engine, options)
}

/// Render both input and constraint notation as a combined TeX fragment.
#[must_use]
pub fn render_full_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput {
    let input = render_input_tex(engine, options);
    let constraints = render_constraints_tex(engine, options);

    let mut warnings = input.warnings;
    warnings.extend(constraints.warnings);

    let tex = match options.section_mode {
        SectionMode::Standalone => {
            let mut out = String::new();
            if !input.tex.is_empty() {
                out.push_str("\\paragraph{入力}\n");
                out.push_str(&input.tex);
            }
            if !constraints.tex.is_empty() {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str("\\paragraph{制約}\n");
                out.push_str(&constraints.tex);
            }
            out
        }
        SectionMode::Fragment => {
            let mut out = String::new();
            if !input.tex.is_empty() {
                out.push_str(&input.tex);
            }
            if !constraints.tex.is_empty() {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&constraints.tex);
            }
            out
        }
    };

    TexOutput { tex, warnings }
}
```

Create `crates/cp-ast-core/src/render_tex/tex_helpers.rs`:

```rust
//! TeX conversion helpers for expressions, references, identifiers.

use crate::constraint::Expression;
use crate::operation::AstEngine;
use crate::structure::{Ident, NodeKind, Reference};

use super::TexWarning;

/// Convert an `Expression` to its TeX representation.
pub(crate) fn expression_to_tex(
    engine: &AstEngine,
    expr: &Expression,
    warnings: &mut Vec<TexWarning>,
) -> String {
    // Stub — implemented in Task T-02
    let _ = (engine, warnings);
    format!("{expr:?}")
}

/// Convert a `Reference` to its TeX representation.
pub(crate) fn reference_to_tex(
    engine: &AstEngine,
    reference: &Reference,
    warnings: &mut Vec<TexWarning>,
) -> String {
    // Stub — implemented in Task T-02
    let _ = warnings;
    match reference {
        Reference::VariableRef(node_id) => {
            if let Some(node) = engine.structure.get(*node_id) {
                match node.kind() {
                    NodeKind::Scalar { name } | NodeKind::Array { name, .. } => {
                        ident_to_tex(name)
                    }
                    _ => format!("?{node_id:?}"),
                }
            } else {
                format!("?{node_id:?}")
            }
        }
        Reference::IndexedRef { target, indices } => {
            let base = if let Some(node) = engine.structure.get(*target) {
                match node.kind() {
                    NodeKind::Scalar { name } | NodeKind::Array { name, .. } => {
                        ident_to_tex(name)
                    }
                    _ => format!("?{target:?}"),
                }
            } else {
                format!("?{target:?}")
            };
            let idx_str = indices.iter().map(|i| i.as_str()).collect::<Vec<_>>().join(",");
            format!("{base}_{{{idx_str}}}")
        }
        Reference::Unresolved(ident) => {
            let name = ident.as_str().to_owned();
            warnings.push(TexWarning::UnresolvedReference { name: name.clone() });
            ident_to_tex(ident)
        }
    }
}

/// Convert an `Ident` to its TeX representation.
/// Single character → as-is; multiple characters → `\mathrm{}`.
pub(crate) fn ident_to_tex(ident: &Ident) -> String {
    let s = ident.as_str();
    if s.len() <= 1 {
        s.to_owned()
    } else {
        format!("\\mathrm{{{s}}}")
    }
}

/// Index variable allocator for array/repeat subscripts.
/// Allocates `i`, `j`, `k`, `l`, ... deterministically.
pub(crate) struct IndexAllocator {
    next: u8,
}

impl IndexAllocator {
    pub(crate) fn new() -> Self {
        Self { next: b'i' }
    }

    pub(crate) fn allocate(&mut self) -> char {
        let c = self.next as char;
        self.next += 1;
        c
    }
}
```

Create `crates/cp-ast-core/src/render_tex/constraint_tex.rs`:

```rust
//! Constraint TeX rendering.

use crate::operation::AstEngine;
use super::{TexOptions, TexOutput, TexWarning};

/// Render all constraints as TeX itemize list.
pub(crate) fn render_constraints_tex_impl(
    engine: &AstEngine,
    _options: &TexOptions,
) -> TexOutput {
    // Stub — implemented in Task T-03
    let _ = engine;
    TexOutput {
        tex: String::new(),
        warnings: Vec::new(),
    }
}
```

Create `crates/cp-ast-core/src/render_tex/input_tex.rs`:

```rust
//! Input format TeX rendering.

use crate::operation::AstEngine;
use super::{TexOptions, TexOutput};

/// Render input format as TeX array layout.
pub(crate) fn render_input_tex_impl(
    engine: &AstEngine,
    _options: &TexOptions,
) -> TexOutput {
    // Stub — implemented in Task T-04
    let _ = engine;
    TexOutput {
        tex: String::new(),
        warnings: Vec::new(),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All tests pass (including the 2 new ones), clippy clean.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(render_tex): add module skeleton with types and stubs (T-01)

New render_tex module with:
- TexOutput, TexOptions, SectionMode, TexWarning types
- render_constraints_tex, render_input_tex, render_full_tex public API
- Stub implementations for constraint_tex, input_tex, tex_helpers
- Module registered in lib.rs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task T-02: TeX helpers — expression, reference, ident conversion

**Files:**
- Modify: `crates/cp-ast-core/src/render_tex/tex_helpers.rs`
- Test: `crates/cp-ast-core/tests/render_tex_basic.rs`

- [ ] **Step 1: Write failing tests for TeX helpers**

Add to `crates/cp-ast-core/tests/render_tex_basic.rs`:

```rust
use cp_ast_core::constraint::{ArithOp, Expression};
use cp_ast_core::render_tex::tex_helpers;
use cp_ast_core::structure::Reference;

// ---- ident_to_tex tests ----

#[test]
fn ident_single_upper() {
    assert_eq!(tex_helpers::ident_to_tex(&Ident::new("N")), "N");
}

#[test]
fn ident_single_lower() {
    assert_eq!(tex_helpers::ident_to_tex(&Ident::new("x")), "x");
}

#[test]
fn ident_multi_char() {
    assert_eq!(tex_helpers::ident_to_tex(&Ident::new("ans")), "\\mathrm{ans}");
}

// ---- expression_to_tex tests ----

#[test]
fn expr_literal_small() {
    let engine = AstEngine::new();
    let mut w = vec![];
    assert_eq!(tex_helpers::expression_to_tex(&engine, &Expression::Lit(42), &mut w), "42");
    assert!(w.is_empty());
}

#[test]
fn expr_literal_power_of_10() {
    let engine = AstEngine::new();
    let mut w = vec![];
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &Expression::Lit(100000), &mut w),
        "10^{5}"
    );
}

#[test]
fn expr_literal_a_times_power_of_10() {
    let engine = AstEngine::new();
    let mut w = vec![];
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &Expression::Lit(200000), &mut w),
        "2 \\times 10^{5}"
    );
}

#[test]
fn expr_pow() {
    let engine = AstEngine::new();
    let mut w = vec![];
    let expr = Expression::Pow {
        base: Box::new(Expression::Lit(10)),
        exp: Box::new(Expression::Lit(9)),
    };
    assert_eq!(tex_helpers::expression_to_tex(&engine, &expr, &mut w), "10^{9}");
}

#[test]
fn expr_binop_mul() {
    let engine = AstEngine::new();
    let mut w = vec![];
    let expr = Expression::BinOp {
        op: ArithOp::Mul,
        lhs: Box::new(Expression::Lit(2)),
        rhs: Box::new(Expression::Pow {
            base: Box::new(Expression::Lit(10)),
            exp: Box::new(Expression::Lit(5)),
        }),
    };
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &expr, &mut w),
        "2 \\times 10^{5}"
    );
}

#[test]
fn expr_binop_add_var() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let mut w = vec![];
    let expr = Expression::BinOp {
        op: ArithOp::Add,
        lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
        rhs: Box::new(Expression::Lit(1)),
    };
    assert_eq!(tex_helpers::expression_to_tex(&engine, &expr, &mut w), "N + 1");
}

#[test]
fn expr_fncall_min() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let m_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("M") });
    let mut w = vec![];
    let expr = Expression::FnCall {
        name: Ident::new("min"),
        args: vec![
            Expression::Var(Reference::VariableRef(n_id)),
            Expression::Var(Reference::VariableRef(m_id)),
        ],
    };
    assert_eq!(tex_helpers::expression_to_tex(&engine, &expr, &mut w), "\\min(N, M)");
}

// ---- reference_to_tex tests ----

#[test]
fn ref_variable_scalar() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let mut w = vec![];
    assert_eq!(
        tex_helpers::reference_to_tex(&engine, &Reference::VariableRef(n_id), &mut w),
        "N"
    );
}

#[test]
fn ref_indexed() {
    let mut engine = AstEngine::new();
    let c_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("C") });
    let mut w = vec![];
    let reference = Reference::IndexedRef {
        target: c_id,
        indices: vec![Ident::new("i"), Ident::new("j")],
    };
    assert_eq!(tex_helpers::reference_to_tex(&engine, &reference, &mut w), "C_{i,j}");
}

#[test]
fn ref_unresolved_emits_warning() {
    let engine = AstEngine::new();
    let mut w = vec![];
    let result = tex_helpers::reference_to_tex(
        &engine,
        &Reference::Unresolved(Ident::new("X")),
        &mut w,
    );
    assert_eq!(result, "X");
    assert_eq!(w.len(), 1);
    assert!(matches!(&w[0], TexWarning::UnresolvedReference { name } if name == "X"));
}

// ---- IndexAllocator tests ----

#[test]
fn index_allocator_sequential() {
    let mut alloc = tex_helpers::IndexAllocator::new();
    assert_eq!(alloc.allocate(), 'i');
    assert_eq!(alloc.allocate(), 'j');
    assert_eq!(alloc.allocate(), 'k');
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test render_tex_basic 2>&1 | tail -30`
Expected: Compilation errors — `tex_helpers` is not public, `expression_to_tex` returns Debug format stub.

- [ ] **Step 3: Make tex_helpers module public and implement expression_to_tex**

In `crates/cp-ast-core/src/render_tex/mod.rs`, change:
```rust
pub mod tex_helpers;
```
(This is already the case — it's declared `pub mod tex_helpers;`)

Replace the entire `crates/cp-ast-core/src/render_tex/tex_helpers.rs` with:

```rust
//! TeX conversion helpers for expressions, references, identifiers.

use crate::constraint::{ArithOp, Expression};
use crate::operation::AstEngine;
use crate::structure::{Ident, NodeKind, Reference};

use super::TexWarning;

/// Convert an `Expression` to its TeX representation.
pub fn expression_to_tex(
    engine: &AstEngine,
    expr: &Expression,
    warnings: &mut Vec<TexWarning>,
) -> String {
    match expr {
        Expression::Lit(n) => lit_to_tex(*n),
        Expression::Var(reference) => reference_to_tex(engine, reference, warnings),
        Expression::BinOp { op, lhs, rhs } => {
            let lhs_str = expression_to_tex(engine, lhs, warnings);
            let rhs_str = expression_to_tex(engine, rhs, warnings);
            let op_str = match op {
                ArithOp::Add => " + ",
                ArithOp::Sub => " - ",
                ArithOp::Mul => " \\times ",
                ArithOp::Div => " \\div ",
            };
            format!("{lhs_str}{op_str}{rhs_str}")
        }
        Expression::Pow { base, exp } => {
            let base_str = expression_to_tex(engine, base, warnings);
            let exp_str = expression_to_tex(engine, exp, warnings);
            format!("{base_str}^{{{exp_str}}}")
        }
        Expression::FnCall { name, args } => {
            let name_str = name.as_str();
            let tex_name = match name_str {
                "min" | "max" | "gcd" | "lcm" | "log" => format!("\\{name_str}"),
                _ => ident_to_tex(name),
            };
            let args_str = args
                .iter()
                .map(|a| expression_to_tex(engine, a, warnings))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{tex_name}({args_str})")
        }
    }
}

/// Convert a literal integer to TeX, auto-decomposing large numbers.
fn lit_to_tex(n: i64) -> String {
    if n < 0 {
        let pos = lit_to_tex(-n);
        return format!("-{pos}");
    }
    if let Some(tex) = decompose_large_number(n) {
        return tex;
    }
    n.to_string()
}

/// Try to decompose n into `a × 10^k` form for TeX.
/// Returns Some(tex_string) if decomposable, None otherwise.
fn decompose_large_number(n: i64) -> Option<String> {
    if n <= 0 {
        return None;
    }
    // Find the largest power of 10 that divides n
    let mut k: u32 = 0;
    let mut remaining = n;
    while remaining >= 100 && remaining % 10 == 0 {
        remaining /= 10;
        k += 1;
    }
    if k < 2 {
        return None;
    }
    if remaining == 1 {
        Some(format!("10^{{{k}}}"))
    } else if remaining < 10 {
        Some(format!("{remaining} \\times 10^{{{k}}}"))
    } else {
        None
    }
}

/// Convert a `Reference` to its TeX representation.
pub fn reference_to_tex(
    engine: &AstEngine,
    reference: &Reference,
    warnings: &mut Vec<TexWarning>,
) -> String {
    match reference {
        Reference::VariableRef(node_id) => {
            if let Some(node) = engine.structure.get(*node_id) {
                match node.kind() {
                    NodeKind::Scalar { name }
                    | NodeKind::Array { name, .. }
                    | NodeKind::Matrix { name, .. } => ident_to_tex(name),
                    _ => format!("?{node_id:?}"),
                }
            } else {
                format!("?{node_id:?}")
            }
        }
        Reference::IndexedRef { target, indices } => {
            let base = if let Some(node) = engine.structure.get(*target) {
                match node.kind() {
                    NodeKind::Scalar { name }
                    | NodeKind::Array { name, .. }
                    | NodeKind::Matrix { name, .. } => ident_to_tex(name),
                    _ => format!("?{target:?}"),
                }
            } else {
                format!("?{target:?}")
            };
            let idx_str = indices
                .iter()
                .map(|i| i.as_str())
                .collect::<Vec<_>>()
                .join(",");
            format!("{base}_{{{idx_str}}}")
        }
        Reference::Unresolved(ident) => {
            let name = ident.as_str().to_owned();
            warnings.push(TexWarning::UnresolvedReference { name });
            ident_to_tex(ident)
        }
    }
}

/// Convert an `Ident` to its TeX representation.
/// Single character → as-is (math italic); multiple characters → `\mathrm{}`.
pub fn ident_to_tex(ident: &Ident) -> String {
    let s = ident.as_str();
    if s.len() <= 1 {
        s.to_owned()
    } else {
        format!("\\mathrm{{{s}}}")
    }
}

/// Index variable allocator for array/repeat subscripts.
/// Allocates `i`, `j`, `k`, `l`, ... deterministically.
pub struct IndexAllocator {
    next: u8,
}

impl IndexAllocator {
    /// Create a new allocator starting at 'i'.
    pub fn new() -> Self {
        Self { next: b'i' }
    }

    /// Allocate the next index variable character.
    pub fn allocate(&mut self) -> char {
        let c = self.next as char;
        self.next += 1;
        c
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All tests pass, clippy clean.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(render_tex): implement TeX helpers — expression, reference, ident (T-02)

- expression_to_tex: Lit (with large-number decomposition), Var, BinOp, Pow, FnCall
- reference_to_tex: VariableRef, IndexedRef, Unresolved (with warning)
- ident_to_tex: single char → as-is, multi-char → \\mathrm{}
- IndexAllocator: deterministic i, j, k, ... allocation
- decompose_large_number: a × 10^k auto-formatting

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task T-03: Constraint TeX rendering

**Files:**
- Modify: `crates/cp-ast-core/src/render_tex/constraint_tex.rs`
- Modify: `crates/cp-ast-core/src/render_tex/tex_helpers.rs` (add `node_is_array_element` helper)
- Test: `crates/cp-ast-core/tests/render_tex_basic.rs`

- [ ] **Step 1: Write failing tests for constraint TeX**

Add to `crates/cp-ast-core/tests/render_tex_basic.rs`:

```rust
use cp_ast_core::constraint::{
    CharSetSpec, Constraint, DistinctUnit, Expression, PropertyTag, SortOrder,
};

// ---- Constraint TeX tests ----

#[test]
fn constraint_tex_scalar_range() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::BinOp {
                op: ArithOp::Mul,
                lhs: Box::new(Expression::Lit(2)),
                rhs: Box::new(Expression::Pow {
                    base: Box::new(Expression::Lit(10)),
                    exp: Box::new(Expression::Lit(5)),
                }),
            },
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $1 \\le N \\le 2 \\times 10^{5}$\n\\end{itemize}\n"
    );
    assert!(result.warnings.is_empty());
}

#[test]
fn constraint_tex_array_element_with_index_range() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Reference::VariableRef(n_id),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id, a_id] });
    }
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::VariableRef(a_id),
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(9)),
            },
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $1 \\le A_i \\le 10^{9} \\ (1 \\le i \\le N)$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_type_decl_skipped() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: cp_ast_core::constraint::ExpectedType::Int,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert!(result.tex.is_empty());
}

#[test]
fn constraint_tex_sum_bound() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }
    engine.constraints.add(
        None,
        Constraint::SumBound {
            variable: Reference::VariableRef(n_id),
            upper: Expression::BinOp {
                op: ArithOp::Mul,
                lhs: Box::new(Expression::Lit(2)),
                rhs: Box::new(Expression::Pow {
                    base: Box::new(Expression::Lit(10)),
                    exp: Box::new(Expression::Lit(5)),
                }),
            },
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $\\sum N \\le 2 \\times 10^{5}$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_distinct() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Reference::VariableRef(n_id),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id, a_id] });
    }
    engine.constraints.add(
        Some(a_id),
        Constraint::Distinct {
            elements: Reference::VariableRef(a_id),
            unit: DistinctUnit::Element,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $A_i \\neq A_j \\ (i \\neq j)$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_sorted() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Reference::VariableRef(n_id),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id, a_id] });
    }
    engine.constraints.add(
        Some(a_id),
        Constraint::Sorted {
            elements: Reference::VariableRef(a_id),
            order: SortOrder::Ascending,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $A_1 \\le A_2 \\le \\cdots \\le A_N$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_string_length() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let s_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("S") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id, s_id] });
    }
    engine.constraints.add(
        Some(s_id),
        Constraint::StringLength {
            target: Reference::VariableRef(s_id),
            min: Expression::Lit(1),
            max: Expression::Var(Reference::VariableRef(n_id)),
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $1 \\le |S| \\le N$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_guarantee() {
    let mut engine = AstEngine::new();
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "The answer always exists.".to_owned(),
            predicate: None,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item The answer always exists.\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_ordering() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }
    // Add in reverse order — Guarantee first, then Range
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "answer exists".to_owned(),
            predicate: None,
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    // Range should come before Guarantee regardless of insertion order
    let lines: Vec<&str> = result.tex.lines().collect();
    assert!(lines[1].contains("1 \\le N \\le 100"));
    assert!(lines[2].contains("answer exists"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test render_tex_basic 2>&1 | tail -30`
Expected: Tests fail — stub returns empty string.

- [ ] **Step 3: Add array element detection helper to tex_helpers.rs**

Add the following to `crates/cp-ast-core/src/render_tex/tex_helpers.rs`:

```rust
use crate::structure::NodeId;

/// Check if a Reference target is an Array node, and if so return (name, length_ref).
pub fn resolve_array_info(
    engine: &AstEngine,
    reference: &Reference,
) -> Option<(String, Reference)> {
    if let Reference::VariableRef(node_id) = reference {
        if let Some(node) = engine.structure.get(*node_id) {
            if let NodeKind::Array { name, length } = node.kind() {
                return Some((ident_to_tex(name), length.clone()));
            }
        }
    }
    None
}

/// Render a Reference as a TeX subscripted element form for array context.
/// E.g., Array A → "A_i" with index from allocator.
pub fn reference_to_tex_subscripted(
    engine: &AstEngine,
    reference: &Reference,
    index_var: char,
    warnings: &mut Vec<TexWarning>,
) -> String {
    if let Reference::VariableRef(node_id) = reference {
        if let Some(node) = engine.structure.get(*node_id) {
            match node.kind() {
                NodeKind::Array { name, .. } | NodeKind::Matrix { name, .. } => {
                    let base = ident_to_tex(name);
                    return format!("{base}_{index_var}");
                }
                NodeKind::Scalar { name } => {
                    return ident_to_tex(name);
                }
                _ => {}
            }
        }
    }
    reference_to_tex(engine, reference, warnings)
}
```

- [ ] **Step 4: Implement constraint_tex.rs**

Replace `crates/cp-ast-core/src/render_tex/constraint_tex.rs` with:

```rust
//! Constraint TeX rendering.

use crate::constraint::{Constraint, RelationOp, SortOrder};
use crate::operation::AstEngine;
use crate::structure::Reference;

use super::tex_helpers::{
    expression_to_tex, ident_to_tex, reference_to_tex, resolve_array_info, IndexAllocator,
};
use super::{TexOptions, TexOutput, TexWarning};

/// Render all constraints as TeX itemize list.
pub(crate) fn render_constraints_tex_impl(
    engine: &AstEngine,
    _options: &TexOptions,
) -> TexOutput {
    let mut warnings = Vec::new();
    let mut alloc = IndexAllocator::new();

    // Group constraints by type in display order (same as render/constraint_text.rs)
    let mut groups: [Vec<String>; 9] = Default::default();

    for (_, constraint) in engine.constraints.iter() {
        let rendered = render_constraint_tex(engine, constraint, &mut alloc, &mut warnings);
        let type_index = match constraint {
            Constraint::Range { .. } => 0,
            Constraint::TypeDecl { .. } | Constraint::RenderHint { .. } => continue,
            Constraint::LengthRelation { .. } => 1,
            Constraint::Relation { .. } => 2,
            Constraint::Distinct { .. } => 3,
            Constraint::Property { .. } => 4,
            Constraint::Sorted { .. } => 5,
            Constraint::SumBound { .. } => 6,
            Constraint::Guarantee { .. }
            | Constraint::CharSet { .. }
            | Constraint::StringLength { .. } => 7,
        };
        groups[type_index].push(rendered);
    }

    let mut items: Vec<String> = Vec::new();
    for group in &groups {
        for item in group {
            items.push(item.clone());
        }
    }

    if items.is_empty() {
        return TexOutput {
            tex: String::new(),
            warnings,
        };
    }

    let mut tex = String::from("\\begin{itemize}\n");
    for item in &items {
        tex.push_str(&format!("  \\item {item}\n"));
    }
    tex.push_str("\\end{itemize}\n");

    TexOutput { tex, warnings }
}

fn render_constraint_tex(
    engine: &AstEngine,
    constraint: &Constraint,
    alloc: &mut IndexAllocator,
    warnings: &mut Vec<TexWarning>,
) -> String {
    match constraint {
        Constraint::Range {
            target,
            lower,
            upper,
        } => {
            let lower_str = expression_to_tex(engine, lower, warnings);
            let upper_str = expression_to_tex(engine, upper, warnings);

            if let Some((name, length_ref)) = resolve_array_info(engine, target) {
                let idx = alloc.allocate();
                let length_str = reference_to_tex(engine, &length_ref, warnings);
                format!(
                    "${lower_str} \\le {name}_{idx} \\le {upper_str} \\ (1 \\le {idx} \\le {length_str})$"
                )
            } else {
                let target_str = reference_to_tex(engine, target, warnings);
                format!("${lower_str} \\le {target_str} \\le {upper_str}$")
            }
        }
        Constraint::LengthRelation { target, length } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let length_str = expression_to_tex(engine, length, warnings);
            format!("$|{target_str}| = {length_str}$")
        }
        Constraint::Relation { lhs, op, rhs } => {
            let lhs_str = expression_to_tex(engine, lhs, warnings);
            let rhs_str = expression_to_tex(engine, rhs, warnings);
            let op_str = match op {
                RelationOp::Le => "\\le",
                RelationOp::Lt => "<",
                RelationOp::Ge => "\\ge",
                RelationOp::Gt => ">",
                RelationOp::Eq => "=",
                RelationOp::Ne => "\\neq",
            };
            format!("${lhs_str} {op_str} {rhs_str}$")
        }
        Constraint::Distinct { elements, .. } => {
            if let Some((name, _)) = resolve_array_info(engine, elements) {
                format!("${name}_i \\neq {name}_j \\ (i \\neq j)$")
            } else {
                let elem_str = reference_to_tex(engine, elements, warnings);
                format!("${elem_str}$ are pairwise distinct")
            }
        }
        Constraint::Sorted { elements, order } => {
            if let Some((name, length_ref)) = resolve_array_info(engine, elements) {
                let length_str = reference_to_tex(engine, &length_ref, warnings);
                let op = match order {
                    SortOrder::Ascending | SortOrder::NonDecreasing => "\\le",
                    SortOrder::Descending | SortOrder::NonIncreasing => "\\ge",
                };
                format!("${name}_1 {op} {name}_2 {op} \\cdots {op} {name}_{length_str}$")
            } else {
                let elem_str = reference_to_tex(engine, elements, warnings);
                let order_str = match order {
                    SortOrder::Ascending | SortOrder::NonDecreasing => "ascending",
                    SortOrder::Descending | SortOrder::NonIncreasing => "descending",
                };
                format!("${elem_str}$ sorted {order_str}")
            }
        }
        Constraint::SumBound { variable, upper } => {
            let var_str = reference_to_tex(engine, variable, warnings);
            let upper_str = expression_to_tex(engine, upper, warnings);
            format!("$\\sum {var_str} \\le {upper_str}$")
        }
        Constraint::Guarantee { description, .. } => description.clone(),
        Constraint::CharSet { target, charset } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let charset_desc = match charset {
                crate::constraint::CharSetSpec::LowerAlpha => "は英小文字からなる",
                crate::constraint::CharSetSpec::UpperAlpha => "は英大文字からなる",
                crate::constraint::CharSetSpec::Alpha => "は英字からなる",
                crate::constraint::CharSetSpec::Digit => "は数字からなる",
                crate::constraint::CharSetSpec::AlphaNumeric => "は英数字からなる",
                crate::constraint::CharSetSpec::Custom(_) | crate::constraint::CharSetSpec::Range(_, _) => {
                    return format!("${target_str}$: charset constraint");
                }
            };
            format!("${target_str}$ {charset_desc}")
        }
        Constraint::StringLength { target, min, max } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let min_str = expression_to_tex(engine, min, warnings);
            let max_str = expression_to_tex(engine, max, warnings);
            format!("${min_str} \\le |{target_str}| \\le {max_str}$")
        }
        Constraint::Property { target, tag } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let tag_desc = match tag {
                crate::constraint::PropertyTag::Simple => "is a simple graph",
                crate::constraint::PropertyTag::Connected => "is connected",
                crate::constraint::PropertyTag::Tree => "is a tree",
                crate::constraint::PropertyTag::Permutation => "is a permutation",
                crate::constraint::PropertyTag::Binary => "is binary",
                crate::constraint::PropertyTag::Odd => "is odd",
                crate::constraint::PropertyTag::Even => "is even",
                crate::constraint::PropertyTag::Custom(s) => {
                    return format!("${target_str}$: {s}");
                }
            };
            format!("${target_str}$ {tag_desc}")
        }
        Constraint::TypeDecl { .. } | Constraint::RenderHint { .. } => {
            // Should never reach here due to `continue` in caller
            String::new()
        }
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All tests pass, clippy clean.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(render_tex): implement constraint TeX rendering (T-03)

- Range constraints with auto index annotation for arrays
- TypeDecl and RenderHint skipped (not displayed in CP)
- Distinct, Sorted with array-aware formatting
- SumBound, StringLength, LengthRelation
- Guarantee and CharSet as text
- Property tag descriptions
- Constraints ordered by type category

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task T-04: Input format TeX rendering

**Files:**
- Modify: `crates/cp-ast-core/src/render_tex/input_tex.rs`
- Test: `crates/cp-ast-core/tests/render_tex_basic.rs`

- [ ] **Step 1: Write failing tests for input format TeX**

Add to `crates/cp-ast-core/tests/render_tex_basic.rs`:

```rust
// ---- Input Format TeX tests ----

#[test]
fn input_tex_single_scalar() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(result.tex, "\\[\n\\begin{array}{l}\nN\n\\end{array}\n\\]\n");
    assert!(result.warnings.is_empty());
}

#[test]
fn input_tex_tuple() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let m_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("M") });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![tuple_id] });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(result.tex, "\\[\n\\begin{array}{l}\nN \\ M\n\\end{array}\n\\]\n");
}

#[test]
fn input_tex_array() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Reference::VariableRef(n_id),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id, a_id] });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\[\n\\begin{array}{l}\nN \\\\\nA_1 \\ A_2 \\ \\cdots \\ A_N\n\\end{array}\n\\]\n"
    );
}

#[test]
fn input_tex_repeat_scalar() {
    let mut engine = AstEngine::new();
    let q_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("Q") });
    let t_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("T") });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Reference::VariableRef(q_id),
        body: vec![t_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![q_id, repeat_id] });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\[\n\\begin{array}{l}\nQ \\\\\nT_1 \\\\\nT_2 \\\\\n\\vdots \\\\\nT_Q\n\\end{array}\n\\]\n"
    );
}

#[test]
fn input_tex_repeat_tuple() {
    let mut engine = AstEngine::new();
    let m_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("M") });
    let u_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("u") });
    let v_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("v") });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Reference::VariableRef(m_id),
        body: vec![tuple_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![m_id, repeat_id] });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\[\n\\begin{array}{l}\nM \\\\\nu_1 \\ v_1 \\\\\nu_2 \\ v_2 \\\\\n\\vdots \\\\\nu_M \\ v_M\n\\end{array}\n\\]\n"
    );
}

#[test]
fn input_tex_hole() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole { expected_kind: None });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![hole_id] });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert!(result.tex.contains("\\texttt{<hole>}"));
    assert_eq!(result.warnings.len(), 1);
    assert!(matches!(&result.warnings[0], TexWarning::HoleEncountered { .. }));
}

#[test]
fn input_tex_combined_n_array_repeat() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let q_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("Q") });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, q_id],
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("D"),
        length: Reference::VariableRef(n_id),
    });
    let t_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("T") });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Reference::VariableRef(q_id),
        body: vec![t_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header, a_id, repeat_id],
        });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    let expected = "\
\\[\n\
\\begin{array}{l}\n\
N \\ Q \\\\\n\
D_1 \\ D_2 \\ \\cdots \\ D_N \\\\\n\
T_1 \\\\\n\
T_2 \\\\\n\
\\vdots \\\\\n\
T_Q\n\
\\end{array}\n\
\\]\n";
    assert_eq!(result.tex, expected);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test render_tex_basic 2>&1 | tail -30`
Expected: Tests fail — stub returns empty string.

- [ ] **Step 3: Implement input_tex.rs**

Replace `crates/cp-ast-core/src/render_tex/input_tex.rs` with:

```rust
//! Input format TeX rendering.

use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind};

use super::tex_helpers::{ident_to_tex, reference_to_tex};
use super::{TexOptions, TexOutput, TexWarning};

/// Render input format as TeX array layout.
pub(crate) fn render_input_tex_impl(
    engine: &AstEngine,
    options: &TexOptions,
) -> TexOutput {
    let mut warnings = Vec::new();
    let root = engine.structure.root();

    let mut lines: Vec<String> = Vec::new();
    collect_lines(engine, root, &mut lines, &mut warnings, options);

    if lines.is_empty() {
        return TexOutput {
            tex: String::new(),
            warnings,
        };
    }

    let mut tex = String::from("\\[\n\\begin{array}{l}\n");
    for (i, line) in lines.iter().enumerate() {
        tex.push_str(line);
        if i < lines.len() - 1 {
            tex.push_str(" \\\\");
        }
        tex.push('\n');
    }
    tex.push_str("\\end{array}\n\\]\n");

    TexOutput { tex, warnings }
}

/// Collect TeX lines from the AST recursively.
fn collect_lines(
    engine: &AstEngine,
    node_id: NodeId,
    lines: &mut Vec<String>,
    warnings: &mut Vec<TexWarning>,
    options: &TexOptions,
) {
    let Some(node) = engine.structure.get(node_id) else {
        lines.push("\\texttt{<?>}".to_owned());
        return;
    };

    match node.kind() {
        NodeKind::Scalar { name } => {
            lines.push(ident_to_tex(name));
        }
        NodeKind::Tuple { elements } => {
            let parts: Vec<String> = elements
                .iter()
                .map(|&eid| {
                    if let Some(enode) = engine.structure.get(eid) {
                        match enode.kind() {
                            NodeKind::Scalar { name } => ident_to_tex(name),
                            NodeKind::Hole { .. } => {
                                if options.include_holes {
                                    warnings.push(TexWarning::HoleEncountered { node_id: eid });
                                    "\\texttt{<hole>}".to_owned()
                                } else {
                                    String::new()
                                }
                            }
                            _ => "\\texttt{<?>}".to_owned(),
                        }
                    } else {
                        "\\texttt{<?>}".to_owned()
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();
            if !parts.is_empty() {
                lines.push(parts.join(" \\ "));
            }
        }
        NodeKind::Array { name, length } => {
            let name_str = ident_to_tex(name);
            let length_str = reference_to_tex(engine, length, warnings);
            lines.push(format!(
                "{name_str}_1 \\ {name_str}_2 \\ \\cdots \\ {name_str}_{length_str}"
            ));
        }
        NodeKind::Matrix { name, rows, cols } => {
            let name_str = ident_to_tex(name);
            let rows_str = reference_to_tex(engine, rows, warnings);
            let cols_str = reference_to_tex(engine, cols, warnings);
            // First row
            lines.push(format!(
                "{name_str}_{{1,1}} \\ {name_str}_{{1,2}} \\ \\cdots \\ {name_str}_{{1,{cols_str}}}"
            ));
            // Second row
            lines.push(format!(
                "{name_str}_{{2,1}} \\ {name_str}_{{2,2}} \\ \\cdots \\ {name_str}_{{2,{cols_str}}}"
            ));
            // Vdots
            lines.push("\\vdots".to_owned());
            // Last row
            lines.push(format!(
                "{name_str}_{{{rows_str},1}} \\ {name_str}_{{{rows_str},2}} \\ \\cdots \\ {name_str}_{{{rows_str},{cols_str}}}"
            ));
        }
        NodeKind::Repeat { count, body } => {
            let count_str = reference_to_tex(engine, count, warnings);
            render_repeat_lines(engine, &count_str, body, lines, warnings, options);
        }
        NodeKind::Section { body, .. } => {
            for &child_id in body {
                collect_lines(engine, child_id, lines, warnings, options);
            }
        }
        NodeKind::Sequence { children } => {
            for &child_id in children {
                collect_lines(engine, child_id, lines, warnings, options);
            }
        }
        NodeKind::Hole { .. } => {
            if options.include_holes {
                warnings.push(TexWarning::HoleEncountered { node_id });
                lines.push("\\texttt{<hole>}".to_owned());
            }
        }
        NodeKind::Choice { .. } => {
            lines.push("\\texttt{(choice)}".to_owned());
        }
    }
}

/// Render Repeat node as subscripted vertical expansion.
fn render_repeat_lines(
    engine: &AstEngine,
    count_str: &str,
    body: &[NodeId],
    lines: &mut Vec<String>,
    warnings: &mut Vec<TexWarning>,
    options: &TexOptions,
) {
    // Check if body is a single Scalar
    if body.len() == 1 {
        if let Some(body_node) = engine.structure.get(body[0]) {
            if let NodeKind::Scalar { name } = body_node.kind() {
                let name_str = ident_to_tex(name);
                lines.push(format!("{name_str}_1"));
                lines.push(format!("{name_str}_2"));
                lines.push("\\vdots".to_owned());
                lines.push(format!("{name_str}_{count_str}"));
                return;
            }
            // Check if body is a single Tuple
            if let NodeKind::Tuple { elements } = body_node.kind() {
                let names: Vec<String> = elements
                    .iter()
                    .filter_map(|&eid| {
                        engine.structure.get(eid).and_then(|n| {
                            if let NodeKind::Scalar { name } = n.kind() {
                                Some(ident_to_tex(name))
                            } else {
                                None
                            }
                        })
                    })
                    .collect();
                if !names.is_empty() {
                    // First line: u_1 \ v_1
                    let first: Vec<String> = names.iter().map(|n| format!("{n}_1")).collect();
                    lines.push(first.join(" \\ "));
                    // Second line: u_2 \ v_2
                    let second: Vec<String> = names.iter().map(|n| format!("{n}_2")).collect();
                    lines.push(second.join(" \\ "));
                    // Vdots
                    lines.push("\\vdots".to_owned());
                    // Last line: u_M \ v_M
                    let last: Vec<String> =
                        names.iter().map(|n| format!("{n}_{count_str}")).collect();
                    lines.push(last.join(" \\ "));
                    return;
                }
            }
        }
    }

    // Fallback: render body children recursively
    for &child_id in body {
        collect_lines(engine, child_id, lines, warnings, options);
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All tests pass, clippy clean.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(render_tex): implement input format TeX rendering (T-04)

- Scalar, Tuple, Array (with \\cdots), Matrix (multi-row with \\vdots)
- Repeat expansion: scalar body, tuple body, general fallback
- Hole and Choice placeholders
- Lines wrapped in \\begin{array}{l}...\\end{array}
- \\[ \\] math display mode

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task T-05: render_full_tex, standalone mode, and end-to-end tests

**Files:**
- Modify: `crates/cp-ast-core/tests/render_tex_basic.rs`

- [ ] **Step 1: Write full_tex and standalone mode tests**

Add to `crates/cp-ast-core/tests/render_tex_basic.rs`:

```rust
// ---- render_full_tex tests ----

#[test]
fn full_tex_fragment_mode() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let result = render_full_tex(&engine, &TexOptions::default());
    // Fragment mode: input + blank line + constraints
    assert!(result.tex.contains("\\begin{array}"));
    assert!(result.tex.contains("\\begin{itemize}"));
}

#[test]
fn full_tex_standalone_mode() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let options = TexOptions {
        section_mode: SectionMode::Standalone,
        include_holes: true,
    };
    let result = render_full_tex(&engine, &options);
    assert!(result.tex.contains("\\paragraph{入力}"));
    assert!(result.tex.contains("\\paragraph{制約}"));
}

// ---- End-to-end test ----

#[test]
fn e2e_graph_problem_tex() {
    // Build a graph problem: N M header, M edges (u_i v_i), constraints
    let mut engine = AstEngine::new();

    // Variables
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let m_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("M") });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });

    let u_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("u") });
    let v_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("v") });
    let edge_tuple = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let edges = engine.structure.add_node(NodeKind::Repeat {
        count: Reference::VariableRef(m_id),
        body: vec![edge_tuple],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header, edges],
        });
    }

    // Constraints
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(m_id),
        Constraint::Range {
            target: Reference::VariableRef(m_id),
            lower: Expression::Lit(0),
            upper: Expression::BinOp {
                op: ArithOp::Div,
                lhs: Box::new(Expression::BinOp {
                    op: ArithOp::Mul,
                    lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
                    rhs: Box::new(Expression::BinOp {
                        op: ArithOp::Sub,
                        lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
                        rhs: Box::new(Expression::Lit(1)),
                    }),
                }),
                rhs: Box::new(Expression::Lit(2)),
            },
        },
    );

    // Input TeX
    let input_result = render_input_tex(&engine, &TexOptions::default());
    let expected_input = "\
\\[\n\
\\begin{array}{l}\n\
N \\ M \\\\\n\
u_1 \\ v_1 \\\\\n\
u_2 \\ v_2 \\\\\n\
\\vdots \\\\\n\
u_M \\ v_M\n\
\\end{array}\n\
\\]\n";
    assert_eq!(input_result.tex, expected_input);

    // Constraint TeX
    let constraint_result = render_constraints_tex(&engine, &TexOptions::default());
    assert!(constraint_result.tex.contains("$1 \\le N \\le 100$"));
    assert!(constraint_result.tex.contains("$0 \\le M \\le"));

    // Full TeX
    let full_result = render_full_tex(
        &engine,
        &TexOptions {
            section_mode: SectionMode::Standalone,
            include_holes: true,
        },
    );
    assert!(full_result.tex.contains("\\paragraph{入力}"));
    assert!(full_result.tex.contains("\\paragraph{制約}"));
    assert!(full_result.warnings.is_empty());
}

#[test]
fn include_holes_false_suppresses_holes() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole { expected_kind: None });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![hole_id] });
    }

    let options = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    let result = render_input_tex(&engine, &options);
    assert!(!result.tex.contains("hole"));
    assert!(result.warnings.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All tests pass. These tests exercise the full pipeline already implemented.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(render_tex): add full_tex, standalone mode, and e2e tests (T-05)

- Fragment vs Standalone section mode tests
- End-to-end graph problem (N M + edges) with full TeX validation
- include_holes: false suppression test
- All golden-test assertions with exact string comparisons

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---
