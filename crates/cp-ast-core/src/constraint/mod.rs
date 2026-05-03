//! Constraint AST: semantic facts attached to structure nodes.
//!
//! Constraints describe facts that are not fully represented by shape alone,
//! such as value ranges, expected scalar types, array distinctness, sortedness,
//! graph-like properties, string character sets, and rendering hints. Store
//! constraints in [`crate::constraint::ConstraintSet`] and refer to nodes through
//! [`crate::structure::Reference`].

/// Constraint enum definitions.
#[allow(clippy::module_inception)]
pub mod constraint;
/// Stable constraint identifier type.
pub mod constraint_id;
/// Arena-backed constraint storage.
pub mod constraint_set;
/// Expected scalar value types.
pub mod expected_type;
/// Arithmetic expressions used by constraints.
pub mod expression;
/// Constraint helper enums.
pub mod types;

pub use constraint::Constraint;
pub use constraint_id::ConstraintId;
pub use constraint_set::ConstraintSet;
pub use expected_type::ExpectedType;
pub use expression::Expression;
pub use types::{
    ArithOp, CharSetSpec, DistinctUnit, PropertyTag, RelationOp, RenderHintKind, Separator,
    SortOrder,
};
