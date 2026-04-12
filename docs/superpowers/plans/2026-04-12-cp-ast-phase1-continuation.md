# Phase 1 Continuation: Rev.1 Domain Model + Operation + Projection + Rendering + Sample Generation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Evolve cp-ast-core from Sprint 5's minimal types to the Rev.1 domain model, then build the full Operation layer, ProjectionAPI, canonical renderer, and sample case generator.

**Architecture:** Arena-based StructureAst + ConstraintSet with ConstraintId addressing. Operation trait mutates the AST via validated Actions. ProjectionAPI provides read-only views. Canonical renderer produces deterministic text. Sample generator uses dependency-graph-ordered constraint-aware generation.

**Tech Stack:** Rust 2021, no external dependencies. TDD. `cargo clippy --all-targets --all-features -- -D warnings`.

**Design References:**
- `doc/design/domain-model.md` §6 — Authoritative Rev.1 type definitions
- `doc/design/final-design.md` §5-10, §13 — Full system design
- `doc/design/projection-operation.md` — ProjectionAPI + Operation detailed spec

---

## File Structure

After all tasks complete, the crate will have this structure:

```
crates/cp-ast-core/src/
  lib.rs                          # pub mod structure, constraint, operation, projection, render, sample
  structure/
    mod.rs                        # re-exports
    node_id.rs                    # NodeId (MODIFIED: remove AtomicU64, add from_raw)
    types.rs                      # NEW: Ident, Literal, NodeKindHint
    reference.rs                  # NEW: Reference enum
    node_kind.rs                  # REWRITE: 9 rich variants with data
    structure_node.rs             # REWRITE: simplified {id, kind}
    structure_ast.rs              # NEW: Arena-based StructureAst
    # REMOVED: slot.rs
  constraint/
    mod.rs                        # re-exports
    constraint_id.rs              # NEW: ConstraintId
    types.rs                      # NEW: RelationOp, ArithOp, DistinctUnit, PropertyTag, SortOrder,
                                  #       CharSetSpec, RenderHintKind, Separator
    expected_type.rs              # REWRITE: simplified to Int/Str/Char
    expression.rs                 # REWRITE: 5 variants (Lit, Var, BinOp, Pow, FnCall)
    constraint.rs                 # REWRITE: 12 variants
    constraint_set.rs             # REWRITE: arena-based with ConstraintId
  operation/
    mod.rs                        # NEW: re-exports
    types.rs                      # NEW: FillContent, VarType, LengthSpec, ConstraintDef, etc.
    action.rs                     # NEW: Action enum (7 variants)
    error.rs                      # NEW: OperationError, ViolationDetail
    result.rs                     # NEW: ApplyResult, PreviewResult
    engine.rs                     # NEW: AstEngine struct implementing Operation
    fill_hole.rs                  # NEW: FillHole logic
    constraint_ops.rs             # NEW: AddConstraint/RemoveConstraint logic
    node_ops.rs                   # NEW: ReplaceNode, AddSlotElement, RemoveSlotElement
    multi_test_case.rs            # NEW: IntroduceMultiTestCase logic
  projection/
    mod.rs                        # NEW: re-exports
    types.rs                      # NEW: ProjectedNode, SlotEntry, NodeDetail, CandidateKind, etc.
    api.rs                        # NEW: ProjectionAPI trait
    projection_impl.rs            # NEW: implementation
  render/
    mod.rs                        # NEW
    input_format.rs               # NEW: Input format renderer
    constraint_text.rs            # NEW: Constraint text renderer
  sample/
    mod.rs                        # NEW
    dependency.rs                 # NEW: Dependency graph + topo sort
    generator.rs                  # NEW: Per-constraint generators
    output.rs                     # NEW: GeneratedSample → text

crates/cp-ast-core/tests/
  structure_primitives.rs         # NEW: tests for Ident, Literal, Reference, NodeKindHint
  structure_ast.rs                # REWRITE (was structure_basic.rs): arena + NodeKind tests
  constraint_types.rs             # NEW: tests for supporting enums
  constraint_ast.rs               # REWRITE (was constraint_basic.rs): Expression + Constraint + Set
  typical_problem.rs              # REWRITE: ABC-style problem with Rev.1 types
  operation_basic.rs              # NEW: FillHole, AddConstraint, etc.
  projection_basic.rs             # NEW: ProjectionAPI tests
  render_basic.rs                 # NEW: Canonical rendering tests
  sample_basic.rs                 # NEW: Sample generation tests
  e2e_abc284c.rs                  # NEW: End-to-end ABC284-C
  # REMOVED: structure_basic.rs, hole_basic.rs, constraint_basic.rs
```

## Task Dependencies

```
Sprint A (parallel):  T-01 ─┐
                      T-02 ─┤
                            ▼
Sprint B:             T-03 (depends: T-01)
                            │
Sprint C:             T-04 (depends: T-01, T-02, T-03)
                            │
                      T-05 (depends: T-04)
                            │
Sprint D:             T-06 (depends: T-03, T-05)
                            │
Sprint E:             T-07 (depends: T-06)
                            │
                      T-08 ─┤ (parallel, depend: T-07)
                      T-09 ─┤
                            ▼
Sprint F:             T-10 (depends: T-08, T-09)
                            │
Sprint G:             T-11 ─┤ (parallel, depend: T-10)
                      T-12 ─┤
                            ▼
Sprint H:             T-13 (depends: T-10, T-11, T-12)
```

---

## Sprint A: Foundation Primitives (non-breaking additions)

### Task 1: add-structure-primitives

**Files:**
- Create: `crates/cp-ast-core/src/structure/types.rs`
- Create: `crates/cp-ast-core/src/structure/reference.rs`
- Modify: `crates/cp-ast-core/src/structure/node_id.rs`
- Modify: `crates/cp-ast-core/src/structure/mod.rs`
- Create: `crates/cp-ast-core/tests/structure_primitives.rs`

- [ ] **Step 1: Write failing tests**

```rust
// tests/structure_primitives.rs
use cp_ast_core::structure::{Ident, Literal, NodeId, NodeKindHint, Reference};

#[test]
fn ident_creation_and_equality() {
    let a = Ident::new("N");
    let b = Ident::new("N");
    assert_eq!(a, b);
    assert_eq!(a.as_str(), "N");
}

#[test]
fn ident_from_str() {
    let id: Ident = "M".into();
    assert_eq!(id.as_str(), "M");
}

#[test]
fn ident_hash_usable_in_collections() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Ident::new("N"));
    set.insert(Ident::new("N"));
    assert_eq!(set.len(), 1);
}

#[test]
fn literal_int() {
    let lit = Literal::IntLit(42);
    assert_eq!(lit, Literal::IntLit(42));
}

#[test]
fn literal_str() {
    let lit = Literal::StrLit("abc".to_owned());
    assert_eq!(lit, Literal::StrLit("abc".to_owned()));
}

#[test]
fn node_id_from_raw() {
    let id = NodeId::from_raw(42);
    assert_eq!(id.value(), 42);
}

#[test]
fn node_id_new_still_works() {
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    assert_ne!(id1, id2);
}

#[test]
fn reference_variable_ref() {
    let id = NodeId::from_raw(1);
    let r = Reference::VariableRef(id);
    assert!(matches!(r, Reference::VariableRef(_)));
}

#[test]
fn reference_indexed_ref() {
    let r = Reference::IndexedRef {
        target: NodeId::from_raw(1),
        indices: vec![Ident::new("i")],
    };
    assert!(matches!(r, Reference::IndexedRef { .. }));
}

#[test]
fn reference_unresolved() {
    let r = Reference::Unresolved(Ident::new("N"));
    assert!(matches!(r, Reference::Unresolved(_)));
}

#[test]
fn node_kind_hint_all_variants() {
    let hints = [
        NodeKindHint::AnyScalar,
        NodeKindHint::AnyArray,
        NodeKindHint::AnyMatrix,
        NodeKindHint::AnyTuple,
        NodeKindHint::AnyRepeat,
        NodeKindHint::AnySection,
        NodeKindHint::AnyChoice,
        NodeKindHint::Any,
    ];
    assert_eq!(hints.len(), 8);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test structure_primitives 2>&1 | head -20`
Expected: compilation error (modules don't exist yet)

- [ ] **Step 3: Implement `types.rs`**

```rust
// src/structure/types.rs

/// Variable name or index name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

impl Ident {
    /// Create a new identifier.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Ident {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Literal value in AST expressions and Choice variant tags.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal.
    IntLit(i64),
    /// String literal.
    StrLit(String),
}

/// Hint for what kind of node is expected at a Hole position.
/// Used by UI to suggest candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKindHint {
    AnyScalar,
    AnyArray,
    AnyMatrix,
    AnyTuple,
    AnyRepeat,
    AnySection,
    AnyChoice,
    Any,
}
```

- [ ] **Step 4: Implement `reference.rs`**

```rust
// src/structure/reference.rs
use super::node_id::NodeId;
use super::types::Ident;

/// Reference to a variable or indexed element.
#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
    /// Direct reference to a variable node.
    VariableRef(NodeId),
    /// Indexed reference: A[i], C[i][j].
    IndexedRef {
        target: NodeId,
        indices: Vec<Ident>,
    },
    /// Unresolved reference (name only, used during construction).
    Unresolved(Ident),
}
```

- [ ] **Step 5: Add `from_raw` to `node_id.rs`**

Add to existing `NodeId` impl block:

```rust
    /// Create a `NodeId` from a raw value.
    /// Used by arenas that manage their own ID allocation.
    #[must_use]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
```

- [ ] **Step 6: Update `structure/mod.rs`**

Add new modules and re-exports (keep existing exports for backward compatibility):

```rust
pub mod node_id;
pub mod node_kind;
pub mod reference;
pub mod slot;
pub mod structure_node;
pub mod types;

pub use node_id::NodeId;
pub use node_kind::NodeKind;
pub use reference::Reference;
pub use slot::{HoleInfo, Slot, SlotValue};
pub use structure_node::StructureNode;
pub use types::{Ident, Literal, NodeKindHint};
```

- [ ] **Step 7: Run all tests**

Run: `cargo test --all-targets 2>&1 | tail -5`
Expected: all tests pass (old tests still work, new tests pass)

- [ ] **Step 8: Run clippy + fmt**

Run: `cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --all -- --check`

- [ ] **Step 9: Commit**

```bash
git add -A && git commit -m "feat(structure): add Rev.1 primitive types (Ident, Literal, Reference, NodeKindHint)

Add foundation types needed for Rev.1 domain model alignment:
- Ident: variable/index name wrapper with Hash+Eq
- Literal: IntLit/StrLit for Choice variant tags
- Reference: VariableRef/IndexedRef/Unresolved for variable references
- NodeKindHint: 8-variant hint for Hole UI candidates
- NodeId::from_raw() for arena-managed ID allocation

Non-breaking: all existing tests continue to pass.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 2: add-constraint-primitives

**Files:**
- Create: `crates/cp-ast-core/src/constraint/constraint_id.rs`
- Create: `crates/cp-ast-core/src/constraint/types.rs`
- Modify: `crates/cp-ast-core/src/constraint/mod.rs`
- Create: `crates/cp-ast-core/tests/constraint_types.rs`

- [ ] **Step 1: Write failing tests**

```rust
// tests/constraint_types.rs
use cp_ast_core::constraint::{
    ArithOp, CharSetSpec, ConstraintId, DistinctUnit, PropertyTag,
    RelationOp, RenderHintKind, Separator, SortOrder,
};

#[test]
fn constraint_id_from_raw_and_value() {
    let id = ConstraintId::from_raw(5);
    assert_eq!(id.value(), 5);
}

#[test]
fn constraint_id_copy_and_eq() {
    let a = ConstraintId::from_raw(1);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn constraint_id_hash_usable() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(ConstraintId::from_raw(1));
    set.insert(ConstraintId::from_raw(1));
    assert_eq!(set.len(), 1);
}

#[test]
fn relation_op_all_variants() {
    let ops = [
        RelationOp::Lt, RelationOp::Le, RelationOp::Gt,
        RelationOp::Ge, RelationOp::Eq, RelationOp::Ne,
    ];
    assert_eq!(ops.len(), 6);
}

#[test]
fn relation_op_copy() {
    let op = RelationOp::Lt;
    let copied = op;
    assert_eq!(op, copied);
}

#[test]
fn arith_op_all_variants() {
    let ops = [ArithOp::Add, ArithOp::Sub, ArithOp::Mul, ArithOp::Div];
    assert_eq!(ops.len(), 4);
}

#[test]
fn distinct_unit_variants() {
    assert_ne!(DistinctUnit::Element, DistinctUnit::Tuple);
}

#[test]
fn property_tag_predefined_and_custom() {
    let tags = [
        PropertyTag::Simple, PropertyTag::Connected, PropertyTag::Tree,
        PropertyTag::Permutation, PropertyTag::Binary,
        PropertyTag::Odd, PropertyTag::Even,
        PropertyTag::Custom("Bipartite".to_owned()),
    ];
    assert_eq!(tags.len(), 8);
}

#[test]
fn sort_order_variants() {
    let orders = [
        SortOrder::Ascending, SortOrder::NonDecreasing,
        SortOrder::Descending, SortOrder::NonIncreasing,
    ];
    assert_eq!(orders.len(), 4);
}

#[test]
fn charset_spec_predefined() {
    assert_ne!(CharSetSpec::LowerAlpha, CharSetSpec::UpperAlpha);
}

#[test]
fn charset_spec_custom() {
    let cs = CharSetSpec::Custom(vec!['a', 'b', 'c']);
    assert!(matches!(cs, CharSetSpec::Custom(_)));
}

#[test]
fn charset_spec_range() {
    let cs = CharSetSpec::Range('a', 'z');
    assert!(matches!(cs, CharSetSpec::Range(_, _)));
}

#[test]
fn render_hint_kind_separator() {
    let hint = RenderHintKind::Separator(Separator::Space);
    assert!(matches!(hint, RenderHintKind::Separator(Separator::Space)));
}

#[test]
fn separator_variants() {
    assert_ne!(Separator::Space, Separator::None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement `constraint_id.rs`**

```rust
// src/constraint/constraint_id.rs

/// Unique identifier for a constraint in the ConstraintSet.
///
/// Used by RemoveConstraint and ViolationDetail for precise constraint addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintId(u64);

impl ConstraintId {
    /// Create a `ConstraintId` from a raw value.
    #[must_use]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw numeric value.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}
```

- [ ] **Step 4: Implement `types.rs`**

```rust
// src/constraint/types.rs

/// Relation operator for variable comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationOp {
    Lt,  // <
    Le,  // ≤
    Gt,  // >
    Ge,  // ≥
    Eq,  // =
    Ne,  // ≠
}

/// Arithmetic operator for expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Unit for distinctness constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DistinctUnit {
    Element,
    Tuple,
}

/// Structural property tag for graph/array properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyTag {
    Simple,
    Connected,
    Tree,
    Permutation,
    Binary,
    Odd,
    Even,
    Custom(String),
}

/// Sort order for sorted constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortOrder {
    Ascending,
    NonDecreasing,
    Descending,
    NonIncreasing,
}

/// Character set specification for string generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharSetSpec {
    LowerAlpha,
    UpperAlpha,
    Alpha,
    Digit,
    AlphaNumeric,
    Custom(Vec<char>),
    Range(char, char),
}

/// Rendering hint kind (separator info moved from StructureAST per S-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderHintKind {
    Separator(Separator),
}

/// Separator between elements in rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Separator {
    Space,
    None,
}
```

- [ ] **Step 5: Update `constraint/mod.rs`**

Add new modules and re-exports (keep existing for backward compatibility):

```rust
#[allow(clippy::module_inception)]
pub mod constraint;
pub mod constraint_id;
pub mod constraint_set;
pub mod expected_type;
pub mod expression;
pub mod types;

pub use constraint::Constraint;
pub use constraint_id::ConstraintId;
pub use constraint_set::ConstraintSet;
pub use expected_type::ExpectedType;
pub use expression::Expression;
pub use types::{
    ArithOp, CharSetSpec, DistinctUnit, PropertyTag, RelationOp,
    RenderHintKind, Separator, SortOrder,
};
```

- [ ] **Step 6: Run all tests, clippy, fmt**
- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat(constraint): add Rev.1 primitive types (ConstraintId, supporting enums)

Add foundation types for Rev.1 constraint model:
- ConstraintId: unique constraint identifier for RemoveConstraint
- RelationOp: 6-variant comparison operator
- ArithOp: 4-variant arithmetic operator
- DistinctUnit, PropertyTag, SortOrder: constraint qualifiers
- CharSetSpec, RenderHintKind, Separator: rendering/generation support

Non-breaking: all existing tests continue to pass.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint B: StructureAST Rewrite

### Task 3: rewrite-structure-ast

**BREAKING CHANGE.** This task replaces the tree-based StructureNode/Slot model with the Rev.1 arena-based StructureAst. All structure tests must be rewritten.

**Files:**
- Rewrite: `crates/cp-ast-core/src/structure/node_kind.rs`
- Rewrite: `crates/cp-ast-core/src/structure/structure_node.rs`
- Create: `crates/cp-ast-core/src/structure/structure_ast.rs`
- Delete: `crates/cp-ast-core/src/structure/slot.rs`
- Modify: `crates/cp-ast-core/src/structure/mod.rs`
- Rewrite: `crates/cp-ast-core/tests/structure_basic.rs` → rename to `tests/structure_ast.rs`
- Delete: `crates/cp-ast-core/tests/hole_basic.rs`
- Modify: `crates/cp-ast-core/tests/typical_problem.rs` (temporarily break — will be fixed in T-06)

**Key design decisions:**
- NodeKind changes from Copy enum (7 variants) to rich enum with data (9 variants)
- StructureNode simplifies from `{id, kind, name, slots}` to `{id, kind}` — name lives inside NodeKind variants
- Hole becomes a NodeKind variant (not SlotValue)
- StructureAst is an arena: `Vec<Option<StructureNode>>` indexed by NodeId
- Slot.rs is completely removed (no more HoleInfo, SlotValue)

- [ ] **Step 1: Write new tests in `tests/structure_ast.rs`**

```rust
// tests/structure_ast.rs
use cp_ast_core::structure::{
    Ident, Literal, NodeId, NodeKind, NodeKindHint, Reference,
    StructureAst, StructureNode,
};

// --- NodeKind tests ---

#[test]
fn node_kind_scalar() {
    let kind = NodeKind::Scalar { name: Ident::new("N") };
    assert!(matches!(kind, NodeKind::Scalar { .. }));
}

#[test]
fn node_kind_array() {
    let kind = NodeKind::Array {
        name: Ident::new("A"),
        length: Reference::Unresolved(Ident::new("N")),
    };
    assert!(matches!(kind, NodeKind::Array { .. }));
}

#[test]
fn node_kind_matrix() {
    let kind = NodeKind::Matrix {
        name: Ident::new("C"),
        rows: Reference::Unresolved(Ident::new("H")),
        cols: Reference::Unresolved(Ident::new("W")),
    };
    assert!(matches!(kind, NodeKind::Matrix { .. }));
}

#[test]
fn node_kind_tuple() {
    let kind = NodeKind::Tuple {
        elements: vec![NodeId::from_raw(1), NodeId::from_raw(2)],
    };
    assert!(matches!(kind, NodeKind::Tuple { .. }));
}

#[test]
fn node_kind_repeat() {
    let kind = NodeKind::Repeat {
        count: Reference::Unresolved(Ident::new("M")),
        body: vec![NodeId::from_raw(5)],
    };
    assert!(matches!(kind, NodeKind::Repeat { .. }));
}

#[test]
fn node_kind_section() {
    let kind = NodeKind::Section {
        header: Some(NodeId::from_raw(1)),
        body: vec![NodeId::from_raw(2)],
    };
    assert!(matches!(kind, NodeKind::Section { .. }));
}

#[test]
fn node_kind_sequence() {
    let kind = NodeKind::Sequence {
        children: vec![NodeId::from_raw(1), NodeId::from_raw(2)],
    };
    assert!(matches!(kind, NodeKind::Sequence { .. }));
}

#[test]
fn node_kind_choice() {
    let kind = NodeKind::Choice {
        tag: Reference::Unresolved(Ident::new("type")),
        variants: vec![
            (Literal::IntLit(1), vec![NodeId::from_raw(10)]),
            (Literal::IntLit(2), vec![NodeId::from_raw(20)]),
        ],
    };
    assert!(matches!(kind, NodeKind::Choice { .. }));
}

#[test]
fn node_kind_hole() {
    let kind = NodeKind::Hole { expected_kind: Some(NodeKindHint::AnyScalar) };
    assert!(matches!(kind, NodeKind::Hole { .. }));
}

#[test]
fn node_kind_hole_no_hint() {
    let kind = NodeKind::Hole { expected_kind: None };
    assert!(matches!(kind, NodeKind::Hole { expected_kind: None }));
}

// --- StructureNode tests ---

#[test]
fn structure_node_creation() {
    let node = StructureNode::new(NodeId::from_raw(1), NodeKind::Scalar { name: Ident::new("N") });
    assert_eq!(node.id(), NodeId::from_raw(1));
    assert!(matches!(node.kind(), &NodeKind::Scalar { .. }));
}

// --- StructureAst tests ---

#[test]
fn ast_new_has_sequence_root() {
    let ast = StructureAst::new();
    let root = ast.get(ast.root()).unwrap();
    assert!(matches!(root.kind(), &NodeKind::Sequence { .. }));
}

#[test]
fn ast_add_and_get_node() {
    let mut ast = StructureAst::new();
    let id = ast.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let node = ast.get(id).unwrap();
    assert!(matches!(node.kind(), &NodeKind::Scalar { .. }));
    assert_eq!(node.id(), id);
}

#[test]
fn ast_add_multiple_nodes() {
    let mut ast = StructureAst::new();
    let id1 = ast.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let id2 = ast.add_node(NodeKind::Scalar { name: Ident::new("M") });
    assert_ne!(id1, id2);
    assert!(ast.contains(id1));
    assert!(ast.contains(id2));
}

#[test]
fn ast_remove_node() {
    let mut ast = StructureAst::new();
    let id = ast.add_node(NodeKind::Scalar { name: Ident::new("N") });
    assert!(ast.contains(id));
    let removed = ast.remove(id);
    assert!(removed.is_some());
    assert!(!ast.contains(id));
}

#[test]
fn ast_get_nonexistent_returns_none() {
    let ast = StructureAst::new();
    assert!(ast.get(NodeId::from_raw(999)).is_none());
}

#[test]
fn ast_get_mut_and_modify() {
    let mut ast = StructureAst::new();
    let id = ast.add_node(NodeKind::Hole { expected_kind: None });
    ast.get_mut(id).unwrap().set_kind(NodeKind::Scalar { name: Ident::new("N") });
    let node = ast.get(id).unwrap();
    assert!(matches!(node.kind(), &NodeKind::Scalar { .. }));
}

#[test]
fn ast_len_counts_only_live_nodes() {
    let mut ast = StructureAst::new();
    assert_eq!(ast.len(), 1); // root Sequence
    let id = ast.add_node(NodeKind::Scalar { name: Ident::new("N") });
    assert_eq!(ast.len(), 2);
    ast.remove(id);
    assert_eq!(ast.len(), 1);
}

#[test]
fn ast_iter_yields_live_nodes() {
    let mut ast = StructureAst::new();
    ast.add_node(NodeKind::Scalar { name: Ident::new("N") });
    ast.add_node(NodeKind::Scalar { name: Ident::new("M") });
    let nodes: Vec<_> = ast.iter().collect();
    assert_eq!(nodes.len(), 3); // root + N + M
}

#[test]
fn ast_add_hole_node() {
    let mut ast = StructureAst::new();
    let hole_id = ast.add_node(NodeKind::Hole { expected_kind: Some(NodeKindHint::AnyScalar) });
    let node = ast.get(hole_id).unwrap();
    assert!(matches!(node.kind(), &NodeKind::Hole { .. }));
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Rewrite `node_kind.rs`**

```rust
// src/structure/node_kind.rs
use super::node_id::NodeId;
use super::reference::Reference;
use super::types::{Ident, Literal, NodeKindHint};

/// The kind of structure node in a competitive programming problem specification.
///
/// Rev.1: Rich variants with embedded data. Type and separator info
/// moved to ConstraintAST (TypeDecl, RenderHint).
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// Single variable: N, M, S, etc.
    Scalar { name: Ident },
    /// 1D array: A_1 ... A_N.
    Array { name: Ident, length: Reference },
    /// 2D grid: C[i][j], A_{i,j}.
    Matrix { name: Ident, rows: Reference, cols: Reference },
    /// Same-line variable group: (N, M, K), (u_i, v_i).
    Tuple { elements: Vec<NodeId> },
    /// Variable-dependent repetition: M lines, T test cases.
    Repeat { count: Reference, body: Vec<NodeId> },
    /// Semantically delimited block: header + body.
    Section { header: Option<NodeId>, body: Vec<NodeId> },
    /// Ordered root of the entire input.
    Sequence { children: Vec<NodeId> },
    /// Tag-dependent branching (query type variants).
    Choice { tag: Reference, variants: Vec<(Literal, Vec<NodeId>)> },
    /// Unfilled position (first-class hole).
    Hole { expected_kind: Option<NodeKindHint> },
}
```

- [ ] **Step 4: Rewrite `structure_node.rs`**

```rust
// src/structure/structure_node.rs
use super::node_id::NodeId;
use super::node_kind::NodeKind;

/// A node in the structure AST.
///
/// Rev.1: Simplified to just id + kind. Name and structural data
/// are embedded in NodeKind variants. Parent-child relationships
/// are managed by the StructureAst arena.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureNode {
    id: NodeId,
    kind: NodeKind,
}

impl StructureNode {
    /// Create a new structure node.
    #[must_use]
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self { id, kind }
    }

    /// Returns the unique ID of this node.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Returns a reference to the kind of this node.
    #[must_use]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Replace the kind of this node (used by FillHole).
    pub fn set_kind(&mut self, kind: NodeKind) {
        self.kind = kind;
    }
}
```

- [ ] **Step 5: Create `structure_ast.rs`**

```rust
// src/structure/structure_ast.rs
use super::node_id::NodeId;
use super::node_kind::NodeKind;
use super::structure_node::StructureNode;

/// Arena-based structure AST.
///
/// Nodes are stored in a Vec indexed by NodeId. Insertion order is preserved
/// for deterministic canonical rendering (Rev.1 M-1).
#[derive(Debug, Clone)]
pub struct StructureAst {
    root: NodeId,
    arena: Vec<Option<StructureNode>>,
    next_id: u64,
}

impl StructureAst {
    /// Create a new AST with an empty Sequence as root.
    #[must_use]
    pub fn new() -> Self {
        let root_id = NodeId::from_raw(0);
        let root_node = StructureNode::new(
            root_id,
            NodeKind::Sequence { children: Vec::new() },
        );
        Self {
            root: root_id,
            arena: vec![Some(root_node)],
            next_id: 1,
        }
    }

    /// Add a node to the arena and return its assigned NodeId.
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId::from_raw(self.next_id);
        self.next_id += 1;
        let node = StructureNode::new(id, kind);
        // Ensure arena is large enough
        let idx = id.value() as usize;
        if idx >= self.arena.len() {
            self.arena.resize_with(idx + 1, || None);
        }
        self.arena[idx] = Some(node);
        id
    }

    /// Get a reference to a node by ID.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&StructureNode> {
        self.arena
            .get(id.value() as usize)
            .and_then(|slot| slot.as_ref())
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut StructureNode> {
        self.arena
            .get_mut(id.value() as usize)
            .and_then(|slot| slot.as_mut())
    }

    /// Remove a node from the arena, returning it if it existed.
    pub fn remove(&mut self, id: NodeId) -> Option<StructureNode> {
        self.arena
            .get_mut(id.value() as usize)
            .and_then(|slot| slot.take())
    }

    /// Returns the root node ID.
    #[must_use]
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Set a new root node ID.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = id;
    }

    /// Check if a node exists in the arena.
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.get(id).is_some()
    }

    /// Returns the count of live (non-removed) nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arena.iter().filter(|s| s.is_some()).count()
    }

    /// Returns true if the arena has no live nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all live nodes in arena order.
    pub fn iter(&self) -> impl Iterator<Item = &StructureNode> {
        self.arena.iter().filter_map(|s| s.as_ref())
    }

    /// Returns the next ID that will be assigned.
    #[must_use]
    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

impl Default for StructureAst {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 6: Delete `slot.rs`**

Remove the file entirely. The Slot/HoleInfo/SlotValue types are replaced by:
- Hole → `NodeKind::Hole` variant
- Slot children → `Vec<NodeId>` in NodeKind variants
- HoleInfo → NodeId of the Hole node itself

- [ ] **Step 7: Update `structure/mod.rs`**

```rust
pub mod node_id;
pub mod node_kind;
pub mod reference;
pub mod structure_ast;
pub mod structure_node;
pub mod types;

pub use node_id::NodeId;
pub use node_kind::NodeKind;
pub use reference::Reference;
pub use structure_ast::StructureAst;
pub use structure_node::StructureNode;
pub use types::{Ident, Literal, NodeKindHint};
```

- [ ] **Step 8: Delete old test files**

Delete `tests/structure_basic.rs` and `tests/hole_basic.rs`. Their coverage is replaced by `tests/structure_ast.rs`.

- [ ] **Step 9: Temporarily disable `typical_problem.rs`**

Comment out the entire file content or delete it. It will be rewritten in T-06 after the constraint rewrite.

- [ ] **Step 10: Run all tests**

Run: `cargo test --all-targets 2>&1 | tail -10`
Expected: structure_ast tests pass, constraint tests still pass (they don't depend on structure node internals — they only use NodeId which is unchanged)

- [ ] **Step 11: Run clippy + fmt**
- [ ] **Step 12: Commit**

```bash
git add -A && git commit -m "refactor(structure): rewrite to Rev.1 arena-based StructureAst

BREAKING: Replace tree-based StructureNode/Slot model with arena-based design.
- NodeKind: 7 simple variants → 9 rich variants with embedded data
- StructureNode: simplified to {id, kind} (name/slots in NodeKind)
- StructureAst: new arena (Vec<Option<StructureNode>>) with NodeId indexing
- Remove: Slot, HoleInfo, SlotValue (Hole is now a NodeKind variant)
- Rewrite all structure tests for new API

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint C: ConstraintAST Rewrite

### Task 4: rewrite-constraint-types

**BREAKING CHANGE.** Rewrites Expression (4→5 variants), ExpectedType (5→3 variants), Constraint (4→12 variants).

**Files:**
- Rewrite: `crates/cp-ast-core/src/constraint/expression.rs`
- Rewrite: `crates/cp-ast-core/src/constraint/expected_type.rs`
- Rewrite: `crates/cp-ast-core/src/constraint/constraint.rs`
- Rewrite: `crates/cp-ast-core/tests/constraint_basic.rs` → rename to `tests/constraint_ast.rs`

- [ ] **Step 1: Write new tests in `tests/constraint_ast.rs`**

Tests should cover:
- Expression: all 5 variants (Lit, Var, BinOp, Pow, FnCall), evaluate_constant for each
- ExpectedType: 3 variants (Int, Str, Char), equality
- Constraint: all 12 variants construction, target() method where applicable
- At least 25 test functions

Key test functions:
```rust
use cp_ast_core::constraint::*;
use cp_ast_core::structure::*;

#[test] fn expression_lit() { ... }
#[test] fn expression_var_with_reference() { ... }
#[test] fn expression_binop_add() { ... }
#[test] fn expression_pow() { ... }
#[test] fn expression_fncall() { ... }
#[test] fn expression_evaluate_lit() { assert_eq!(Expression::Lit(42).evaluate_constant(), Some(42)); }
#[test] fn expression_evaluate_pow() { ... }  // base^exp
#[test] fn expression_evaluate_binop_mul() { ... }  // 2 * 10^5 = 200_000
#[test] fn expression_evaluate_var_returns_none() { ... }
#[test] fn expression_evaluate_fncall_returns_none() { ... }
#[test] fn expected_type_equality() { assert_eq!(ExpectedType::Int, ExpectedType::Int); }
#[test] fn expected_type_all_variants() { ... }  // Int, Str, Char
#[test] fn constraint_range() { ... }
#[test] fn constraint_type_decl() { ... }
#[test] fn constraint_length_relation() { ... }
#[test] fn constraint_relation() { ... }
#[test] fn constraint_distinct() { ... }
#[test] fn constraint_property() { ... }
#[test] fn constraint_sum_bound() { ... }
#[test] fn constraint_sorted() { ... }
#[test] fn constraint_guarantee() { ... }
#[test] fn constraint_charset() { ... }
#[test] fn constraint_string_length() { ... }
#[test] fn constraint_render_hint() { ... }
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Rewrite `expression.rs`**

```rust
// src/constraint/expression.rs
use crate::structure::{Ident, Reference};
use super::types::ArithOp;

/// Expression in constraints — represents numeric formulas.
///
/// Rev.1: 5 variants replacing the old 4.
/// Lit/Var/BinOp/Pow/FnCall.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal: 1, 42, 1000000007.
    Lit(i64),
    /// Variable reference: N, A[i].
    Var(Reference),
    /// Binary arithmetic: lhs op rhs.
    BinOp {
        op: ArithOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    /// Power: base^exp (e.g., 10^9, 2^30).
    Pow {
        base: Box<Expression>,
        exp: Box<Expression>,
    },
    /// Function call: min(a,b), max(a,b), abs(x), len(arr).
    FnCall {
        name: Ident,
        args: Vec<Expression>,
    },
}

impl Expression {
    /// Evaluate to a constant if possible (no variable references).
    #[must_use]
    pub fn evaluate_constant(&self) -> Option<i64> {
        match self {
            Self::Lit(v) => Some(*v),
            Self::Var(_) => None,
            Self::BinOp { op, lhs, rhs } => {
                let l = lhs.evaluate_constant()?;
                let r = rhs.evaluate_constant()?;
                match op {
                    ArithOp::Add => Some(l + r),
                    ArithOp::Sub => Some(l - r),
                    ArithOp::Mul => Some(l * r),
                    ArithOp::Div => {
                        if r == 0 { None } else { Some(l / r) }
                    }
                }
            }
            Self::Pow { base, exp } => {
                let b = base.evaluate_constant()?;
                let e = exp.evaluate_constant()?;
                let e_u32 = u32::try_from(e).ok()?;
                Some(b.pow(e_u32))
            }
            Self::FnCall { .. } => None,
        }
    }
}
```

- [ ] **Step 4: Rewrite `expected_type.rs`**

```rust
// src/constraint/expected_type.rs

/// Expected type for TypeDecl constraints.
///
/// Rev.1: Simplified to 3 variants. Array/Tuple/Float removed.
/// Complex type info is expressed via constraint composition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedType {
    Int,
    Str,
    Char,
}
```

- [ ] **Step 5: Rewrite `constraint.rs`**

```rust
// src/constraint/constraint.rs
use super::expected_type::ExpectedType;
use super::expression::Expression;
use super::types::{
    CharSetSpec, DistinctUnit, PropertyTag, RelationOp, RenderHintKind, SortOrder,
};
use crate::structure::Reference;

/// A constraint on the structure AST.
///
/// Rev.1: 12 variants covering all competitive programming constraint patterns.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Value range: lower ≤ target ≤ upper.
    Range {
        target: Reference,
        lower: Expression,
        upper: Expression,
    },
    /// Type declaration (single source of truth per S-1).
    TypeDecl {
        target: Reference,
        expected: ExpectedType,
    },
    /// Length relation: len(target) = length.
    LengthRelation {
        target: Reference,
        length: Expression,
    },
    /// Variable relation: lhs op rhs.
    Relation {
        lhs: Expression,
        op: RelationOp,
        rhs: Expression,
    },
    /// All elements are distinct.
    Distinct {
        elements: Reference,
        unit: DistinctUnit,
    },
    /// Structural property (graph/array).
    Property {
        target: Reference,
        tag: PropertyTag,
    },
    /// Sum bound across test cases.
    SumBound {
        variable: Reference,
        upper: Expression,
    },
    /// Elements are sorted.
    Sorted {
        elements: Reference,
        order: SortOrder,
    },
    /// Existence/validity guarantee (human-readable).
    Guarantee {
        description: String,
        predicate: Option<Expression>,
    },
    /// Character set constraint for strings.
    CharSet {
        target: Reference,
        charset: CharSetSpec,
    },
    /// String length constraint.
    StringLength {
        target: Reference,
        min: Expression,
        max: Expression,
    },
    /// Rendering hint (separator, moved from StructureAST per S-1).
    RenderHint {
        target: Reference,
        hint: RenderHintKind,
    },
}
```

- [ ] **Step 6: Delete old test file, rename**

Delete `tests/constraint_basic.rs`. The new `tests/constraint_ast.rs` replaces it.

- [ ] **Step 7: Update `constraint/mod.rs`** — Ensure all re-exports are correct.

- [ ] **Step 8: Run all tests, clippy, fmt**
- [ ] **Step 9: Commit**

```bash
git add -A && git commit -m "refactor(constraint): rewrite to Rev.1 types (Expression, Constraint 12 variants)

BREAKING: Replace simplified constraint types with full Rev.1 model.
- Expression: 4 variants → 5 (Lit, Var, BinOp, Pow, FnCall)
- ExpectedType: 5 variants → 3 (Int, Str, Char)
- Constraint: 4 variants → 12 (Range, TypeDecl, LengthRelation, Relation,
  Distinct, Property, SumBound, Sorted, Guarantee, CharSet, StringLength, RenderHint)
- Constraint now uses Reference instead of raw NodeId

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 5: rewrite-constraint-set

**Files:**
- Rewrite: `crates/cp-ast-core/src/constraint/constraint_set.rs`
- Add tests to: `crates/cp-ast-core/tests/constraint_ast.rs`

- [ ] **Step 1: Add ConstraintSet tests to `constraint_ast.rs`**

```rust
// Additional tests appended to tests/constraint_ast.rs

#[test]
fn constraint_set_empty() {
    let set = ConstraintSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn constraint_set_add_returns_id() {
    let mut set = ConstraintSet::new();
    let id = set.add(Some(NodeId::from_raw(1)), Constraint::Range {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        lower: Expression::Lit(1),
        upper: Expression::Lit(100),
    });
    assert_eq!(id.value(), 0); // first ID assigned
}

#[test]
fn constraint_set_get_by_id() {
    let mut set = ConstraintSet::new();
    let id = set.add(Some(NodeId::from_raw(1)), Constraint::TypeDecl {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        expected: ExpectedType::Int,
    });
    assert!(set.get(id).is_some());
}

#[test]
fn constraint_set_remove_by_id() {
    let mut set = ConstraintSet::new();
    let id = set.add(None, Constraint::Guarantee {
        description: "test".to_owned(),
        predicate: None,
    });
    assert_eq!(set.len(), 1);
    let removed = set.remove(id);
    assert!(removed.is_some());
    assert_eq!(set.len(), 0);
    assert!(set.get(id).is_none());
}

#[test]
fn constraint_set_for_node_returns_matching() {
    let mut set = ConstraintSet::new();
    let n = NodeId::from_raw(1);
    let m = NodeId::from_raw(2);
    set.add(Some(n), Constraint::Range {
        target: Reference::VariableRef(n),
        lower: Expression::Lit(1), upper: Expression::Lit(100),
    });
    set.add(Some(n), Constraint::TypeDecl {
        target: Reference::VariableRef(n),
        expected: ExpectedType::Int,
    });
    set.add(Some(m), Constraint::Range {
        target: Reference::VariableRef(m),
        lower: Expression::Lit(0), upper: Expression::Lit(50),
    });
    let n_ids = set.for_node(n);
    assert_eq!(n_ids.len(), 2);
    let m_ids = set.for_node(m);
    assert_eq!(m_ids.len(), 1);
}

#[test]
fn constraint_set_global_constraints() {
    let mut set = ConstraintSet::new();
    let g_id = set.add(None, Constraint::Guarantee {
        description: "Input is valid".to_owned(),
        predicate: None,
    });
    assert!(set.global().contains(&g_id));
    assert_eq!(set.global().len(), 1);
}

#[test]
fn constraint_set_iter_all() {
    let mut set = ConstraintSet::new();
    set.add(Some(NodeId::from_raw(1)), Constraint::TypeDecl {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        expected: ExpectedType::Int,
    });
    set.add(None, Constraint::Guarantee {
        description: "test".to_owned(), predicate: None,
    });
    let all: Vec<_> = set.iter().collect();
    assert_eq!(all.len(), 2);
}
```

- [ ] **Step 2: Rewrite `constraint_set.rs`**

```rust
// src/constraint/constraint_set.rs
use super::constraint::Constraint;
use super::constraint_id::ConstraintId;
use crate::structure::NodeId;

/// Arena-based constraint set with ConstraintId addressing.
///
/// Rev.1 S-2: Constraints are identified by ConstraintId for precise
/// RemoveConstraint operations. Supports per-node and global constraints.
#[derive(Debug, Clone, Default)]
pub struct ConstraintSet {
    arena: Vec<Option<Constraint>>,
    by_node: Vec<(NodeId, Vec<ConstraintId>)>,
    global: Vec<ConstraintId>,
    next_id: u64,
}

impl ConstraintSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a constraint. If `target` is Some, it's per-node; if None, it's global.
    /// Returns the assigned ConstraintId.
    pub fn add(&mut self, target: Option<NodeId>, constraint: Constraint) -> ConstraintId {
        let id = ConstraintId::from_raw(self.next_id);
        self.next_id += 1;
        let idx = id.value() as usize;
        if idx >= self.arena.len() {
            self.arena.resize_with(idx + 1, || None);
        }
        self.arena[idx] = Some(constraint);
        match target {
            Some(node_id) => {
                if let Some(entry) = self.by_node.iter_mut().find(|(n, _)| *n == node_id) {
                    entry.1.push(id);
                } else {
                    self.by_node.push((node_id, vec![id]));
                }
            }
            None => {
                self.global.push(id);
            }
        }
        id
    }

    /// Remove a constraint by ID.
    pub fn remove(&mut self, id: ConstraintId) -> Option<Constraint> {
        let constraint = self.arena.get_mut(id.value() as usize)?.take()?;
        // Remove from by_node index
        for (_, ids) in &mut self.by_node {
            ids.retain(|cid| *cid != id);
        }
        // Remove from global
        self.global.retain(|cid| *cid != id);
        Some(constraint)
    }

    /// Get a constraint by ID.
    #[must_use]
    pub fn get(&self, id: ConstraintId) -> Option<&Constraint> {
        self.arena.get(id.value() as usize)?.as_ref()
    }

    /// Get constraint IDs for a specific node.
    #[must_use]
    pub fn for_node(&self, node: NodeId) -> Vec<ConstraintId> {
        self.by_node
            .iter()
            .find(|(n, _)| *n == node)
            .map_or_else(Vec::new, |(_, ids)| ids.clone())
    }

    /// Get global constraint IDs.
    #[must_use]
    pub fn global(&self) -> &[ConstraintId] {
        &self.global
    }

    /// Count of live constraints.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arena.iter().filter(|c| c.is_some()).count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all live (ConstraintId, &Constraint) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (ConstraintId, &Constraint)> {
        self.arena
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                slot.as_ref().map(|c| {
                    (ConstraintId::from_raw(u64::try_from(i).expect("index fits u64")), c)
                })
            })
    }
}
```

- [ ] **Step 3: Run all tests, clippy, fmt**
- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "refactor(constraint): rewrite ConstraintSet to arena-based with ConstraintId

Rev.1 S-2: ConstraintSet uses arena + ConstraintId for precise addressing.
- add() returns ConstraintId
- remove(ConstraintId) for precise deletion
- for_node(NodeId) returns per-node constraint IDs
- global() returns global constraint IDs
- Supports both per-node and global constraints

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint D: Rev.1 Integration Tests

### Task 6: rev1-integration-test

Rewrite the typical problem integration test using the new Rev.1 types. Builds an ABC-style problem (N + array A) with StructureAst + ConstraintSet.

**Files:**
- Rewrite: `crates/cp-ast-core/tests/typical_problem.rs`

- [ ] **Step 1: Write the integration test**

```rust
// tests/typical_problem.rs
//! Integration test: express a typical AtCoder ABC problem with Rev.1 types.
//!
//! Problem: N + array A of length N.
//! Input:
//!   N
//!   A_1 A_2 ... A_N
//! Constraints:
//!   1 ≤ N ≤ 2×10^5
//!   0 ≤ A_i ≤ 10^9
//!   All values are integers

use cp_ast_core::constraint::*;
use cp_ast_core::structure::*;

#[test]
fn express_n_plus_array_rev1() {
    // --- Build StructureAST ---
    let mut ast = StructureAst::new();

    // Scalar N
    let n_id = ast.add_node(NodeKind::Scalar { name: Ident::new("N") });

    // Array A with length referencing N
    let a_id = ast.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Reference::VariableRef(n_id),
    });

    // Tuple for header line: (N) — single element on first line
    let header_id = ast.add_node(NodeKind::Tuple { elements: vec![n_id] });

    // Connect root Sequence → [header, A]
    if let Some(root) = ast.get_mut(ast.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header_id, a_id],
        });
    }

    // Verify structure
    assert_eq!(ast.len(), 4); // root + N + A + header
    assert!(ast.contains(n_id));
    assert!(ast.contains(a_id));

    // --- Build ConstraintAST ---
    let mut constraints = ConstraintSet::new();

    // N: Int
    constraints.add(Some(n_id), Constraint::TypeDecl {
        target: Reference::VariableRef(n_id),
        expected: ExpectedType::Int,
    });

    // 1 ≤ N ≤ 2×10^5
    constraints.add(Some(n_id), Constraint::Range {
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
    });

    // A elements: Int
    constraints.add(Some(a_id), Constraint::TypeDecl {
        target: Reference::VariableRef(a_id),
        expected: ExpectedType::Int,
    });

    // 0 ≤ A_i ≤ 10^9
    constraints.add(Some(a_id), Constraint::Range {
        target: Reference::IndexedRef {
            target: a_id,
            indices: vec![Ident::new("i")],
        },
        lower: Expression::Lit(0),
        upper: Expression::Pow {
            base: Box::new(Expression::Lit(10)),
            exp: Box::new(Expression::Lit(9)),
        },
    });

    // All values are integers (global guarantee)
    constraints.add(None, Constraint::Guarantee {
        description: "All values are integers".to_owned(),
        predicate: None,
    });

    assert_eq!(constraints.len(), 5);
    assert_eq!(constraints.for_node(n_id).len(), 2);
    assert_eq!(constraints.for_node(a_id).len(), 2);
    assert_eq!(constraints.global().len(), 1);
}

#[test]
fn express_problem_with_holes_rev1() {
    let mut ast = StructureAst::new();

    let n_id = ast.add_node(NodeKind::Scalar { name: Ident::new("N") });
    let hole_id = ast.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyArray),
    });

    if let Some(root) = ast.get_mut(ast.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, hole_id],
        });
    }

    // Verify hole exists
    let hole_node = ast.get(hole_id).unwrap();
    assert!(matches!(hole_node.kind(), NodeKind::Hole { .. }));

    // Count holes
    let hole_count = ast.iter()
        .filter(|n| matches!(n.kind(), NodeKind::Hole { .. }))
        .count();
    assert_eq!(hole_count, 1);
}
```

- [ ] **Step 2: Run all tests, clippy, fmt**
- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "test: rewrite integration tests for Rev.1 domain model

Rewrite typical_problem.rs to use arena-based StructureAst + ConstraintSet
with ConstraintId addressing. Covers N+array problem and hole scenarios.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Sprint E: Operation Layer

### Task 7: add-operation-framework

Create the operation module with all type definitions: Action, FillContent, OperationError, Operation trait.

**Files:**
- Modify: `crates/cp-ast-core/src/lib.rs` (add `pub mod operation;`)
- Create: `crates/cp-ast-core/src/operation/mod.rs`
- Create: `crates/cp-ast-core/src/operation/types.rs`
- Create: `crates/cp-ast-core/src/operation/action.rs`
- Create: `crates/cp-ast-core/src/operation/error.rs`
- Create: `crates/cp-ast-core/src/operation/result.rs`
- Create: `crates/cp-ast-core/src/operation/engine.rs`
- Create: `crates/cp-ast-core/tests/operation_basic.rs`

**Type definitions** (from `doc/design/projection-operation.md` §2, `doc/design/final-design.md` §8):

`operation/types.rs`:
```rust
use crate::structure::{Ident, NodeId, Reference};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarType { Int, Str, Char }

#[derive(Debug, Clone, PartialEq)]
pub enum LengthSpec {
    Fixed(usize),
    RefVar(NodeId),
    Expr(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FillContent {
    Scalar { name: String, typ: VarType },
    Array { name: String, element_type: VarType, length: LengthSpec },
    Grid { name: String, rows: LengthSpec, cols: LengthSpec, cell_type: VarType },
    Section { label: String },
    OutputSingleValue { typ: VarType },
    OutputYesNo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintDef {
    pub kind: ConstraintDefKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintDefKind {
    Range { lower: String, upper: String },
    TypeDecl { typ: VarType },
    Relation { op: crate::constraint::RelationOp, rhs: String },
    Distinct,
    Sorted { order: crate::constraint::SortOrder },
    Property { tag: String },
    SumBound { over_var: String, upper: String },
    Guarantee { description: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SumBoundDef {
    pub bound_var: String,
    pub upper: String,
}
```

`operation/action.rs`:
```rust
use crate::structure::NodeId;
use crate::constraint::ConstraintId;
use super::types::{FillContent, ConstraintDef, SumBoundDef};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    FillHole { target: NodeId, fill: FillContent },
    ReplaceNode { target: NodeId, replacement: FillContent },
    AddConstraint { target: NodeId, constraint: ConstraintDef },
    RemoveConstraint { constraint_id: ConstraintId },
    IntroduceMultiTestCase { count_var_name: String, sum_bound: Option<SumBoundDef> },
    AddSlotElement { parent: NodeId, slot_name: String, element: FillContent },
    RemoveSlotElement { parent: NodeId, slot_name: String, child: NodeId },
}
```

`operation/error.rs`:
```rust
use crate::structure::NodeId;
use crate::constraint::{ConstraintId, ExpectedType};

#[derive(Debug, Clone, PartialEq)]
pub struct ViolationDetail {
    pub constraint_id: ConstraintId,
    pub description: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationError {
    TypeMismatch { expected: ExpectedType, actual: String, context: String },
    NodeNotFound { node: NodeId },
    SlotOccupied { node: NodeId, current_occupant: String },
    ConstraintViolation { violated_constraints: Vec<ViolationDetail> },
    InvalidOperation { action: String, reason: String },
}
```

`operation/result.rs`:
```rust
use crate::structure::NodeId;
use crate::constraint::ConstraintId;
use super::types::FillContent;

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyResult {
    pub created_nodes: Vec<NodeId>,
    pub removed_nodes: Vec<NodeId>,
    pub created_constraints: Vec<ConstraintId>,
    pub affected_constraints: Vec<ConstraintId>,
}

// For projection/preview
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewResult {
    pub new_holes_created: Vec<NodeId>,
    pub constraints_affected: Vec<ConstraintId>,
}
```

`operation/engine.rs`:
```rust
use crate::structure::StructureAst;
use crate::constraint::ConstraintSet;
use super::action::Action;
use super::error::OperationError;
use super::result::{ApplyResult, PreviewResult};

/// The main AST engine that owns both Structure and Constraint data.
#[derive(Debug, Clone)]
pub struct AstEngine {
    pub structure: StructureAst,
    pub constraints: ConstraintSet,
}

impl AstEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            structure: StructureAst::new(),
            constraints: ConstraintSet::new(),
        }
    }

    pub fn apply(&mut self, action: Action) -> Result<ApplyResult, OperationError> {
        match action {
            Action::FillHole { .. } => todo!("T-08"),
            Action::AddConstraint { .. } => todo!("T-09"),
            Action::RemoveConstraint { .. } => todo!("T-09"),
            Action::ReplaceNode { .. } => todo!("T-09"),
            Action::AddSlotElement { .. } => todo!("T-09"),
            Action::RemoveSlotElement { .. } => todo!("T-09"),
            Action::IntroduceMultiTestCase { .. } => todo!("T-09"),
        }
    }

    pub fn preview(&self, action: &Action) -> Result<PreviewResult, OperationError> {
        let _ = action;
        todo!("T-10")
    }
}

impl Default for AstEngine {
    fn default() -> Self {
        Self::new()
    }
}
```

Tests: Verify all types compile, AstEngine can be constructed, Action variants exist.

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

```bash
git commit -m "feat(operation): add operation framework types (Action, Error, AstEngine)

- Action: 7 variants (FillHole, ReplaceNode, AddConstraint, etc.)
- FillContent: high-level fill intent for Builder layer
- OperationError: 5 error kinds with ViolationDetail
- ApplyResult/PreviewResult: operation output types
- AstEngine: owns StructureAst + ConstraintSet, apply()/preview() stubs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 8: impl-fill-hole-and-constraint-ops

Implement FillHole, AddConstraint, RemoveConstraint in AstEngine.

**Files:**
- Create: `crates/cp-ast-core/src/operation/fill_hole.rs`
- Create: `crates/cp-ast-core/src/operation/constraint_ops.rs`
- Modify: `crates/cp-ast-core/src/operation/engine.rs`
- Add tests to: `crates/cp-ast-core/tests/operation_basic.rs`

**FillHole logic** (from `doc/design/final-design.md` §8.4):
1. Verify target exists and is a Hole node
2. Expand FillContent to NodeKind (Scalar → NodeKind::Scalar, Array → NodeKind::Array + child holes, etc.)
3. Replace the Hole node with the new node(s)
4. Transfer any pre-existing constraints from the Hole to the new node
5. Return ApplyResult with created_nodes

**AddConstraint logic:**
1. Verify target exists
2. Parse ConstraintDef → Constraint
3. Add to ConstraintSet with target node
4. Return ApplyResult with created_constraints

**RemoveConstraint logic:**
1. Verify constraint_id exists
2. Remove from ConstraintSet
3. Return ApplyResult with affected_constraints

**Key tests:**
```rust
#[test] fn fill_hole_scalar_success() { ... }
#[test] fn fill_hole_nonexistent_node_fails() { ... }
#[test] fn fill_hole_non_hole_fails() { ... }
#[test] fn fill_hole_array_creates_child_holes() { ... }
#[test] fn add_constraint_range_success() { ... }
#[test] fn add_constraint_to_hole_allowed() { ... }  // Rev.1 L-4
#[test] fn add_constraint_node_not_found_fails() { ... }
#[test] fn remove_constraint_success() { ... }
#[test] fn remove_constraint_not_found_fails() { ... }
```

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

---

### Task 9: impl-remaining-ops

Implement ReplaceNode, AddSlotElement, RemoveSlotElement, IntroduceMultiTestCase.

**Files:**
- Create: `crates/cp-ast-core/src/operation/node_ops.rs`
- Create: `crates/cp-ast-core/src/operation/multi_test_case.rs`
- Modify: `crates/cp-ast-core/src/operation/engine.rs`
- Add tests to: `crates/cp-ast-core/tests/operation_basic.rs`

**ReplaceNode logic** (§8.4):
1. Verify target exists and is NOT a Hole
2. Check no unsafe dependents (constraints referencing this node)
3. Expand replacement FillContent
4. Replace the node, invalidating old references

**AddSlotElement logic:**
1. Verify parent exists and has a variable-length slot (Sequence.children, Tuple.elements, etc.)
2. Expand element FillContent
3. Append to the appropriate Vec<NodeId> in parent's NodeKind

**RemoveSlotElement logic:**
1. Verify parent and child exist, child is in parent's children
2. Remove child subtree
3. Remove constraints referencing removed nodes

**IntroduceMultiTestCase logic** (§8.4):
1. Verify no MultiTestCase already exists (no Repeat with "T" count)
2. Create new Repeat node with count var
3. Wrap existing structure as body
4. Optionally add SumBound constraint

**Key tests:**
```rust
#[test] fn replace_node_success() { ... }
#[test] fn replace_node_with_dependents_fails() { ... }
#[test] fn add_slot_element_to_sequence() { ... }
#[test] fn remove_slot_element_from_sequence() { ... }
#[test] fn introduce_multi_test_case_success() { ... }
#[test] fn introduce_multi_test_case_already_exists_fails() { ... }
```

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

---

## Sprint F: ProjectionAPI + Preview

### Task 10: impl-projection-api

Implement the full ProjectionAPI as a method set on AstEngine.

**Files:**
- Modify: `crates/cp-ast-core/src/lib.rs` (add `pub mod projection;`)
- Create: `crates/cp-ast-core/src/projection/mod.rs`
- Create: `crates/cp-ast-core/src/projection/types.rs`
- Create: `crates/cp-ast-core/src/projection/api.rs`
- Create: `crates/cp-ast-core/src/projection/projection_impl.rs`
- Create: `crates/cp-ast-core/tests/projection_basic.rs`

**ProjectionAPI trait** (from `doc/design/final-design.md` §7):
```rust
pub trait ProjectionAPI {
    fn nodes(&self) -> Vec<ProjectedNode>;
    fn children(&self, node: NodeId) -> Vec<SlotEntry>;
    fn inspect(&self, node: NodeId) -> Option<NodeDetail>;
    fn hole_candidates(&self, hole: NodeId) -> Vec<CandidateKind>;
    fn available_actions(&self) -> Vec<AvailableAction>;
    fn why_not_editable(&self, node: NodeId) -> Option<NotEditableReason>;
    fn completeness(&self) -> CompletenessSummary;
}
```

**Supporting types** (`projection/types.rs`):
```rust
pub struct ProjectedNode {
    pub id: NodeId,
    pub label: String,
    pub depth: usize,
    pub is_hole: bool,
}

pub struct SlotEntry {
    pub name: String,
    pub child: NodeId,
}

pub struct NodeDetail {
    pub id: NodeId,
    pub kind_label: String,
    pub constraints: Vec<String>,  // human-readable
}

pub enum CandidateKind {
    IntroduceScalar { suggested_names: Vec<String> },
    IntroduceArray { suggested_names: Vec<String> },
    IntroduceMatrix,
    IntroduceSection,
}

pub struct AvailableAction {
    pub action: crate::operation::Action,
    pub target: NodeId,
    pub description: String,
}

pub enum NotEditableReason {
    HasDependents { dependents: Vec<NodeId> },
    IsRoot,
}

pub struct CompletenessSummary {
    pub total_holes: usize,
    pub filled_slots: usize,
    pub unsatisfied_constraints: usize,
    pub is_complete: bool,
}
```

**Implementation:** DFS traversal for `nodes()`, parent-child inspection for `children()`, constraint lookup for `inspect()`, etc.

**Key tests** (based on §7.3 ABC284-C example):
```rust
#[test] fn nodes_returns_dfs_order() { ... }
#[test] fn children_of_sequence() { ... }
#[test] fn inspect_scalar_node() { ... }
#[test] fn hole_candidates_for_hole() { ... }
#[test] fn completeness_with_holes() { ... }
#[test] fn completeness_all_filled() { ... }
#[test] fn why_not_editable_root() { ... }
```

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

---

## Sprint G: Canonical Rendering + Preview

### Task 11: impl-canonical-renderer

Implement input format and constraint text rendering.

**Files:**
- Modify: `crates/cp-ast-core/src/lib.rs` (add `pub mod render;`)
- Create: `crates/cp-ast-core/src/render/mod.rs`
- Create: `crates/cp-ast-core/src/render/input_format.rs`
- Create: `crates/cp-ast-core/src/render/constraint_text.rs`
- Create: `crates/cp-ast-core/tests/render_basic.rs`

**Rendering rules** (from `doc/design/final-design.md` §9):
- Sequence.children order is master
- Array(length N): `A_1 A_2 … A_N`
- Repeat(M, Tuple(u,v)): M lines of `u_i v_i`
- Matrix(H×W) + RenderHint(sep=None): H lines, each W chars concatenated
- Variable naming: Scalars uppercase, array elements name+subscript
- Constraint display order: Range → TypeDecl → LengthRelation → Relation → Distinct → Property → Sorted → SumBound → Guarantee

**Key tests:**
```rust
#[test] fn render_scalar_n() { assert_eq!(render_input(&engine), "N\n"); }
#[test] fn render_n_plus_array() { assert_eq!(render_input(&engine), "N\nA_1 A_2 … A_N\n"); }
#[test] fn render_edge_list() { ... }  // "N M\nu_1 v_1\n...\nu_M v_M\n"
#[test] fn render_constraints_range() { assert!(text.contains("1 ≤ N ≤ 100")); }
#[test] fn render_constraints_sorted_by_type() { ... }
```

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

---

### Task 12: impl-preview

Implement preview (dry-run) for AstEngine.

**Files:**
- Modify: `crates/cp-ast-core/src/operation/engine.rs`
- Add tests to: `crates/cp-ast-core/tests/operation_basic.rs`

**Preview logic** (Rev.1 M-2):
- `preview(&self, action)` is read-only (takes `&self`)
- Validates the action (same checks as `apply`)
- Returns PreviewResult showing what would happen (new holes, affected constraints)
- Does NOT modify the AST

**Key tests:**
```rust
#[test] fn preview_fill_hole_shows_new_holes() { ... }
#[test] fn preview_invalid_action_returns_error() { ... }
#[test] fn preview_does_not_mutate_ast() { ... }
```

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

---

## Sprint H: Sample Generation + End-to-End

### Task 13: impl-sample-generation

Implement dependency graph, topological sort, and constraint-aware sample generators.

**Files:**
- Modify: `crates/cp-ast-core/src/lib.rs` (add `pub mod sample;`)
- Create: `crates/cp-ast-core/src/sample/mod.rs`
- Create: `crates/cp-ast-core/src/sample/dependency.rs`
- Create: `crates/cp-ast-core/src/sample/generator.rs`
- Create: `crates/cp-ast-core/src/sample/output.rs`
- Create: `crates/cp-ast-core/tests/sample_basic.rs`

**Pipeline** (from `doc/design/final-design.md` §10):
```
StructureAST + ConstraintAST
    → DependencyGraph (DAG)
    → TopologicalSort → GenerationOrder
    → Per-constraint generators → GeneratedSample
    → Text output
```

**L1 generators (Guaranteed):**
- Range: uniform random in [lower, upper]
- LengthRelation: generate length first, then array
- TypeDecl(Int): combined with Range

**L2 generators (HighProbability):**
- Distinct: Fisher-Yates / rejection sampling
- Sorted: generate then sort
- Property(Permutation): Fisher-Yates shuffle
- Property(Tree): Prüfer sequence
- Property(Simple): edge set management

**Key types:**
```rust
pub struct GeneratedSample {
    pub values: std::collections::HashMap<NodeId, SampleValue>,
    pub warnings: Vec<String>,
    pub guarantee_level: GuaranteeLevel,
}

pub enum SampleValue {
    Int(i64),
    Str(String),
    Array(Vec<SampleValue>),
    Grid(Vec<Vec<SampleValue>>),
}

pub enum GuaranteeLevel { L1Guaranteed, L2HighProbability, L3BestEffort }
```

**Key tests:**
```rust
#[test] fn dependency_graph_simple() { ... }
#[test] fn topo_sort_linear() { ... }
#[test] fn topo_sort_detects_cycle() { ... }
#[test] fn generate_range_within_bounds() { ... }  // property test: value ∈ [lo, hi]
#[test] fn generate_array_correct_length() { ... }
#[test] fn generate_distinct_all_unique() { ... }
#[test] fn generate_sorted_is_sorted() { ... }
#[test] fn sample_to_text_n_plus_array() { ... }
```

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

Note: Use `rand` crate as a dependency for random generation. Add to `crates/cp-ast-core/Cargo.toml`:
```toml
[dependencies]
rand = "0.8"
```

---

### Task 14 (FINAL): impl-e2e-tests (T-20 from §13.2)

End-to-end integration tests verifying the full pipeline: build AST → apply operations → render → generate samples → verify constraints.

**Files:**
- Create: `crates/cp-ast-core/tests/e2e_abc284c.rs`
- Modify: `crates/cp-ast-core/tests/typical_problem.rs` (add operation-based building)

**Test scenarios** (from §13.3):

1. **ABC284-C (Graph):** Build via FillHole operations → AddConstraint → verify completeness → render input format → generate 5 samples → verify all samples satisfy constraints

2. **ABC300-A (Simple arithmetic):** FillHole scalar variables → AddConstraint ranges → render → generate samples

3. **N + Array (typical):** Full pipeline with the standard pattern

**Key test:**
```rust
#[test]
fn e2e_abc284c_full_pipeline() {
    let mut engine = AstEngine::new();

    // Step 1: FillHole to build structure
    // (header with N, M; edge list; output)

    // Step 2: AddConstraint for all bounds

    // Step 3: Verify completeness
    let summary = engine.completeness();
    assert!(summary.is_complete);

    // Step 4: Render
    let input_text = cp_ast_core::render::render_input(&engine);
    assert!(input_text.contains("N M"));
    assert!(input_text.contains("u_"));

    let constraint_text = cp_ast_core::render::render_constraints(&engine);
    assert!(constraint_text.contains("1 ≤ N ≤ 100"));

    // Step 5: Generate samples
    let sample = cp_ast_core::sample::generate(&engine, 42);  // seed
    assert!(sample.warnings.is_empty() || sample.guarantee_level != GuaranteeLevel::L1Guaranteed);

    // Step 6: Verify sample satisfies constraints
    // (each value within declared range, array lengths correct, etc.)
}
```

- [ ] **Steps: Write tests → implement → clippy/fmt → commit**

```bash
git commit -m "test: add end-to-end integration tests (ABC284-C, ABC300-A)

Full pipeline verification: build → operate → render → sample → validate.
Covers §13.3 T-20 requirements from final design document.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Notes

- **Breaking changes in T-03/T-04**: These tasks intentionally break existing tests. Old test files are deleted and replaced. Ensure `cargo test` passes after each task completes.
- **NodeId migration**: `NodeId::new()` (AtomicU64) is preserved for backward compat but `from_raw()` is the primary constructor for arena-managed IDs. Consider deprecating `new()` after the arena is fully adopted.
- **Constraint references**: Constraints now use `Reference` instead of raw `NodeId`. This enables richer targeting (indexed refs, unresolved names) needed for Operation semantics.
- **FillContent scope**: The plan implements a practical subset of FillContent variants (Scalar, Array, Grid, Section, OutputSingleValue, OutputYesNo). Additional variants (EdgeList, QueryList, TriangularBlock, Interactive, etc.) can be added incrementally.
- **Sample generation dependency**: Task 13 introduces the first external dependency (`rand`). All prior tasks have zero dependencies.
