//! Tests for editor projection and action DTOs.

use cp_ast_core::constraint::{ConstraintId, Expression, RelationOp};
use cp_ast_core::operation::{
    error::ViolationDetail, result::ApplyResult, Action, ConstraintDef, ConstraintDefKind,
    FillContent, LengthSpec, OperationError, SlotId, SlotKind, VarType,
};
use cp_ast_core::projection::{
    CompletenessInfo, Diagnostic, DiagnosticLevel, FullProjection, OutlineNode,
};
use cp_ast_core::structure::NodeId;
use cp_ast_json::{
    deserialize_action, serialize_action, serialize_apply_result, serialize_full_projection,
    serialize_operation_error, ConversionError,
};

#[test]
fn test_full_projection_serialization() {
    let projection = FullProjection {
        outline: vec![OutlineNode {
            id: NodeId::from_raw(1),
            label: "arr: array[n] of int".to_string(),
            kind_label: "Array".to_string(),
            depth: 0,
            is_hole: false,
            child_ids: vec![NodeId::from_raw(2)],
        }],
        diagnostics: vec![Diagnostic {
            level: DiagnosticLevel::Error,
            message: "Array length not specified".to_string(),
            node_id: Some(NodeId::from_raw(1)),
            constraint_id: None,
        }],
        completeness: CompletenessInfo {
            total_holes: 2,
            is_complete: false,
            missing_constraints: vec!["Range constraint missing".to_string()],
        },
    };

    let json = serialize_full_projection(&projection).expect("Failed to serialize");

    // Verify JSON contains expected structure
    assert!(json.contains("Array"));
    assert!(json.contains("error"));
    assert!(json.contains("total_holes"));

    // Verify JSON is valid by parsing it back
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert_eq!(parsed["outline"][0]["label"], "arr: array[n] of int");
    assert_eq!(parsed["diagnostics"][0]["level"], "error");
    assert_eq!(parsed["completeness"]["total_holes"], 2);
}

#[test]
fn test_action_fill_hole_roundtrip() {
    let action = Action::FillHole {
        target: NodeId::from_raw(42),
        fill: FillContent::Scalar {
            name: "x".to_string(),
            typ: VarType::Int,
        },
    };

    let json = serialize_action(&action).expect("Failed to serialize");
    let parsed_action = deserialize_action(&json).expect("Failed to deserialize");

    assert_eq!(action, parsed_action);
}

#[test]
fn test_action_set_expr_roundtrip() {
    let action = Action::SetExpr {
        slot: SlotId {
            owner: NodeId::from_raw(10),
            kind: SlotKind::ArrayLength,
        },
        expr: Expression::Lit(5),
    };

    let json = serialize_action(&action).expect("Failed to serialize");
    let parsed_action = deserialize_action(&json).expect("Failed to deserialize");

    assert_eq!(action, parsed_action);
}

#[test]
fn test_action_array_with_ref_var_roundtrip() {
    let action = Action::FillHole {
        target: NodeId::from_raw(1),
        fill: FillContent::Array {
            name: "data".to_string(),
            element_type: VarType::Int,
            length: LengthSpec::RefVar(NodeId::from_raw(2)),
        },
    };

    let json = serialize_action(&action).expect("Failed to serialize");
    let parsed_action = deserialize_action(&json).expect("Failed to deserialize");

    assert_eq!(action, parsed_action);
}

#[test]
fn test_action_add_constraint_roundtrip() {
    let action = Action::AddConstraint {
        target: NodeId::from_raw(3),
        constraint: ConstraintDef {
            kind: ConstraintDefKind::Relation {
                op: RelationOp::Le,
                rhs: "100".to_string(),
            },
        },
    };

    let json = serialize_action(&action).expect("Failed to serialize");
    let parsed_action = deserialize_action(&json).expect("Failed to deserialize");

    assert_eq!(action, parsed_action);
}

#[test]
fn test_action_grid_roundtrip() {
    let action = Action::ReplaceNode {
        target: NodeId::from_raw(5),
        replacement: FillContent::Grid {
            name: "matrix".to_string(),
            rows: LengthSpec::Fixed(3),
            cols: LengthSpec::Expr("n + 1".to_string()),
            cell_type: VarType::Char,
        },
    };

    let json = serialize_action(&action).expect("Failed to serialize");
    let parsed_action = deserialize_action(&json).expect("Failed to deserialize");

    assert_eq!(action, parsed_action);
}

#[test]
fn test_operation_error_serialization() {
    let error = OperationError::ConstraintViolation {
        violated_constraints: vec![ViolationDetail {
            constraint_id: ConstraintId::from_raw(1),
            description: "Value must be positive".to_string(),
            suggestion: Some("Try using a value > 0".to_string()),
        }],
    };

    let json = serialize_operation_error(&error).expect("Failed to serialize");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert_eq!(parsed["kind"], "ConstraintViolation");
    assert_eq!(
        parsed["violations"][0]["description"],
        "Value must be positive"
    );
}

#[test]
fn test_apply_result_serialization() {
    let result = ApplyResult {
        created_nodes: vec![NodeId::from_raw(10), NodeId::from_raw(11)],
        removed_nodes: vec![NodeId::from_raw(5)],
        created_constraints: vec![ConstraintId::from_raw(3)],
        affected_constraints: vec![ConstraintId::from_raw(1), ConstraintId::from_raw(2)],
    };

    let json = serialize_apply_result(&result).expect("Failed to serialize");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert_eq!(parsed["created_nodes"], serde_json::json!(["10", "11"]));
    assert_eq!(parsed["removed_nodes"], serde_json::json!(["5"]));
}

#[test]
fn test_invalid_action_deserialization() {
    let bad_json = r#"{"kind": "FillHole", "target": "not_a_number", "fill": {"kind": "Scalar", "name": "x", "typ": "Int"}}"#;

    let result = deserialize_action(bad_json);
    assert!(result.is_err());

    if let Err(ConversionError::InvalidId(id)) = result {
        assert_eq!(id, "not_a_number");
    } else {
        panic!("Expected InvalidId error");
    }
}

#[test]
fn test_slot_kind_serialization() {
    // Test each SlotKind variant
    let kinds = [
        SlotKind::ArrayLength,
        SlotKind::RepeatCount,
        SlotKind::RangeLower,
        SlotKind::RangeUpper,
        SlotKind::RelationLhs,
        SlotKind::RelationRhs,
        SlotKind::LengthLength,
    ];

    for kind in kinds {
        let action = Action::SetExpr {
            slot: SlotId {
                owner: NodeId::from_raw(1),
                kind,
            },
            expr: Expression::Lit(42),
        };

        let json = serialize_action(&action).expect("Failed to serialize");
        let parsed_action = deserialize_action(&json).expect("Failed to deserialize");

        assert_eq!(action, parsed_action);
    }
}

#[test]
fn test_length_spec_variants() {
    let variants = vec![
        LengthSpec::Fixed(10),
        LengthSpec::RefVar(NodeId::from_raw(3)),
        LengthSpec::Expr("2 * n".to_string()),
    ];

    for length in variants {
        let action = Action::FillHole {
            target: NodeId::from_raw(1),
            fill: FillContent::Array {
                name: "test".to_string(),
                element_type: VarType::Int,
                length: length.clone(),
            },
        };

        let json = serialize_action(&action).expect("Failed to serialize");
        let parsed_action = deserialize_action(&json).expect("Failed to deserialize");

        assert_eq!(action, parsed_action);
    }
}
