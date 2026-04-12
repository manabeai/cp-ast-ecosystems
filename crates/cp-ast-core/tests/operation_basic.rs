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
