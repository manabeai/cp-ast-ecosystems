# AST Viewer Frontend — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Phase 1 web-based AST viewer that renders competitive programming input format, constraints, TeX, and sample output from the existing cp-ast-core Rust backend via WebAssembly.

**Architecture:** New `cp-ast-wasm` crate wraps cp-ast-core/tree/json APIs with wasm-bindgen. A Preact+Vite frontend in `web/` calls these wasm functions through a JSON String API — the frontend never interprets AST structure directly. 9 preset ASTs are hardcoded in Rust and served to the frontend.

**Tech Stack:** Rust (wasm-bindgen, wasm-pack), TypeScript, Preact, Vite, KaTeX, Preact Signals

**Design Spec:** `docs/superpowers/specs/2026-04-14-ast-viewer-frontend-design.md`

---

## Prerequisites

- `wasm-pack` installed: `cargo install wasm-pack`
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Node.js 18+ with npm

---

## File Structure

### New crate: cp-ast-wasm

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `crates/cp-ast-wasm/Cargo.toml` | Crate manifest |
| Create | `crates/cp-ast-wasm/src/lib.rs` | wasm-bindgen exported functions (10 functions) |
| Create | `crates/cp-ast-wasm/src/presets.rs` | 9 preset AstEngine builders + list/build API |

### New directory: web/

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `web/package.json` | npm dependencies |
| Create | `web/vite.config.ts` | Vite configuration |
| Create | `web/tsconfig.json` | TypeScript configuration |
| Create | `web/index.html` | HTML entry point |
| Create | `web/src/main.tsx` | Application bootstrap + wasm init |
| Create | `web/src/app.tsx` | Root component + hash router |
| Create | `web/src/state.ts` | Preact Signals state management |
| Create | `web/src/wasm.ts` | wasm module loader + re-exports |
| Create | `web/src/tex-renderer.ts` | KaTeX rendering helpers |
| Create | `web/src/index.css` | Global styles (dark theme) |
| Create | `web/src/components/viewer/ViewerPage.tsx` | 3-column viewer layout |
| Create | `web/src/components/viewer/StructurePane.tsx` | Input format + AST toggle |
| Create | `web/src/components/viewer/ConstraintPane.tsx` | Constraints + AST toggle |
| Create | `web/src/components/viewer/PreviewPane.tsx` | TeX + Sample tabs |
| Create | `web/src/components/viewer/Toolbar.tsx` | Preset selector, seed, shuffle |
| Create | `web/src/components/preview/PreviewPage.tsx` | Card gallery page |
| Create | `web/src/components/preview/PreviewCard.tsx` | Individual preset card |

### Modify existing

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `.gitignore` | Add web/wasm/, web/node_modules/, web/dist/ |

---

## Task Dependency Graph

```
T1 (wasm skeleton) ──→ T2 (render functions) ──→ T3 (presets) ──┐
                                                                  ├──→ T5 (wasm integration) ──→ T6 (app shell) ──┬──→ T7 (viewer panes)
T4 (web scaffold) ────────────────────────────────────────────────┘                                               ├──→ T8 (preview + toolbar)
                                                                                                                  └──→ T9 (preview page)
                                                                                                                        ↓
                                                                                                                  T10 (final)
```

Parallel groups: [T1, T4], [T7, T8, T9]

---

## Task 1: cp-ast-wasm crate skeleton

**Files:**
- Create: `crates/cp-ast-wasm/Cargo.toml`
- Create: `crates/cp-ast-wasm/src/lib.rs`

- [ ] **Step 1: Create `Cargo.toml`**

Create `crates/cp-ast-wasm/Cargo.toml`:

```toml
[package]
name = "cp-ast-wasm"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "WebAssembly bindings for cp-ast-core AST viewer"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cp-ast-core = { path = "../cp-ast-core" }
cp-ast-tree = { path = "../cp-ast-tree" }
cp-ast-json = { path = "../cp-ast-json" }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[lints]
workspace = true
```

- [ ] **Step 2: Create minimal `lib.rs`**

Create `crates/cp-ast-wasm/src/lib.rs`:

```rust
//! WebAssembly bindings for the cp-ast-core AST viewer.
//!
//! All functions accept and return strings (JSON String API).
//! The frontend never interprets AST structure directly.

use wasm_bindgen::prelude::*;

/// Returns the crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Renders input format as human-readable text.
///
/// Input: cp-ast-json document JSON string.
/// Output: formatted input specification (e.g., "N\nA_1 A_2 ... A_N").
#[wasm_bindgen]
pub fn render_input_format(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_core::render::render_input(&engine))
}

fn deserialize(json: &str) -> Result<cp_ast_core::operation::AstEngine, JsError> {
    cp_ast_json::deserialize_ast(json).map_err(|e| JsError::new(&e.to_string()))
}
```

- [ ] **Step 3: Verify native compilation**

Run: `cargo build -p cp-ast-wasm`
Expected: compiles successfully.

- [ ] **Step 4: Build with wasm-pack**

Run: `wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm`
Expected: produces `web/wasm/` with `cp_ast_wasm.js`, `cp_ast_wasm_bg.wasm`, `cp_ast_wasm.d.ts`.

Note: `web/wasm/` is a build output — do NOT commit it. It will be added to `.gitignore` in Task 10.

- [ ] **Step 5: Commit**

```bash
git add crates/cp-ast-wasm/
git commit -m "feat(wasm): create cp-ast-wasm crate skeleton

Minimal wasm-bindgen crate with version() and render_input_format().
wasm-pack build verified.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 2: All wasm render functions

**Files:**
- Modify: `crates/cp-ast-wasm/src/lib.rs`

- [ ] **Step 1: Replace `lib.rs` with complete wasm API**

Replace `crates/cp-ast-wasm/src/lib.rs` with:

```rust
//! WebAssembly bindings for the cp-ast-core AST viewer.
//!
//! All functions accept and return strings (JSON String API).
//! The frontend never interprets AST structure directly.

use wasm_bindgen::prelude::*;

use cp_ast_core::render_tex::{SectionMode, TexOptions};
use cp_ast_tree::TreeOptions;

mod presets;

/// Returns the crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Renders input format as human-readable text.
///
/// Example output: `"N\nA_1 A_2 ... A_N"`
#[wasm_bindgen]
pub fn render_input_format(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_core::render::render_input(&engine))
}

/// Renders structure AST as an ASCII tree.
///
/// Example output: `"Sequence (#0)\n├── Scalar(N) (#1)\n└── Array(A) (#2)"`
#[wasm_bindgen]
pub fn render_structure_tree(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_tree::render_structure_tree(
        &engine,
        &TreeOptions::default(),
    ))
}

/// Renders constraints as human-readable text.
///
/// Example output: `"1 ≤ N ≤ 2×10^5\n0 ≤ A_i ≤ 10^9"`
#[wasm_bindgen]
pub fn render_constraints_text(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_core::render::render_constraints(&engine))
}

/// Renders constraint AST as an ASCII tree.
#[wasm_bindgen]
pub fn render_constraint_tree(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_tree::render_constraint_tree(
        &engine,
        &TreeOptions::default(),
    ))
}

/// Renders input format as TeX (KaTeX-compatible display math).
///
/// Output is wrapped in `\[\begin{array}{l}...\end{array}\]`.
#[wasm_bindgen]
pub fn render_input_tex(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let opts = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    Ok(cp_ast_core::render_tex::render_input_tex(&engine, &opts).tex)
}

/// Renders constraints as TeX (itemize list with inline math).
///
/// Output uses `\begin{itemize}\item $...$\end{itemize}` format.
#[wasm_bindgen]
pub fn render_constraints_tex(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let opts = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    Ok(cp_ast_core::render_tex::render_constraints_tex(&engine, &opts).tex)
}

/// Renders full TeX (input + constraints combined, Fragment mode).
#[wasm_bindgen]
pub fn render_full_tex(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let opts = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    Ok(cp_ast_core::render_tex::render_full_tex(&engine, &opts).tex)
}

/// Generates a sample test case from the AST.
///
/// `seed` is u32 (JS Number compatible, no BigInt needed).
/// Same seed → same output (deterministic).
#[wasm_bindgen]
pub fn generate_sample(document_json: &str, seed: u32) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let sample = cp_ast_core::sample::generate(&engine, u64::from(seed))
        .map_err(|e| JsError::new(&format!("{e:?}")))?;
    Ok(cp_ast_core::sample::sample_to_text(&engine, &sample))
}

/// Returns preset list as JSON: `[{"name": "...", "description": "..."}]`
#[wasm_bindgen]
pub fn list_presets() -> String {
    serde_json::to_string(&presets::list()).expect("preset list serialization should not fail")
}

/// Returns preset document JSON for the given name.
#[wasm_bindgen]
pub fn get_preset(name: &str) -> Result<String, JsError> {
    let engine = presets::build(name)
        .ok_or_else(|| JsError::new(&format!("unknown preset: {name}")))?;
    cp_ast_json::serialize_ast(&engine).map_err(|e| JsError::new(&e.to_string()))
}

fn deserialize(json: &str) -> Result<cp_ast_core::operation::AstEngine, JsError> {
    cp_ast_json::deserialize_ast(json).map_err(|e| JsError::new(&e.to_string()))
}
```

Note: This references `mod presets;` which doesn't exist yet — create a placeholder:

Create `crates/cp-ast-wasm/src/presets.rs`:

```rust
//! Preset AST definitions — populated in Task 3.

use serde::Serialize;

#[derive(Serialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
}

pub fn list() -> Vec<PresetInfo> {
    vec![]
}

pub fn build(_name: &str) -> Option<cp_ast_core::operation::AstEngine> {
    None
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo build -p cp-ast-wasm`
Expected: compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add crates/cp-ast-wasm/src/
git commit -m "feat(wasm): add all render/generate/preset wasm API functions

10 wasm-bindgen functions: render_input_format, render_structure_tree,
render_constraints_text, render_constraint_tree, render_input_tex,
render_constraints_tex, render_full_tex, generate_sample,
list_presets, get_preset. Presets module is a placeholder.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 3: Presets module

**Files:**
- Modify: `crates/cp-ast-wasm/src/presets.rs`

- [ ] **Step 1: Implement all 9 presets**

Replace `crates/cp-ast-wasm/src/presets.rs` with:

```rust
//! Preset AST definitions for the viewer frontend.
//!
//! Each preset builds a representative `AstEngine` covering a different
//! competitive programming input pattern.

use cp_ast_core::constraint::types::{
    ArithOp, CharSetSpec, DistinctUnit, PropertyTag, SortOrder,
};
use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, Literal, NodeId, NodeKind, NodeKindHint, Reference};
use serde::Serialize;

#[derive(Serialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
}

#[must_use]
pub fn list() -> Vec<PresetInfo> {
    vec![
        PresetInfo {
            name: "scalar_only".into(),
            description: "スカラー値のみ (N)".into(),
        },
        PresetInfo {
            name: "scalar_array".into(),
            description: "N + A_1..A_N".into(),
        },
        PresetInfo {
            name: "tuple_repeat".into(),
            description: "N + (A_i, B_i) × N".into(),
        },
        PresetInfo {
            name: "matrix".into(),
            description: "H×W 行列".into(),
        },
        PresetInfo {
            name: "choice".into(),
            description: "タグ分岐 (クエリ型)".into(),
        },
        PresetInfo {
            name: "graph_simple".into(),
            description: "N頂点M辺の単純グラフ".into(),
        },
        PresetInfo {
            name: "sorted_distinct".into(),
            description: "ソート済み + 相異なる配列".into(),
        },
        PresetInfo {
            name: "string_problem".into(),
            description: "文字列制約 (CharSet, StringLength)".into(),
        },
        PresetInfo {
            name: "hole_structure".into(),
            description: "Hole を含む未完成 AST".into(),
        },
    ]
}

#[must_use]
pub fn build(name: &str) -> Option<AstEngine> {
    match name {
        "scalar_only" => Some(scalar_only()),
        "scalar_array" => Some(scalar_array()),
        "tuple_repeat" => Some(tuple_repeat()),
        "matrix" => Some(matrix()),
        "choice" => Some(choice()),
        "graph_simple" => Some(graph_simple()),
        "sorted_distinct" => Some(sorted_distinct()),
        "string_problem" => Some(string_problem()),
        "hole_structure" => Some(hole_structure()),
        _ => None,
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn pow10(exp: i64) -> Expression {
    Expression::Pow {
        base: Box::new(Expression::Lit(10)),
        exp: Box::new(Expression::Lit(exp)),
    }
}

fn var_ref(id: NodeId) -> Reference {
    Reference::VariableRef(id)
}

fn var_expr(id: NodeId) -> Expression {
    Expression::Var(var_ref(id))
}

fn indexed_ref(target: NodeId, index: &str) -> Reference {
    Reference::IndexedRef {
        target,
        indices: vec![Ident::new(index)],
    }
}

fn set_root_sequence(engine: &mut AstEngine, children: Vec<NodeId>) {
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence { children });
}

fn add_int_type(engine: &mut AstEngine, node: NodeId, target: Reference) {
    engine.constraints.add(
        Some(node),
        Constraint::TypeDecl {
            target,
            expected: ExpectedType::Int,
        },
    );
}

fn add_range(engine: &mut AstEngine, node: NodeId, target: Reference, lo: Expression, hi: Expression) {
    engine.constraints.add(
        Some(node),
        Constraint::Range {
            target,
            lower: lo,
            upper: hi,
        },
    );
}

// ── Presets ─────────────────────────────────────────────────────────

/// Simplest: just N.
/// Input: `N`
/// Constraints: 1 ≤ N ≤ 10^9, N is integer
fn scalar_only() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    set_root_sequence(&mut engine, vec![n]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(9));
    engine
}

/// Classic array: N then A_1..A_N.
/// Input: `N\nA_1 A_2 ... A_N`
/// Constraints: 1 ≤ N ≤ 10^5, 1 ≤ A_i ≤ 10^9
fn scalar_array() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    let a = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: var_expr(n),
    });
    set_root_sequence(&mut engine, vec![n, a]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, a, indexed_ref(a, "i"));
    add_range(
        &mut engine,
        a,
        indexed_ref(a, "i"),
        Expression::Lit(1),
        pow10(9),
    );
    engine
}

/// Tuple repeat: N then (A_i, B_i) × N.
/// Input: `N\nA_1 B_1\n...\nA_N B_N`
fn tuple_repeat() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    let a = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("A") });
    let b = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("B") });
    let pair = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![a, b] });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(n),
        index_var: Some(Ident::new("i")),
        body: vec![pair],
    });
    set_root_sequence(&mut engine, vec![n, repeat]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, a, var_ref(a));
    add_range(&mut engine, a, var_ref(a), Expression::Lit(1), pow10(9));
    add_int_type(&mut engine, b, var_ref(b));
    add_range(&mut engine, b, var_ref(b), Expression::Lit(1), pow10(9));
    engine
}

/// Matrix: H×W grid.
/// Input: `H W\nA_{1,1}..A_{1,W}\n...\nA_{H,1}..A_{H,W}`
fn matrix() -> AstEngine {
    let mut engine = AstEngine::new();
    let h = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("H") });
    let w = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("W") });
    let header = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![h, w] });
    let mat = engine.structure.add_node(NodeKind::Matrix {
        name: Ident::new("A"),
        rows: var_ref(h),
        cols: var_ref(w),
    });
    set_root_sequence(&mut engine, vec![header, mat]);
    add_int_type(&mut engine, h, var_ref(h));
    add_range(
        &mut engine,
        h,
        var_ref(h),
        Expression::Lit(1),
        Expression::Lit(1000),
    );
    add_int_type(&mut engine, w, var_ref(w));
    add_range(
        &mut engine,
        w,
        var_ref(w),
        Expression::Lit(1),
        Expression::Lit(1000),
    );
    let mat_ref = Reference::IndexedRef {
        target: mat,
        indices: vec![Ident::new("i"), Ident::new("j")],
    };
    add_int_type(&mut engine, mat, mat_ref.clone());
    add_range(
        &mut engine,
        mat,
        mat_ref,
        Expression::Lit(0),
        Expression::Lit(9),
    );
    engine
}

/// Query-type with Choice: N, Q then Q queries with tag branching.
/// Query type 1: `1 x` (add), type 2: `2 k` (get k-th)
fn choice() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    let q = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("Q") });
    let header = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![n, q] });
    let x = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("x") });
    let k = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("k") });
    let t = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("T") });
    let query = engine.structure.add_node(NodeKind::Choice {
        tag: var_ref(t),
        variants: vec![
            (Literal::IntLit(1), vec![x]),
            (Literal::IntLit(2), vec![k]),
        ],
    });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(q),
        index_var: Some(Ident::new("i")),
        body: vec![query],
    });
    set_root_sequence(&mut engine, vec![header, repeat]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, q, var_ref(q));
    add_range(&mut engine, q, var_ref(q), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, x, var_ref(x));
    add_range(&mut engine, x, var_ref(x), Expression::Lit(1), pow10(9));
    add_int_type(&mut engine, k, var_ref(k));
    add_range(&mut engine, k, var_ref(k), Expression::Lit(1), var_expr(n));
    engine
}

/// Simple graph: N vertices, M edges.
/// Input: `N M\nu_1 v_1\n...\nu_M v_M`
fn graph_simple() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    let m = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("M") });
    let header = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![n, m] });
    let u = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("u") });
    let v = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("v") });
    let edge = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![u, v] });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(m),
        index_var: Some(Ident::new("i")),
        body: vec![edge],
    });
    set_root_sequence(&mut engine, vec![header, repeat]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(2), pow10(5));
    add_int_type(&mut engine, m, var_ref(m));
    add_range(&mut engine, m, var_ref(m), Expression::Lit(1), var_expr(n));
    add_int_type(&mut engine, u, var_ref(u));
    add_range(&mut engine, u, var_ref(u), Expression::Lit(1), var_expr(n));
    add_int_type(&mut engine, v, var_ref(v));
    add_range(&mut engine, v, var_ref(v), Expression::Lit(1), var_expr(n));
    engine.constraints.add(
        None,
        Constraint::Property {
            target: Reference::Unresolved(Ident::new("G")),
            tag: PropertyTag::Simple,
        },
    );
    engine.constraints.add(
        None,
        Constraint::Property {
            target: Reference::Unresolved(Ident::new("G")),
            tag: PropertyTag::Connected,
        },
    );
    engine
}

/// Sorted + distinct array.
/// Input: `N\nA_1 A_2 ... A_N` (sorted ascending, all distinct)
fn sorted_distinct() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    let a = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: var_expr(n),
    });
    set_root_sequence(&mut engine, vec![n, a]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, a, indexed_ref(a, "i"));
    add_range(
        &mut engine,
        a,
        indexed_ref(a, "i"),
        Expression::Lit(1),
        pow10(9),
    );
    engine.constraints.add(
        Some(a),
        Constraint::Sorted {
            elements: var_ref(a),
            order: SortOrder::Ascending,
        },
    );
    engine.constraints.add(
        Some(a),
        Constraint::Distinct {
            elements: var_ref(a),
            unit: DistinctUnit::Element,
        },
    );
    engine
}

/// String problem: N strings with charset and length constraints.
/// Input: `N\nS_1\n...\nS_N`
fn string_problem() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    let s = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("S") });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(n),
        index_var: Some(Ident::new("i")),
        body: vec![s],
    });
    set_root_sequence(&mut engine, vec![n, repeat]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    engine.constraints.add(
        Some(s),
        Constraint::TypeDecl {
            target: var_ref(s),
            expected: ExpectedType::Str,
        },
    );
    engine.constraints.add(
        Some(s),
        Constraint::CharSet {
            target: var_ref(s),
            charset: CharSetSpec::LowerAlpha,
        },
    );
    engine.constraints.add(
        Some(s),
        Constraint::StringLength {
            target: var_ref(s),
            min: Expression::Lit(1),
            max: pow10(5),
        },
    );
    engine.constraints.add(
        None,
        Constraint::SumBound {
            variable: Reference::Unresolved(Ident::new("|S_i|")),
            upper: Expression::BinOp {
                op: ArithOp::Mul,
                lhs: Box::new(Expression::Lit(2)),
                rhs: Box::new(pow10(5)),
            },
        },
    );
    engine
}

/// Incomplete AST with Hole nodes.
/// Input: `N\n<hole: AnyArray>\n<hole>`
fn hole_structure() -> AstEngine {
    let mut engine = AstEngine::new();
    let n = engine
        .structure
        .add_node(NodeKind::Scalar { name: Ident::new("N") });
    let hole1 = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyArray),
    });
    let hole2 = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    set_root_sequence(&mut engine, vec![n, hole1, hole2]);
    add_int_type(&mut engine, n, var_ref(n));
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_presets_build_and_serialize() {
        for info in list() {
            let engine = build(&info.name)
                .unwrap_or_else(|| panic!("preset '{}' not found", info.name));
            let json = cp_ast_json::serialize_ast(&engine)
                .unwrap_or_else(|e| panic!("preset '{}' failed to serialize: {e}", info.name));
            assert!(!json.is_empty(), "preset '{}' produced empty JSON", info.name);
            // Verify roundtrip
            let _ = cp_ast_json::deserialize_ast(&json)
                .unwrap_or_else(|e| panic!("preset '{}' failed roundtrip: {e}", info.name));
        }
    }

    #[test]
    fn all_presets_render() {
        for info in list() {
            let engine = build(&info.name).unwrap();
            // All render functions should succeed (not panic)
            let _ = cp_ast_core::render::render_input(&engine);
            let _ = cp_ast_core::render::render_constraints(&engine);
            let _ = cp_ast_tree::render_structure_tree(&engine, &TreeOptions::default());
            let _ = cp_ast_tree::render_constraint_tree(&engine, &TreeOptions::default());
        }
    }

    #[test]
    fn unknown_preset_returns_none() {
        assert!(build("nonexistent").is_none());
    }

    #[test]
    fn preset_list_has_9_entries() {
        assert_eq!(list().len(), 9);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p cp-ast-wasm -v`
Expected: all 4 tests pass.

- [ ] **Step 3: Run clippy**

Run: `cargo clippy -p cp-ast-wasm --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 4: Build wasm to verify**

Run: `wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm`
Expected: compiles successfully.

- [ ] **Step 5: Commit**

```bash
git add crates/cp-ast-wasm/src/presets.rs
git commit -m "feat(wasm): add 9 preset AST definitions with tests

scalar_only, scalar_array, tuple_repeat, matrix, choice,
graph_simple, sorted_distinct, string_problem, hole_structure.
All presets verified: build, serialize, roundtrip, render.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 4: web/ Preact + Vite scaffold

**Files:**
- Create: `web/package.json`
- Create: `web/vite.config.ts`
- Create: `web/tsconfig.json`
- Create: `web/index.html`
- Create: `web/src/main.tsx`

- [ ] **Step 1: Create `package.json`**

Create `web/package.json`:

```json
{
  "name": "cp-ast-viewer",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "preact": "^10.19.0",
    "@preact/signals": "^1.2.0",
    "katex": "^0.16.0"
  },
  "devDependencies": {
    "@preact/preset-vite": "^2.8.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0",
    "@types/katex": "^0.16.0"
  }
}
```

- [ ] **Step 2: Create `vite.config.ts`**

Create `web/vite.config.ts`:

```typescript
import { defineConfig } from 'vite';
import preact from '@preact/preset-vite';

export default defineConfig({
  plugins: [preact()],
  root: '.',
  build: {
    outDir: 'dist',
  },
});
```

- [ ] **Step 3: Create `tsconfig.json`**

Create `web/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "jsxImportSource": "preact",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true
  },
  "include": ["src"]
}
```

- [ ] **Step 4: Create `index.html`**

Create `web/index.html`:

```html
<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>AST Viewer</title>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.tsx"></script>
</body>
</html>
```

- [ ] **Step 5: Create `main.tsx` (hello world)**

Create `web/src/main.tsx`:

```tsx
import { render } from 'preact';

function App() {
  return <h1>AST Viewer — scaffold OK</h1>;
}

render(<App />, document.getElementById('app')!);
```

- [ ] **Step 6: Install dependencies and verify**

Run:
```bash
cd web && npm install
```
Expected: `node_modules/` created, no errors.

Run:
```bash
cd web && npx vite --host 0.0.0.0 --port 5173 &
sleep 3
curl -s http://localhost:5173/ | head -5
# Kill the server
kill %1
```
Expected: HTML page served.

- [ ] **Step 7: Commit**

```bash
git add web/package.json web/vite.config.ts web/tsconfig.json web/index.html web/src/main.tsx
git commit -m "feat(web): Preact + Vite scaffold

Hello world app with TypeScript, KaTeX dependency installed.
Dev server verified.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 5: wasm integration

**Files:**
- Create: `web/src/wasm.ts`
- Modify: `web/src/main.tsx`

Prerequisite: Tasks 1-3 (wasm crate built) and Task 4 (web scaffold).

- [ ] **Step 1: Build wasm to web/wasm/**

Run:
```bash
wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm
```
Expected: `web/wasm/` populated with JS glue, wasm binary, and TS declarations.

- [ ] **Step 2: Create `wasm.ts`**

Create `web/src/wasm.ts`:

```typescript
import init from '../wasm/cp_ast_wasm';

export {
  render_input_format,
  render_structure_tree,
  render_constraints_text,
  render_constraint_tree,
  render_input_tex,
  render_constraints_tex,
  render_full_tex,
  generate_sample,
  list_presets,
  get_preset,
  version,
} from '../wasm/cp_ast_wasm';

let initialized = false;

export async function initWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}
```

- [ ] **Step 3: Update `main.tsx` to verify wasm loading**

Replace `web/src/main.tsx` with:

```tsx
import { render } from 'preact';
import { initWasm, list_presets, version } from './wasm';

async function main() {
  await initWasm();
  const ver = version();
  const presets = JSON.parse(list_presets());
  console.log(`wasm v${ver} loaded, ${presets.length} presets`);

  render(
    <div>
      <h1>🌳 AST Viewer — wasm loaded (v{ver})</h1>
      <p>{presets.length} presets available</p>
      <ul>
        {presets.map((p: { name: string; description: string }) => (
          <li key={p.name}><strong>{p.name}</strong> — {p.description}</li>
        ))}
      </ul>
    </div>,
    document.getElementById('app')!,
  );
}

main().catch(console.error);
```

- [ ] **Step 4: Verify in browser**

Run:
```bash
cd web && npx vite --host 0.0.0.0 --port 5173
```
Open `http://localhost:5173/` in browser. Expected: page showing version and 9 preset names.

Check browser console for `wasm v0.1.0 loaded, 9 presets`.

Stop the dev server after verification.

- [ ] **Step 5: Commit**

```bash
git add web/src/wasm.ts web/src/main.tsx
git commit -m "feat(web): integrate wasm module loading

wasm.ts re-exports all wasm functions. main.tsx verifies
wasm init and lists all 9 presets on page.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 6: App shell + state + CSS

**Files:**
- Create: `web/src/state.ts`
- Create: `web/src/app.tsx`
- Create: `web/src/index.css`
- Modify: `web/src/main.tsx`
- Create: `web/src/components/viewer/ViewerPage.tsx` (shell)
- Create: `web/src/components/preview/PreviewPage.tsx` (shell)

- [ ] **Step 1: Create `state.ts`**

Create `web/src/state.ts`:

```typescript
import { signal, computed } from '@preact/signals';
import {
  render_input_format,
  render_structure_tree,
  render_constraints_text,
  render_constraint_tree,
  render_input_tex,
  render_constraints_tex,
  generate_sample,
  get_preset,
} from './wasm';

// ── Page routing ────────────────────────────────────────────────────

export const currentPage = signal<'viewer' | 'preview'>(
  window.location.hash === '#/preview' ? 'preview' : 'viewer',
);

window.addEventListener('hashchange', () => {
  currentPage.value = window.location.hash === '#/preview' ? 'preview' : 'viewer';
});

// ── Viewer state ────────────────────────────────────────────────────

export const documentJson = signal<string>('');
export const activePreset = signal<string>('scalar_array');
export const sampleSeed = signal<number>(0);
export const activePreviewTab = signal<'tex' | 'sample'>('tex');
export const structureAstMode = signal<boolean>(false);
export const constraintAstMode = signal<boolean>(false);

// ── Derived state ───────────────────────────────────────────────────

function safeCall<T>(fn: () => T, fallback: T): T {
  try {
    return fn();
  } catch (e) {
    console.error(e);
    return fallback;
  }
}

export const structureText = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(
    () =>
      structureAstMode.value
        ? render_structure_tree(documentJson.value)
        : render_input_format(documentJson.value),
    'Error rendering structure',
  );
});

export const constraintText = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(
    () =>
      constraintAstMode.value
        ? render_constraint_tree(documentJson.value)
        : render_constraints_text(documentJson.value),
    'Error rendering constraints',
  );
});

export const inputTexString = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(() => render_input_tex(documentJson.value), '');
});

export const constraintsTexString = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(() => render_constraints_tex(documentJson.value), '');
});

export const sampleText = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(
    () => generate_sample(documentJson.value, sampleSeed.value),
    'Error generating sample',
  );
});

// ── Actions ─────────────────────────────────────────────────────────

export function loadPreset(name: string): void {
  try {
    documentJson.value = get_preset(name);
    activePreset.value = name;
  } catch (e) {
    console.error('Failed to load preset:', e);
  }
}

export function shuffleSeed(): void {
  sampleSeed.value = Math.floor(Math.random() * 0xffffffff);
}
```

- [ ] **Step 2: Create `app.tsx`**

Create `web/src/app.tsx`:

```tsx
import { currentPage } from './state';
import { ViewerPage } from './components/viewer/ViewerPage';
import { PreviewPage } from './components/preview/PreviewPage';

export function App() {
  return (
    <div class="app">
      <header class="header">
        <h1 class="header-title">🌳 AST Viewer</h1>
        <nav class="header-nav">
          <a
            href="#/viewer"
            class={`nav-link ${currentPage.value === 'viewer' ? 'active' : ''}`}
          >
            Viewer
          </a>
          <a
            href="#/preview"
            class={`nav-link ${currentPage.value === 'preview' ? 'active' : ''}`}
          >
            Preview
          </a>
        </nav>
      </header>
      <main class="main">
        {currentPage.value === 'viewer' ? <ViewerPage /> : <PreviewPage />}
      </main>
    </div>
  );
}
```

- [ ] **Step 3: Create ViewerPage shell**

Create directories: `mkdir -p web/src/components/viewer web/src/components/preview`

Create `web/src/components/viewer/ViewerPage.tsx`:

```tsx
export function ViewerPage() {
  return (
    <div class="viewer-page">
      <div class="viewer-panes">
        <div class="pane">
          <div class="pane-header"><span class="pane-title">📝 入力形式</span></div>
          <pre class="pane-content">Structure pane placeholder</pre>
        </div>
        <div class="pane">
          <div class="pane-header"><span class="pane-title">📋 制約</span></div>
          <pre class="pane-content">Constraint pane placeholder</pre>
        </div>
        <div class="pane">
          <div class="pane-header"><span class="pane-title">Preview</span></div>
          <pre class="pane-content">Preview pane placeholder</pre>
        </div>
      </div>
      <div class="toolbar">Toolbar placeholder</div>
    </div>
  );
}
```

- [ ] **Step 4: Create PreviewPage shell**

Create `web/src/components/preview/PreviewPage.tsx`:

```tsx
export function PreviewPage() {
  return (
    <div class="preview-page">
      <p>Preview page — card gallery coming in Task 9</p>
    </div>
  );
}
```

- [ ] **Step 5: Create `index.css`**

Create `web/src/index.css`:

```css
@import 'katex/dist/katex.min.css';

:root {
  --bg-primary: #1e1e2e;
  --bg-secondary: #313244;
  --bg-surface: #45475a;
  --text-primary: #cdd6f4;
  --text-secondary: #a6adc8;
  --accent: #89b4fa;
  --accent-hover: #74c7ec;
  --border: #585b70;
  --font-mono: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  --font-sans: 'Inter', system-ui, -apple-system, sans-serif;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: var(--font-sans);
  background: var(--bg-primary);
  color: var(--text-primary);
  line-height: 1.6;
}

/* ── App layout ────────────────────────────────────────────────── */

.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 1rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.header-title { font-size: 1.2rem; font-weight: 600; }
.header-nav { display: flex; gap: 0.5rem; }

.nav-link {
  color: var(--text-secondary);
  text-decoration: none;
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  transition: all 0.2s;
}
.nav-link:hover { color: var(--text-primary); background: var(--bg-surface); }
.nav-link.active { color: var(--accent); background: var(--bg-surface); }

.main { flex: 1; overflow: hidden; }

/* ── Viewer page ───────────────────────────────────────────────── */

.viewer-page {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.viewer-panes {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 1px;
  flex: 1;
  overflow: hidden;
  background: var(--border);
}

/* ── Pane ──────────────────────────────────────────────────────── */

.pane {
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
}

.pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  min-height: 2.5rem;
}

.pane-title { font-size: 0.875rem; font-weight: 600; }

.pane-content {
  flex: 1;
  overflow: auto;
  padding: 0.75rem;
  font-family: var(--font-mono);
  font-size: 0.8rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.pane-content-scroll {
  flex: 1;
  overflow: auto;
  padding: 0.75rem;
}

/* ── Toggle / Tab buttons ──────────────────────────────────────── */

.toggle-btn {
  font-size: 0.7rem;
  padding: 0.15rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}
.toggle-btn:hover { border-color: var(--accent); color: var(--accent); }
.toggle-btn.active {
  background: var(--accent);
  color: var(--bg-primary);
  border-color: var(--accent);
}

.tab-buttons { display: flex; gap: 0.25rem; }

.tab-btn {
  font-size: 0.75rem;
  padding: 0.15rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}
.tab-btn.active {
  background: var(--accent);
  color: var(--bg-primary);
  border-color: var(--accent);
}

/* ── TeX rendering ─────────────────────────────────────────────── */

.tex-tab { padding: 0.5rem 0; }
.tex-section { margin-bottom: 1rem; }
.tex-section-label {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
  font-weight: 600;
}

.tex-fallback {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  padding: 0.5rem;
  background: var(--bg-secondary);
  border-radius: 4px;
}

.constraint-tex-list {
  list-style: disc;
  padding-left: 1.5rem;
  font-size: 0.9rem;
}
.constraint-tex-list li { margin-bottom: 0.25rem; }

.sample-output {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  line-height: 1.5;
  white-space: pre-wrap;
}

/* ── Toolbar ───────────────────────────────────────────────────── */

.toolbar {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 1rem;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
  flex-wrap: wrap;
}

.toolbar-group { display: flex; align-items: center; gap: 0.5rem; }
.toolbar-label { font-size: 0.8rem; color: var(--text-secondary); font-weight: 600; }

.toolbar-select {
  font-size: 0.8rem;
  padding: 0.25rem 0.5rem;
  background: var(--bg-surface);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: 4px;
  min-width: 200px;
}

.toolbar-input {
  font-size: 0.8rem;
  padding: 0.25rem 0.5rem;
  background: var(--bg-surface);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: 4px;
  width: 120px;
  font-family: var(--font-mono);
}

.toolbar-btn {
  font-size: 0.8rem;
  padding: 0.25rem 0.75rem;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s;
}
.toolbar-btn:hover { background: var(--accent-hover); }

.toolbar-status {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-left: auto;
}

/* ── Preview page ──────────────────────────────────────────────── */

.preview-page { padding: 1rem; overflow: auto; height: 100%; }

.preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 1rem;
}

/* ── Preview card ──────────────────────────────────────────────── */

.preview-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: border-color 0.2s, transform 0.2s;
}
.preview-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--bg-surface);
}
.card-title { font-size: 0.85rem; font-weight: 600; }
.card-name { font-size: 0.7rem; color: var(--text-secondary); font-family: var(--font-mono); }

.card-section {
  padding: 0.5rem 0.75rem;
  border-top: 1px solid var(--border);
}
.card-section-label {
  font-size: 0.7rem;
  color: var(--text-secondary);
  font-weight: 600;
  margin-bottom: 0.25rem;
}
.card-content {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  line-height: 1.4;
  max-height: 6rem;
  overflow: hidden;
  white-space: pre-wrap;
}

/* ── KaTeX overrides ───────────────────────────────────────────── */

.katex-display { margin: 0.5rem 0; }
.katex { color: var(--text-primary); }
```

- [ ] **Step 6: Update `main.tsx` to use App**

Replace `web/src/main.tsx` with:

```tsx
import { render } from 'preact';
import { initWasm } from './wasm';
import { loadPreset } from './state';
import { App } from './app';
import './index.css';

async function main() {
  await initWasm();
  loadPreset('scalar_array');
  render(<App />, document.getElementById('app')!);
}

main().catch(console.error);
```

- [ ] **Step 7: Verify in browser**

Run: `cd web && npx vite --host 0.0.0.0 --port 5173`

Expected: dark-themed page with header ("🌳 AST Viewer", Viewer/Preview nav links), 3-column placeholder panes, toolbar placeholder. Page routing works (click Viewer/Preview).

Stop the dev server after verification.

- [ ] **Step 8: Commit**

```bash
git add web/src/
git commit -m "feat(web): app shell with state management and dark theme CSS

Hash-based router (Viewer/Preview pages), Preact Signals state,
computed render outputs, full CSS with Catppuccin-inspired dark theme.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 7: StructurePane + ConstraintPane + ViewerPage wiring

**Files:**
- Create: `web/src/components/viewer/StructurePane.tsx`
- Create: `web/src/components/viewer/ConstraintPane.tsx`
- Modify: `web/src/components/viewer/ViewerPage.tsx`

- [ ] **Step 1: Create `StructurePane.tsx`**

Create `web/src/components/viewer/StructurePane.tsx`:

```tsx
import { structureText, structureAstMode } from '../../state';

export function StructurePane() {
  return (
    <div class="pane">
      <div class="pane-header">
        <span class="pane-title">📝 入力形式</span>
        <button
          class={`toggle-btn ${structureAstMode.value ? 'active' : ''}`}
          onClick={() => {
            structureAstMode.value = !structureAstMode.value;
          }}
        >
          AST
        </button>
      </div>
      <pre class="pane-content">{structureText.value}</pre>
    </div>
  );
}
```

- [ ] **Step 2: Create `ConstraintPane.tsx`**

Create `web/src/components/viewer/ConstraintPane.tsx`:

```tsx
import { constraintText, constraintAstMode } from '../../state';

export function ConstraintPane() {
  return (
    <div class="pane">
      <div class="pane-header">
        <span class="pane-title">📋 制約</span>
        <button
          class={`toggle-btn ${constraintAstMode.value ? 'active' : ''}`}
          onClick={() => {
            constraintAstMode.value = !constraintAstMode.value;
          }}
        >
          AST
        </button>
      </div>
      <pre class="pane-content">{constraintText.value}</pre>
    </div>
  );
}
```

- [ ] **Step 3: Update `ViewerPage.tsx`**

Replace `web/src/components/viewer/ViewerPage.tsx` with:

```tsx
import { StructurePane } from './StructurePane';
import { ConstraintPane } from './ConstraintPane';

export function ViewerPage() {
  return (
    <div class="viewer-page">
      <div class="viewer-panes">
        <StructurePane />
        <ConstraintPane />
        <div class="pane">
          <div class="pane-header">
            <span class="pane-title">Preview</span>
          </div>
          <pre class="pane-content">Preview pane — coming in Task 8</pre>
        </div>
      </div>
      <div class="toolbar">Toolbar — coming in Task 8</div>
    </div>
  );
}
```

- [ ] **Step 4: Verify in browser**

Run: `cd web && npx vite --host 0.0.0.0 --port 5173`

Expected:
- Left pane shows `scalar_array` input format (e.g., "N\nA_1 A_2 ... A_N")
- Middle pane shows constraints (e.g., "1 ≤ N ≤ 10^5\n...")
- Clicking AST toggle switches to tree view (e.g., "Sequence (#0)\n├── Scalar(N) (#1)\n...")
- Clicking AST again switches back

Stop dev server after verification.

- [ ] **Step 5: Commit**

```bash
git add web/src/components/viewer/
git commit -m "feat(web): StructurePane and ConstraintPane with AST toggle

Both panes show human-readable text by default, switch to raw
AST tree when toggle is active. Wired to computed signals.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 8: PreviewPane + Toolbar + TeX rendering

**Files:**
- Create: `web/src/tex-renderer.ts`
- Create: `web/src/components/viewer/PreviewPane.tsx`
- Create: `web/src/components/viewer/Toolbar.tsx`
- Modify: `web/src/components/viewer/ViewerPage.tsx`

- [ ] **Step 1: Create `tex-renderer.ts`**

Create `web/src/tex-renderer.ts`:

```typescript
import katex from 'katex';

/**
 * Render input TeX (display math) to HTML.
 *
 * Input TeX is wrapped in `\[...\]` — we strip delimiters before
 * passing to KaTeX displayMode.
 */
export function renderInputTex(tex: string): string {
  if (!tex.trim()) return '';
  let content = tex.trim();
  // Strip \[...\] display math delimiters
  if (content.startsWith('\\[')) content = content.slice(2);
  if (content.endsWith('\\]')) content = content.slice(0, -2);
  content = content.trim();
  if (!content) return '';

  try {
    return katex.renderToString(content, {
      displayMode: true,
      throwOnError: false,
    });
  } catch {
    return `<pre class="tex-fallback">${escapeHtml(tex)}</pre>`;
  }
}

/**
 * Render constraint TeX to HTML.
 *
 * Constraint TeX uses `\begin{itemize}\item $...$\end{itemize}` format.
 * We parse individual items and render the `$...$` math content with KaTeX inline.
 */
export function renderConstraintsTex(tex: string): string {
  if (!tex.trim()) return '';

  // Extract \item lines
  const itemRegex = /\\item\s+(.+)/g;
  const items: string[] = [];
  let match;
  while ((match = itemRegex.exec(tex)) !== null) {
    items.push(match[1].trim());
  }

  if (items.length === 0) {
    return `<pre class="tex-fallback">${escapeHtml(tex)}</pre>`;
  }

  const rendered = items.map((item) => {
    // Render $...$ segments with KaTeX inline, keep other text as-is
    const html = item.replace(/\$([^$]+)\$/g, (_, math: string) => {
      try {
        return katex.renderToString(math, {
          displayMode: false,
          throwOnError: false,
        });
      } catch {
        return `<code>$${escapeHtml(math)}$</code>`;
      }
    });
    return `<li>${html}</li>`;
  });

  return `<ul class="constraint-tex-list">${rendered.join('\n')}</ul>`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}
```

- [ ] **Step 2: Create `PreviewPane.tsx`**

Create `web/src/components/viewer/PreviewPane.tsx`:

```tsx
import { useMemo } from 'preact/hooks';
import {
  activePreviewTab,
  inputTexString,
  constraintsTexString,
  sampleText,
} from '../../state';
import { renderInputTex, renderConstraintsTex } from '../../tex-renderer';

function TexTab() {
  const inputHtml = useMemo(
    () => renderInputTex(inputTexString.value),
    [inputTexString.value],
  );
  const constraintsHtml = useMemo(
    () => renderConstraintsTex(constraintsTexString.value),
    [constraintsTexString.value],
  );

  return (
    <div class="tex-tab">
      {inputHtml && (
        <div class="tex-section">
          <h4 class="tex-section-label">入力</h4>
          <div dangerouslySetInnerHTML={{ __html: inputHtml }} />
        </div>
      )}
      {constraintsHtml && (
        <div class="tex-section">
          <h4 class="tex-section-label">制約</h4>
          <div dangerouslySetInnerHTML={{ __html: constraintsHtml }} />
        </div>
      )}
    </div>
  );
}

function SampleTab() {
  return <pre class="sample-output">{sampleText.value}</pre>;
}

export function PreviewPane() {
  return (
    <div class="pane">
      <div class="pane-header">
        <span class="pane-title">Preview</span>
        <div class="tab-buttons">
          <button
            class={`tab-btn ${activePreviewTab.value === 'tex' ? 'active' : ''}`}
            onClick={() => {
              activePreviewTab.value = 'tex';
            }}
          >
            TeX
          </button>
          <button
            class={`tab-btn ${activePreviewTab.value === 'sample' ? 'active' : ''}`}
            onClick={() => {
              activePreviewTab.value = 'sample';
            }}
          >
            Sample
          </button>
        </div>
      </div>
      <div class="pane-content-scroll">
        {activePreviewTab.value === 'tex' ? <TexTab /> : <SampleTab />}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Create `Toolbar.tsx`**

Create `web/src/components/viewer/Toolbar.tsx`:

```tsx
import { useMemo } from 'preact/hooks';
import {
  activePreset,
  sampleSeed,
  loadPreset,
  shuffleSeed,
  documentJson,
} from '../../state';
import { list_presets } from '../../wasm';

interface PresetInfo {
  name: string;
  description: string;
}

export function Toolbar() {
  const presets: PresetInfo[] = useMemo(() => JSON.parse(list_presets()), []);

  return (
    <div class="toolbar">
      <div class="toolbar-group">
        <label class="toolbar-label">Preset:</label>
        <select
          class="toolbar-select"
          value={activePreset.value}
          onChange={(e) =>
            loadPreset((e.target as HTMLSelectElement).value)
          }
        >
          {presets.map((p) => (
            <option key={p.name} value={p.name}>
              {p.name} — {p.description}
            </option>
          ))}
        </select>
      </div>
      <div class="toolbar-group">
        <label class="toolbar-label">Seed:</label>
        <input
          class="toolbar-input"
          type="number"
          min={0}
          max={4294967295}
          value={sampleSeed.value}
          onInput={(e) => {
            const val = parseInt(
              (e.target as HTMLInputElement).value,
              10,
            );
            if (!isNaN(val) && val >= 0) sampleSeed.value = val;
          }}
        />
        <button class="toolbar-btn" onClick={shuffleSeed}>
          🔀 Shuffle
        </button>
      </div>
      <div class="toolbar-status">
        {documentJson.value ? '✓ Document loaded' : '— No document'}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Update ViewerPage to wire everything together**

Replace `web/src/components/viewer/ViewerPage.tsx` with:

```tsx
import { StructurePane } from './StructurePane';
import { ConstraintPane } from './ConstraintPane';
import { PreviewPane } from './PreviewPane';
import { Toolbar } from './Toolbar';

export function ViewerPage() {
  return (
    <div class="viewer-page">
      <div class="viewer-panes">
        <StructurePane />
        <ConstraintPane />
        <PreviewPane />
      </div>
      <Toolbar />
    </div>
  );
}
```

- [ ] **Step 5: Verify in browser**

Run: `cd web && npx vite --host 0.0.0.0 --port 5173`

Expected:
- All 3 panes populated with real content
- TeX tab shows KaTeX-rendered input format + constraint items
- Sample tab shows generated sample text (seed=0)
- Preset dropdown has 9 options, switching presets updates all panes
- Shuffle button generates new seed, Sample tab updates
- Seed input field allows direct entry

Stop dev server after verification.

- [ ] **Step 6: Commit**

```bash
git add web/src/
git commit -m "feat(web): PreviewPane (TeX + Sample), Toolbar, KaTeX rendering

TeX tab renders input math with KaTeX display mode, parses
constraint itemize items for inline KaTeX rendering.
Toolbar with preset selector, seed input, shuffle button.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 9: PreviewPage + PreviewCard

**Files:**
- Modify: `web/src/components/preview/PreviewPage.tsx`
- Create: `web/src/components/preview/PreviewCard.tsx`

- [ ] **Step 1: Create `PreviewCard.tsx`**

Create `web/src/components/preview/PreviewCard.tsx`:

```tsx
import { useMemo } from 'preact/hooks';
import {
  get_preset,
  render_input_format,
  render_constraints_text,
  generate_sample,
} from '../../wasm';
import { loadPreset } from '../../state';

interface PresetInfo {
  name: string;
  description: string;
}

export function PreviewCard({ preset }: { preset: PresetInfo }) {
  const data = useMemo(() => {
    try {
      const json = get_preset(preset.name);
      return {
        structure: render_input_format(json),
        constraints: render_constraints_text(json),
        sample: generate_sample(json, 0),
      };
    } catch (e) {
      return {
        structure: `Error: ${e}`,
        constraints: '',
        sample: '',
      };
    }
  }, [preset.name]);

  const handleClick = () => {
    loadPreset(preset.name);
    window.location.hash = '#/viewer';
  };

  return (
    <div class="preview-card" onClick={handleClick}>
      <div class="card-header">
        <span class="card-title">📝 {preset.description}</span>
        <span class="card-name">{preset.name}</span>
      </div>
      <div class="card-section">
        <div class="card-section-label">Structure</div>
        <pre class="card-content">{data.structure}</pre>
      </div>
      <div class="card-section">
        <div class="card-section-label">Constraints</div>
        <pre class="card-content">{data.constraints}</pre>
      </div>
      <div class="card-section">
        <div class="card-section-label">Sample (seed=0)</div>
        <pre class="card-content">{data.sample}</pre>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Update `PreviewPage.tsx`**

Replace `web/src/components/preview/PreviewPage.tsx` with:

```tsx
import { useMemo } from 'preact/hooks';
import { list_presets } from '../../wasm';
import { PreviewCard } from './PreviewCard';

interface PresetInfo {
  name: string;
  description: string;
}

export function PreviewPage() {
  const presets: PresetInfo[] = useMemo(() => JSON.parse(list_presets()), []);

  return (
    <div class="preview-page">
      <div class="preview-grid">
        {presets.map((p) => (
          <PreviewCard key={p.name} preset={p} />
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Verify in browser**

Run: `cd web && npx vite --host 0.0.0.0 --port 5173`

Navigate to Preview page (`#/preview`). Expected:
- 9 cards in a responsive grid
- Each card shows structure, constraints, and sample for seed=0
- Clicking a card navigates to Viewer page with that preset loaded
- `hole_structure` card may show limited output (expected — holes have no sample data)

Stop dev server after verification.

- [ ] **Step 4: Commit**

```bash
git add web/src/components/preview/
git commit -m "feat(web): PreviewPage with card gallery for all 9 presets

Each card displays structure, constraints, and sample (seed=0).
Clicking a card navigates to Viewer page with preset loaded.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 10: .gitignore + final verification

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Update `.gitignore`**

Append to `.gitignore`:

```
# Web frontend build outputs
web/wasm/
web/node_modules/
web/dist/
```

- [ ] **Step 2: Run full Rust test suite**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: all pass. The `cp-ast-wasm` tests (preset tests) pass natively.

- [ ] **Step 3: Rebuild wasm and verify frontend build**

Run:
```bash
wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm
cd web && npm run build
```
Expected: `web/dist/` produced with production build.

- [ ] **Step 4: Final browser verification**

Run: `cd web && npx vite --host 0.0.0.0 --port 5173`

Verify checklist:
- [ ] Viewer page: 3 panes populated
- [ ] AST toggle works on StructurePane
- [ ] AST toggle works on ConstraintPane
- [ ] TeX tab renders with KaTeX
- [ ] Sample tab shows generated output
- [ ] Preset dropdown switches all panes
- [ ] Seed input changes sample output
- [ ] Shuffle generates random seed
- [ ] Preview page shows 9 cards
- [ ] Card click navigates to Viewer with preset loaded
- [ ] All 9 presets work without errors in console

Stop dev server.

- [ ] **Step 5: Commit**

```bash
git add .gitignore
git commit -m "chore: add web build outputs to .gitignore

web/wasm/ (wasm-pack output), web/node_modules/, web/dist/.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Notes

### wasm-pack build command

```bash
wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm
```

The `--out-dir` is relative to the crate directory, hence `../../web/wasm`.

### TeX rendering strategy

The Rust TeX output has two parts with different KaTeX compatibility:

1. **Input TeX** (`render_input_tex`): Produces `\[\begin{array}{l}...\end{array}\]` — pure display math. KaTeX handles this natively after stripping `\[...\]` delimiters.

2. **Constraint TeX** (`render_constraints_tex`): Produces `\begin{itemize}\item $...$\end{itemize}`. KaTeX cannot render `\begin{itemize}` (it's a document command, not math). The frontend parses `\item` lines and renders the `$...$` math content with KaTeX inline mode.

Fallback: if KaTeX fails on any segment, raw TeX is shown in a `<pre>` block.

### Seed u32 limitation

The wasm API uses `u32` for seed (not `u64`) because JavaScript's `Number` type loses precision above 2^53. The `u32` range (0–4,294,967,295) provides sufficient randomness for sample generation. Internally, the Rust code casts `u64::from(seed)`.

### Preact Signals reactivity

All state lives in Preact Signals (`signal()` and `computed()`). When a signal value changes (e.g., toggling AST mode, changing preset), all dependent computed values automatically recompute, and components that read those values re-render. No manual state plumbing needed.

### Development workflow

```bash
# Terminal 1: Rust wasm rebuild (run after Rust code changes)
wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm

# Terminal 2: Vite dev server (auto-reloads on frontend changes)
cd web && npm run dev
```

For Rust-only changes (presets, render functions), rebuild wasm and refresh the browser.
For frontend-only changes, Vite HMR handles live reload automatically.
