//! Projection API for read-only AST views.
//!
//! This module provides the `ProjectionAPI` trait which transforms
//! internal AST representations into UI-friendly formats.

pub mod api;
pub mod full_projection;
mod projection_impl;
pub mod types;

// Re-export public types and traits
pub use api::ProjectionAPI;
pub use full_projection::project_full;
pub use types::*;
