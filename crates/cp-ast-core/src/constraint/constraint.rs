use super::expected_type::ExpectedType;
use super::expression::Expression;
use crate::structure::NodeId;

/// A constraint on a position in the structure AST.
///
/// Constraints define what is allowed at each position. They are used for
/// validation, candidate enumeration, and sample case generation.
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Value range constraint (e.g., `1 <= N <= 2*10^5`)
    Range {
        target: NodeId,
        min: Expression,
        max: Expression,
    },
    /// Array length tied to another variable (e.g., `len(A) = N`)
    Length { array: NodeId, length_ref: NodeId },
    /// Per-element constraint on an array (e.g., `0 <= A[i] <= 10^9`)
    Element {
        array: NodeId,
        element_constraint: Box<Constraint>,
    },
    /// Expected type for a position (e.g., `N: Int`)
    Type {
        target: NodeId,
        expected: ExpectedType,
    },
}

impl Constraint {
    /// Create a range constraint.
    #[must_use]
    pub fn range(target: NodeId, min: Expression, max: Expression) -> Self {
        Self::Range { target, min, max }
    }

    /// Create a length constraint.
    #[must_use]
    pub fn length(array: NodeId, length_ref: NodeId) -> Self {
        Self::Length { array, length_ref }
    }

    /// Create an element constraint.
    #[must_use]
    pub fn element(array: NodeId, element_constraint: Constraint) -> Self {
        Self::Element {
            array,
            element_constraint: Box::new(element_constraint),
        }
    }

    /// Create a type constraint.
    #[must_use]
    pub fn expected_type(target: NodeId, expected: ExpectedType) -> Self {
        Self::Type { target, expected }
    }

    /// Returns the primary target `NodeId` of this constraint.
    #[must_use]
    pub fn target(&self) -> NodeId {
        match self {
            Self::Range { target, .. } | Self::Type { target, .. } => *target,
            Self::Length { array, .. } | Self::Element { array, .. } => *array,
        }
    }
}
