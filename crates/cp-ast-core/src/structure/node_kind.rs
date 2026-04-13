use super::node_id::NodeId;
use super::reference::Reference;
use super::types::{Ident, Literal, NodeKindHint};
use crate::constraint::Expression;

/// The kind of structure node in a competitive programming problem specification.
///
/// Rev.1: Rich variants with embedded data. Type and separator info
/// moved to `ConstraintAST` (`TypeDecl`, `RenderHint`).
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// Single variable: N, M, S, etc.
    Scalar { name: Ident },
    /// 1D array: `A_1` ... `A_N`.
    Array { name: Ident, length: Expression },
    /// 2D grid: C[i][j], A_{i,j}.
    Matrix {
        name: Ident,
        rows: Reference,
        cols: Reference,
    },
    /// Same-line variable group: (N, M, K), (`u_i`, `v_i`).
    Tuple { elements: Vec<NodeId> },
    /// Variable-dependent repetition: M lines, T test cases.
    Repeat {
        count: Expression,
        index_var: Option<Ident>,
        body: Vec<NodeId>,
    },
    /// Semantically delimited block: header + body.
    Section {
        header: Option<NodeId>,
        body: Vec<NodeId>,
    },
    /// Ordered root of the entire input.
    Sequence { children: Vec<NodeId> },
    /// Tag-dependent branching (query type variants).
    Choice {
        tag: Reference,
        variants: Vec<(Literal, Vec<NodeId>)>,
    },
    /// Unfilled position (first-class hole).
    Hole { expected_kind: Option<NodeKindHint> },
}
