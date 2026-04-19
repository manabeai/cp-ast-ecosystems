use cp_ast_core::operation::*;
use cp_ast_core::structure::*;

/// Helper: create an engine and add a scalar "N" to the root sequence.
/// Returns (engine, N node id).
fn engine_with_n() -> (AstEngine, NodeId) {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();
    let n_id = *result.created_nodes.last().unwrap();
    (engine, n_id)
}

// ── EdgeList ────────────────────────────────────────────────────────────

#[test]
fn edge_list_creates_repeat_with_tuple_body() {
    let (mut engine, n_id) = engine_with_n();
    let root = engine.structure.root();

    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::EdgeList {
                edge_count: LengthSpec::RefVar(n_id),
            },
        })
        .unwrap();

    // The top-level created node is the Repeat
    let repeat_id = *result.created_nodes.last().unwrap();
    let repeat_node = engine.structure.get(repeat_id).unwrap();

    if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
        assert_eq!(body.len(), 1, "Repeat body should have exactly 1 child");
        let tuple_id = body[0];
        let tuple_node = engine.structure.get(tuple_id).unwrap();
        if let NodeKind::Tuple { elements } = tuple_node.kind() {
            assert_eq!(elements.len(), 2, "EdgeList tuple should have u, v");
            // Verify both are Scalar
            for &elem_id in elements {
                assert!(
                    matches!(
                        engine.structure.get(elem_id).unwrap().kind(),
                        NodeKind::Scalar { .. }
                    ),
                    "Each element should be a Scalar"
                );
            }
            // Verify names
            if let NodeKind::Scalar { name } = engine.structure.get(elements[0]).unwrap().kind() {
                assert_eq!(name.as_str(), "u");
            }
            if let NodeKind::Scalar { name } = engine.structure.get(elements[1]).unwrap().kind() {
                assert_eq!(name.as_str(), "v");
            }
        } else {
            panic!("Expected Tuple in Repeat body, got {:?}", tuple_node.kind());
        }
    } else {
        panic!("Expected Repeat, got {:?}", repeat_node.kind());
    }

    // EdgeList should not produce a TypeDecl constraint
    assert!(result.created_constraints.is_empty());
}

// ── WeightedEdgeList ────────────────────────────────────────────────────

#[test]
fn weighted_edge_list_creates_repeat_with_triple() {
    let (mut engine, n_id) = engine_with_n();
    let root = engine.structure.root();

    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::WeightedEdgeList {
                edge_count: LengthSpec::RefVar(n_id),
                weight_name: "w".to_owned(),
                weight_type: VarType::Int,
            },
        })
        .unwrap();

    let repeat_id = *result.created_nodes.last().unwrap();
    let repeat_node = engine.structure.get(repeat_id).unwrap();

    if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
        assert_eq!(body.len(), 1);
        let tuple_node = engine.structure.get(body[0]).unwrap();
        if let NodeKind::Tuple { elements } = tuple_node.kind() {
            assert_eq!(
                elements.len(),
                3,
                "WeightedEdgeList tuple should have u, v, w"
            );
            // Verify third element name matches weight_name
            if let NodeKind::Scalar { name } = engine.structure.get(elements[2]).unwrap().kind() {
                assert_eq!(name.as_str(), "w");
            }
        } else {
            panic!("Expected Tuple");
        }
    } else {
        panic!("Expected Repeat");
    }

    // WeightedEdgeList via AddSlotElement does not auto-add constraints
    // (TypeDecl constraint is only added by FillHole)
    assert!(result.created_constraints.is_empty());
}

// ── QueryList ───────────────────────────────────────────────────────────

#[test]
fn query_list_creates_repeat_with_choice() {
    let (mut engine, _n_id) = engine_with_n();
    let root = engine.structure.root();

    // Add Q scalar for query count
    let q_result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "Q".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();
    let q_id = *q_result.created_nodes.last().unwrap();

    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::QueryList {
                query_count: LengthSpec::RefVar(q_id),
            },
        })
        .unwrap();

    let repeat_id = *result.created_nodes.last().unwrap();
    let repeat_node = engine.structure.get(repeat_id).unwrap();

    if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
        assert_eq!(body.len(), 1);
        let choice_node = engine.structure.get(body[0]).unwrap();
        if let NodeKind::Choice { tag, variants } = choice_node.kind() {
            // Tag should be Unresolved("type")
            assert!(
                matches!(tag, Reference::Unresolved(ident) if ident.as_str() == "type"),
                "Choice tag should be Unresolved(\"type\")"
            );
            // Variants start empty
            assert!(variants.is_empty(), "QueryList starts with no variants");
        } else {
            panic!("Expected Choice, got {:?}", choice_node.kind());
        }
    } else {
        panic!("Expected Repeat");
    }

    // No TypeDecl constraint
    assert!(result.created_constraints.is_empty());
}

// ── MultiTestCaseTemplate ───────────────────────────────────────────────

#[test]
fn multi_test_case_template_creates_repeat_with_hole() {
    let (mut engine, _n_id) = engine_with_n();
    let root = engine.structure.root();

    // Add T scalar for test case count
    let t_result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "T".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();
    let t_id = *t_result.created_nodes.last().unwrap();

    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::MultiTestCaseTemplate {
                count: LengthSpec::RefVar(t_id),
            },
        })
        .unwrap();

    let repeat_id = *result.created_nodes.last().unwrap();
    let repeat_node = engine.structure.get(repeat_id).unwrap();

    if let NodeKind::Repeat { body, .. } = repeat_node.kind() {
        assert_eq!(body.len(), 1, "Repeat body should have exactly 1 Hole");
        let hole_node = engine.structure.get(body[0]).unwrap();
        assert!(
            matches!(hole_node.kind(), NodeKind::Hole { .. }),
            "Body element should be a Hole"
        );
    } else {
        panic!("Expected Repeat");
    }

    // No TypeDecl constraint
    assert!(result.created_constraints.is_empty());
}

// ── GridTemplate ────────────────────────────────────────────────────────

#[test]
fn grid_template_creates_matrix() {
    let (mut engine, n_id) = engine_with_n();
    let root = engine.structure.root();

    // Add M scalar for cols
    let m_result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "M".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();
    let m_id = *m_result.created_nodes.last().unwrap();

    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::GridTemplate {
                name: "A".to_owned(),
                rows: LengthSpec::RefVar(n_id),
                cols: LengthSpec::RefVar(m_id),
                cell_type: VarType::Int,
            },
        })
        .unwrap();

    let matrix_id = *result.created_nodes.last().unwrap();
    let matrix_node = engine.structure.get(matrix_id).unwrap();

    if let NodeKind::Matrix { name, .. } = matrix_node.kind() {
        assert_eq!(name.as_str(), "A");
    } else {
        panic!("Expected Matrix, got {:?}", matrix_node.kind());
    }

    // GridTemplate via AddSlotElement does not auto-add constraints
    assert!(result.created_constraints.is_empty());
}

// ── Variant construction tests ──────────────────────────────────────────

#[test]
fn fill_content_new_variants_construction() {
    let edge_list = FillContent::EdgeList {
        edge_count: LengthSpec::Fixed(10),
    };
    let weighted = FillContent::WeightedEdgeList {
        edge_count: LengthSpec::Expr("M".to_owned()),
        weight_name: "c".to_owned(),
        weight_type: VarType::Int,
    };
    let query = FillContent::QueryList {
        query_count: LengthSpec::Fixed(5),
    };
    let multi = FillContent::MultiTestCaseTemplate {
        count: LengthSpec::Fixed(100),
    };
    let grid = FillContent::GridTemplate {
        name: "G".to_owned(),
        rows: LengthSpec::Fixed(3),
        cols: LengthSpec::Fixed(3),
        cell_type: VarType::Char,
    };

    assert!(matches!(edge_list, FillContent::EdgeList { .. }));
    assert!(matches!(weighted, FillContent::WeightedEdgeList { .. }));
    assert!(matches!(query, FillContent::QueryList { .. }));
    assert!(matches!(multi, FillContent::MultiTestCaseTemplate { .. }));
    assert!(matches!(grid, FillContent::GridTemplate { .. }));
}

// ── EdgeList with Fixed count ───────────────────────────────────────────

#[test]
fn edge_list_with_fixed_count() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::EdgeList {
                edge_count: LengthSpec::Fixed(5),
            },
        })
        .unwrap();

    let repeat_id = *result.created_nodes.last().unwrap();
    let repeat_node = engine.structure.get(repeat_id).unwrap();
    assert!(matches!(repeat_node.kind(), NodeKind::Repeat { .. }));
}

// ── FillHole adds TypeDecl for WeightedEdgeList and GridTemplate ────────

#[test]
fn fill_hole_weighted_edge_list_adds_type_constraint() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let result = engine
        .apply(&Action::FillHole {
            target: hole_id,
            fill: FillContent::WeightedEdgeList {
                edge_count: LengthSpec::Fixed(5),
                weight_name: "w".to_owned(),
                weight_type: VarType::Int,
            },
        })
        .unwrap();

    assert!(
        !result.created_constraints.is_empty(),
        "FillHole should add TypeDecl for weight_type"
    );
}

#[test]
fn fill_hole_grid_template_adds_type_constraint() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let result = engine
        .apply(&Action::FillHole {
            target: hole_id,
            fill: FillContent::GridTemplate {
                name: "A".to_owned(),
                rows: LengthSpec::Fixed(3),
                cols: LengthSpec::Fixed(3),
                cell_type: VarType::Int,
            },
        })
        .unwrap();

    assert!(
        !result.created_constraints.is_empty(),
        "FillHole should add TypeDecl for cell_type"
    );
}
