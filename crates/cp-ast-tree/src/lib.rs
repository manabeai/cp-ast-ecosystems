//! ASCII tree renderer for inspecting `cp-ast-core` ASTs.
//!
//! Provides three rendering modes:
//! - `render_structure_tree` — structure AST only
//! - `render_constraint_tree` — constraints grouped by target node
//! - `render_combined_tree` — structure tree with inline constraint annotations

pub mod constraint_tree;
mod drawing;
pub mod structure_tree;

pub use constraint_tree::render_constraint_tree;
pub use structure_tree::render_structure_tree;

/// Options controlling tree rendering output.
#[derive(Debug, Clone, Default)]
pub struct TreeOptions {
    /// Show `NodeId` next to each label (e.g. "#3 Scalar(N)").
    pub show_node_ids: bool,
    /// Show `ConstraintId` next to each constraint line.
    pub show_constraint_ids: bool,
}
