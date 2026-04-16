use crate::constraint::ConstraintId;
use crate::operation::SlotKind;
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

/// Full projection of the AST for UI display.
#[derive(Debug, Clone, PartialEq)]
pub struct FullProjection {
    pub outline: Vec<OutlineNode>,
    pub diagnostics: Vec<Diagnostic>,
    pub completeness: CompletenessInfo,
}

/// A node in the outline view with rich metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineNode {
    pub id: NodeId,
    pub label: String,
    pub kind_label: String,
    pub depth: usize,
    pub is_hole: bool,
    pub child_ids: Vec<NodeId>,
}

/// A diagnostic message for the UI.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub node_id: Option<NodeId>,
    pub constraint_id: Option<ConstraintId>,
}

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

/// Summary of AST completeness.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletenessInfo {
    pub total_holes: usize,
    pub is_complete: bool,
    pub missing_constraints: Vec<String>,
}

/// Detailed view of a node's expression slots and related constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeDetailProjection {
    pub slots: Vec<SlotInfo>,
    pub related_constraints: Vec<ConstraintSummary>,
}

/// Information about an expression slot on a node.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotInfo {
    pub kind: SlotKind,
    pub current_expr: Option<String>,
    pub is_editable: bool,
}

/// Summary of a constraint for display.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintSummary {
    pub id: ConstraintId,
    pub label: String,
    pub kind_label: String,
}

/// A candidate type for filling a hole.
#[derive(Debug, Clone, PartialEq)]
pub struct HoleCandidate {
    pub kind: String,
    pub suggested_names: Vec<String>,
}

/// Menu of expression candidates.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprCandidateMenu {
    pub references: Vec<ReferenceCandidate>,
    pub literals: Vec<i64>,
}

/// A candidate reference for expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceCandidate {
    pub node_id: NodeId,
    pub label: String,
}

/// Menu of constraint target nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintTargetMenu {
    pub targets: Vec<ReferenceCandidate>,
}
