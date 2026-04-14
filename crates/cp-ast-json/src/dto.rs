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
