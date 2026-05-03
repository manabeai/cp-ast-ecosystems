//! High-level editing operations over an AST document.
//!
//! [`crate::operation::AstEngine`] owns both the structure and constraint arenas. Callers can
//! mutate documents by applying [`crate::operation::Action`] values, which keeps editor logic at
//! the domain level instead of editing arena nodes by hand.

/// User-facing edit actions.
pub mod action;
mod constraint_ops;
/// Engine that owns structure and constraint state.
pub mod engine;
/// Operation error and violation detail types.
pub mod error;
mod fill_hole;
mod multi_test_case;
mod node_ops;
/// Apply/preview result types.
pub mod result;
/// Action payload and constraint definition types.
pub mod types;

pub use action::Action;
pub use engine::AstEngine;
pub use error::{OperationError, ViolationDetail};
pub use result::{ApplyResult, PreviewResult};
pub use types::{ConstraintDef, ConstraintDefKind, FillContent, LengthSpec, SumBoundDef, VarType};
