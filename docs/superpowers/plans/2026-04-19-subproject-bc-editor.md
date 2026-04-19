# Sub-project B+C: Projectional Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a fully interactive AST editor that passes all 26 E2E tests by extending the Rust WASM backend (Projection + Actions) and building a Preact frontend.

**Architecture:** TEA (The Elm Architecture) with stateless WASM. `documentJson` (Preact Signal) is the Model; `apply_action()` is Update; `project_full()` is View. DTO↔AstEngine conversion happens ONLY at `#[wasm_bindgen]` boundaries. Popup intermediate states never touch WASM.

**Tech Stack:** Rust 2021 / wasm-pack / cp-ast-core / cp-ast-json / Preact 10 + @preact/signals / TypeScript / Vitest / Playwright

---

## File Structure Overview

### Rust (Sub-project B) — New/Modified Files

| File | Action | Responsibility |
|------|--------|----------------|
| `crates/cp-ast-core/src/operation/types.rs` | Modify | Add EdgeList, WeightedEdgeList, QueryList, MultiTestCaseTemplate, GridTemplate to FillContent |
| `crates/cp-ast-core/src/operation/action.rs` | Modify | Add AddSibling, AddChoiceVariant action variants |
| `crates/cp-ast-core/src/operation/engine.rs` | Modify | Dispatch new action variants |
| `crates/cp-ast-core/src/operation/fill_hole.rs` | Modify | Expand new FillContent variants into NodeKind trees |
| `crates/cp-ast-core/src/operation/node_ops.rs` | Modify | Add add_sibling() and add_choice_variant() methods |
| `crates/cp-ast-core/src/operation/mod.rs` | Modify | Re-export if needed |
| `crates/cp-ast-core/src/projection/types.rs` | Modify | Add FullProjection, Hotspot, DraftConstraint, CompletedConstraint, ExprCandidate types |
| `crates/cp-ast-core/src/projection/full_projection.rs` | Create | project_full() implementation |
| `crates/cp-ast-core/src/projection/mod.rs` | Modify | Add full_projection module |
| `crates/cp-ast-json/src/dto.rs` | Modify | Add ActionDto, FillContentDto, LengthSpecDto, ConstraintDefDto, SumBoundDefDto, FullProjectionDto, HotspotDto, DraftConstraintDto, CompletedConstraintDto, ExprCandidateDto, HoleCandidateDetailDto |
| `crates/cp-ast-json/src/action_dto.rs` | Create | Action ↔ DTO conversion (bidirectional) |
| `crates/cp-ast-json/src/projection_dto.rs` | Create | FullProjection → DTO conversion (serialize only) |
| `crates/cp-ast-json/src/lib.rs` | Modify | Export new serialization functions |
| `crates/cp-ast-wasm/src/lib.rs` | Modify | Add 6 new wasm_bindgen functions |

### Frontend (Sub-project C) — New/Modified Files

| File | Action | Responsibility |
|------|--------|----------------|
| `web/package.json` | Modify | Add vitest, @testing-library/preact, jsdom |
| `web/vitest.config.ts` | Create | Vitest configuration |
| `web/src/wasm.ts` | Modify | Export 6 new WASM functions |
| `web/src/app.tsx` | Modify | Add Editor page as default route |
| `web/src/editor/editor-state.ts` | Create | Editor signals + dispatchAction |
| `web/src/editor/popup-state.ts` | Create | Popup local state signals |
| `web/src/editor/action-builder.ts` | Create | UI events → Action JSON builder |
| `web/src/editor/EditorPage.tsx` | Create | 3-pane layout component |
| `web/src/editor/StructurePane.tsx` | Create | Structure tree + hotspots |
| `web/src/editor/NodePopup.tsx` | Create | Node creation wizard |
| `web/src/editor/ExpressionBuilder.tsx` | Create | Expression builder (N-1 etc.) |
| `web/src/editor/ConstraintPane.tsx` | Create | Draft/completed constraints |
| `web/src/editor/ConstraintEditor.tsx` | Create | Constraint editing popup |
| `web/src/editor/ValueInput.tsx` | Create | Value input popup (var list + literal) |
| `web/src/editor/PreviewPane.tsx` | Create | TeX + sample preview |
| `web/src/index.css` | Modify | Add editor layout styles |

### Test Files

| File | Action | Responsibility |
|------|--------|----------------|
| `web/tests/unit/action-builder.test.ts` | Create | Action builder unit tests |
| `web/tests/unit/editor-state.test.ts` | Create | State management unit tests |

---

## Task 1: Extend FillContent with Template Variants

**Files:**
- Modify: `crates/cp-ast-core/src/operation/types.rs:27-49`
- Modify: `crates/cp-ast-core/src/operation/fill_hole.rs:62-104`
- Test: `crates/cp-ast-core/tests/fill_content_templates.rs` (create)

- [ ] **Step 1: Write failing tests for new FillContent variants**

Create `crates/cp-ast-core/tests/fill_content_templates.rs`:

```rust
use cp_ast_core::operation::engine::AstEngine;
use cp_ast_core::operation::types::{FillContent, LengthSpec, VarType};
use cp_ast_core::operation::action::Action;
use cp_ast_core::structure::NodeKind;

#[test]
fn edge_list_creates_repeat_with_tuple_uv() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // Fill root's first hole (we need a hole first)
    // Add a scalar N, then add edge list below
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    // Add edge list with count = RefVar to N (node id 1)
    let n_id = cp_ast_core::structure::NodeId::from_raw(1);
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::EdgeList {
            edge_count: LengthSpec::RefVar(n_id),
        },
    }).unwrap();

    // Verify: root should have 2 children (N, Repeat)
    if let Some(root_node) = engine.structure.get(root) {
        if let NodeKind::Sequence { children } = root_node.kind() {
            assert_eq!(children.len(), 2);
            // Second child should be Repeat
            let repeat_id = children[1];
            let repeat_node = engine.structure.get(repeat_id).unwrap();
            if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
                assert_eq!(body.len(), 1);
                // Body should be Tuple with u, v
                let tuple_id = body[0];
                let tuple_node = engine.structure.get(tuple_id).unwrap();
                if let NodeKind::Tuple { elements } = tuple_node.kind() {
                    assert_eq!(elements.len(), 2);
                } else {
                    panic!("Expected Tuple in Repeat body");
                }
            } else {
                panic!("Expected Repeat node");
            }
        } else {
            panic!("Root should be Sequence");
        }
    }
}

#[test]
fn weighted_edge_list_creates_repeat_with_tuple_uvw() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "M".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let m_id = cp_ast_core::structure::NodeId::from_raw(1);
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::WeightedEdgeList {
            edge_count: LengthSpec::RefVar(m_id),
            weight_name: "w".to_owned(),
            weight_type: VarType::Int,
        },
    }).unwrap();

    if let Some(root_node) = engine.structure.get(root) {
        if let NodeKind::Sequence { children } = root_node.kind() {
            let repeat_id = children[1];
            let repeat_node = engine.structure.get(repeat_id).unwrap();
            if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
                let tuple_id = body[0];
                let tuple_node = engine.structure.get(tuple_id).unwrap();
                if let NodeKind::Tuple { elements } = tuple_node.kind() {
                    assert_eq!(elements.len(), 3, "u, v, w");
                } else {
                    panic!("Expected Tuple");
                }
            } else {
                panic!("Expected Repeat");
            }
        }
    }
}

#[test]
fn query_list_creates_repeat_with_choice() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "Q".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let q_id = cp_ast_core::structure::NodeId::from_raw(1);
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::QueryList {
            query_count: LengthSpec::RefVar(q_id),
        },
    }).unwrap();

    if let Some(root_node) = engine.structure.get(root) {
        if let NodeKind::Sequence { children } = root_node.kind() {
            let repeat_id = children[1];
            let repeat_node = engine.structure.get(repeat_id).unwrap();
            if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
                assert_eq!(body.len(), 1);
                let choice_id = body[0];
                let choice_node = engine.structure.get(choice_id).unwrap();
                assert!(matches!(choice_node.kind(), NodeKind::Choice { .. }));
            } else {
                panic!("Expected Repeat");
            }
        }
    }
}

#[test]
fn multi_testcase_template_creates_repeat_with_hole() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::MultiTestCaseTemplate {
            count: LengthSpec::RefVar(cp_ast_core::structure::NodeId::from_raw(99)),
        },
    }).unwrap();

    if let Some(root_node) = engine.structure.get(root) {
        if let NodeKind::Sequence { children } = root_node.kind() {
            assert_eq!(children.len(), 1);
            let repeat_id = children[0];
            let repeat_node = engine.structure.get(repeat_id).unwrap();
            if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
                assert_eq!(body.len(), 1);
                let hole_id = body[0];
                let hole_node = engine.structure.get(hole_id).unwrap();
                assert!(matches!(hole_node.kind(), NodeKind::Hole { .. }));
            } else {
                panic!("Expected Repeat");
            }
        }
    }
}

#[test]
fn grid_template_creates_matrix_node() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::GridTemplate {
            name: "S".to_owned(),
            rows: LengthSpec::RefVar(cp_ast_core::structure::NodeId::from_raw(1)),
            cols: LengthSpec::RefVar(cp_ast_core::structure::NodeId::from_raw(2)),
            cell_type: VarType::Char,
        },
    }).unwrap();

    if let Some(root_node) = engine.structure.get(root) {
        if let NodeKind::Sequence { children } = root_node.kind() {
            let matrix_id = children[0];
            let matrix_node = engine.structure.get(matrix_id).unwrap();
            assert!(matches!(matrix_node.kind(), NodeKind::Matrix { .. }));
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test fill_content_templates`
Expected: FAIL — `FillContent::EdgeList` etc. not found

- [ ] **Step 3: Add new FillContent variants**

In `crates/cp-ast-core/src/operation/types.rs`, add these variants to `FillContent`:

```rust
/// Fill with an edge list (u_i, v_i pairs).
EdgeList {
    edge_count: LengthSpec,
},
/// Fill with a weighted edge list (u_i, v_i, w_i triples).
WeightedEdgeList {
    edge_count: LengthSpec,
    weight_name: String,
    weight_type: VarType,
},
/// Fill with a query list (Choice inside Repeat).
QueryList {
    query_count: LengthSpec,
},
/// Fill with a multi-testcase repeat block (Repeat with Hole body).
MultiTestCaseTemplate {
    count: LengthSpec,
},
/// Fill with a grid template (Matrix node).
GridTemplate {
    name: String,
    rows: LengthSpec,
    cols: LengthSpec,
    cell_type: VarType,
},
```

- [ ] **Step 4: Implement expand_fill_content for new variants**

In `crates/cp-ast-core/src/operation/fill_hole.rs`, add match arms to `expand_fill_content()`:

```rust
FillContent::EdgeList { edge_count } => {
    let u_id = self.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    created.push(u_id);
    let v_id = self.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    created.push(v_id);
    let tuple_id = self.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    created.push(tuple_id);
    NodeKind::Repeat {
        count: length_spec_to_expression(edge_count),
        index_var: None,
        body: vec![tuple_id],
    }
}
FillContent::WeightedEdgeList {
    edge_count,
    weight_name,
    weight_type: _,
} => {
    let u_id = self.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    created.push(u_id);
    let v_id = self.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    created.push(v_id);
    let w_id = self.structure.add_node(NodeKind::Scalar {
        name: Ident::new(weight_name),
    });
    created.push(w_id);
    let tuple_id = self.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id, w_id],
    });
    created.push(tuple_id);
    NodeKind::Repeat {
        count: length_spec_to_expression(edge_count),
        index_var: None,
        body: vec![tuple_id],
    }
}
FillContent::QueryList { query_count } => {
    // Create a Choice node with an empty tag ref and no variants
    let tag_ref = Reference::Unresolved(Ident::new("type"));
    let choice_id = self.structure.add_node(NodeKind::Choice {
        tag: tag_ref,
        variants: vec![],
    });
    created.push(choice_id);
    NodeKind::Repeat {
        count: length_spec_to_expression(query_count),
        index_var: None,
        body: vec![choice_id],
    }
}
FillContent::MultiTestCaseTemplate { count } => {
    let hole_id = self.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    created.push(hole_id);
    NodeKind::Repeat {
        count: length_spec_to_expression(count),
        index_var: None,
        body: vec![hole_id],
    }
}
FillContent::GridTemplate {
    name,
    rows,
    cols,
    ..
} => {
    let rows_ref = length_spec_to_reference(rows);
    let cols_ref = length_spec_to_reference(cols);
    NodeKind::Matrix {
        name: Ident::new(name),
        rows: rows_ref,
        cols: cols_ref,
    }
}
```

Also update `var_type_to_expected` to handle new variants that need TypeDecl:

```rust
FillContent::WeightedEdgeList { weight_type, .. } => {
    Some(var_type_to_expected_type(weight_type))
}
FillContent::GridTemplate { cell_type, .. } => {
    Some(var_type_to_expected_type(cell_type))
}
FillContent::EdgeList { .. }
| FillContent::QueryList { .. }
| FillContent::MultiTestCaseTemplate { .. } => None,
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test fill_content_templates`
Expected: PASS

- [ ] **Step 6: Run full test suite + clippy**

Run: `cargo test --all-targets && cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/cp-ast-core/src/operation/types.rs crates/cp-ast-core/src/operation/fill_hole.rs crates/cp-ast-core/tests/fill_content_templates.rs
git commit -m "feat(core): add EdgeList, WeightedEdgeList, QueryList, MultiTestCaseTemplate, GridTemplate to FillContent

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 2: Add AddSibling and AddChoiceVariant Actions

**Files:**
- Modify: `crates/cp-ast-core/src/operation/action.rs:1-41`
- Modify: `crates/cp-ast-core/src/operation/engine.rs:32-58`
- Modify: `crates/cp-ast-core/src/operation/node_ops.rs`
- Test: `crates/cp-ast-core/tests/add_sibling.rs` (create)

- [ ] **Step 1: Write failing tests for AddSibling**

Create `crates/cp-ast-core/tests/add_sibling.rs`:

```rust
use cp_ast_core::operation::engine::AstEngine;
use cp_ast_core::operation::types::{FillContent, VarType};
use cp_ast_core::operation::action::Action;
use cp_ast_core::structure::{NodeId, NodeKind};

#[test]
fn add_sibling_wraps_scalar_in_tuple() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // Add scalar H
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "H".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let h_id = NodeId::from_raw(1);

    // AddSibling W next to H → should create Tuple[H, W]
    engine.apply(&Action::AddSibling {
        target: h_id,
        element: FillContent::Scalar {
            name: "W".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    // Root's first child should now be a Tuple
    if let Some(root_node) = engine.structure.get(root) {
        if let NodeKind::Sequence { children } = root_node.kind() {
            assert_eq!(children.len(), 1, "Tuple should replace H in Sequence");
            let tuple_id = children[0];
            let tuple_node = engine.structure.get(tuple_id).unwrap();
            if let NodeKind::Tuple { elements } = tuple_node.kind() {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], h_id);
                // elements[1] is the new W scalar
                let w_node = engine.structure.get(elements[1]).unwrap();
                assert!(matches!(w_node.kind(), NodeKind::Scalar { .. }));
            } else {
                panic!("Expected Tuple, got {:?}", tuple_node.kind());
            }
        }
    }
}

#[test]
fn add_sibling_to_existing_tuple_appends() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // Add H
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "H".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let h_id = NodeId::from_raw(1);

    // Add W as sibling → creates Tuple[H, W]
    engine.apply(&Action::AddSibling {
        target: h_id,
        element: FillContent::Scalar {
            name: "W".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let w_id = NodeId::from_raw(2); // W scalar

    // Add K as sibling to W → should append to existing Tuple[H, W, K]
    engine.apply(&Action::AddSibling {
        target: w_id,
        element: FillContent::Scalar {
            name: "K".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    if let Some(root_node) = engine.structure.get(root) {
        if let NodeKind::Sequence { children } = root_node.kind() {
            assert_eq!(children.len(), 1);
            let tuple_id = children[0];
            let tuple_node = engine.structure.get(tuple_id).unwrap();
            if let NodeKind::Tuple { elements } = tuple_node.kind() {
                assert_eq!(elements.len(), 3, "H, W, K");
            } else {
                panic!("Expected Tuple");
            }
        }
    }
}

#[test]
fn add_choice_variant_adds_to_choice() {
    use cp_ast_core::structure::Literal;

    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // Build: Q (scalar) + QueryList
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "Q".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let q_id = cp_ast_core::structure::NodeId::from_raw(1);
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::QueryList {
            query_count: cp_ast_core::operation::types::LengthSpec::RefVar(q_id),
        },
    }).unwrap();

    // Find the Choice node (inside the Repeat)
    let mut choice_id = None;
    for node in engine.structure.iter() {
        if matches!(node.kind(), NodeKind::Choice { .. }) {
            choice_id = Some(node.id());
            break;
        }
    }
    let choice_id = choice_id.expect("Should have a Choice node");

    // Add variant with tag=1 and body element "a"
    engine.apply(&Action::AddChoiceVariant {
        choice: choice_id,
        tag_value: Literal::Int(1),
        first_element: FillContent::Scalar {
            name: "a".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let choice_node = engine.structure.get(choice_id).unwrap();
    if let NodeKind::Choice { variants, .. } = choice_node.kind() {
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].0, Literal::Int(1));
        assert_eq!(variants[0].1.len(), 1);
    } else {
        panic!("Expected Choice");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test add_sibling`
Expected: FAIL — `Action::AddSibling` variant not found

- [ ] **Step 3: Add AddSibling and AddChoiceVariant to Action enum**

In `crates/cp-ast-core/src/operation/action.rs`, add:

```rust
/// Add a sibling element next to a target node.
/// If target is a direct child of Sequence/Section/Repeat body,
/// wraps target + new element in a Tuple.
/// If target is already inside a Tuple, appends to that Tuple.
AddSibling {
    target: NodeId,
    element: FillContent,
},
/// Add a variant to a Choice node.
AddChoiceVariant {
    choice: NodeId,
    tag_value: crate::structure::Literal,
    first_element: FillContent,
},
```

- [ ] **Step 4: Implement add_sibling() in node_ops.rs**

Add to `crates/cp-ast-core/src/operation/node_ops.rs`:

```rust
/// Add a sibling element next to a target node.
///
/// Two cases:
/// 1. Target is a direct child of Sequence/Section/Repeat → wrap in Tuple
/// 2. Target is already in a Tuple → append to that Tuple
///
/// # Errors
/// Returns `OperationError` if target doesn't exist or parent can't be found.
pub(crate) fn add_sibling(
    &mut self,
    target: NodeId,
    element: &FillContent,
) -> Result<ApplyResult, OperationError> {
    if !self.structure.contains(target) {
        return Err(OperationError::NodeNotFound { node: target });
    }

    // Find the parent of target
    let parent_info = self.find_parent(target);

    let mut created_nodes = Vec::new();
    let new_kind = self.expand_fill_content(element, &mut created_nodes);
    let new_node_id = self.structure.add_node(new_kind);
    created_nodes.push(new_node_id);

    // Auto-add TypeDecl constraint if applicable
    let mut created_constraints = Vec::new();
    if let Some(expected_type) = super::fill_hole::var_type_to_expected_from_fill(element) {
        let cid = self.constraints.add(
            Some(new_node_id),
            crate::constraint::Constraint::TypeDecl {
                target: crate::structure::Reference::VariableRef(new_node_id),
                expected: expected_type,
            },
        );
        created_constraints.push(cid);
    }

    match parent_info {
        Some((parent_id, ParentSlot::Tuple)) => {
            // Target is in a Tuple — append new element
            let parent_node = self.structure.get_mut(parent_id)
                .ok_or(OperationError::NodeNotFound { node: parent_id })?;
            if let NodeKind::Tuple { elements } = parent_node.kind() {
                let mut new_elements = elements.clone();
                new_elements.push(new_node_id);
                parent_node.set_kind(NodeKind::Tuple {
                    elements: new_elements,
                });
            }
        }
        Some((parent_id, slot)) => {
            // Target is in Sequence/Section/Repeat body — wrap in Tuple
            let tuple_id = self.structure.add_node(NodeKind::Tuple {
                elements: vec![target, new_node_id],
            });
            created_nodes.push(tuple_id);

            // Replace target with tuple in parent's slot
            let parent_node = self.structure.get_mut(parent_id)
                .ok_or(OperationError::NodeNotFound { node: parent_id })?;
            Self::replace_child_in_slot(parent_node, &slot, target, tuple_id);
        }
        None => {
            return Err(OperationError::InvalidOperation {
                action: "AddSibling".to_owned(),
                reason: "Cannot find parent of target node".to_owned(),
            });
        }
    }

    Ok(ApplyResult {
        created_nodes,
        removed_nodes: vec![],
        created_constraints,
        affected_constraints: vec![],
    })
}

/// Add a variant to a Choice node.
pub(crate) fn add_choice_variant(
    &mut self,
    choice: NodeId,
    tag_value: &crate::structure::Literal,
    first_element: &FillContent,
) -> Result<ApplyResult, OperationError> {
    if !self.structure.contains(choice) {
        return Err(OperationError::NodeNotFound { node: choice });
    }

    let mut created_nodes = Vec::new();
    let new_kind = self.expand_fill_content(first_element, &mut created_nodes);
    let new_node_id = self.structure.add_node(new_kind);
    created_nodes.push(new_node_id);

    let choice_node = self.structure.get_mut(choice)
        .ok_or(OperationError::NodeNotFound { node: choice })?;

    if let NodeKind::Choice { tag, variants } = choice_node.kind() {
        let mut new_variants = variants.clone();
        new_variants.push((tag_value.clone(), vec![new_node_id]));
        choice_node.set_kind(NodeKind::Choice {
            tag: tag.clone(),
            variants: new_variants,
        });
    } else {
        return Err(OperationError::InvalidOperation {
            action: "AddChoiceVariant".to_owned(),
            reason: "Target is not a Choice node".to_owned(),
        });
    }

    Ok(ApplyResult {
        created_nodes,
        removed_nodes: vec![],
        created_constraints: vec![],
        affected_constraints: vec![],
    })
}
```

Also add the `find_parent` and `replace_child_in_slot` helper methods:

```rust
enum ParentSlot {
    Sequence,
    SectionBody,
    RepeatBody,
    Tuple,
}

fn find_parent(&self, target: NodeId) -> Option<(NodeId, ParentSlot)> {
    for node in self.structure.iter() {
        match node.kind() {
            NodeKind::Sequence { children } => {
                if children.contains(&target) {
                    return Some((node.id(), ParentSlot::Sequence));
                }
            }
            NodeKind::Section { body, .. } => {
                if body.contains(&target) {
                    return Some((node.id(), ParentSlot::SectionBody));
                }
            }
            NodeKind::Repeat { body, .. } => {
                if body.contains(&target) {
                    return Some((node.id(), ParentSlot::RepeatBody));
                }
            }
            NodeKind::Tuple { elements } => {
                if elements.contains(&target) {
                    return Some((node.id(), ParentSlot::Tuple));
                }
            }
            _ => {}
        }
    }
    None
}

fn replace_child_in_slot(
    parent: &mut crate::structure::StructureNode,
    slot: &ParentSlot,
    old_child: NodeId,
    new_child: NodeId,
) {
    match (slot, parent.kind()) {
        (ParentSlot::Sequence, NodeKind::Sequence { children }) => {
            let new_children: Vec<_> = children.iter()
                .map(|&id| if id == old_child { new_child } else { id })
                .collect();
            parent.set_kind(NodeKind::Sequence { children: new_children });
        }
        (ParentSlot::SectionBody, NodeKind::Section { header, body }) => {
            let new_body: Vec<_> = body.iter()
                .map(|&id| if id == old_child { new_child } else { id })
                .collect();
            parent.set_kind(NodeKind::Section { header: *header, body: new_body });
        }
        (ParentSlot::RepeatBody, NodeKind::Repeat { count, index_var, body }) => {
            let new_body: Vec<_> = body.iter()
                .map(|&id| if id == old_child { new_child } else { id })
                .collect();
            parent.set_kind(NodeKind::Repeat {
                count: count.clone(),
                index_var: index_var.clone(),
                body: new_body,
            });
        }
        _ => {}
    }
}
```

- [ ] **Step 5: Dispatch new actions in engine.rs**

In `crates/cp-ast-core/src/operation/engine.rs`, add to `apply()`:

```rust
Action::AddSibling { target, element } => self.add_sibling(*target, element),
Action::AddChoiceVariant {
    choice,
    tag_value,
    first_element,
} => self.add_choice_variant(*choice, tag_value, first_element),
```

- [ ] **Step 6: Make fill_hole helper function pub(crate)**

In `crates/cp-ast-core/src/operation/fill_hole.rs`, extract `var_type_to_expected` as pub(crate):

```rust
pub(crate) fn var_type_to_expected_from_fill(fill: &FillContent) -> Option<ExpectedType> {
    var_type_to_expected(fill)
}
```

- [ ] **Step 7: Run tests**

Run: `cargo test --test add_sibling && cargo test --test fill_content_templates && cargo test --all-targets`
Expected: PASS

- [ ] **Step 8: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

- [ ] **Step 9: Commit**

```bash
git add crates/cp-ast-core/src/operation/
git add crates/cp-ast-core/tests/add_sibling.rs
git commit -m "feat(core): add AddSibling and AddChoiceVariant actions

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 3: Add FullProjection Types

**Files:**
- Modify: `crates/cp-ast-core/src/projection/types.rs`
- Test: existing tests + compilation check

- [ ] **Step 1: Add FullProjection and supporting types**

In `crates/cp-ast-core/src/projection/types.rs`, add:

```rust
use serde::Serialize;

/// Rich projection of the entire AST for the editor UI.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FullProjection {
    pub nodes: Vec<ProjectedNode>,
    pub hotspots: Vec<Hotspot>,
    pub constraints: ProjectedConstraints,
    pub available_vars: Vec<ExprCandidate>,
    pub completeness: CompletenessSummary,
}

/// Projected constraints split into draft and completed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedConstraints {
    pub drafts: Vec<DraftConstraint>,
    pub completed: Vec<CompletedConstraint>,
}

/// An insertion point in the UI.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hotspot {
    pub parent_id: NodeId,
    pub direction: HotspotDirection,
    pub candidates: Vec<String>,
}

/// Direction of a hotspot insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HotspotDirection {
    Below,
    Right,
    Inside,
    Variant,
}

/// An unfilled constraint generated on-the-fly by projection.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DraftConstraint {
    pub index: usize,
    pub target_id: NodeId,
    pub target_name: String,
    pub display: String,
    pub template: String,
}

/// A fully specified constraint from the AST.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletedConstraint {
    pub index: usize,
    pub constraint_id: String,
    pub display: String,
}

/// A variable available for use in expressions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExprCandidate {
    pub name: String,
    pub node_id: NodeId,
}

/// Detailed candidate info for hole filling.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HoleCandidateDetail {
    pub kind: String,
    pub label: String,
    pub fields: Vec<CandidateField>,
}

/// A field required to complete a candidate fill.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CandidateField {
    pub name: String,
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}
```

Also add `Serialize` derive to `ProjectedNode` and `CompletenessSummary`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedNode { ... }  // existing fields unchanged

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletenessSummary { ... }  // existing fields unchanged
```

Add the `serde` dependency if not already present in `cp-ast-core/Cargo.toml`:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build --all-targets`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/cp-ast-core/src/projection/types.rs crates/cp-ast-core/Cargo.toml
git commit -m "feat(core): add FullProjection types for editor UI

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 4: Implement project_full()

**Files:**
- Create: `crates/cp-ast-core/src/projection/full_projection.rs`
- Modify: `crates/cp-ast-core/src/projection/mod.rs`
- Test: `crates/cp-ast-core/tests/full_projection.rs` (create)

- [ ] **Step 1: Write failing tests**

Create `crates/cp-ast-core/tests/full_projection.rs`:

```rust
use cp_ast_core::operation::engine::AstEngine;
use cp_ast_core::operation::types::{FillContent, LengthSpec, VarType};
use cp_ast_core::operation::action::Action;
use cp_ast_core::projection::types::{FullProjection, HotspotDirection};

#[test]
fn empty_engine_has_below_hotspot() {
    let engine = AstEngine::new();
    let proj = cp_ast_core::projection::project_full(&engine);
    assert!(!proj.hotspots.is_empty(), "Empty engine should have a below hotspot");
    assert_eq!(proj.hotspots[0].direction, HotspotDirection::Below);
}

#[test]
fn scalar_generates_draft_range() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    assert_eq!(proj.constraints.drafts.len(), 1, "Scalar Int should generate 1 draft Range");
    assert!(proj.constraints.drafts[0].display.contains("N"));
}

#[test]
fn scalar_has_right_hotspot() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    let right = proj.hotspots.iter().find(|h| h.direction == HotspotDirection::Right);
    assert!(right.is_some(), "Scalar in Sequence should have Right hotspot");
}

#[test]
fn available_vars_includes_scalar() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    assert!(proj.available_vars.iter().any(|v| v.name == "N"));
}

#[test]
fn existing_range_suppresses_draft() {
    use cp_ast_core::operation::types::{ConstraintDef, ConstraintDefKind};

    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine.apply(&Action::AddSlotElement {
        parent: root,
        slot_name: "children".to_owned(),
        element: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    }).unwrap();

    let n_id = cp_ast_core::structure::NodeId::from_raw(1);
    engine.apply(&Action::AddConstraint {
        target: n_id,
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Range {
                lower: "1".to_owned(),
                upper: "100".to_owned(),
            },
        },
    }).unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    // Draft should be empty since real Range exists
    assert_eq!(proj.constraints.drafts.len(), 0);
    assert_eq!(proj.constraints.completed.len(), 1);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test full_projection`
Expected: FAIL — `cp_ast_core::projection::project_full` not found

- [ ] **Step 3: Implement project_full()**

Create `crates/cp-ast-core/src/projection/full_projection.rs`:

```rust
//! Full projection for the editor UI.
//!
//! Generates a complete UI-ready view of the AST including nodes,
//! hotspots, draft/completed constraints, and available variables.

use super::api::ProjectionAPI;
use super::types::{
    CompletedConstraint, DraftConstraint, ExprCandidate, FullProjection, Hotspot,
    HotspotDirection, ProjectedConstraints,
};
use crate::constraint::{Constraint, ExpectedType};
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind, Reference};

/// Generate a full projection of the AST for the editor UI.
///
/// This is the main "View" function in the TEA architecture.
/// It produces everything the frontend needs to render, including:
/// - Projected nodes (display tree)
/// - Hotspots (insertion points with candidates)
/// - Draft constraints (unfilled, generated on-the-fly)
/// - Completed constraints (existing in the AST)
/// - Available variables (for expression inputs)
/// - Completeness summary
#[must_use]
pub fn project_full(engine: &AstEngine) -> FullProjection {
    let nodes = engine.nodes();
    let hotspots = generate_hotspots(engine);
    let constraints = generate_constraints(engine);
    let available_vars = collect_available_vars(engine);
    let completeness = engine.completeness();

    FullProjection {
        nodes,
        hotspots,
        constraints,
        available_vars,
        completeness,
    }
}

fn generate_hotspots(engine: &AstEngine) -> Vec<Hotspot> {
    let mut hotspots = Vec::new();
    let all_candidates = vec![
        "scalar".to_owned(),
        "array".to_owned(),
        "grid-template".to_owned(),
        "edge-list".to_owned(),
        "weighted-edge-list".to_owned(),
        "query-list".to_owned(),
        "multi-testcase".to_owned(),
    ];
    let scalar_only = vec!["scalar".to_owned()];

    for node in engine.structure.iter() {
        match node.kind() {
            NodeKind::Sequence { children } => {
                // Below hotspot after last child (or if empty)
                hotspots.push(Hotspot {
                    parent_id: node.id(),
                    direction: HotspotDirection::Below,
                    candidates: all_candidates.clone(),
                });

                // Right hotspot for each Scalar/Array direct child
                for &child_id in children {
                    if let Some(child) = engine.structure.get(child_id) {
                        match child.kind() {
                            NodeKind::Scalar { .. } | NodeKind::Array { .. } => {
                                hotspots.push(Hotspot {
                                    parent_id: child_id,
                                    direction: HotspotDirection::Right,
                                    candidates: scalar_only.clone(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            NodeKind::Repeat { body, .. } => {
                // Check if body contains a Hole → Inside hotspot
                let has_hole = body.iter().any(|&id| {
                    engine
                        .structure
                        .get(id)
                        .is_some_and(|n| matches!(n.kind(), NodeKind::Hole { .. }))
                });
                if has_hole {
                    hotspots.push(Hotspot {
                        parent_id: node.id(),
                        direction: HotspotDirection::Inside,
                        candidates: all_candidates.clone(),
                    });
                } else {
                    // Below hotspot inside repeat body
                    hotspots.push(Hotspot {
                        parent_id: node.id(),
                        direction: HotspotDirection::Below,
                        candidates: all_candidates.clone(),
                    });

                    // Right hotspots for direct children
                    for &child_id in body {
                        if let Some(child) = engine.structure.get(child_id) {
                            match child.kind() {
                                NodeKind::Scalar { .. } | NodeKind::Array { .. } => {
                                    hotspots.push(Hotspot {
                                        parent_id: child_id,
                                        direction: HotspotDirection::Right,
                                        candidates: scalar_only.clone(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            NodeKind::Choice { .. } => {
                hotspots.push(Hotspot {
                    parent_id: node.id(),
                    direction: HotspotDirection::Variant,
                    candidates: vec![],
                });
            }
            _ => {}
        }
    }

    hotspots
}

fn generate_constraints(engine: &AstEngine) -> ProjectedConstraints {
    let mut drafts = Vec::new();
    let mut completed = Vec::new();

    // Collect existing constraints as completed
    let mut nodes_with_range: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
    let mut nodes_with_charset: std::collections::HashSet<NodeId> = std::collections::HashSet::new();

    let mut completed_idx = 0usize;
    for c_entry in engine.constraints.iter() {
        let display = format_constraint_display(c_entry.1);
        completed.push(CompletedConstraint {
            index: completed_idx,
            constraint_id: c_entry.0.value().to_string(),
            display,
        });
        completed_idx += 1;

        // Track which nodes already have Range/CharSet
        match c_entry.1 {
            Constraint::Range { target, .. } => {
                if let Reference::VariableRef(id) = target {
                    nodes_with_range.insert(*id);
                }
            }
            Constraint::CharSet { target, .. } => {
                if let Reference::VariableRef(id) = target {
                    nodes_with_charset.insert(*id);
                }
            }
            _ => {}
        }
    }

    // Generate drafts for nodes missing constraints
    let mut draft_idx = 0usize;
    for node in engine.structure.iter() {
        let node_id = node.id();
        match node.kind() {
            NodeKind::Scalar { name } => {
                // Check if this scalar has Int type (via TypeDecl constraint)
                let is_int = is_int_typed(engine, node_id);
                if is_int && !nodes_with_range.contains(&node_id) {
                    drafts.push(DraftConstraint {
                        index: draft_idx,
                        target_id: node_id,
                        target_name: name.as_str().to_owned(),
                        display: format!("? \\le {} \\le ?", name.as_str()),
                        template: "range".to_owned(),
                    });
                    draft_idx += 1;
                }
            }
            NodeKind::Array { name, .. } => {
                let is_int = is_int_typed(engine, node_id);
                if is_int && !nodes_with_range.contains(&node_id) {
                    drafts.push(DraftConstraint {
                        index: draft_idx,
                        target_id: node_id,
                        target_name: format!("{}_i", name.as_str()),
                        display: format!("? \\le {}_i \\le ?", name.as_str()),
                        template: "range".to_owned(),
                    });
                    draft_idx += 1;
                }
            }
            NodeKind::Matrix { name, .. } => {
                if !nodes_with_charset.contains(&node_id) {
                    let is_str_or_char = is_str_typed(engine, node_id) || is_char_typed(engine, node_id);
                    if is_str_or_char {
                        drafts.push(DraftConstraint {
                            index: draft_idx,
                            target_id: node_id,
                            target_name: name.as_str().to_owned(),
                            display: format!("charset({}) = ?", name.as_str()),
                            template: "charset".to_owned(),
                        });
                        draft_idx += 1;
                    }
                }
            }
            _ => {}
        }
    }

    ProjectedConstraints { drafts, completed }
}

fn is_int_typed(engine: &AstEngine, node_id: NodeId) -> bool {
    let constraints = engine.constraints.for_node(node_id);
    for &cid in &constraints {
        if let Some(Constraint::TypeDecl { expected, .. }) = engine.constraints.get(cid) {
            return *expected == ExpectedType::Int;
        }
    }
    false
}

fn is_str_typed(engine: &AstEngine, node_id: NodeId) -> bool {
    let constraints = engine.constraints.for_node(node_id);
    for &cid in &constraints {
        if let Some(Constraint::TypeDecl { expected, .. }) = engine.constraints.get(cid) {
            return *expected == ExpectedType::Str;
        }
    }
    false
}

fn is_char_typed(engine: &AstEngine, node_id: NodeId) -> bool {
    let constraints = engine.constraints.for_node(node_id);
    for &cid in &constraints {
        if let Some(Constraint::TypeDecl { expected, .. }) = engine.constraints.get(cid) {
            return *expected == ExpectedType::Char;
        }
    }
    false
}

fn collect_available_vars(engine: &AstEngine) -> Vec<ExprCandidate> {
    let mut vars = Vec::new();
    for node in engine.structure.iter() {
        match node.kind() {
            NodeKind::Scalar { name } => {
                vars.push(ExprCandidate {
                    name: name.as_str().to_owned(),
                    node_id: node.id(),
                });
            }
            NodeKind::Array { name, .. } | NodeKind::Matrix { name, .. } => {
                vars.push(ExprCandidate {
                    name: name.as_str().to_owned(),
                    node_id: node.id(),
                });
            }
            _ => {}
        }
    }
    vars
}

fn format_constraint_display(c: &Constraint) -> String {
    match c {
        Constraint::Range { target, lower, upper } => {
            let name = ref_to_name(target);
            format!("{} \\le {} \\le {}", format_expr_display(lower), name, format_expr_display(upper))
        }
        Constraint::TypeDecl { target, expected } => {
            format!("{}: {:?}", ref_to_name(target), expected)
        }
        Constraint::Property { tag, .. } => {
            format!("{:?}", tag)
        }
        Constraint::SumBound { variable, upper } => {
            format!("\\sum {} \\le {}", ref_to_name(variable), format_expr_display(upper))
        }
        Constraint::CharSet { target, charset } => {
            format!("charset({}) = {}", ref_to_name(target), charset)
        }
        _ => format!("{c:?}"),
    }
}

fn ref_to_name(r: &Reference) -> String {
    match r {
        Reference::VariableRef(_id) => "var".to_owned(), // Will be enhanced
        Reference::IndexedRef { target, .. } => format!("{target:?}"),
        Reference::Unresolved(name) => name.as_str().to_owned(),
    }
}

fn format_expr_display(e: &crate::constraint::Expression) -> String {
    use crate::constraint::Expression;
    match e {
        Expression::Lit(n) => n.to_string(),
        Expression::Var(r) => ref_to_name(r),
        Expression::BinOp { op, lhs, rhs } => {
            format!("{} {:?} {}", format_expr_display(lhs), op, format_expr_display(rhs))
        }
        Expression::Pow { base, exp } => {
            format!("{}^{{{}}}", format_expr_display(base), format_expr_display(exp))
        }
        Expression::FnCall { name, args } => {
            let args_str: Vec<_> = args.iter().map(format_expr_display).collect();
            format!("{}({})", name.as_str(), args_str.join(", "))
        }
    }
}
```

- [ ] **Step 4: Wire up module**

In `crates/cp-ast-core/src/projection/mod.rs`, add:

```rust
pub mod full_projection;

pub use full_projection::project_full;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test full_projection && cargo test --all-targets`
Expected: PASS

- [ ] **Step 6: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/cp-ast-core/src/projection/
git add crates/cp-ast-core/tests/full_projection.rs
git commit -m "feat(core): implement project_full() with hotspots and draft constraints

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 5: Add Action + Projection DTOs to cp-ast-json

**Files:**
- Modify: `crates/cp-ast-json/src/dto.rs`
- Create: `crates/cp-ast-json/src/action_dto.rs`
- Create: `crates/cp-ast-json/src/projection_dto.rs`
- Modify: `crates/cp-ast-json/src/lib.rs`
- Test: `crates/cp-ast-json/tests/action_roundtrip.rs` (create)

- [ ] **Step 1: Write failing tests for Action deserialization**

Create `crates/cp-ast-json/tests/action_roundtrip.rs`:

```rust
use cp_ast_json::{deserialize_action, serialize_action};
use cp_ast_core::operation::action::Action;
use cp_ast_core::operation::types::{FillContent, VarType, LengthSpec, ConstraintDef, ConstraintDefKind};
use cp_ast_core::structure::NodeId;

#[test]
fn fill_hole_scalar_roundtrip() {
    let action = Action::FillHole {
        target: NodeId::from_raw(5),
        fill: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_slot_element_array_roundtrip() {
    let action = Action::AddSlotElement {
        parent: NodeId::from_raw(0),
        slot_name: "children".to_owned(),
        element: FillContent::Array {
            name: "A".to_owned(),
            element_type: VarType::Int,
            length: LengthSpec::RefVar(NodeId::from_raw(1)),
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_sibling_roundtrip() {
    let action = Action::AddSibling {
        target: NodeId::from_raw(1),
        element: FillContent::Scalar {
            name: "W".to_owned(),
            typ: VarType::Int,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_constraint_range_roundtrip() {
    let action = Action::AddConstraint {
        target: NodeId::from_raw(1),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Range {
                lower: "1".to_owned(),
                upper: "100000".to_owned(),
            },
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn edge_list_roundtrip() {
    let action = Action::AddSlotElement {
        parent: NodeId::from_raw(0),
        slot_name: "children".to_owned(),
        element: FillContent::EdgeList {
            edge_count: LengthSpec::Expr("N-1".to_owned()),
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_choice_variant_roundtrip() {
    use cp_ast_core::structure::Literal;

    let action = Action::AddChoiceVariant {
        choice: NodeId::from_raw(5),
        tag_value: Literal::Int(1),
        first_element: FillContent::Scalar {
            name: "a".to_owned(),
            typ: VarType::Int,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test action_roundtrip -p cp-ast-json`
Expected: FAIL — `serialize_action` / `deserialize_action` not found

- [ ] **Step 3: Add ActionDto types to dto.rs**

In `crates/cp-ast-json/src/dto.rs`, add:

```rust
// ── Actions ─────────────────────────────────────────────────────────

/// Discriminated union for editor actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ActionDto {
    FillHole {
        target: String,
        fill: FillContentDto,
    },
    ReplaceNode {
        target: String,
        replacement: FillContentDto,
    },
    AddConstraint {
        target: String,
        constraint: ConstraintDefDto,
    },
    RemoveConstraint {
        constraint_id: String,
    },
    IntroduceMultiTestCase {
        count_var_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sum_bound: Option<SumBoundDefDto>,
    },
    AddSlotElement {
        parent: String,
        slot_name: String,
        element: FillContentDto,
    },
    RemoveSlotElement {
        parent: String,
        slot_name: String,
        child: String,
    },
    AddSibling {
        target: String,
        element: FillContentDto,
    },
    AddChoiceVariant {
        choice: String,
        tag_value: LiteralDto,
        first_element: FillContentDto,
    },
}

/// Fill content for creating nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum FillContentDto {
    Scalar { name: String, typ: String },
    Array { name: String, element_type: String, length: LengthSpecDto },
    Grid { name: String, rows: LengthSpecDto, cols: LengthSpecDto, cell_type: String },
    Section { label: String },
    OutputSingleValue { typ: String },
    OutputYesNo,
    EdgeList { edge_count: LengthSpecDto },
    WeightedEdgeList { edge_count: LengthSpecDto, weight_name: String, weight_type: String },
    QueryList { query_count: LengthSpecDto },
    MultiTestCaseTemplate { count: LengthSpecDto },
    GridTemplate { name: String, rows: LengthSpecDto, cols: LengthSpecDto, cell_type: String },
}

/// Length specification DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LengthSpecDto {
    Fixed { value: usize },
    RefVar { node_id: String },
    Expr { expr: String },
}

/// Constraint definition DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConstraintDefDto {
    Range { lower: String, upper: String },
    TypeDecl { typ: String },
    Relation { op: String, rhs: String },
    Distinct,
    Sorted { order: String },
    Property { tag: String },
    SumBound { over_var: String, upper: String },
    Guarantee { description: String },
}

/// SumBound definition DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SumBoundDefDto {
    pub bound_var: String,
    pub upper: String,
}

// ── FullProjection DTOs ─────────────────────────────────────────────

/// Full projection DTO for JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullProjectionDto {
    pub nodes: Vec<ProjectedNodeDto>,
    pub hotspots: Vec<HotspotDto>,
    pub constraints: ProjectedConstraintsDto,
    pub available_vars: Vec<ExprCandidateDto>,
    pub completeness: CompletenessSummaryDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedNodeDto {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub depth: usize,
    pub is_hole: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotDto {
    pub parent_id: String,
    pub direction: String,
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedConstraintsDto {
    pub drafts: Vec<DraftConstraintDto>,
    pub completed: Vec<CompletedConstraintDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftConstraintDto {
    pub index: usize,
    pub target_id: String,
    pub target_name: String,
    pub display: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedConstraintDto {
    pub index: usize,
    pub constraint_id: String,
    pub display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprCandidateDto {
    pub name: String,
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessSummaryDto {
    pub total_holes: usize,
    pub filled_slots: usize,
    pub unsatisfied_constraints: usize,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoleCandidateDetailDto {
    pub kind: String,
    pub label: String,
    pub fields: Vec<CandidateFieldDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateFieldDto {
    pub name: String,
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}
```

- [ ] **Step 4: Implement Action ↔ DTO conversion**

Create `crates/cp-ast-json/src/action_dto.rs`:

This file implements bidirectional conversion between `Action` and `ActionDto`, `FillContent` and `FillContentDto`, etc. Following the same patterns as `to_dto.rs` and `from_dto.rs`.

Key conversion functions:
- `action_to_dto(action: &Action) -> ActionDto`
- `dto_to_action(dto: &ActionDto) -> Result<Action, ConversionError>`
- Helper functions for FillContent, LengthSpec, ConstraintDef, SumBoundDef

- [ ] **Step 5: Implement FullProjection → DTO conversion**

Create `crates/cp-ast-json/src/projection_dto.rs`:

One-way conversion (serialize only) from `FullProjection` to `FullProjectionDto`.

Key function: `projection_to_dto(proj: &FullProjection) -> FullProjectionDto`

- [ ] **Step 6: Wire up in lib.rs**

In `crates/cp-ast-json/src/lib.rs`, add:

```rust
mod action_dto;
mod projection_dto;

pub use action_dto::{serialize_action, deserialize_action};
pub use projection_dto::serialize_projection;
```

- [ ] **Step 7: Run tests**

Run: `cargo test --test action_roundtrip -p cp-ast-json && cargo test --all-targets`
Expected: PASS

- [ ] **Step 8: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

- [ ] **Step 9: Commit**

```bash
git add crates/cp-ast-json/
git commit -m "feat(json): add Action and FullProjection DTOs with conversion

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 6: Add WASM Editor Functions

**Files:**
- Modify: `crates/cp-ast-wasm/src/lib.rs`
- Modify: `crates/cp-ast-wasm/Cargo.toml` (if needed)
- Test: compilation + manual testing via web

- [ ] **Step 1: Add 6 new wasm_bindgen functions**

In `crates/cp-ast-wasm/src/lib.rs`, add:

```rust
/// Creates a new empty document as JSON.
#[wasm_bindgen]
pub fn new_document() -> Result<String, JsError> {
    let engine = cp_ast_core::operation::AstEngine::new();
    serialize(&engine)
}

/// Returns a full UI projection of the document as JSON.
#[wasm_bindgen]
pub fn project_full(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let projection = cp_ast_core::projection::project_full(&engine);
    cp_ast_json::serialize_projection(&projection)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Applies an action to the document, returning the new document JSON.
#[wasm_bindgen]
pub fn apply_action(document_json: &str, action_json: &str) -> Result<String, JsError> {
    let mut engine = deserialize(document_json)?;
    let action = cp_ast_json::deserialize_action(action_json)
        .map_err(|e| JsError::new(&e.to_string()))?;
    engine
        .apply(&action)
        .map_err(|e| JsError::new(&format!("{e:?}")))?;
    serialize(&engine)
}

/// Returns hole candidates for a specific hole node.
#[wasm_bindgen]
pub fn get_hole_candidates(document_json: &str, hole_id: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let node_id = hole_id
        .parse::<u64>()
        .map(cp_ast_core::structure::NodeId::from_raw)
        .map_err(|_| JsError::new(&format!("Invalid node ID: {hole_id}")))?;

    let candidates = cp_ast_core::projection::generate_hole_candidates(&engine, node_id);
    serde_json::to_string(&candidates).map_err(|e| JsError::new(&e.to_string()))
}

/// Returns available variables for expression input.
#[wasm_bindgen]
pub fn get_expr_candidates(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let projection = cp_ast_core::projection::project_full(&engine);
    serde_json::to_string(&projection.available_vars)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Returns nodes that can be targets for property/sumbound constraints.
#[wasm_bindgen]
pub fn get_constraint_targets(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let projection = cp_ast_core::projection::project_full(&engine);
    // Return available_vars as constraint targets
    serde_json::to_string(&projection.available_vars)
        .map_err(|e| JsError::new(&e.to_string()))
}
```

Also add a `serialize` helper:

```rust
fn serialize(engine: &cp_ast_core::operation::AstEngine) -> Result<String, JsError> {
    cp_ast_json::serialize_ast(engine).map_err(|e| JsError::new(&e.to_string()))
}
```

- [ ] **Step 2: Build WASM**

Run: `cargo build -p cp-ast-wasm --target wasm32-unknown-unknown`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/cp-ast-wasm/
git commit -m "feat(wasm): add 6 editor WASM functions (new_document, project_full, apply_action, etc.)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 7: Frontend Tooling Setup

**Files:**
- Modify: `web/package.json`
- Create: `web/vitest.config.ts`
- Modify: `web/tsconfig.json` (if needed for vitest types)

- [ ] **Step 1: Install frontend dependencies**

```bash
cd web
npm install -D vitest @testing-library/preact @testing-library/jest-dom jsdom happy-dom
```

- [ ] **Step 2: Create vitest.config.ts**

Create `web/vitest.config.ts`:

```typescript
import { defineConfig } from 'vitest/config';
import preact from '@preact/preset-vite';

export default defineConfig({
  plugins: [preact()],
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['tests/unit/**/*.test.{ts,tsx}'],
  },
});
```

- [ ] **Step 3: Add test:unit script to package.json**

In `web/package.json`, add to scripts:

```json
"test:unit": "vitest run"
```

- [ ] **Step 4: Verify vitest runs (empty suite)**

```bash
cd web && npm run test:unit
```
Expected: 0 tests pass (no tests yet)

- [ ] **Step 5: Commit**

```bash
git add web/package.json web/package-lock.json web/vitest.config.ts
git commit -m "chore(web): add vitest + @testing-library/preact for unit tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 8: Build WASM and Update Frontend Exports

**Files:**
- Modify: `web/src/wasm.ts`
- Build: `wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm`

- [ ] **Step 1: Build WASM**

```bash
wasm-pack build crates/cp-ast-wasm --target web --out-dir ../../web/wasm
```
Expected: BUILD PASS

- [ ] **Step 2: Update wasm.ts with new exports**

In `web/src/wasm.ts`, add the 6 new functions:

```typescript
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
  new_document,
  project_full,
  apply_action,
  get_hole_candidates,
  get_expr_candidates,
  get_constraint_targets,
} from '../wasm/cp_ast_wasm';
```

- [ ] **Step 3: Verify TypeScript compilation**

```bash
cd web && npx tsc --noEmit
```
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add web/src/wasm.ts web/wasm/
git commit -m "build: update WASM bundle and export 6 new editor functions

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 9: Editor State Management

**Files:**
- Create: `web/src/editor/editor-state.ts`
- Create: `web/src/editor/popup-state.ts`
- Create: `web/src/editor/action-builder.ts`
- Test: `web/tests/unit/action-builder.test.ts` (create)

- [ ] **Step 1: Create editor-state.ts**

Create `web/src/editor/editor-state.ts`:

```typescript
import { signal, computed } from '@preact/signals';
import {
  new_document,
  project_full,
  apply_action,
  render_input_tex,
  render_constraints_tex,
  generate_sample,
} from '../wasm';

export const documentJson = signal<string>('');
export const sampleSeed = signal<number>(42);

function safeCall<T>(fn: () => T, fallback: T): T {
  try {
    return fn();
  } catch (e) {
    console.error(e);
    return fallback;
  }
}

export const projection = computed(() => {
  if (!documentJson.value) return null;
  return safeCall(
    () => JSON.parse(project_full(documentJson.value)),
    null,
  );
});

export const texInput = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(() => render_input_tex(documentJson.value), '');
});

export const texConstraints = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(() => render_constraints_tex(documentJson.value), '');
});

export const sampleOutput = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(
    () => generate_sample(documentJson.value, sampleSeed.value),
    '',
  );
});

export function dispatchAction(actionJson: string): void {
  try {
    documentJson.value = apply_action(documentJson.value, actionJson);
  } catch (e) {
    console.error('Action failed:', e);
  }
}

export function initEditor(): void {
  try {
    documentJson.value = new_document();
  } catch (e) {
    console.error('Failed to init editor:', e);
  }
}
```

- [ ] **Step 2: Create popup-state.ts**

Create `web/src/editor/popup-state.ts`:

```typescript
import { signal } from '@preact/signals';

export type PopupMode =
  | { type: 'closed' }
  | { type: 'node'; parentId: string; direction: string; candidates: string[] }
  | { type: 'constraint'; draftIndex: number; targetId: string; bound: 'lower' | 'upper' | null }
  | { type: 'charset'; draftIndex: number; targetId: string }
  | { type: 'property' }
  | { type: 'sumbound' };

export const popupMode = signal<PopupMode>({ type: 'closed' });

// Node creation wizard state
export const selectedCandidate = signal<string | null>(null);
export const nameValue = signal<string>('');
export const typeValue = signal<string>('number');
export const lengthValues = signal<string[]>([]);

// Expression builder state
export const exprBase = signal<string | null>(null);
export const exprOp = signal<string | null>(null);
export const exprOperand = signal<string>('');

// Constraint editor state
export const lowerExpr = signal<any>(null);
export const upperExpr = signal<any>(null);

// Weight name (for weighted edge list)
export const weightName = signal<string>('w');

// Variant tag
export const variantTag = signal<string>('');

export function resetPopup(): void {
  popupMode.value = { type: 'closed' };
  selectedCandidate.value = null;
  nameValue.value = '';
  typeValue.value = 'number';
  lengthValues.value = [];
  exprBase.value = null;
  exprOp.value = null;
  exprOperand.value = '';
  lowerExpr.value = null;
  upperExpr.value = null;
  weightName.value = 'w';
  variantTag.value = '';
}
```

- [ ] **Step 3: Create action-builder.ts**

Create `web/src/editor/action-builder.ts`:

```typescript
/**
 * Build Action JSON objects for WASM dispatch.
 * This module converts UI-level events into Action DTOs.
 */

export interface ExpressionDto {
  kind: string;
  value?: number;
  reference?: { kind: string; node_id?: string; name?: string };
  op?: string;
  lhs?: ExpressionDto;
  rhs?: ExpressionDto;
  base?: ExpressionDto;
  exp?: ExpressionDto;
}

export function buildFillHole(
  target: string,
  fill: Record<string, unknown>,
): string {
  return JSON.stringify({ action: 'FillHole', target, fill });
}

export function buildAddSlotElement(
  parent: string,
  slotName: string,
  element: Record<string, unknown>,
): string {
  return JSON.stringify({
    action: 'AddSlotElement',
    parent,
    slot_name: slotName,
    element,
  });
}

export function buildAddSibling(
  target: string,
  element: Record<string, unknown>,
): string {
  return JSON.stringify({ action: 'AddSibling', target, element });
}

export function buildAddChoiceVariant(
  choice: string,
  tagValue: number,
  firstName: string,
  firstType: string,
): string {
  return JSON.stringify({
    action: 'AddChoiceVariant',
    choice,
    tag_value: { kind: 'IntLit', value: tagValue },
    first_element: { kind: 'Scalar', name: firstName, typ: firstType },
  });
}

export function buildAddConstraint(
  target: string,
  constraintDef: Record<string, unknown>,
): string {
  return JSON.stringify({
    action: 'AddConstraint',
    target,
    constraint: constraintDef,
  });
}

export function buildScalarFill(
  name: string,
  typ: string = 'Int',
): Record<string, unknown> {
  return { kind: 'Scalar', name, typ };
}

export function buildArrayFill(
  name: string,
  lengthNodeId: string,
  elementType: string = 'Int',
): Record<string, unknown> {
  return {
    kind: 'Array',
    name,
    element_type: elementType,
    length: { kind: 'RefVar', node_id: lengthNodeId },
  };
}

export function buildEdgeListFill(
  countSpec: Record<string, unknown>,
): Record<string, unknown> {
  return { kind: 'EdgeList', edge_count: countSpec };
}

export function buildWeightedEdgeListFill(
  countSpec: Record<string, unknown>,
  weightNameStr: string,
  weightType: string = 'Int',
): Record<string, unknown> {
  return {
    kind: 'WeightedEdgeList',
    edge_count: countSpec,
    weight_name: weightNameStr,
    weight_type: weightType,
  };
}

export function buildQueryListFill(
  countSpec: Record<string, unknown>,
): Record<string, unknown> {
  return { kind: 'QueryList', query_count: countSpec };
}

export function buildMultiTestCaseFill(
  countSpec: Record<string, unknown>,
): Record<string, unknown> {
  return { kind: 'MultiTestCaseTemplate', count: countSpec };
}

export function buildGridTemplateFill(
  name: string,
  rowsSpec: Record<string, unknown>,
  colsSpec: Record<string, unknown>,
  cellType: string = 'Char',
): Record<string, unknown> {
  return {
    kind: 'GridTemplate',
    name,
    rows: rowsSpec,
    cols: colsSpec,
    cell_type: cellType,
  };
}

export function buildRefVarSpec(
  nodeId: string,
): Record<string, unknown> {
  return { kind: 'RefVar', node_id: nodeId };
}

export function buildExprSpec(expr: string): Record<string, unknown> {
  return { kind: 'Expr', expr };
}

export function buildRangeConstraint(
  lower: string,
  upper: string,
): Record<string, unknown> {
  return { kind: 'Range', lower, upper };
}

export function buildPropertyConstraint(
  tag: string,
): Record<string, unknown> {
  return { kind: 'Property', tag };
}

export function buildSumBoundConstraint(
  overVar: string,
  upper: string,
): Record<string, unknown> {
  return { kind: 'SumBound', over_var: overVar, upper };
}
```

- [ ] **Step 4: Write action-builder unit tests**

Create `web/tests/unit/action-builder.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import {
  buildAddSlotElement,
  buildScalarFill,
  buildArrayFill,
  buildAddSibling,
  buildRangeConstraint,
  buildAddConstraint,
  buildEdgeListFill,
  buildRefVarSpec,
} from '../../src/editor/action-builder';

describe('action-builder', () => {
  it('builds FillHole for scalar', () => {
    const fill = buildScalarFill('N', 'Int');
    expect(fill).toEqual({ kind: 'Scalar', name: 'N', typ: 'Int' });
  });

  it('builds AddSlotElement for array', () => {
    const json = buildAddSlotElement(
      '0',
      'children',
      buildArrayFill('A', '1', 'Int'),
    );
    const parsed = JSON.parse(json);
    expect(parsed.action).toBe('AddSlotElement');
    expect(parsed.element.kind).toBe('Array');
    expect(parsed.element.length.node_id).toBe('1');
  });

  it('builds AddSibling', () => {
    const json = buildAddSibling('1', buildScalarFill('W'));
    const parsed = JSON.parse(json);
    expect(parsed.action).toBe('AddSibling');
    expect(parsed.target).toBe('1');
  });

  it('builds AddConstraint with range', () => {
    const json = buildAddConstraint('1', buildRangeConstraint('1', '100'));
    const parsed = JSON.parse(json);
    expect(parsed.action).toBe('AddConstraint');
    expect(parsed.constraint.kind).toBe('Range');
  });

  it('builds EdgeList fill', () => {
    const fill = buildEdgeListFill(buildRefVarSpec('1'));
    expect(fill.kind).toBe('EdgeList');
  });
});
```

- [ ] **Step 5: Run unit tests**

```bash
cd web && npm run test:unit
```
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add web/src/editor/ web/tests/unit/
git commit -m "feat(web): add editor state management, popup state, and action builder

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 10: Editor Page + Structure Pane + Hotspots

**Files:**
- Create: `web/src/editor/EditorPage.tsx`
- Create: `web/src/editor/StructurePane.tsx`
- Modify: `web/src/app.tsx`

- [ ] **Step 1: Create EditorPage.tsx**

Create `web/src/editor/EditorPage.tsx`:

```tsx
import { useEffect } from 'preact/hooks';
import { initEditor, projection, texInput, texConstraints, sampleOutput } from './editor-state';
import { StructurePane } from './StructurePane';
import { ConstraintPane } from './ConstraintPane';
import { PreviewPane } from './PreviewPane';

export function EditorPage() {
  useEffect(() => {
    initEditor();
  }, []);

  return (
    <div class="editor-layout" data-testid="editor-page">
      <StructurePane projection={projection.value} />
      <ConstraintPane projection={projection.value} />
      <PreviewPane
        texInput={texInput.value}
        texConstraints={texConstraints.value}
        sampleOutput={sampleOutput.value}
      />
    </div>
  );
}
```

- [ ] **Step 2: Create StructurePane.tsx**

Create `web/src/editor/StructurePane.tsx` implementing:
- Renders `projection.nodes` with depth-based indentation
- Shows `insertion-hotspot-below`, `insertion-hotspot-right`, `insertion-hotspot-inside`, `insertion-hotspot-variant` based on `projection.hotspots`
- Hotspot clicks open NodePopup by setting `popupMode`
- Includes the NodePopup component

Key data-testid requirements:
- `structure-pane` on the container
- `insertion-hotspot-below`, `insertion-hotspot-right`, `insertion-hotspot-inside`, `insertion-hotspot-variant` on hotspot buttons

- [ ] **Step 3: Update app.tsx routing**

In `web/src/app.tsx`, add EditorPage as the default route:

```tsx
import { currentPage } from './state';
import { ViewerPage } from './components/viewer/ViewerPage';
import { PreviewPage } from './components/preview/PreviewPage';
import { EditorPage } from './editor/EditorPage';

export function App() {
  return (
    <div class="app">
      <header class="header">
        <h1 class="header-title">🌳 AST Editor</h1>
        <nav class="header-nav">
          <a
            href="#/"
            class={`nav-link ${currentPage.value === 'editor' ? 'active' : ''}`}
          >
            Editor
          </a>
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
        {currentPage.value === 'editor' && <EditorPage />}
        {currentPage.value === 'viewer' && <ViewerPage />}
        {currentPage.value === 'preview' && <PreviewPage />}
      </main>
    </div>
  );
}
```

Update `web/src/state.ts` routing to include `'editor'`:

```typescript
export const currentPage = signal<'editor' | 'viewer' | 'preview'>(
  window.location.hash === '#/preview'
    ? 'preview'
    : window.location.hash === '#/viewer'
      ? 'viewer'
      : 'editor',
);

window.addEventListener('hashchange', () => {
  if (window.location.hash === '#/preview') {
    currentPage.value = 'preview';
  } else if (window.location.hash === '#/viewer') {
    currentPage.value = 'viewer';
  } else {
    currentPage.value = 'editor';
  }
});
```

- [ ] **Step 4: Verify dev server starts and shows editor**

```bash
cd web && npm run dev
```
Navigate to localhost:5173 — should see the editor layout.

- [ ] **Step 5: Commit**

```bash
git add web/src/editor/EditorPage.tsx web/src/editor/StructurePane.tsx web/src/app.tsx web/src/state.ts
git commit -m "feat(web): add EditorPage with StructurePane and hotspot rendering

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 11: NodePopup + ExpressionBuilder Components

**Files:**
- Create: `web/src/editor/NodePopup.tsx`
- Create: `web/src/editor/ExpressionBuilder.tsx`

- [ ] **Step 1: Create NodePopup.tsx**

This component handles:
- Popup visibility based on `popupMode.type === 'node'`
- Candidate list rendering (`popup-option-scalar`, `popup-option-array`, etc.)
- Name input (`name-input`)
- Type select (`type-select`)
- Length select (`length-select`) — populated from `available_vars`
- Count field with ExpressionBuilder (`count-field`)
- Weight name input (`weight-name-input`)
- Variant tag input (`variant-tag-input`)
- Confirm button (`confirm-button`)
- On confirm: builds Action JSON via action-builder, dispatches via `dispatchAction()`, resets popup

Key data-testid:
- `node-popup`, `popup-option-{x}`, `name-input`, `type-select`, `length-select`, `weight-name-input`, `confirm-button`, `variant-tag-input`

- [ ] **Step 2: Create ExpressionBuilder.tsx**

This component handles the multi-step expression building:
- `count-field` click → show variable options
- `count-var-option-{var}` → select variable as base
- `expression-element-{var}` → click to show function ops
- `function-op-{op}` → select operation (subtract, add, multiply, divide, power, min, max)
- `function-operand-input` → enter operand, press Enter to confirm

Key data-testid:
- `count-field`, `count-var-option-{x}`, `expression-element-{x}`, `function-op-{x}`, `function-operand-input`

- [ ] **Step 3: Verify popup works with dev server**

Test manually: click hotspot → see popup → select option → fill fields → confirm

- [ ] **Step 4: Commit**

```bash
git add web/src/editor/NodePopup.tsx web/src/editor/ExpressionBuilder.tsx
git commit -m "feat(web): add NodePopup and ExpressionBuilder components

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 12: ConstraintPane + ConstraintEditor + ValueInput

**Files:**
- Create: `web/src/editor/ConstraintPane.tsx`
- Create: `web/src/editor/ConstraintEditor.tsx`
- Create: `web/src/editor/ValueInput.tsx`

- [ ] **Step 1: Create ConstraintPane.tsx**

This component renders:
- Draft constraints list (`draft-constraint-{index}`)
- Completed constraints list (`completed-constraint-{index}`)
- Property shortcut button (`property-shortcut`) → opens property options (`property-option-{name}`)
- SumBound shortcut button (`sumbound-shortcut`) → opens SumBound editor (`sumbound-var-select`, `sumbound-upper-input`)
- CharSet option (`charset-option-lowercase`)

Key data-testid:
- `constraint-pane`, `draft-constraint-{i}`, `completed-constraint-{i}`, `property-shortcut`, `property-option-{x}`, `sumbound-shortcut`, `sumbound-var-select`, `sumbound-upper-input`, `sumbound-upper-expression`, `charset-option-lowercase`

- [ ] **Step 2: Create ConstraintEditor.tsx**

This component handles editing a draft Range constraint:
- Lower bound area (`constraint-lower-input`)
- Upper bound area (`constraint-upper-input`)
- Expression display areas (`constraint-lower-expression`, `constraint-upper-expression`)
- Confirm button (`constraint-confirm`)

When bound is clicked → opens ValueInput popup.
On confirm → builds AddConstraint action → dispatch → close.

- [ ] **Step 3: Create ValueInput.tsx**

This popup component appears when `?` / bound area is clicked:
- Shows available variables as clickable options (`constraint-var-option-{var}`)
- Free integer input field (`constraint-value-literal`)
- After variable selection, clicking the variable element opens function popup (reuses ExpressionBuilder pattern)
- Enter confirms literal input

Key data-testid:
- `constraint-value-literal`, `constraint-var-option-{x}`

- [ ] **Step 4: Verify constraint flow manually**

Test: Add scalar N → see draft → click draft → fill bounds → confirm → see completed

- [ ] **Step 5: Commit**

```bash
git add web/src/editor/ConstraintPane.tsx web/src/editor/ConstraintEditor.tsx web/src/editor/ValueInput.tsx
git commit -m "feat(web): add ConstraintPane, ConstraintEditor, and ValueInput components

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 13: PreviewPane + CSS

**Files:**
- Create: `web/src/editor/PreviewPane.tsx`
- Modify: `web/src/index.css`

- [ ] **Step 1: Create PreviewPane.tsx**

```tsx
import { renderTeX } from '../tex-renderer';

interface PreviewPaneProps {
  texInput: string;
  texConstraints: string;
  sampleOutput: string;
}

export function PreviewPane({ texInput, texConstraints, sampleOutput }: PreviewPaneProps) {
  return (
    <div class="preview-pane" data-testid="preview-pane">
      <section class="preview-section">
        <h3>Input Format</h3>
        <div
          data-testid="tex-input-format"
          dangerouslySetInnerHTML={{ __html: renderTeX(texInput) }}
        />
      </section>
      <section class="preview-section">
        <h3>Constraints</h3>
        <div
          data-testid="tex-constraints"
          dangerouslySetInnerHTML={{ __html: renderTeX(texConstraints) }}
        />
      </section>
      <section class="preview-section">
        <h3>Sample</h3>
        <pre data-testid="sample-output">{sampleOutput}</pre>
      </section>
    </div>
  );
}
```

- [ ] **Step 2: Add editor CSS**

Add to `web/src/index.css`:

```css
.editor-layout {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 1rem;
  height: calc(100vh - 4rem);
  padding: 1rem;
}

/* Structure pane, constraint pane, preview pane styling */
/* Popup styling */
/* Hotspot button styling */
```

- [ ] **Step 3: Verify all three panes render**

```bash
cd web && npm run dev
```
Check that all three panes are visible and populated.

- [ ] **Step 4: Commit**

```bash
git add web/src/editor/PreviewPane.tsx web/src/index.css
git commit -m "feat(web): add PreviewPane and editor CSS layout

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 14: E2E Test Integration — Pass All 26 Tests

**Files:**
- All files from Tasks 1-13 may need refinement
- No E2E test files are modified

- [ ] **Step 1: Run E2E tests and collect failures**

```bash
cd web && npx playwright test --config tests/e2e/playwright.config.ts 2>&1 | head -100
```
Expected: Some tests pass, some fail. Collect failure list.

- [ ] **Step 2: Fix basic-array.spec.ts failures**

The 5 basic-array tests exercise: addScalar, addArray, openDraft, fillBoundLiteral, applyBoundFunction, confirmConstraint, expectRightPanePopulated, expectSampleLines.

Debug and fix any issues with:
- Initial hotspot visibility
- Popup opening/closing flow
- Draft constraint generation
- Constraint editing flow (lower/upper bound input)
- TeX and sample rendering

- [ ] **Step 3: Fix grid.spec.ts failures**

The 4 grid tests exercise: addScalar, addScalarRight (AddSibling), grid-template popup, selectLength (rows/cols), charset editing.

Debug and fix any issues with:
- AddSibling creating Tuple correctly
- GridTemplate fill creating Matrix node
- Charset draft and `charset-option-lowercase` click

- [ ] **Step 4: Fix tree.spec.ts failures**

The 4 tree tests exercise: addScalar, edge-list popup, buildCountExpression (N-1), addProperty('tree').

Debug and fix:
- EdgeList fill creating Repeat[Tuple[u,v]]
- buildCountExpression multi-step flow
- fillBoundVar (variable reference in upper bound)
- Property shortcut and property-option-tree

- [ ] **Step 5: Fix query.spec.ts failures**

The 4 query tests exercise: addScalarRight, query-list popup, insertion-hotspot-variant, AddChoiceVariant with tag input.

Debug and fix:
- QueryList fill creating Repeat[Choice]
- Variant hotspot visibility
- AddChoiceVariant action
- variant-tag-input + scalar creation inside variant body
- addScalarRight inside variant body

- [ ] **Step 6: Fix multi-testcase.spec.ts failures**

The 5 multi-testcase tests exercise: multi-testcase popup, insertion-hotspot-inside, addSumBound, addSumBoundExpression.

Debug and fix:
- MultiTestCaseTemplate fill creating Repeat[Hole]
- Inside hotspot after filling Hole
- SumBound shortcut flow
- SumBound expression builder (multiply operand)

- [ ] **Step 7: Fix graph.spec.ts failures**

The 4 graph tests exercise: edge-list with simple count (selectLength), weighted-edge-list (addWeightedEdgeList).

Debug and fix:
- EdgeList with simple RefVar count
- WeightedEdgeList with weight-name-input
- Multiple constraint filling in sequence

- [ ] **Step 8: All 26 tests pass**

```bash
cd web && npx playwright test --config tests/e2e/playwright.config.ts
```
Expected: 26/26 PASS

- [ ] **Step 9: Run full validation**

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets
cd web && npm run test:unit && npx tsc --noEmit && npx playwright test --config tests/e2e/playwright.config.ts
```
Expected: ALL PASS

- [ ] **Step 10: Final commit**

```bash
git add -A
git commit -m "feat: all 26 E2E tests passing — editor MVP complete

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Self-Review Checklist

1. **Spec coverage**: Every section of the design spec (2026-04-19-subproject-bc-editor-design.md) maps to at least one task:
   - §1 Architecture → Tasks 9-10 (state, routing)
   - §2 WASM API → Tasks 6, 8
   - §3 Rust extensions → Tasks 1-4
   - §4 Frontend → Tasks 10-13
   - §5 testid contracts → Tasks 10-13 (all testid mentioned in spec are assigned to components)

2. **Placeholder scan**: No TBD/TODO items. All tasks have code or clear implementation steps.

3. **Type consistency**: ActionDto in Task 5 matches Action variants in Tasks 1-2. FullProjection types in Task 3 match DTOs in Task 5. Testid names in Tasks 10-13 match E2E POM (editor-page.ts) exactly.

4. **E2E test immutability**: No E2E test files are modified in any task. Task 14 explicitly states "No E2E test files are modified."

5. **DTO boundary**: Task 6 shows DTO conversion happening only in `#[wasm_bindgen]` functions. Task 9's action-builder builds JSON for the DTO boundary. Internal Rust functions (Tasks 1-4) use AstEngine directly.
