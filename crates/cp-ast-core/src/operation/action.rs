use super::types::{ConstraintDef, FillContent, SumBoundDef};
use crate::constraint::ConstraintId;
use crate::structure::NodeId;

/// An action that can be applied to the AST.
///
/// Actions are the atomic operations that modify the `StructureAST` and `ConstraintSet`.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fill a hole with concrete content.
    FillHole { target: NodeId, fill: FillContent },
    /// Replace an existing node with new content.
    ReplaceNode {
        target: NodeId,
        replacement: FillContent,
    },
    /// Add a constraint to a node.
    AddConstraint {
        target: NodeId,
        constraint: ConstraintDef,
    },
    /// Remove a constraint by ID.
    RemoveConstraint { constraint_id: ConstraintId },
    /// Introduce multi-test-case wrapper.
    IntroduceMultiTestCase {
        count_var_name: String,
        sum_bound: Option<SumBoundDef>,
    },
    /// Add an element to a parent's slot (e.g., add child to Sequence).
    AddSlotElement {
        parent: NodeId,
        slot_name: String,
        element: FillContent,
    },
    /// Remove an element from a parent's slot.
    RemoveSlotElement {
        parent: NodeId,
        slot_name: String,
        child: NodeId,
    },
}
