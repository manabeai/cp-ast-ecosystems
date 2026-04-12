use super::expected_type::ExpectedType;
use super::expression::Expression;
use super::types::{CharSetSpec, DistinctUnit, PropertyTag, RelationOp, RenderHintKind, SortOrder};
use crate::structure::Reference;

/// A constraint on the structure AST.
///
/// Rev.1: 12 variants covering all competitive programming constraint patterns.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Value range: lower ≤ target ≤ upper.
    Range {
        target: Reference,
        lower: Expression,
        upper: Expression,
    },
    /// Type declaration (single source of truth per S-1).
    TypeDecl {
        target: Reference,
        expected: ExpectedType,
    },
    /// Length relation: len(target) = length.
    LengthRelation {
        target: Reference,
        length: Expression,
    },
    /// Variable relation: lhs op rhs.
    Relation {
        lhs: Expression,
        op: RelationOp,
        rhs: Expression,
    },
    /// All elements are distinct.
    Distinct {
        elements: Reference,
        unit: DistinctUnit,
    },
    /// Structural property (graph/array).
    Property { target: Reference, tag: PropertyTag },
    /// Sum bound across test cases.
    SumBound {
        variable: Reference,
        upper: Expression,
    },
    /// Elements are sorted.
    Sorted {
        elements: Reference,
        order: SortOrder,
    },
    /// Existence/validity guarantee (human-readable).
    Guarantee {
        description: String,
        predicate: Option<Expression>,
    },
    /// Character set constraint for strings.
    CharSet {
        target: Reference,
        charset: CharSetSpec,
    },
    /// String length constraint.
    StringLength {
        target: Reference,
        min: Expression,
        max: Expression,
    },
    /// Rendering hint (separator, moved from `StructureAST` per S-1).
    RenderHint {
        target: Reference,
        hint: RenderHintKind,
    },
}
