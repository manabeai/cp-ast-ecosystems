use cp_ast_core::operation::action::Action;
use cp_ast_core::operation::types::{
    ConstraintDef, ConstraintDefKind, FillContent, LengthSpec, VarType,
};
use cp_ast_core::structure::NodeId;
use cp_ast_json::{deserialize_action, serialize_action};

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
        tag_value: Literal::IntLit(1),
        first_element: FillContent::Scalar {
            name: "a".to_owned(),
            typ: VarType::Int,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn weighted_edge_list_roundtrip() {
    let action = Action::AddSlotElement {
        parent: NodeId::from_raw(0),
        slot_name: "children".to_owned(),
        element: FillContent::WeightedEdgeList {
            edge_count: LengthSpec::RefVar(NodeId::from_raw(2)),
            weight_name: "w".to_owned(),
            weight_type: VarType::Int,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn grid_template_roundtrip() {
    let action = Action::AddSlotElement {
        parent: NodeId::from_raw(0),
        slot_name: "children".to_owned(),
        element: FillContent::GridTemplate {
            name: "S".to_owned(),
            rows: LengthSpec::RefVar(NodeId::from_raw(1)),
            cols: LengthSpec::RefVar(NodeId::from_raw(2)),
            cell_type: VarType::Char,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn multi_testcase_roundtrip() {
    let action = Action::AddSlotElement {
        parent: NodeId::from_raw(0),
        slot_name: "children".to_owned(),
        element: FillContent::MultiTestCaseTemplate {
            count: LengthSpec::RefVar(NodeId::from_raw(1)),
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn query_list_roundtrip() {
    let action = Action::AddSlotElement {
        parent: NodeId::from_raw(0),
        slot_name: "children".to_owned(),
        element: FillContent::QueryList {
            query_count: LengthSpec::RefVar(NodeId::from_raw(3)),
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn replace_node_roundtrip() {
    let action = Action::ReplaceNode {
        target: NodeId::from_raw(3),
        replacement: FillContent::Scalar {
            name: "X".to_owned(),
            typ: VarType::Str,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn remove_constraint_roundtrip() {
    use cp_ast_core::constraint::ConstraintId;

    let action = Action::RemoveConstraint {
        constraint_id: ConstraintId::from_raw(42),
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn introduce_multi_test_case_roundtrip() {
    use cp_ast_core::operation::types::SumBoundDef;

    let action = Action::IntroduceMultiTestCase {
        count_var_name: "T".to_owned(),
        sum_bound: Some(SumBoundDef {
            bound_var: "N".to_owned(),
            upper: "200000".to_owned(),
        }),
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn introduce_multi_test_case_no_bound_roundtrip() {
    let action = Action::IntroduceMultiTestCase {
        count_var_name: "T".to_owned(),
        sum_bound: None,
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn remove_slot_element_roundtrip() {
    let action = Action::RemoveSlotElement {
        parent: NodeId::from_raw(0),
        slot_name: "children".to_owned(),
        child: NodeId::from_raw(3),
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn output_single_value_roundtrip() {
    let action = Action::FillHole {
        target: NodeId::from_raw(7),
        fill: FillContent::OutputSingleValue { typ: VarType::Int },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn output_yes_no_roundtrip() {
    let action = Action::FillHole {
        target: NodeId::from_raw(8),
        fill: FillContent::OutputYesNo,
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn section_roundtrip() {
    let action = Action::FillHole {
        target: NodeId::from_raw(2),
        fill: FillContent::Section {
            label: "Output".to_owned(),
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn grid_roundtrip() {
    let action = Action::FillHole {
        target: NodeId::from_raw(4),
        fill: FillContent::Grid {
            name: "G".to_owned(),
            rows: LengthSpec::Fixed(3),
            cols: LengthSpec::Fixed(4),
            cell_type: VarType::Int,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_constraint_distinct_roundtrip() {
    let action = Action::AddConstraint {
        target: NodeId::from_raw(2),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Distinct,
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_constraint_sorted_roundtrip() {
    use cp_ast_core::constraint::SortOrder;

    let action = Action::AddConstraint {
        target: NodeId::from_raw(2),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Sorted {
                order: SortOrder::NonDecreasing,
            },
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_constraint_relation_roundtrip() {
    use cp_ast_core::constraint::RelationOp;

    let action = Action::AddConstraint {
        target: NodeId::from_raw(1),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Relation {
                op: RelationOp::Le,
                rhs: "N".to_owned(),
            },
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_constraint_guarantee_roundtrip() {
    let action = Action::AddConstraint {
        target: NodeId::from_raw(0),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Guarantee {
                description: "The graph is connected".to_owned(),
            },
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn add_constraint_string_length_roundtrip() {
    let action = Action::AddConstraint {
        target: NodeId::from_raw(0),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::StringLength {
                min: "1".to_owned(),
                max: "N".to_owned(),
            },
        },
    };
    let json = serialize_action(&action).unwrap();
    let restored = deserialize_action(&json).unwrap();
    assert_eq!(action, restored);
}

#[test]
fn projection_serializes_to_json() {
    use cp_ast_core::operation::engine::AstEngine;
    use cp_ast_core::projection::project_full;
    use cp_ast_json::serialize_projection;

    let engine = AstEngine::new();
    let proj = project_full(&engine);
    let json = serialize_projection(&proj).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["hotspots"].is_array());
    assert!(parsed["nodes"].is_array());
    assert!(parsed["completeness"]["is_complete"].is_boolean());
}
