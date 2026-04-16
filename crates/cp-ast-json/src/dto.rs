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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// ── Editor Projection ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullProjectionDto {
    pub outline: Vec<OutlineNodeDto>,
    pub diagnostics: Vec<DiagnosticDto>,
    pub completeness: CompletenessInfoDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineNodeDto {
    pub id: String,
    pub label: String,
    pub kind_label: String,
    pub depth: usize,
    pub is_hole: bool,
    pub child_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticDto {
    pub level: String, // "error", "warning", "info"
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessInfoDto {
    pub total_holes: usize,
    pub is_complete: bool,
    pub missing_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetailProjectionDto {
    pub slots: Vec<SlotInfoDto>,
    pub related_constraints: Vec<ConstraintSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfoDto {
    pub kind: String, // "ArrayLength", "RepeatCount", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_expr: Option<String>,
    pub is_editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSummaryDto {
    pub id: String,
    pub label: String,
    pub kind_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoleCandidateDto {
    pub kind: String,
    pub suggested_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprCandidateMenuDto {
    pub references: Vec<ReferenceCandidateDto>,
    pub literals: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCandidateDto {
    pub node_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintTargetMenuDto {
    pub targets: Vec<ReferenceCandidateDto>,
}

// ── SlotKind / SlotId ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotIdDto {
    pub owner: String,
    pub kind: String, // "ArrayLength" etc.
}

// ── Action ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
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
    SetExpr {
        slot: SlotIdDto,
        expr: ExpressionDto,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LengthSpecDto {
    Fixed { value: usize },
    RefVar { node_id: String },
    Expr { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintDefDto {
    pub kind: ConstraintDefKindDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConstraintDefKindDto {
    Range { lower: String, upper: String },
    TypeDecl { typ: String },
    Relation { op: String, rhs: String },
    Distinct,
    Sorted { order: String },
    Property { tag: String },
    SumBound { over_var: String, upper: String },
    Guarantee { description: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SumBoundDefDto {
    pub bound_var: String,
    pub upper: String,
}

// ── OperationError ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum OperationErrorDto {
    TypeMismatch {
        expected: String,
        actual: String,
        context: String,
    },
    NodeNotFound {
        node_id: String,
    },
    SlotOccupied {
        node_id: String,
        current_occupant: String,
    },
    ConstraintViolation {
        violations: Vec<ViolationDetailDto>,
    },
    InvalidOperation {
        action: String,
        reason: String,
    },
    InvalidFill {
        reason: String,
    },
    DeserializationError {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationDetailDto {
    pub constraint_id: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

// ── Operation Result ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResultDto {
    pub created_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub created_constraints: Vec<String>,
    pub affected_constraints: Vec<String>,
}
