use super::types::{
    AvailableAction, CandidateKind, CompletenessSummary, NodeDetail, NotEditableReason,
    ProjectedNode, SlotEntry,
};
use crate::structure::NodeId;

/// The main projection API for viewing AST structure and constraints.
///
/// This trait provides read-only views of the AST for UI consumption,
/// transforming internal representations into display-ready formats.
pub trait ProjectionAPI {
    /// Return all nodes in DFS traversal order with display information.
    #[must_use]
    fn nodes(&self) -> Vec<ProjectedNode>;

    /// Get named slot entries (children) for a specific node.
    #[must_use]
    fn children(&self, node: NodeId) -> Vec<SlotEntry>;

    /// Get detailed information about a node including constraints.
    #[must_use]
    fn inspect(&self, node: NodeId) -> Option<NodeDetail>;

    /// Get candidate types that can fill a hole node.
    #[must_use]
    fn hole_candidates(&self, hole: NodeId) -> Vec<CandidateKind>;

    /// Get all available actions that can be performed on the AST.
    #[must_use]
    fn available_actions(&self) -> Vec<AvailableAction>;

    /// Check if a node cannot be edited and return the reason.
    #[must_use]
    fn why_not_editable(&self, node: NodeId) -> Option<NotEditableReason>;

    /// Get a summary of AST completeness status.
    #[must_use]
    fn completeness(&self) -> CompletenessSummary;
}
