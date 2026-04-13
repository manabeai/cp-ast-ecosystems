# AST Tree Viewer — Design Spec

## Overview

A console tree-rendering module for inspecting `StructureAST` and `ConstraintSet` as human-readable ASCII trees. Designed for AST development and debugging.

## Goals

1. Render Structure AST as an indented tree with `├──` / `└──` / `│` connectors
2. Render Constraint set as a flat grouped listing (by target node)
3. Render combined view (structure tree with constraints annotated inline)
4. Return `String` — caller decides where to print
5. Minimize coupling: NodeKind variant additions should not require changes in the tree viewer crate

## Architecture

### Two-layer design

```
┌────────────────────────────┐
│   cp-ast-tree (new crate)  │   Tree rendering logic
│   - render_structure_tree  │   Uses TreeVisitor trait
│   - render_constraint_tree │
│   - render_combined_tree   │
└──────────┬─────────────────┘
           │ depends on
┌──────────▼─────────────────┐
│      cp-ast-core           │
│   - TreeVisitor trait      │   Walk logic + default labels
│   - NodeKind, Constraint   │   Domain types
│   - AstEngine              │   Owns both trees
└────────────────────────────┘
```

### Layer 1: `TreeVisitor` trait in `cp-ast-core`

Location: `crates/cp-ast-core/src/structure/tree_visitor.rs`

```rust
/// Information about a single node for tree rendering.
pub struct NodeInfo {
    pub node_id: NodeId,
    pub label: String,           // e.g. "Scalar(N)", "Repeat(count=M)"
    pub children: Vec<NodeId>,   // ordered child node IDs
}

/// Trait for walking the structure AST in a visitor pattern.
/// Default implementation handles all NodeKind variants.
pub trait TreeVisitor {
    /// Extract display information from a node.
    fn node_info(&self, engine: &AstEngine, node_id: NodeId) -> Option<NodeInfo>;
}

/// Default visitor that knows about all NodeKind variants.
pub struct DefaultTreeVisitor;
```

The `DefaultTreeVisitor` implements `node_info()` with a match on `NodeKind`. When a new variant is added, only this single match in core needs updating. The external crate never matches on `NodeKind` directly.

**Label format per variant:**

| NodeKind | Label |
|----------|-------|
| Scalar | `Scalar(N)` |
| Array | `Array(A, len=N)` |
| Matrix | `Matrix(C, rows=H, cols=W)` |
| Tuple | `Tuple` |
| Repeat | `Repeat(count=M)` or `Repeat(count=M, i)` |
| Section | `Section` |
| Sequence | `Sequence` |
| Choice | `Choice(tag=T)` with variant sub-labels |
| Hole | `Hole` or `Hole(expected=AnyArray)` |

**Children extraction:** Each variant returns its structural children in order. For Choice, each variant `(literal, children)` is represented as a virtual child whose label is the literal value (e.g. `Variant(1)`, `Variant("ADD")`), containing the variant's child nodes.

Example Choice output:
```
Choice(tag=T)
├── Variant(1)
│   ├── Scalar(X)
│   └── Scalar(Y)
└── Variant(2)
    └── Scalar(Z)
```

### Layer 2: `cp-ast-tree` crate

Location: `crates/cp-ast-tree/`

Dependencies: `cp-ast-core`

#### Public API

```rust
/// Options for tree rendering.
#[derive(Debug, Clone)]
pub struct TreeOptions {
    /// Show NodeId next to each label (e.g. "#3 Scalar(N)").
    pub show_node_ids: bool,
    /// Show ConstraintId next to each constraint.
    pub show_constraint_ids: bool,
}

/// Render the structure AST as an ASCII tree.
pub fn render_structure_tree(engine: &AstEngine, options: &TreeOptions) -> String;

/// Render constraints grouped by target node.
pub fn render_constraint_tree(engine: &AstEngine, options: &TreeOptions) -> String;

/// Render structure tree with constraints annotated on each node.
pub fn render_combined_tree(engine: &AstEngine, options: &TreeOptions) -> String;
```

#### Output format

**Structure tree:**
```
Sequence
├── Tuple
│   ├── Scalar(N)
│   └── Scalar(M)
└── Repeat(count=M)
    └── Tuple
        ├── Scalar(u)
        └── Scalar(v)
```

**Constraint tree (grouped by node):**
```
Constraints
├── N
│   ├── Range: 1 ≤ N ≤ 100000
│   └── TypeDecl: Int
├── M
│   ├── Range: 1 ≤ M ≤ min(N*(N-1)/2, 200000)
│   └── TypeDecl: Int
├── u
│   └── Range: 1 ≤ u ≤ N
├── v
│   └── Range: 1 ≤ v ≤ N
└── (global)
    └── Property: Simple graph
```

**Combined tree:**
```
Sequence
├── Tuple
│   ├── Scalar(N)  [Range: 1..10^5, Int]
│   └── Scalar(M)  [Range: 1..min(...), Int]
└── Repeat(count=M)
    └── Tuple
        ├── Scalar(u)  [Range: 1..N, Int]
        └── Scalar(v)  [Range: 1..N, Int]
(global) Property: Simple graph
```

#### Tree drawing implementation

Hand-rolled ASCII tree renderer (~60 lines). Recursive function:

```rust
fn render_tree_lines(
    engine: &AstEngine,
    visitor: &impl TreeVisitor,
    node_id: NodeId,
    prefix: &str,
    is_last: bool,
    options: &TreeOptions,
    output: &mut String,
);
```

Uses `├── ` / `└── ` for branching, `│   ` / `    ` for continuation.

### Constraint rendering

Constraints don't form a tree, so `render_constraint_tree` builds a two-level grouping:

1. Top level: target nodes (resolved to names via `render_reference`)
2. Second level: individual constraints (using existing `render_constraints` from `cp-ast-core::render`)

Reuses `cp-ast-core::render::render_constraints` / `render_expression` / `render_reference` for constraint text. No duplication of rendering logic.

**Visibility note:** `render_reference` and `render_expression` in `render/mod.rs` are currently `pub(crate)`. This is fine because `DefaultTreeVisitor` lives in `cp-ast-core` and uses them for label generation. However, `render_constraint` (per-constraint) in `constraint_text.rs` is private. We will expose a new public function `render_single_constraint(engine, constraint) -> String` for the tree crate's per-node constraint listing.

### Combined view

`render_combined_tree` walks the structure tree and, for each leaf-like node (Scalar, Array, Matrix), appends a compact constraint summary in `[...]` brackets. Global constraints are listed at the bottom.

## Coupling analysis

| Event | What changes |
|-------|-------------|
| New NodeKind variant added | `DefaultTreeVisitor::node_info()` in core (one match arm) |
| New Constraint variant added | Nothing in tree crate; reuses `render_constraints` from core |
| Expression/Reference changes | Nothing; label generation uses existing `render_expression` |
| StructureAst API changes | `TreeVisitor` trait in core is updated alongside |

The tree viewer crate **never** matches on `NodeKind` or `Constraint` directly. All variant-specific logic lives in core's `DefaultTreeVisitor` and `render` module.

## File plan

```
crates/cp-ast-core/src/structure/tree_visitor.rs   (new)  — TreeVisitor trait + DefaultTreeVisitor
crates/cp-ast-core/src/structure/mod.rs            (edit) — pub mod tree_visitor + re-exports
crates/cp-ast-tree/Cargo.toml                      (new)  — workspace member
crates/cp-ast-tree/src/lib.rs                      (new)  — public API
crates/cp-ast-tree/src/structure_tree.rs            (new)  — render_structure_tree
crates/cp-ast-tree/src/constraint_tree.rs           (new)  — render_constraint_tree
crates/cp-ast-tree/src/combined_tree.rs             (new)  — render_combined_tree
crates/cp-ast-tree/src/drawing.rs                   (new)  — ASCII tree drawing primitives
crates/cp-ast-tree/tests/tree_basic.rs              (new)  — integration tests
Cargo.toml                                          (edit) — add to workspace members
```

## Testing

- Unit tests in `cp-ast-tree/tests/tree_basic.rs` using known AST structures
- Verify output strings match expected tree format
- Test with NodeId display on/off
- Test empty AST edge case
- Test constraint-only view
- Test combined view
