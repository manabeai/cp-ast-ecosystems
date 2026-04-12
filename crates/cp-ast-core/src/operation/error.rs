use crate::constraint::{ConstraintId, ExpectedType};
use crate::structure::NodeId;

/// Detail about a constraint violation.
#[derive(Debug, Clone, PartialEq)]
pub struct ViolationDetail {
    /// ID of the violated constraint.
    pub constraint_id: ConstraintId,
    /// Human-readable description of the violation.
    pub description: String,
    /// Optional suggestion for fixing the violation.
    pub suggestion: Option<String>,
}

/// Errors that can occur when applying operations.
#[derive(Debug, Clone, PartialEq)]
pub enum OperationError {
    /// Type mismatch between expected and actual.
    TypeMismatch {
        expected: ExpectedType,
        actual: String,
        context: String,
    },
    /// Target node not found in the AST.
    NodeNotFound { node: NodeId },
    /// Slot is already occupied.
    SlotOccupied {
        node: NodeId,
        current_occupant: String,
    },
    /// One or more constraints were violated.
    ConstraintViolation {
        violated_constraints: Vec<ViolationDetail>,
    },
    /// The operation is invalid for the given context.
    InvalidOperation { action: String, reason: String },
}
