use crate::structure::NodeId;

/// Variable type for operation layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarType {
    /// Integer type.
    Int,
    /// String type.
    Str,
    /// Character type.
    Char,
}

/// Length specification for arrays/grids.
#[derive(Debug, Clone, PartialEq)]
pub enum LengthSpec {
    /// Fixed numeric length.
    Fixed(usize),
    /// Length referenced by another node.
    RefVar(NodeId),
    /// Length as a string expression.
    Expr(String),
}

/// High-level fill intent for hole filling.
#[derive(Debug, Clone, PartialEq)]
pub enum FillContent {
    /// Fill with a scalar variable.
    Scalar { name: String, typ: VarType },
    /// Fill with an array.
    Array {
        name: String,
        element_type: VarType,
        length: LengthSpec,
    },
    /// Fill with a generic repeat block.
    Repeat { count: LengthSpec },
    /// Fill with a 2D grid.
    Grid {
        name: String,
        rows: LengthSpec,
        cols: LengthSpec,
        cell_type: VarType,
    },
    /// Fill with a section (grouping).
    Section { label: String },
    /// Fill with a single output value.
    OutputSingleValue { typ: VarType },
    /// Fill with Yes/No output.
    OutputYesNo,
    /// Fill with an edge list (`u_i`, `v_i` pairs).
    EdgeList { edge_count: LengthSpec },
    /// Fill with a weighted edge list (`u_i`, `v_i`, `w_i` triples).
    WeightedEdgeList {
        edge_count: LengthSpec,
        weight_name: String,
        weight_type: VarType,
    },
    /// Fill with a query list (Choice inside Repeat).
    QueryList { query_count: LengthSpec },
    /// Fill with a multi-testcase repeat block (Repeat with Hole body).
    MultiTestCaseTemplate { count: LengthSpec },
    /// Fill with a grid template (Matrix node).
    GridTemplate {
        name: String,
        rows: LengthSpec,
        cols: LengthSpec,
        cell_type: VarType,
    },
}

/// Constraint definition from the builder layer.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintDef {
    /// The kind of constraint to add.
    pub kind: ConstraintDefKind,
}

/// Kinds of constraint definitions.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintDefKind {
    /// Value range constraint.
    Range { lower: String, upper: String },
    /// Type declaration.
    TypeDecl { typ: VarType },
    /// Relational constraint.
    Relation {
        op: crate::constraint::RelationOp,
        rhs: String,
    },
    /// All-distinct constraint.
    Distinct,
    /// Sorted order constraint.
    Sorted { order: crate::constraint::SortOrder },
    /// Structural property.
    Property { tag: String },
    /// Sum bound across test cases.
    SumBound { over_var: String, upper: String },
    /// Character set constraint.
    CharSet { spec: String },
    /// String length bounds.
    StringLength { min: String, max: String },
    /// Human-readable guarantee.
    Guarantee { description: String },
}

/// Sum bound definition for multi-test-case introduction.
#[derive(Debug, Clone, PartialEq)]
pub struct SumBoundDef {
    /// Variable whose sum is bounded.
    pub bound_var: String,
    /// Upper bound expression.
    pub upper: String,
}
