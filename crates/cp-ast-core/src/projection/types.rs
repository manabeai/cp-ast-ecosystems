use crate::structure::NodeId;

/// A node projected for UI display with depth and metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedNode {
    pub id: NodeId,
    pub label: String,
    pub depth: usize,
    pub is_hole: bool,
}

/// A named slot entry representing a child relationship.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotEntry {
    pub name: String,
    pub child: NodeId,
}

/// Detailed information about a node including constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeDetail {
    pub id: NodeId,
    pub kind_label: String,
    pub constraints: Vec<String>, // human-readable
}

/// Types of candidates that can fill a hole node.
#[derive(Debug, Clone, PartialEq)]
pub enum CandidateKind {
    IntroduceScalar { suggested_names: Vec<String> },
    IntroduceArray { suggested_names: Vec<String> },
    IntroduceMatrix,
    IntroduceSection,
}

/// An action that can be performed on the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct AvailableAction {
    pub target: NodeId,
    pub description: String,
}

/// Reason why a node cannot be edited.
#[derive(Debug, Clone, PartialEq)]
pub enum NotEditableReason {
    HasDependents { dependents: Vec<NodeId> },
    IsRoot,
}

/// Summary of AST completeness and satisfaction status.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletenessSummary {
    pub total_holes: usize,
    pub filled_slots: usize,
    pub unsatisfied_constraints: usize,
    pub is_complete: bool,
}
