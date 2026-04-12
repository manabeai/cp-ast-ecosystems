//! Projection API for read-only AST views.
//!
//! This module provides the `ProjectionAPI` trait which transforms
//! internal AST representations into UI-friendly formats.

pub mod api;
mod projection_impl;
pub mod types;

// Re-export public types and traits
pub use api::ProjectionAPI;
pub use types::*;
