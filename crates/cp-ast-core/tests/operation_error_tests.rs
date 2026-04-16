use cp_ast_core::constraint::{ConstraintId, ExpectedType};
use cp_ast_core::operation::{OperationError, ViolationDetail};
use cp_ast_core::structure::NodeId;

#[test]
fn test_operation_error_display() {
    // TypeMismatch
    let err = OperationError::TypeMismatch {
        expected: ExpectedType::Int,
        actual: "string".to_string(),
        context: "variable assignment".to_string(),
    };
    assert_eq!(
        format!("{err}"),
        "Type mismatch in variable assignment: expected integer, got string"
    );

    // NodeNotFound
    let err = OperationError::NodeNotFound {
        node: NodeId::from_raw(42),
    };
    assert_eq!(format!("{err}"), "Node not found: 42");

    // SlotOccupied
    let err = OperationError::SlotOccupied {
        node: NodeId::from_raw(10),
        current_occupant: "existing_value".to_string(),
    };
    assert_eq!(
        format!("{err}"),
        "Slot at node 10 is already occupied by: existing_value"
    );

    // ConstraintViolation - single violation
    let violation = ViolationDetail {
        constraint_id: ConstraintId::from_raw(1),
        description: "Value out of range".to_string(),
        suggestion: Some("Use value between 1 and 10".to_string()),
    };
    let err = OperationError::ConstraintViolation {
        violated_constraints: vec![violation],
    };
    assert_eq!(
        format!("{err}"),
        "Constraint violation: Constraint 1 violated: Value out of range (Suggestion: Use value between 1 and 10)"
    );

    // ConstraintViolation - multiple violations
    let violation1 = ViolationDetail {
        constraint_id: ConstraintId::from_raw(1),
        description: "First violation".to_string(),
        suggestion: None,
    };
    let violation2 = ViolationDetail {
        constraint_id: ConstraintId::from_raw(2),
        description: "Second violation".to_string(),
        suggestion: Some("Fix this".to_string()),
    };
    let err = OperationError::ConstraintViolation {
        violated_constraints: vec![violation1, violation2],
    };
    let output = format!("{err}");
    assert!(output.starts_with("Multiple constraint violations:"));
    assert!(output.contains("- Constraint 1 violated: First violation"));
    assert!(output.contains("- Constraint 2 violated: Second violation (Suggestion: Fix this)"));

    // InvalidOperation
    let err = OperationError::InvalidOperation {
        action: "DeleteNode".to_string(),
        reason: "Node has dependencies".to_string(),
    };
    assert_eq!(
        format!("{err}"),
        "Invalid operation 'DeleteNode': Node has dependencies"
    );

    // InvalidFill
    let err = OperationError::InvalidFill {
        reason: "Content does not match expected format".to_string(),
    };
    assert_eq!(
        format!("{err}"),
        "Invalid fill content: Content does not match expected format"
    );

    // DeserializationError
    let err = OperationError::DeserializationError {
        message: "Unexpected token at position 42".to_string(),
    };
    assert_eq!(
        format!("{err}"),
        "Deserialization error: Unexpected token at position 42"
    );
}

#[test]
fn test_violation_detail_display() {
    // ViolationDetail with suggestion
    let violation = ViolationDetail {
        constraint_id: ConstraintId::from_raw(42),
        description: "Value must be positive".to_string(),
        suggestion: Some("Use a value greater than zero".to_string()),
    };
    assert_eq!(
        format!("{violation}"),
        "Constraint 42 violated: Value must be positive (Suggestion: Use a value greater than zero)"
    );

    // ViolationDetail without suggestion
    let violation = ViolationDetail {
        constraint_id: ConstraintId::from_raw(123),
        description: "Invalid format".to_string(),
        suggestion: None,
    };
    assert_eq!(
        format!("{violation}"),
        "Constraint 123 violated: Invalid format"
    );
}

#[test]
fn test_operation_error_is_error() {
    let err = OperationError::InvalidFill {
        reason: "test error".to_string(),
    };
    // This test ensures OperationError implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_new_variants_can_be_constructed() {
    // Test InvalidFill variant
    let err = OperationError::InvalidFill {
        reason: "Invalid format".to_string(),
    };
    assert!(matches!(err, OperationError::InvalidFill { .. }));

    // Test DeserializationError variant
    let err = OperationError::DeserializationError {
        message: "JSON parse error".to_string(),
    };
    assert!(matches!(err, OperationError::DeserializationError { .. }));
}

#[test]
fn test_expected_type_display() {
    assert_eq!(format!("{}", ExpectedType::Int), "integer");
    assert_eq!(format!("{}", ExpectedType::Str), "string");
    assert_eq!(format!("{}", ExpectedType::Char), "character");
}

#[test]
fn test_operation_error_debug_clone() {
    let err = OperationError::InvalidFill {
        reason: "test".to_string(),
    };

    // Test Debug derive
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("InvalidFill"));
    assert!(debug_str.contains("test"));

    // Test Clone derive
    let cloned_err = err.clone();
    assert_eq!(err, cloned_err);
}
