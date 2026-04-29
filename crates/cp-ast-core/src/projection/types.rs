use serde::Serialize;

use crate::structure::NodeId;

/// A node projected for UI display with depth and metadata.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedNode {
    pub id: NodeId,
    pub label: String,
    pub depth: usize,
    pub is_hole: bool,
}

/// A named slot entry representing a child relationship.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SlotEntry {
    pub name: String,
    pub child: NodeId,
}

/// Detailed information about a node including constraints.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeDetail {
    pub id: NodeId,
    pub kind_label: String,
    pub constraints: Vec<String>, // human-readable
}

/// Types of candidates that can fill a hole node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CandidateKind {
    IntroduceScalar { suggested_names: Vec<String> },
    IntroduceArray { suggested_names: Vec<String> },
    IntroduceMatrix,
    IntroduceSection,
}

/// An action that can be performed on the AST.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AvailableAction {
    pub target: NodeId,
    pub description: String,
}

/// Reason why a node cannot be edited.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NotEditableReason {
    HasDependents { dependents: Vec<NodeId> },
    IsRoot,
}

/// Summary of AST completeness and satisfaction status.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletenessSummary {
    pub total_holes: usize,
    pub filled_slots: usize,
    pub unsatisfied_constraints: usize,
    pub is_complete: bool,
}

/// Rich projection of the entire AST for the editor UI.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FullProjection {
    pub nodes: Vec<ProjectedNode>,
    pub structure_lines: Vec<StructureLine>,
    pub hotspots: Vec<Hotspot>,
    pub constraints: ProjectedConstraints,
    pub available_vars: Vec<ExprCandidate>,
    pub completeness: CompletenessSummary,
}

/// A display row in the Structure pane.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructureLine {
    pub depth: usize,
    pub nodes: Vec<ProjectedNode>,
}

/// Projected constraints split into draft and completed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedConstraints {
    pub items: Vec<ConstraintItem>,
    pub drafts: Vec<DraftConstraint>,
    pub completed: Vec<CompletedConstraint>,
}

/// A stable display row for the Constraint pane.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConstraintItem {
    pub index: usize,
    pub status: ConstraintItemStatus,
    pub target_id: NodeId,
    pub target_name: String,
    pub display: String,
    pub template: Option<String>,
    pub constraint_id: Option<String>,
    pub draft_index: Option<usize>,
    pub completed_index: Option<usize>,
}

/// Whether a projected constraint row is still draft or already completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintItemStatus {
    Draft,
    Completed,
}

/// An insertion point in the UI.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hotspot {
    pub parent_id: NodeId,
    pub direction: HotspotDirection,
    pub candidates: Vec<String>,
}

/// Direction of a hotspot insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HotspotDirection {
    Below,
    Right,
    Inside,
    Variant,
}

/// An unfilled constraint generated on-the-fly by projection.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DraftConstraint {
    pub index: usize,
    pub target_id: NodeId,
    pub target_name: String,
    pub display: String,
    pub template: String,
}

/// A fully specified constraint from the AST.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletedConstraint {
    pub index: usize,
    pub constraint_id: String,
    pub display: String,
}

/// A variable available for use in expressions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExprCandidate {
    pub name: String,
    pub node_id: NodeId,
}

/// Detailed candidate info for hole filling.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HoleCandidateDetail {
    pub kind: String,
    pub label: String,
    pub fields: Vec<CandidateField>,
}

/// A field required to complete a candidate fill.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CandidateField {
    pub name: String,
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}
