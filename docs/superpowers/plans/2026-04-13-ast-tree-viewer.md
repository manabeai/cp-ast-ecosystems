# AST Tree Viewer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a console tree-rendering module (`cp-ast-tree`) that displays StructureAST and ConstraintSet as human-readable ASCII trees, with a `TreeVisitor` trait in `cp-ast-core` for decoupling.

**Architecture:** Two-layer design. Layer 1: `TreeVisitor` trait + `DefaultTreeVisitor` in `cp-ast-core` handles all NodeKind-specific logic (label generation, child extraction). Layer 2: `cp-ast-tree` crate consumes `NodeInfo` structs from the visitor and renders ASCII trees with `├──`/`└──`/`│` connectors. The tree crate never matches on `NodeKind` or `Constraint` directly.

**Tech Stack:** Rust 2021 edition, zero external dependencies beyond `cp-ast-core`.

---

## File Structure

```
crates/cp-ast-core/src/structure/tree_visitor.rs   (new)   — NodeInfo, TreeVisitor trait, DefaultTreeVisitor
crates/cp-ast-core/src/structure/mod.rs            (edit)  — add pub mod tree_visitor + re-exports
crates/cp-ast-core/src/render/constraint_text.rs   (edit)  — make render_constraint pub
crates/cp-ast-core/src/render/mod.rs               (edit)  — re-export render_single_constraint
crates/cp-ast-tree/Cargo.toml                      (new)   — crate manifest
crates/cp-ast-tree/src/lib.rs                      (new)   — public API + TreeOptions + re-exports
crates/cp-ast-tree/src/drawing.rs                  (new)   — ASCII tree drawing primitives
crates/cp-ast-tree/src/structure_tree.rs           (new)   — render_structure_tree
crates/cp-ast-tree/src/constraint_tree.rs          (new)   — render_constraint_tree
crates/cp-ast-tree/src/combined_tree.rs            (new)   — render_combined_tree
crates/cp-ast-tree/tests/tree_basic.rs             (new)   — integration tests
Cargo.toml                                         (edit)  — workspace already has members = ["crates/*"]
```

---

## Task 1: TreeVisitor trait + DefaultTreeVisitor in cp-ast-core

**Files:**
- Create: `crates/cp-ast-core/src/structure/tree_visitor.rs`
- Modify: `crates/cp-ast-core/src/structure/mod.rs`
- Test: `crates/cp-ast-core/tests/tree_visitor_basic.rs`

This task adds the `TreeVisitor` trait and `DefaultTreeVisitor` to `cp-ast-core`. The visitor extracts `NodeInfo` (label + children) from each `NodeKind` variant. All variant-specific logic lives here, keeping the external crate decoupled.

- [ ] **Step 1: Create `tree_visitor.rs` with `NodeInfo`, `TreeVisitor` trait, and `DefaultTreeVisitor`**

Create `crates/cp-ast-core/src/structure/tree_visitor.rs`:

```rust
//! Visitor pattern for walking the structure AST.
//!
//! Provides `NodeInfo` (label + children) for each node, and `DefaultTreeVisitor`
//! which handles all `NodeKind` variants. External crates use the visitor
//! to avoid matching on `NodeKind` directly.

use crate::operation::AstEngine;
use crate::render::{render_expression, render_reference};
use crate::structure::{Ident, Literal, NodeId, NodeKind, NodeKindHint, Reference};

/// Display information for a single AST node.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// The node's unique ID.
    pub node_id: NodeId,
    /// Human-readable label (e.g. "Scalar(N)", "Repeat(count=M)").
    pub label: String,
    /// Ordered child node IDs for tree traversal.
    pub children: Vec<ChildEntry>,
}

/// A child entry in the tree — either a real node or a virtual grouping.
#[derive(Debug, Clone)]
pub enum ChildEntry {
    /// A real node in the StructureAst arena.
    Node(NodeId),
    /// A virtual grouping node (e.g. Choice variant) with its own label and children.
    Virtual { label: String, children: Vec<NodeId> },
}

/// Trait for extracting display information from the structure AST.
///
/// Implement this to customize how nodes are labeled and traversed.
/// `DefaultTreeVisitor` handles all built-in `NodeKind` variants.
pub trait TreeVisitor {
    /// Extract display information for a node. Returns `None` if the node doesn't exist.
    fn node_info(&self, engine: &AstEngine, node_id: NodeId) -> Option<NodeInfo>;
}

/// Default visitor that handles all `NodeKind` variants.
///
/// When new variants are added to `NodeKind`, only this implementation
/// needs updating — external crates using `TreeVisitor` are unaffected.
#[derive(Debug, Clone, Copy)]
pub struct DefaultTreeVisitor;

impl TreeVisitor for DefaultTreeVisitor {
    fn node_info(&self, engine: &AstEngine, node_id: NodeId) -> Option<NodeInfo> {
        let node = engine.structure.get(node_id)?;
        let (label, children) = match node.kind() {
            NodeKind::Scalar { name } => (
                format!("Scalar({})", name.as_str()),
                Vec::new(),
            ),
            NodeKind::Array { name, length } => (
                format!(
                    "Array({}, len={})",
                    name.as_str(),
                    render_expression(engine, length)
                ),
                Vec::new(),
            ),
            NodeKind::Matrix { name, rows, cols } => (
                format!(
                    "Matrix({}, rows={}, cols={})",
                    name.as_str(),
                    render_reference(engine, rows),
                    render_reference(engine, cols)
                ),
                Vec::new(),
            ),
            NodeKind::Tuple { elements } => (
                "Tuple".to_owned(),
                elements.iter().map(|id| ChildEntry::Node(*id)).collect(),
            ),
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                let label = match index_var {
                    Some(var) => format!(
                        "Repeat(count={}, {})",
                        render_expression(engine, count),
                        var.as_str()
                    ),
                    None => format!("Repeat(count={})", render_expression(engine, count)),
                };
                (label, body.iter().map(|id| ChildEntry::Node(*id)).collect())
            }
            NodeKind::Section { header, body } => {
                let mut children: Vec<ChildEntry> = Vec::new();
                if let Some(h) = header {
                    children.push(ChildEntry::Node(*h));
                }
                children.extend(body.iter().map(|id| ChildEntry::Node(*id)));
                ("Section".to_owned(), children)
            }
            NodeKind::Sequence { children: kids } => (
                "Sequence".to_owned(),
                kids.iter().map(|id| ChildEntry::Node(*id)).collect(),
            ),
            NodeKind::Choice { tag, variants } => {
                let label = format!("Choice(tag={})", render_reference(engine, tag));
                let children = variants
                    .iter()
                    .map(|(lit, kids)| {
                        let variant_label = match lit {
                            Literal::IntLit(v) => format!("Variant({v})"),
                            Literal::StrLit(s) => format!("Variant(\"{s}\")"),
                        };
                        ChildEntry::Virtual {
                            label: variant_label,
                            children: kids.clone(),
                        }
                    })
                    .collect();
                (label, children)
            }
            NodeKind::Hole { expected_kind } => {
                let label = match expected_kind {
                    Some(hint) => format!("Hole(expected={hint:?})"),
                    None => "Hole".to_owned(),
                };
                (label, Vec::new())
            }
        };
        Some(NodeInfo {
            node_id,
            label,
            children,
        })
    }
}

/// Get a short display name for a node (just the name, no variant prefix).
///
/// Returns the variable name for named nodes (Scalar/Array/Matrix),
/// or the kind name for structural nodes (Tuple/Repeat/etc.).
/// External crates use this instead of matching on `NodeKind` directly.
#[must_use]
pub fn node_display_name(engine: &AstEngine, node_id: NodeId) -> String {
    engine
        .structure
        .get(node_id)
        .map(|node| match node.kind() {
            NodeKind::Scalar { name }
            | NodeKind::Array { name, .. }
            | NodeKind::Matrix { name, .. } => name.as_str().to_owned(),
            NodeKind::Tuple { .. } => "Tuple".to_owned(),
            NodeKind::Repeat { .. } => "Repeat".to_owned(),
            NodeKind::Section { .. } => "Section".to_owned(),
            NodeKind::Sequence { .. } => "Sequence".to_owned(),
            NodeKind::Choice { .. } => "Choice".to_owned(),
            NodeKind::Hole { .. } => "Hole".to_owned(),
        })
        .unwrap_or_else(|| format!("#{}", node_id.value()))
}
```

- [ ] **Step 2: Register module in `structure/mod.rs`**

Add to `crates/cp-ast-core/src/structure/mod.rs`:

```rust
pub mod tree_visitor;
```

And add re-exports:

```rust
pub use tree_visitor::{ChildEntry, DefaultTreeVisitor, NodeInfo, TreeVisitor, node_display_name};
```

- [ ] **Step 3: Widen visibility of `render_reference` and `render_expression`**

In `crates/cp-ast-core/src/render/mod.rs`, change both functions from `pub(crate)` to `pub` so they can be used from `tree_visitor.rs` (same crate) and also by the `cp-ast-tree` crate for constraint rendering:

```rust
// Change:
pub(crate) fn render_reference(engine: &AstEngine, reference: &Reference) -> String {
// To:
pub fn render_reference(engine: &AstEngine, reference: &Reference) -> String {

// Change:
pub(crate) fn render_expression(engine: &AstEngine, expr: &Expression) -> String {
// To:
pub fn render_expression(engine: &AstEngine, expr: &Expression) -> String {
```

- [ ] **Step 4: Write tests for DefaultTreeVisitor**

Create `crates/cp-ast-core/tests/tree_visitor_basic.rs`:

```rust
use cp_ast_core::constraint::Expression;
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{
    ChildEntry, DefaultTreeVisitor, Ident, Literal, NodeKind, Reference, TreeVisitor,
};

fn setup_graph_engine() -> AstEngine {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let tuple_header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    let tuple_edge = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(m_id)),
        index_var: Some(Ident::new("i")),
        body: vec![tuple_edge],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tuple_header, repeat],
        });
    }
    engine
}

#[test]
fn visitor_scalar_label() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, n_id).unwrap();
    assert_eq!(info.label, "Scalar(N)");
    assert!(info.children.is_empty());
}

#[test]
fn visitor_array_label() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, a_id).unwrap();
    assert_eq!(info.label, "Array(A, len=N)");
    assert!(info.children.is_empty());
}

#[test]
fn visitor_tuple_children() {
    let engine = setup_graph_engine();
    let visitor = DefaultTreeVisitor;
    let root_info = visitor.node_info(&engine, engine.structure.root()).unwrap();
    assert_eq!(root_info.label, "Sequence");
    assert_eq!(root_info.children.len(), 2);
}

#[test]
fn visitor_repeat_label_with_index() {
    let engine = setup_graph_engine();
    let visitor = DefaultTreeVisitor;
    // The repeat node is the second child of root Sequence
    let root_info = visitor.node_info(&engine, engine.structure.root()).unwrap();
    if let ChildEntry::Node(repeat_id) = &root_info.children[1] {
        let repeat_info = visitor.node_info(&engine, *repeat_id).unwrap();
        assert_eq!(repeat_info.label, "Repeat(count=M, i)");
        assert_eq!(repeat_info.children.len(), 1);
    } else {
        panic!("expected Node entry");
    }
}

#[test]
fn visitor_choice_virtual_children() {
    let mut engine = AstEngine::new();
    let tag_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let choice = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(tag_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id]),
        ],
    });
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, choice).unwrap();
    assert_eq!(info.label, "Choice(tag=T)");
    assert_eq!(info.children.len(), 2);
    match &info.children[0] {
        ChildEntry::Virtual { label, children } => {
            assert_eq!(label, "Variant(1)");
            assert_eq!(children.len(), 1);
        }
        _ => panic!("expected Virtual entry"),
    }
}

#[test]
fn visitor_hole_label() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(cp_ast_core::structure::NodeKindHint::AnyArray),
    });
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, hole_id).unwrap();
    assert_eq!(info.label, "Hole(expected=AnyArray)");
}

#[test]
fn visitor_nonexistent_node_returns_none() {
    let engine = AstEngine::new();
    let visitor = DefaultTreeVisitor;
    let fake_id = cp_ast_core::structure::NodeId::from_raw(999);
    assert!(visitor.node_info(&engine, fake_id).is_none());
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p cp-ast-core --test tree_visitor_basic`
Expected: All 7 tests pass.

- [ ] **Step 6: Run full validation**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All pass, no warnings.

- [ ] **Step 7: Commit**

```bash
git add crates/cp-ast-core/src/structure/tree_visitor.rs \
       crates/cp-ast-core/src/structure/mod.rs \
       crates/cp-ast-core/src/render/mod.rs \
       crates/cp-ast-core/tests/tree_visitor_basic.rs
git commit -m "feat: add TreeVisitor trait and DefaultTreeVisitor to cp-ast-core

- NodeInfo struct with label + ChildEntry (Node | Virtual)
- DefaultTreeVisitor handles all 9 NodeKind variants
- Widen render_reference/render_expression to pub

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 2: Expose per-constraint rendering + add `nodes_with_constraints()`

**Files:**
- Modify: `crates/cp-ast-core/src/render/constraint_text.rs`
- Modify: `crates/cp-ast-core/src/render/mod.rs`
- Modify: `crates/cp-ast-core/src/constraint/constraint_set.rs`

The tree crate needs to render individual constraints and iterate over (NodeId, constraints) pairs. Currently `render_constraint` is private and `ConstraintSet` has no way to iterate `by_node`. This task exposes both.

- [ ] **Step 1: Make `render_constraint` public in `constraint_text.rs`**

In `crates/cp-ast-core/src/render/constraint_text.rs`, rename and make public:

```rust
// Change line 46:
fn render_constraint(engine: &AstEngine, constraint: &Constraint) -> String {
// To:
/// Render a single constraint to a human-readable string.
#[must_use]
pub fn render_single_constraint(engine: &AstEngine, constraint: &Constraint) -> String {
```

Update the call site in `render_constraints` (line 15):

```rust
// Change:
        let rendered = render_constraint(engine, constraint);
// To:
        let rendered = render_single_constraint(engine, constraint);
```

- [ ] **Step 2: Re-export from `render/mod.rs`**

In `crates/cp-ast-core/src/render/mod.rs`, update the re-exports:

```rust
// Change:
pub use constraint_text::render_constraints;
// To:
pub use constraint_text::{render_constraints, render_single_constraint};
```

- [ ] **Step 3: Add `nodes_with_constraints()` to `ConstraintSet`**

In `crates/cp-ast-core/src/constraint/constraint_set.rs`, add this method after `global()`:

```rust
    /// Iterate over (NodeId, &[ConstraintId]) pairs for all nodes with constraints.
    pub fn nodes_with_constraints(&self) -> impl Iterator<Item = (NodeId, &[ConstraintId])> {
        self.by_node
            .iter()
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(node_id, ids)| (*node_id, ids.as_slice()))
    }
```

- [ ] **Step 4: Run full validation**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All pass. The existing `render_constraints` tests still pass since the function signature didn't change.

- [ ] **Step 5: Commit**

```bash
git add crates/cp-ast-core/src/render/constraint_text.rs \
       crates/cp-ast-core/src/render/mod.rs \
       crates/cp-ast-core/src/constraint/constraint_set.rs
git commit -m "feat: expose render_single_constraint and nodes_with_constraints

- Make per-constraint render function public
- Add ConstraintSet::nodes_with_constraints() iterator
- Needed by cp-ast-tree crate

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 3: Create cp-ast-tree crate with drawing primitives + structure tree

**Files:**
- Create: `crates/cp-ast-tree/Cargo.toml`
- Create: `crates/cp-ast-tree/src/lib.rs`
- Create: `crates/cp-ast-tree/src/drawing.rs`
- Create: `crates/cp-ast-tree/src/structure_tree.rs`
- Test: `crates/cp-ast-tree/tests/tree_basic.rs`

This task creates the new crate, implements the ASCII tree drawing primitives, and implements `render_structure_tree`.

- [ ] **Step 1: Create `crates/cp-ast-tree/Cargo.toml`**

```toml
[package]
name = "cp-ast-tree"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "ASCII tree renderer for cp-ast-core AST inspection"

[dependencies]
cp-ast-core = { path = "../cp-ast-core" }

[lints]
workspace = true
```

- [ ] **Step 2: Create `crates/cp-ast-tree/src/drawing.rs`**

```rust
//! ASCII tree drawing primitives.
//!
//! Provides `draw_tree` which renders a label+children tree structure
//! using `├──`/`└──`/`│` box-drawing connectors.

use std::fmt::Write;

use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{ChildEntry, NodeId, TreeVisitor};

/// Draw an ASCII tree starting from `node_id`, using the given visitor.
///
/// `label_fn` is called for each node to produce the final display label
/// (allowing the caller to append constraint annotations, NodeIds, etc.).
pub(crate) fn draw_tree(
    engine: &AstEngine,
    visitor: &impl TreeVisitor,
    node_id: NodeId,
    label_fn: &impl Fn(NodeId, &str) -> String,
    output: &mut String,
) {
    if let Some(info) = visitor.node_info(engine, node_id) {
        output.push_str(&label_fn(info.node_id, &info.label));
        output.push('\n');
        draw_children(engine, visitor, &info.children, "", label_fn, output);
    }
}

fn draw_children(
    engine: &AstEngine,
    visitor: &impl TreeVisitor,
    children: &[ChildEntry],
    prefix: &str,
    label_fn: &impl Fn(NodeId, &str) -> String,
    output: &mut String,
) {
    for (i, child) in children.iter().enumerate() {
        let is_last = i + 1 == children.len();
        let connector = if is_last { "└── " } else { "├── " };
        let continuation = if is_last { "    " } else { "│   " };

        match child {
            ChildEntry::Node(child_id) => {
                if let Some(child_info) = visitor.node_info(engine, *child_id) {
                    let _ = write!(
                        output,
                        "{prefix}{connector}{}",
                        label_fn(child_info.node_id, &child_info.label)
                    );
                    output.push('\n');
                    let new_prefix = format!("{prefix}{continuation}");
                    draw_children(
                        engine,
                        visitor,
                        &child_info.children,
                        &new_prefix,
                        label_fn,
                        output,
                    );
                }
            }
            ChildEntry::Virtual { label, children: kids } => {
                let _ = write!(output, "{prefix}{connector}{label}");
                output.push('\n');
                let new_prefix = format!("{prefix}{continuation}");
                let virtual_children: Vec<ChildEntry> =
                    kids.iter().map(|id| ChildEntry::Node(*id)).collect();
                draw_children(
                    engine,
                    visitor,
                    &virtual_children,
                    &new_prefix,
                    label_fn,
                    output,
                );
            }
        }
    }
}
```

- [ ] **Step 3: Create `crates/cp-ast-tree/src/structure_tree.rs`**

```rust
//! Structure AST tree rendering.

use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::DefaultTreeVisitor;

use crate::TreeOptions;
use crate::drawing::draw_tree;

/// Render the structure AST as an ASCII tree.
#[must_use]
pub fn render_structure_tree(engine: &AstEngine, options: &TreeOptions) -> String {
    let visitor = DefaultTreeVisitor;
    let root = engine.structure.root();
    let mut output = String::new();

    let label_fn = |node_id, label: &str| {
        if options.show_node_ids {
            format!("#{} {}", node_id.value(), label)
        } else {
            label.to_owned()
        }
    };

    draw_tree(engine, &visitor, root, &label_fn, &mut output);
    output
}
```

- [ ] **Step 4: Create `crates/cp-ast-tree/src/lib.rs`**

```rust
//! ASCII tree renderer for inspecting `cp-ast-core` ASTs.
//!
//! Provides three rendering modes:
//! - `render_structure_tree` — structure AST only
//! - `render_constraint_tree` — constraints grouped by target node
//! - `render_combined_tree` — structure tree with inline constraint annotations

mod drawing;
pub mod structure_tree;

pub use structure_tree::render_structure_tree;

/// Options controlling tree rendering output.
#[derive(Debug, Clone)]
pub struct TreeOptions {
    /// Show NodeId next to each label (e.g. "#3 Scalar(N)").
    pub show_node_ids: bool,
    /// Show ConstraintId next to each constraint line.
    pub show_constraint_ids: bool,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            show_node_ids: false,
            show_constraint_ids: false,
        }
    }
}
```

- [ ] **Step 5: Write integration tests for structure tree**

Create `crates/cp-ast-tree/tests/tree_basic.rs`:

```rust
use cp_ast_core::constraint::Expression;
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, Literal, NodeKind, Reference};
use cp_ast_tree::{render_structure_tree, TreeOptions};

fn setup_graph_engine() -> AstEngine {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let tuple_header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    let tuple_edge = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(m_id)),
        index_var: Some(Ident::new("i")),
        body: vec![tuple_edge],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tuple_header, repeat],
        });
    }
    engine
}

#[test]
fn structure_tree_graph_problem() {
    let engine = setup_graph_engine();
    let output = render_structure_tree(&engine, &TreeOptions::default());
    let expected = "\
Sequence
├── Tuple
│   ├── Scalar(N)
│   └── Scalar(M)
└── Repeat(count=M, i)
    └── Tuple
        ├── Scalar(u)
        └── Scalar(v)
";
    assert_eq!(output, expected);
}

#[test]
fn structure_tree_with_node_ids() {
    let engine = setup_graph_engine();
    let options = TreeOptions {
        show_node_ids: true,
        ..TreeOptions::default()
    };
    let output = render_structure_tree(&engine, &options);
    // Root is #0, N is #1, M is #2, tuple_header is #3, etc.
    assert!(output.contains("#0 Sequence"));
    assert!(output.contains("#1 Scalar(N)"));
}

#[test]
fn structure_tree_empty_ast() {
    let engine = AstEngine::new();
    let output = render_structure_tree(&engine, &TreeOptions::default());
    // Empty AST has only the root Sequence with no children
    assert_eq!(output, "Sequence\n");
}

#[test]
fn structure_tree_single_scalar() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    let output = render_structure_tree(&engine, &TreeOptions::default());
    assert_eq!(output, "Sequence\n└── Scalar(N)\n");
}

#[test]
fn structure_tree_choice() {
    let mut engine = AstEngine::new();
    let tag_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let choice = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(tag_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id]),
        ],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tag_id, choice],
        });
    }
    let output = render_structure_tree(&engine, &TreeOptions::default());
    let expected = "\
Sequence
├── Scalar(T)
└── Choice(tag=T)
    ├── Variant(1)
    │   └── Scalar(X)
    └── Variant(2)
        └── Scalar(Y)
";
    assert_eq!(output, expected);
}
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p cp-ast-tree`
Expected: All 5 tests pass.

- [ ] **Step 7: Run full validation**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All pass.

- [ ] **Step 8: Commit**

```bash
git add crates/cp-ast-tree/ crates/cp-ast-tree/tests/
git commit -m "feat: create cp-ast-tree crate with structure tree rendering

- ASCII tree drawing primitives (drawing.rs)
- render_structure_tree with TreeOptions (show_node_ids)
- Integration tests for graph, choice, empty, single-scalar ASTs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 4: Constraint tree rendering

**Files:**
- Create: `crates/cp-ast-tree/src/constraint_tree.rs`
- Modify: `crates/cp-ast-tree/src/lib.rs`
- Test: `crates/cp-ast-tree/tests/tree_basic.rs` (append)

This task implements `render_constraint_tree` — constraints grouped by target node in a two-level tree.

- [ ] **Step 1: Create `crates/cp-ast-tree/src/constraint_tree.rs`**

```rust
//! Constraint tree rendering — constraints grouped by target node.

use std::fmt::Write;

use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::render_single_constraint;
use cp_ast_core::structure::node_display_name;

use crate::TreeOptions;

/// Render constraints grouped by target node as an ASCII tree.
///
/// Output format:
/// ```text
/// Constraints
/// ├── N
/// │   ├── 1 ≤ N ≤ 100000
/// │   └── N is integer
/// └── (global)
///     └── Property: Simple graph
/// ```
#[must_use]
pub fn render_constraint_tree(engine: &AstEngine, options: &TreeOptions) -> String {
    let mut output = String::from("Constraints\n");

    let mut groups: Vec<GroupEntry> = Vec::new();

    // Collect per-node constraint groups
    for (node_id, constraint_ids) in engine.constraints.nodes_with_constraints() {
        let node_label = node_display_name(engine, node_id);

        let mut items: Vec<ConstraintItem> = Vec::new();
        for &cid in constraint_ids {
            if let Some(constraint) = engine.constraints.get(cid) {
                let text = render_single_constraint(engine, constraint);
                if text.is_empty() {
                    continue;
                }
                items.push(ConstraintItem {
                    text,
                    constraint_id: if options.show_constraint_ids {
                        Some(cid.value())
                    } else {
                        None
                    },
                });
            }
        }
        if !items.is_empty() {
            groups.push(GroupEntry {
                label: node_label,
                items,
            });
        }
    }

    // Collect global constraints
    let global_ids = engine.constraints.global();
    if !global_ids.is_empty() {
        let mut items: Vec<ConstraintItem> = Vec::new();
        for &cid in global_ids {
            if let Some(constraint) = engine.constraints.get(cid) {
                let text = render_single_constraint(engine, constraint);
                if text.is_empty() {
                    continue;
                }
                items.push(ConstraintItem {
                    text,
                    constraint_id: if options.show_constraint_ids {
                        Some(cid.value())
                    } else {
                        None
                    },
                });
            }
        }
        if !items.is_empty() {
            groups.push(GroupEntry {
                label: "(global)".to_owned(),
                items,
            });
        }
    }

    // Render groups as a tree
    for (gi, group) in groups.iter().enumerate() {
        let is_last_group = gi + 1 == groups.len();
        let group_connector = if is_last_group { "└── " } else { "├── " };
        let group_continuation = if is_last_group { "    " } else { "│   " };

        let _ = writeln!(output, "{group_connector}{}", group.label);

        for (ci, item) in group.items.iter().enumerate() {
            let is_last_item = ci + 1 == group.items.len();
            let item_connector = if is_last_item { "└── " } else { "├── " };

            let label = match item.constraint_id {
                Some(id) => format!("[C{}] {}", id, item.text),
                None => item.text.clone(),
            };
            let _ = writeln!(output, "{group_continuation}{item_connector}{label}");
        }
    }

    output
}

struct GroupEntry {
    label: String,
    items: Vec<ConstraintItem>,
}

struct ConstraintItem {
    text: String,
    constraint_id: Option<u64>,
}

```

- [ ] **Step 2: Register in `lib.rs`**

In `crates/cp-ast-tree/src/lib.rs`, add:

```rust
pub mod constraint_tree;
pub use constraint_tree::render_constraint_tree;
```

- [ ] **Step 3: Write constraint tree tests**

Append to `crates/cp-ast-tree/tests/tree_basic.rs`:

```rust
use cp_ast_core::constraint::{Constraint, ExpectedType};
use cp_ast_tree::render_constraint_tree;

#[test]
fn constraint_tree_basic() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(5)),
            },
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );

    let output = render_constraint_tree(&engine, &TreeOptions::default());
    assert!(output.contains("Constraints"));
    assert!(output.contains("N"));
    assert!(output.contains("1 ≤ N ≤ 10^5"));
    assert!(output.contains("N is integer"));
}

#[test]
fn constraint_tree_with_global() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "The answer exists".to_owned(),
            predicate: None,
        },
    );

    let output = render_constraint_tree(&engine, &TreeOptions::default());
    assert!(output.contains("(global)"));
    assert!(output.contains("The answer exists"));
}

#[test]
fn constraint_tree_empty() {
    let engine = AstEngine::new();
    let output = render_constraint_tree(&engine, &TreeOptions::default());
    assert_eq!(output, "Constraints\n");
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p cp-ast-tree`
Expected: All tests pass (5 structure + 3 constraint = 8 total).

- [ ] **Step 5: Run full validation**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All pass.

- [ ] **Step 6: Commit**

```bash
git add crates/cp-ast-tree/src/constraint_tree.rs \
       crates/cp-ast-tree/src/lib.rs \
       crates/cp-ast-tree/tests/tree_basic.rs
git commit -m "feat: add render_constraint_tree to cp-ast-tree

- Two-level tree: group by target node, then individual constraints
- Global constraints shown as (global) group
- Skips RenderHint constraints
- Optional ConstraintId display

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 5: Combined tree rendering

**Files:**
- Create: `crates/cp-ast-tree/src/combined_tree.rs`
- Modify: `crates/cp-ast-tree/src/lib.rs`
- Test: `crates/cp-ast-tree/tests/tree_basic.rs` (append)

This task implements `render_combined_tree` — the structure tree with constraints annotated inline in `[...]` brackets, plus global constraints at the bottom.

- [ ] **Step 1: Create `crates/cp-ast-tree/src/combined_tree.rs`**

```rust
//! Combined tree rendering — structure tree with inline constraint annotations.

use std::fmt::Write;

use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::render_single_constraint;
use cp_ast_core::structure::{DefaultTreeVisitor, NodeId};

use crate::TreeOptions;
use crate::drawing::draw_tree;

/// Render the structure tree with constraints annotated on each node.
///
/// Each node shows its constraints in `[...]` brackets. Global constraints
/// are listed at the bottom.
#[must_use]
pub fn render_combined_tree(engine: &AstEngine, options: &TreeOptions) -> String {
    let visitor = DefaultTreeVisitor;
    let root = engine.structure.root();
    let mut output = String::new();

    let label_fn = |node_id: NodeId, label: &str| -> String {
        let base = if options.show_node_ids {
            format!("#{} {}", node_id.value(), label)
        } else {
            label.to_owned()
        };

        let annotation = build_constraint_annotation(engine, node_id, options);
        if annotation.is_empty() {
            base
        } else {
            format!("{base}  [{annotation}]")
        }
    };

    draw_tree(engine, &visitor, root, &label_fn, &mut output);

    // Append global constraints
    let global_ids = engine.constraints.global();
    let global_lines: Vec<String> = global_ids
        .iter()
        .filter_map(|&cid| engine.constraints.get(cid))
        .map(|c| render_single_constraint(engine, c))
        .filter(|s| !s.is_empty())
        .collect();

    if !global_lines.is_empty() {
        let _ = writeln!(output, "(global) {}", global_lines.join(", "));
    }

    output
}

fn build_constraint_annotation(
    engine: &AstEngine,
    node_id: NodeId,
    options: &TreeOptions,
) -> String {
    let constraint_ids = engine.constraints.for_node(node_id);
    let parts: Vec<String> = constraint_ids
        .iter()
        .filter_map(|&cid| {
            let constraint = engine.constraints.get(cid)?;
            let text = render_single_constraint(engine, constraint);
            if text.is_empty() {
                return None;
            }
            if options.show_constraint_ids {
                Some(format!("C{}:{text}", cid.value()))
            } else {
                Some(text)
            }
        })
        .collect();
    parts.join(", ")
}
```

- [ ] **Step 2: Register in `lib.rs`**

In `crates/cp-ast-tree/src/lib.rs`, add:

```rust
pub mod combined_tree;
pub use combined_tree::render_combined_tree;
```

- [ ] **Step 3: Write combined tree tests**

Append to `crates/cp-ast-tree/tests/tree_basic.rs`:

```rust
use cp_ast_tree::render_combined_tree;

#[test]
fn combined_tree_basic() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );

    let output = render_combined_tree(&engine, &TreeOptions::default());
    assert!(output.contains("Scalar(N)  [1 ≤ N ≤ 100, N is integer]"));
    assert!(output.contains("Array(A, len=N)"));
}

#[test]
fn combined_tree_with_global_constraints() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "The answer is unique".to_owned(),
            predicate: None,
        },
    );

    let output = render_combined_tree(&engine, &TreeOptions::default());
    assert!(output.contains("(global) The answer is unique"));
}

#[test]
fn combined_tree_no_constraints() {
    let engine = setup_graph_engine();
    let output = render_combined_tree(&engine, &TreeOptions::default());
    // Same as structure tree when no constraints exist
    let structure_output = render_structure_tree(&engine, &TreeOptions::default());
    assert_eq!(output, structure_output);
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p cp-ast-tree`
Expected: All 11 tests pass (5 structure + 3 constraint + 3 combined).

- [ ] **Step 5: Run full validation**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All pass.

- [ ] **Step 6: Commit**

```bash
git add crates/cp-ast-tree/src/combined_tree.rs \
       crates/cp-ast-tree/src/lib.rs \
       crates/cp-ast-tree/tests/tree_basic.rs
git commit -m "feat: add render_combined_tree to cp-ast-tree

- Structure tree with inline [constraint] annotations
- Global constraints appended at bottom
- Optional NodeId and ConstraintId display

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 6: Documentation update + final push

**Files:**
- Modify: `doc/plan/processing.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: Update `doc/plan/processing.md`**

Add a new section for the tree viewer phase, marking it complete.

- [ ] **Step 2: Update `AGENTS.md`**

Add `cp-ast-tree` to the project structure section, noting it depends on `cp-ast-core` and provides ASCII tree rendering for AST inspection.

- [ ] **Step 3: Run full validation one final time**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`
Expected: All pass.

- [ ] **Step 4: Commit and push**

```bash
git add doc/plan/processing.md AGENTS.md
git commit -m "docs: add cp-ast-tree to project docs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
git push
```
