use crate::constraint::ConstraintId;
use crate::structure::NodeId;

/// Result of successfully applying an action.
#[derive(Debug, Clone, PartialEq)]
pub struct ApplyResult {
    /// Nodes created by this operation.
    pub created_nodes: Vec<NodeId>,
    /// Nodes removed by this operation.
    pub removed_nodes: Vec<NodeId>,
    /// Constraints created by this operation.
    pub created_constraints: Vec<ConstraintId>,
    /// Constraints affected (but not removed) by this operation.
    pub affected_constraints: Vec<ConstraintId>,
}

/// Result of previewing an action (dry run).
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewResult {
    /// Holes that would be created.
    pub new_holes_created: Vec<NodeId>,
    /// Constraints that would be affected.
    pub constraints_affected: Vec<ConstraintId>,
}
