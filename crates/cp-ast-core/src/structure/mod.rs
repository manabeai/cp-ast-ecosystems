//! Structure AST: the shape of a competitive-programming input format.
//!
//! The structure tree stores nodes in an arena and refers to them by
//! [`crate::structure::NodeId`]. Use [`crate::structure::StructureAst`] for
//! storage, [`crate::structure::NodeKind`] for shape, and
//! [`crate::structure::Reference`] when constraints or expressions need to
//! point at nodes.

/// Stable structure node identifier type.
pub mod node_id;
/// Structure node variants.
pub mod node_kind;
/// References to nodes or unresolved names.
pub mod reference;
/// Arena-backed structure storage.
pub mod structure_ast;
/// Individual structure node wrapper.
pub mod structure_node;
/// Visitor API for tree inspection.
pub mod tree_visitor;
/// Identifiers, literals, and node-kind hints.
pub mod types;

pub use node_id::NodeId;
pub use node_kind::NodeKind;
pub use reference::Reference;
pub use structure_ast::StructureAst;
pub use structure_node::StructureNode;
pub use tree_visitor::{ChildEntry, DefaultTreeVisitor, NodeInfo, TreeVisitor, node_display_name};
pub use types::{Ident, Literal, NodeKindHint};
