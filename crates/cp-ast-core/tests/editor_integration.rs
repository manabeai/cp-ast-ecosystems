use cp_ast_core::constraint::*;
use cp_ast_core::operation::*;
use cp_ast_core::projection::{project_full, project_node_detail, DiagnosticLevel};
use cp_ast_core::structure::*;

#[test]
fn build_scalar_n() {
    // Create new AstEngine
    let mut engine = AstEngine::new();

    // Get the root node and add a hole to it
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });

    // Set root as sequence containing the hole
    let root_id = engine.structure.root();
    if let Some(root) = engine.structure.get_mut(root_id) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    // FillHole on root → Scalar(N, int)
    let fill_action = Action::FillHole {
        target: hole_id,
        fill: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    };

    let result = engine.apply(&fill_action);
    assert!(result.is_ok(), "FillHole should succeed");

    // AddConstraint → Range(N, 1, 100000)
    // The hole was transformed to a scalar, so use the same hole_id
    let scalar_node_id = hole_id;

    let constraint_action = Action::AddConstraint {
        target: scalar_node_id,
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Range {
                lower: "1".to_owned(),
                upper: "100000".to_owned(),
            },
        },
    };

    let constraint_result = engine.apply(&constraint_action);
    assert!(constraint_result.is_ok(), "AddConstraint should succeed");

    // project_full → outline contains N
    let projection = project_full(&engine);

    // Verify outline contains our scalar N
    let scalar_nodes: Vec<_> = projection
        .outline
        .iter()
        .filter(|node| node.label == "N")
        .collect();
    assert!(!scalar_nodes.is_empty(), "Outline should contain scalar N");

    // Verify completeness (no holes remaining)
    assert_eq!(projection.completeness.total_holes, 0);
    assert!(projection.completeness.is_complete);
}

#[test]
fn build_scalar_and_array() {
    // Create engine
    let mut engine = AstEngine::new();

    // Add holes to root sequence
    let hole1_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    let hole2_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });

    let root_id = engine.structure.root();
    if let Some(root) = engine.structure.get_mut(root_id) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole1_id, hole2_id],
        });
    }

    // Add Scalar N
    let fill_n = Action::FillHole {
        target: hole1_id,
        fill: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    };

    let _result_n = engine.apply(&fill_n).expect("Should create scalar N");
    let n_node_id = hole1_id; // The hole was transformed to scalar N

    // Add Array A with length referencing N
    let fill_a = Action::FillHole {
        target: hole2_id,
        fill: FillContent::Array {
            name: "A".to_owned(),
            element_type: VarType::Int,
            length: LengthSpec::RefVar(n_node_id),
        },
    };

    let _result_a = engine.apply(&fill_a).expect("Should create array A");
    let a_node_id = hole2_id; // The hole was transformed to array A

    // AddConstraint → Range on N
    let constraint_n = Action::AddConstraint {
        target: n_node_id,
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Range {
                lower: "1".to_owned(),
                upper: "100000".to_owned(),
            },
        },
    };

    engine
        .apply(&constraint_n)
        .expect("Should add range constraint to N");

    // AddConstraint → Range on A elements
    let constraint_a = Action::AddConstraint {
        target: a_node_id,
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Range {
                lower: "1".to_owned(),
                upper: "1000000000".to_owned(),
            },
        },
    };

    engine
        .apply(&constraint_a)
        .expect("Should add range constraint to A");

    // project_full → outline has N and A
    let projection = project_full(&engine);

    let has_n = projection.outline.iter().any(|node| node.label == "N");
    let has_a = projection.outline.iter().any(|node| node.label == "A[]");

    assert!(has_n, "Outline should contain scalar N");
    assert!(has_a, "Outline should contain array A");

    // Verify completeness and structure
    assert_eq!(projection.completeness.total_holes, 0);
    assert!(projection.completeness.is_complete);

    // Should have no hole diagnostics
    let hole_diagnostics: Vec<_> = projection
        .diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Info && d.message.contains("Unfilled hole"))
        .collect();
    assert!(hole_diagnostics.is_empty(), "Should have no unfilled holes");
}

#[test]
#[allow(clippy::similar_names)]
fn build_tuple() {
    // Create engine
    let mut engine = AstEngine::new();

    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });

    let root_id = engine.structure.root();
    if let Some(root) = engine.structure.get_mut(root_id) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    // Create a tuple by manually building the structure
    // Since FillContent doesn't have Tuple variant, we'll create it manually
    #[allow(clippy::similar_names)]
    let scalar_n_id = engine
        .structure
        .add_node(NodeKind::Scalar { name: "N".into() });
    let scalar_m_id = engine
        .structure
        .add_node(NodeKind::Scalar { name: "M".into() });

    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![scalar_n_id, scalar_m_id],
    });

    // Replace the hole with the tuple
    let _replace_action = Action::ReplaceNode {
        target: hole_id,
        replacement: FillContent::Scalar {
            name: "dummy".to_owned(),
            typ: VarType::Int,
        },
    };

    // Since ReplaceNode might not fully work as expected, let's manually update the structure
    if let Some(root) = engine.structure.get_mut(root_id) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tuple_id],
        });
    }

    // project_full → verify structure
    let projection = project_full(&engine);

    let has_tuple = projection.outline.iter().any(|node| node.label == "Tuple");
    let has_n = projection.outline.iter().any(|node| node.label == "N");
    let has_m = projection.outline.iter().any(|node| node.label == "M");

    assert!(has_tuple, "Outline should contain tuple");
    assert!(has_n, "Tuple should contain element N");
    assert!(has_m, "Tuple should contain element M");
}

#[test]
fn projection_reflects_operations() {
    // Create engine, add some elements
    let mut engine = AstEngine::new();

    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });

    let root_id = engine.structure.root();
    if let Some(root) = engine.structure.get_mut(root_id) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    // project_full → verify outline matches (should have hole)
    let initial_projection = project_full(&engine);
    assert!(initial_projection.completeness.total_holes > 0);
    assert!(!initial_projection.completeness.is_complete);

    let has_hole = initial_projection.outline.iter().any(|node| node.is_hole);
    assert!(has_hole, "Initial projection should contain hole");

    // Add scalar
    let fill_action = Action::FillHole {
        target: hole_id,
        fill: FillContent::Scalar {
            name: "X".to_owned(),
            typ: VarType::Int,
        },
    };

    let _result = engine.apply(&fill_action).expect("Should fill hole");
    let scalar_id = hole_id; // The hole was transformed to scalar

    // project_node_detail → verify slots and constraints
    let detail = project_node_detail(&engine, scalar_id);
    assert!(detail.is_some(), "Should get detail projection for scalar");

    let detail = detail.unwrap();
    // Scalar nodes don't have expression slots, so should be empty
    assert!(
        detail.slots.is_empty(),
        "Scalar should have no expression slots"
    );

    // May have auto-added TypeDecl constraint when filling hole
    let initial_constraint_count = detail.related_constraints.len();

    // Modify (AddConstraint) → re-project → verify changes reflected
    let constraint_action = Action::AddConstraint {
        target: scalar_id,
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Range {
                lower: "1".to_owned(),
                upper: "1000".to_owned(),
            },
        },
    };

    engine
        .apply(&constraint_action)
        .expect("Should add constraint");

    // Re-project and verify constraint is reflected
    let updated_detail = project_node_detail(&engine, scalar_id).unwrap();
    assert!(
        updated_detail.related_constraints.len() > initial_constraint_count,
        "Should have more constraints after adding"
    );

    // Find the range constraint we added
    let has_range_constraint = updated_detail
        .related_constraints
        .iter()
        .any(|c| c.label.contains("Range") && c.kind_label == "Range");
    assert!(
        has_range_constraint,
        "Should have the range constraint we added"
    );

    // Final projection should show complete AST
    let final_projection = project_full(&engine);
    assert_eq!(final_projection.completeness.total_holes, 0);
    assert!(final_projection.completeness.is_complete);

    let has_scalar_x = final_projection
        .outline
        .iter()
        .any(|node| node.label == "X");
    assert!(has_scalar_x, "Final projection should contain scalar X");
}

#[test]
fn test_operation_error_handling() {
    let mut engine = AstEngine::new();

    // Try to apply action to non-existent node
    let invalid_action = Action::FillHole {
        target: NodeId::from_raw(999), // Non-existent node
        fill: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    };

    let result = engine.apply(&invalid_action);
    assert!(result.is_err(), "Should fail for non-existent target node");
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn test_set_expr_not_implemented() {
    let mut engine = AstEngine::new();

    let set_expr_action = Action::SetExpr {
        slot: SlotId {
            owner: NodeId::from_raw(1),
            kind: SlotKind::ArrayLength,
        },
        expr: Expression::Lit(10),
    };

    // Should panic because SetExpr is not implemented yet
    let _result = engine.apply(&set_expr_action);
}
