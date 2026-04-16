//! Projection API for read-only AST views.
//!
//! This module provides the `ProjectionAPI` trait which transforms
//! internal AST representations into UI-friendly formats.

pub mod api;
pub mod editor_projection;
mod projection_impl;
pub mod types;

// Re-export public types and traits
pub use api::ProjectionAPI;
pub use types::*;

// Re-export editor projection functions
pub use editor_projection::{
    get_constraint_targets, get_expr_candidates, get_hole_candidates, project_full,
    project_node_detail,
};
