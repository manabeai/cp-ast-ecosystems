use crate::constraint::{ConstraintId, ExpectedType};
use crate::structure::NodeId;
use std::fmt;

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
    /// Fill content is invalid.
    InvalidFill { reason: String },
    /// Error during deserialization from JSON/wasm boundary.
    DeserializationError { message: String },
}

impl fmt::Display for ViolationDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Constraint {} violated: {}",
            self.constraint_id, self.description
        )?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (Suggestion: {suggestion})")?;
        }
        Ok(())
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch {
                expected,
                actual,
                context,
            } => {
                write!(
                    f,
                    "Type mismatch in {context}: expected {expected}, got {actual}"
                )
            }
            Self::NodeNotFound { node } => {
                write!(f, "Node not found: {node}")
            }
            Self::SlotOccupied {
                node,
                current_occupant,
            } => {
                write!(
                    f,
                    "Slot at node {node} is already occupied by: {current_occupant}"
                )
            }
            Self::ConstraintViolation {
                violated_constraints,
            } => {
                if violated_constraints.len() == 1 {
                    write!(f, "Constraint violation: {}", violated_constraints[0])
                } else {
                    writeln!(f, "Multiple constraint violations:")?;
                    for violation in violated_constraints {
                        writeln!(f, "  - {violation}")?;
                    }
                    Ok(())
                }
            }
            Self::InvalidOperation { action, reason } => {
                write!(f, "Invalid operation '{action}': {reason}")
            }
            Self::InvalidFill { reason } => {
                write!(f, "Invalid fill content: {reason}")
            }
            Self::DeserializationError { message } => {
                write!(f, "Deserialization error: {message}")
            }
        }
    }
}

impl std::error::Error for OperationError {}
