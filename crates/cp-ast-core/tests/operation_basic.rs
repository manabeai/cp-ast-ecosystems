use cp_ast_core::constraint::*;
use cp_ast_core::operation::*;
use cp_ast_core::structure::*;

#[test]
fn engine_construction() {
    let engine = AstEngine::new();
    assert!(engine.structure.contains(engine.structure.root()));
    assert!(engine.constraints.is_empty());
}

#[test]
fn engine_default() {
    let engine = AstEngine::default();
    assert_eq!(engine.structure.len(), 1); // root node
}

#[test]
fn action_fill_hole_construction() {
    let action = Action::FillHole {
        target: NodeId::from_raw(1),
        fill: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    };
    assert!(matches!(action, Action::FillHole { .. }));
}

#[test]
fn action_add_constraint_construction() {
    let action = Action::AddConstraint {
        target: NodeId::from_raw(1),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Range {
                lower: "1".to_owned(),
                upper: "100".to_owned(),
            },
        },
    };
    assert!(matches!(action, Action::AddConstraint { .. }));
}

#[test]
fn action_remove_constraint_construction() {
    let action = Action::RemoveConstraint {
        constraint_id: ConstraintId::from_raw(0),
    };
    assert!(matches!(action, Action::RemoveConstraint { .. }));
}

#[test]
fn operation_error_node_not_found() {
    let err = OperationError::NodeNotFound {
        node: NodeId::from_raw(99),
    };
    assert!(matches!(err, OperationError::NodeNotFound { .. }));
}

#[test]
fn operation_error_constraint_violation() {
    let err = OperationError::ConstraintViolation {
        violated_constraints: vec![ViolationDetail {
            constraint_id: ConstraintId::from_raw(0),
            description: "out of range".to_owned(),
            suggestion: Some("use value within 1..100".to_owned()),
        }],
    };
    assert!(matches!(err, OperationError::ConstraintViolation { .. }));
}

#[test]
fn apply_result_construction() {
    let result = ApplyResult {
        created_nodes: vec![NodeId::from_raw(1)],
        removed_nodes: vec![],
        created_constraints: vec![ConstraintId::from_raw(0)],
        affected_constraints: vec![],
    };
    assert_eq!(result.created_nodes.len(), 1);
}

#[test]
fn fill_content_all_variants() {
    let scalar = FillContent::Scalar {
        name: "N".to_owned(),
        typ: VarType::Int,
    };
    let array = FillContent::Array {
        name: "A".to_owned(),
        element_type: VarType::Int,
        length: LengthSpec::RefVar(NodeId::from_raw(1)),
    };
    let grid = FillContent::Grid {
        name: "G".to_owned(),
        rows: LengthSpec::Fixed(3),
        cols: LengthSpec::Fixed(3),
        cell_type: VarType::Int,
    };
    let section = FillContent::Section {
        label: "Input".to_owned(),
    };
    let output_val = FillContent::OutputSingleValue { typ: VarType::Int };
    let output_yn = FillContent::OutputYesNo;

    // Verify all variants can be created
    assert!(matches!(scalar, FillContent::Scalar { .. }));
    assert!(matches!(array, FillContent::Array { .. }));
    assert!(matches!(grid, FillContent::Grid { .. }));
    assert!(matches!(section, FillContent::Section { .. }));
    assert!(matches!(output_val, FillContent::OutputSingleValue { .. }));
    assert!(matches!(output_yn, FillContent::OutputYesNo));
}

#[test]
fn constraint_def_all_kinds() {
    let range = ConstraintDefKind::Range {
        lower: "1".to_owned(),
        upper: "100".to_owned(),
    };
    let type_decl = ConstraintDefKind::TypeDecl { typ: VarType::Int };
    let relation = ConstraintDefKind::Relation {
        op: RelationOp::Le,
        rhs: "N".to_owned(),
    };
    let distinct = ConstraintDefKind::Distinct;
    let sorted = ConstraintDefKind::Sorted {
        order: SortOrder::Ascending,
    };
    let property = ConstraintDefKind::Property {
        tag: "simple".to_owned(),
    };
    let sum_bound = ConstraintDefKind::SumBound {
        over_var: "N".to_owned(),
        upper: "2*10^5".to_owned(),
    };
    let guarantee = ConstraintDefKind::Guarantee {
        description: "valid".to_owned(),
    };

    // Verify all constraint kinds can be created
    assert!(matches!(range, ConstraintDefKind::Range { .. }));
    assert!(matches!(type_decl, ConstraintDefKind::TypeDecl { .. }));
    assert!(matches!(relation, ConstraintDefKind::Relation { .. }));
    assert!(matches!(distinct, ConstraintDefKind::Distinct));
    assert!(matches!(sorted, ConstraintDefKind::Sorted { .. }));
    assert!(matches!(property, ConstraintDefKind::Property { .. }));
    assert!(matches!(sum_bound, ConstraintDefKind::SumBound { .. }));
    assert!(matches!(guarantee, ConstraintDefKind::Guarantee { .. }));
}

// New tests for the implemented operations

#[test]
fn fill_hole_scalar_success() {
    let mut engine = AstEngine::new();
    // Add a hole to the structure
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let result = engine.apply(&Action::FillHole {
        target: hole_id,
        fill: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    });

    let result = result.unwrap();
    // The hole itself is replaced (not a new node)
    assert!(matches!(
        engine.structure.get(hole_id).unwrap().kind(),
        NodeKind::Scalar { .. }
    ));
    // TypeDecl constraint auto-added
    assert!(!result.created_constraints.is_empty());
}

#[test]
fn fill_hole_nonexistent_node_fails() {
    let mut engine = AstEngine::new();
    let result = engine.apply(&Action::FillHole {
        target: NodeId::from_raw(999),
        fill: FillContent::Scalar {
            name: "N".to_owned(),
            typ: VarType::Int,
        },
    });
    assert!(matches!(result, Err(OperationError::NodeNotFound { .. })));
}

#[test]
fn fill_hole_non_hole_fails() {
    let mut engine = AstEngine::new();
    let scalar_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let result = engine.apply(&Action::FillHole {
        target: scalar_id,
        fill: FillContent::Scalar {
            name: "M".to_owned(),
            typ: VarType::Int,
        },
    });
    assert!(matches!(
        result,
        Err(OperationError::InvalidOperation { .. })
    ));
}

#[test]
fn fill_hole_array_creates_structure() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, hole_id],
        });
    }

    let _result = engine
        .apply(&Action::FillHole {
            target: hole_id,
            fill: FillContent::Array {
                name: "A".to_owned(),
                element_type: VarType::Int,
                length: LengthSpec::RefVar(n_id),
            },
        })
        .unwrap();

    // The hole is now an Array
    assert!(matches!(
        engine.structure.get(hole_id).unwrap().kind(),
        NodeKind::Array { .. }
    ));
}

#[test]
fn add_constraint_range_success() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    let result = engine
        .apply(&Action::AddConstraint {
            target: n_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "100".to_owned(),
                },
            },
        })
        .unwrap();

    assert_eq!(result.created_constraints.len(), 1);
    assert!(engine
        .constraints
        .get(result.created_constraints[0])
        .is_some());
}

#[test]
fn add_constraint_to_hole_allowed() {
    // Rev.1 L-4: constraints can be pre-attached to holes
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });

    let result = engine
        .apply(&Action::AddConstraint {
            target: hole_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::TypeDecl { typ: VarType::Int },
            },
        })
        .unwrap();

    assert_eq!(result.created_constraints.len(), 1);
}

#[test]
fn add_constraint_node_not_found_fails() {
    let mut engine = AstEngine::new();
    let result = engine.apply(&Action::AddConstraint {
        target: NodeId::from_raw(999),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Distinct,
        },
    });
    assert!(matches!(result, Err(OperationError::NodeNotFound { .. })));
}

#[test]
fn remove_constraint_success() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    // First add a constraint
    let add_result = engine
        .apply(&Action::AddConstraint {
            target: n_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::TypeDecl { typ: VarType::Int },
            },
        })
        .unwrap();

    let cid = add_result.created_constraints[0];

    // Now remove it
    let remove_result = engine
        .apply(&Action::RemoveConstraint { constraint_id: cid })
        .unwrap();
    assert!(remove_result.affected_constraints.contains(&cid));
    assert!(engine.constraints.get(cid).is_none());
}

#[test]
fn remove_constraint_not_found_fails() {
    let mut engine = AstEngine::new();
    let result = engine.apply(&Action::RemoveConstraint {
        constraint_id: ConstraintId::from_raw(999),
    });
    assert!(matches!(
        result,
        Err(OperationError::InvalidOperation { .. })
    ));
}

#[test]
fn replace_node_success() {
    let mut engine = AstEngine::new();
    let scalar_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    let result = engine
        .apply(&Action::ReplaceNode {
            target: scalar_id,
            replacement: FillContent::Array {
                name: "A".to_owned(),
                element_type: VarType::Int,
                length: LengthSpec::Expr("N".to_owned()),
            },
        })
        .unwrap();

    assert!(matches!(
        engine.structure.get(scalar_id).unwrap().kind(),
        NodeKind::Array { .. }
    ));
    assert!(result.removed_nodes.is_empty()); // replace is in-place
}

#[test]
fn replace_node_with_dependents_fails() {
    let mut engine = AstEngine::new();
    let scalar_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    // Add a constraint to create a dependent
    engine
        .apply(&Action::AddConstraint {
            target: scalar_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "100".to_owned(),
                },
            },
        })
        .unwrap();

    let result = engine.apply(&Action::ReplaceNode {
        target: scalar_id,
        replacement: FillContent::Scalar {
            name: "M".to_owned(),
            typ: VarType::Int,
        },
    });

    assert!(matches!(
        result,
        Err(OperationError::InvalidOperation { .. })
    ));
}

#[test]
fn add_slot_element_to_sequence() {
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

    assert_eq!(result.created_nodes.len(), 1);
    // Verify the new node is in root's children
    if let NodeKind::Sequence { children } = engine.structure.get(root).unwrap().kind() {
        assert!(children.contains(&result.created_nodes[0]));
    } else {
        panic!("Root should be Sequence");
    }
}

#[test]
fn remove_slot_element_from_sequence() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // First add an element
    let add_result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    let child_id = add_result.created_nodes[0];

    // Now remove it
    let remove_result = engine
        .apply(&Action::RemoveSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            child: child_id,
        })
        .unwrap();

    assert!(remove_result.removed_nodes.contains(&child_id));
    assert!(!engine.structure.contains(child_id));
}

#[test]
fn introduce_multi_test_case_success() {
    let mut engine = AstEngine::new();
    // Add some structure first
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }

    let result = engine
        .apply(&Action::IntroduceMultiTestCase {
            count_var_name: "T".to_owned(),
            sum_bound: Some(SumBoundDef {
                bound_var: "N".to_owned(),
                upper: "200000".to_owned(),
            }),
        })
        .unwrap();

    // Should have created count var + repeat node
    assert!(result.created_nodes.len() >= 2);
    // Should have created SumBound constraint
    assert!(!result.created_constraints.is_empty());
}

#[test]
fn introduce_multi_test_case_already_exists_fails() {
    let mut engine = AstEngine::new();
    // Add a Repeat node manually (simulating existing multi-test-case)
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Reference::Unresolved(Ident::new("T")),
        body: vec![],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![repeat_id],
        });
    }

    let result = engine.apply(&Action::IntroduceMultiTestCase {
        count_var_name: "T".to_owned(),
        sum_bound: None,
    });

    assert!(matches!(
        result,
        Err(OperationError::InvalidOperation { .. })
    ));
}
