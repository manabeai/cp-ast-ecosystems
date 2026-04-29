//! Data Transfer Objects for lossless AST JSON serialization.
//!
//! These types form the JSON schema contract. Internal types are never
//! directly serialized — conversion goes through these DTOs.

use serde::{Deserialize, Serialize};

/// Schema version. Bump when breaking the JSON format.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

// ── Top-level envelope ──────────────────────────────────────────────

/// Versioned envelope wrapping the AST document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDocumentEnvelope {
    pub schema_version: u32,
    pub document: AstDocumentDto,
}

/// The complete AST document (structure + constraints).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDocumentDto {
    pub structure: StructureAstDto,
    pub constraints: ConstraintSetDto,
}

// ── Structure ───────────────────────────────────────────────────────

/// Arena-based structure AST.
///
/// `arena` preserves insertion order and tombstone (`null`) slots.
/// IDs are decimal strings to avoid JS 53-bit precision loss.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureAstDto {
    pub root: String,
    pub next_id: String,
    pub arena: Vec<Option<StructureNodeDto>>,
}

/// A single structure node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureNodeDto {
    pub id: String,
    pub kind: NodeKindDto,
}

/// Discriminated union for node kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum NodeKindDto {
    Scalar {
        name: String,
    },
    Array {
        name: String,
        length: ExpressionDto,
    },
    Matrix {
        name: String,
        rows: ReferenceDto,
        cols: ReferenceDto,
    },
    Tuple {
        elements: Vec<String>,
    },
    Repeat {
        count: ExpressionDto,
        #[serde(skip_serializing_if = "Option::is_none")]
        index_var: Option<String>,
        body: Vec<String>,
    },
    Section {
        #[serde(skip_serializing_if = "Option::is_none")]
        header: Option<String>,
        body: Vec<String>,
    },
    Sequence {
        children: Vec<String>,
    },
    Choice {
        tag: ReferenceDto,
        variants: Vec<ChoiceVariantDto>,
    },
    Hole {
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_kind: Option<String>,
    },
}

/// A variant arm in a `Choice` node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceVariantDto {
    pub tag_value: LiteralDto,
    pub body: Vec<String>,
}

// ── Constraints ─────────────────────────────────────────────────────

/// Arena-based constraint set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSetDto {
    pub next_id: String,
    pub arena: Vec<Option<ConstraintEntryDto>>,
    pub by_node: Vec<ByNodeEntryDto>,
    pub global: Vec<String>,
}

/// A single constraint entry in the arena.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintEntryDto {
    pub id: String,
    pub constraint: ConstraintDto,
}

/// Per-node constraint index entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByNodeEntryDto {
    pub node_id: String,
    pub constraints: Vec<String>,
}

/// Discriminated union for constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConstraintDto {
    Range {
        target: ReferenceDto,
        lower: ExpressionDto,
        upper: ExpressionDto,
    },
    TypeDecl {
        target: ReferenceDto,
        expected: String,
    },
    LengthRelation {
        target: ReferenceDto,
        length: ExpressionDto,
    },
    Relation {
        lhs: ExpressionDto,
        op: String,
        rhs: ExpressionDto,
    },
    Distinct {
        elements: ReferenceDto,
        unit: String,
    },
    Property {
        target: ReferenceDto,
        tag: PropertyTagDto,
    },
    SumBound {
        variable: ReferenceDto,
        upper: ExpressionDto,
    },
    Sorted {
        elements: ReferenceDto,
        order: String,
    },
    Guarantee {
        description: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        predicate: Option<ExpressionDto>,
    },
    CharSet {
        target: ReferenceDto,
        charset: CharSetSpecDto,
    },
    StringLength {
        target: ReferenceDto,
        min: ExpressionDto,
        max: ExpressionDto,
    },
    RenderHint {
        target: ReferenceDto,
        hint: RenderHintKindDto,
    },
}

// ── Expressions ─────────────────────────────────────────────────────

/// Discriminated union for numeric expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ExpressionDto {
    Lit {
        value: i64,
    },
    Var {
        reference: ReferenceDto,
    },
    BinOp {
        op: String,
        lhs: Box<ExpressionDto>,
        rhs: Box<ExpressionDto>,
    },
    Pow {
        base: Box<ExpressionDto>,
        exp: Box<ExpressionDto>,
    },
    FnCall {
        name: String,
        args: Vec<ExpressionDto>,
    },
}

/// Discriminated union for references.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ReferenceDto {
    VariableRef {
        node_id: String,
    },
    IndexedRef {
        target: String,
        indices: Vec<String>,
    },
    Unresolved {
        name: String,
    },
}

/// Discriminated union for literal values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum LiteralDto {
    IntLit { value: i64 },
    StrLit { value: String },
}

// ── Small enums ─────────────────────────────────────────────────────

/// Property tag (discriminated union with Custom having a value).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PropertyTagDto {
    Simple,
    Connected,
    Tree,
    Permutation,
    Binary,
    Odd,
    Even,
    Custom { value: String },
}

/// Character set specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum CharSetSpecDto {
    LowerAlpha,
    UpperAlpha,
    Alpha,
    Digit,
    AlphaNumeric,
    Custom { chars: Vec<char> },
    Range { from: char, to: char },
}

/// Render hint kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RenderHintKindDto {
    Separator { value: String },
}

// ── Actions ─────────────────────────────────────────────────────────

/// Discriminated union for builder-layer actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action")]
pub enum ActionDto {
    FillHole {
        target: String,
        fill: FillContentDto,
    },
    ReplaceNode {
        target: String,
        replacement: FillContentDto,
    },
    AddConstraint {
        target: String,
        constraint: ConstraintDefDto,
    },
    RemoveConstraint {
        constraint_id: String,
    },
    IntroduceMultiTestCase {
        count_var_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sum_bound: Option<SumBoundDefDto>,
    },
    AddSlotElement {
        parent: String,
        slot_name: String,
        element: FillContentDto,
    },
    RemoveSlotElement {
        parent: String,
        slot_name: String,
        child: String,
    },
    AddSibling {
        target: String,
        element: FillContentDto,
    },
    AddChoiceVariant {
        choice: String,
        tag_value: LiteralDto,
        first_element: FillContentDto,
    },
}

/// High-level fill intent for hole filling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum FillContentDto {
    Scalar {
        name: String,
        typ: String,
    },
    Array {
        name: String,
        element_type: String,
        length: LengthSpecDto,
    },
    Repeat {
        count: LengthSpecDto,
    },
    Grid {
        name: String,
        rows: LengthSpecDto,
        cols: LengthSpecDto,
        cell_type: String,
    },
    Section {
        label: String,
    },
    OutputSingleValue {
        typ: String,
    },
    OutputYesNo,
    EdgeList {
        edge_count: LengthSpecDto,
    },
    WeightedEdgeList {
        edge_count: LengthSpecDto,
        weight_name: String,
        weight_type: String,
    },
    QueryList {
        query_count: LengthSpecDto,
    },
    MultiTestCaseTemplate {
        count: LengthSpecDto,
    },
    GridTemplate {
        name: String,
        rows: LengthSpecDto,
        cols: LengthSpecDto,
        cell_type: String,
    },
}

/// Length specification for arrays/grids.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum LengthSpecDto {
    Fixed { value: usize },
    RefVar { node_id: String },
    Expr { expr: String },
}

/// Constraint definition from the builder layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ConstraintDefDto {
    Range { lower: String, upper: String },
    TypeDecl { typ: String },
    Relation { op: String, rhs: String },
    Distinct,
    Sorted { order: String },
    Property { tag: String },
    SumBound { over_var: String, upper: String },
    CharSet { charset: CharSetSpecDto },
    StringLength { min: String, max: String },
    Guarantee { description: String },
}

/// Sum bound definition for multi-test-case introduction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SumBoundDefDto {
    pub bound_var: String,
    pub upper: String,
}

// ── FullProjection DTOs ─────────────────────────────────────────────

/// Rich projection of the entire AST for the editor UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullProjectionDto {
    pub nodes: Vec<ProjectedNodeDto>,
    pub structure_lines: Vec<StructureLineDto>,
    pub hotspots: Vec<HotspotDto>,
    pub constraints: ProjectedConstraintsDto,
    pub available_vars: Vec<ExprCandidateDto>,
    pub completeness: CompletenessSummaryDto,
}

/// A display row in the Structure pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureLineDto {
    pub depth: usize,
    pub nodes: Vec<ProjectedNodeDto>,
}

/// A projected node for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedNodeDto {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub is_hole: bool,
    pub edit: Option<NodeEditProjectionDto>,
}

/// Semantic edit metadata for a projected structure node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEditProjectionDto {
    pub kind: String,
    pub name: String,
    pub value_type: String,
    pub length_expr: Option<String>,
    pub allowed_kinds: Vec<String>,
    pub allowed_types: Vec<String>,
}

/// An insertion point in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotDto {
    pub parent_id: String,
    pub direction: String,
    pub candidates: Vec<String>,
    pub candidate_details: Vec<HoleCandidateDetailDto>,
    pub action: HotspotActionDto,
}

/// Declarative action routing for a projected hotspot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotActionDto {
    pub kind: String,
    pub target_id: String,
    pub slot_name: Option<String>,
}

/// Detailed field schema for a candidate shown in the node popup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoleCandidateDetailDto {
    pub kind: String,
    pub label: String,
    pub fields: Vec<CandidateFieldDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateFieldDto {
    pub name: String,
    pub field_type: String,
    pub label: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
    pub default_value: Option<String>,
}

/// Projected constraints split into draft and completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedConstraintsDto {
    pub items: Vec<ConstraintItemDto>,
    pub drafts: Vec<DraftConstraintDto>,
    pub completed: Vec<CompletedConstraintDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintItemDto {
    pub index: usize,
    pub status: String,
    pub target_id: String,
    pub target_name: String,
    pub display: String,
    pub template: Option<String>,
    pub constraint_id: Option<String>,
    pub draft_index: Option<usize>,
    pub completed_index: Option<usize>,
    pub edit: Option<ConstraintEditProjectionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConstraintEditProjectionDto {
    Range {
        lower: String,
        upper: String,
        constraint_id: Option<String>,
    },
    CharSet {
        charset: CharSetSpecDto,
        constraint_id: Option<String>,
    },
    StringLength {
        min: String,
        max: String,
        constraint_id: Option<String>,
    },
}

/// An unfilled constraint generated on-the-fly by projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftConstraintDto {
    pub index: usize,
    pub target_id: String,
    pub target_name: String,
    pub display: String,
    pub template: String,
}

/// A fully specified constraint from the AST.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedConstraintDto {
    pub index: usize,
    pub constraint_id: String,
    pub display: String,
}

/// A variable available for use in expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprCandidateDto {
    pub name: String,
    pub node_id: String,
    pub value_type: String,
    pub node_kind: String,
}

/// Summary of AST completeness and satisfaction status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessSummaryDto {
    pub total_holes: usize,
    pub filled_slots: usize,
    pub unsatisfied_constraints: usize,
    pub is_complete: bool,
}
