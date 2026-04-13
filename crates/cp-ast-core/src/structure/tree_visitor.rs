//! Visitor pattern for walking the structure AST.
//!
//! Provides `NodeInfo` (label + children) for each node, and `DefaultTreeVisitor`
//! which handles all `NodeKind` variants. External crates use the visitor
//! to avoid matching on `NodeKind` directly.

use crate::operation::AstEngine;
use crate::render::{render_expression, render_reference};
use crate::structure::{Literal, NodeId, NodeKind};

/// Display information for a single AST node.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// The node's unique ID.
    pub node_id: NodeId,
    /// Human-readable label (e.g. "Scalar(N)", "Repeat(count=M)").
    pub label: String,
    /// Ordered child node IDs for tree traversal.
    pub children: Vec<ChildEntry>,
}

/// A child entry in the tree — either a real node or a virtual grouping.
#[derive(Debug, Clone)]
pub enum ChildEntry {
    /// A real node in the `StructureAst` arena.
    Node(NodeId),
    /// A virtual grouping node (e.g. Choice variant) with its own label and children.
    Virtual {
        label: String,
        children: Vec<NodeId>,
    },
}

/// Trait for extracting display information from the structure AST.
///
/// Implement this to customize how nodes are labeled and traversed.
/// `DefaultTreeVisitor` handles all built-in `NodeKind` variants.
pub trait TreeVisitor {
    /// Extract display information for a node. Returns `None` if the node doesn't exist.
    fn node_info(&self, engine: &AstEngine, node_id: NodeId) -> Option<NodeInfo>;
}

/// Default visitor that handles all `NodeKind` variants.
///
/// When new variants are added to `NodeKind`, only this implementation
/// needs updating — external crates using `TreeVisitor` are unaffected.
#[derive(Debug, Clone, Copy)]
pub struct DefaultTreeVisitor;

impl TreeVisitor for DefaultTreeVisitor {
    fn node_info(&self, engine: &AstEngine, node_id: NodeId) -> Option<NodeInfo> {
        let node = engine.structure.get(node_id)?;
        let (label, children) = match node.kind() {
            NodeKind::Scalar { name } => (format!("Scalar({})", name.as_str()), Vec::new()),
            NodeKind::Array { name, length } => (
                format!(
                    "Array({}, len={})",
                    name.as_str(),
                    render_expression(engine, length)
                ),
                Vec::new(),
            ),
            NodeKind::Matrix { name, rows, cols } => (
                format!(
                    "Matrix({}, rows={}, cols={})",
                    name.as_str(),
                    render_reference(engine, rows),
                    render_reference(engine, cols)
                ),
                Vec::new(),
            ),
            NodeKind::Tuple { elements } => (
                "Tuple".to_owned(),
                elements.iter().map(|id| ChildEntry::Node(*id)).collect(),
            ),
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                let label = match index_var {
                    Some(var) => format!(
                        "Repeat(count={}, {})",
                        render_expression(engine, count),
                        var.as_str()
                    ),
                    None => format!("Repeat(count={})", render_expression(engine, count)),
                };
                (label, body.iter().map(|id| ChildEntry::Node(*id)).collect())
            }
            NodeKind::Section { header, body } => {
                let mut children: Vec<ChildEntry> = Vec::new();
                if let Some(h) = header {
                    children.push(ChildEntry::Node(*h));
                }
                children.extend(body.iter().map(|id| ChildEntry::Node(*id)));
                ("Section".to_owned(), children)
            }
            NodeKind::Sequence { children: kids } => (
                "Sequence".to_owned(),
                kids.iter().map(|id| ChildEntry::Node(*id)).collect(),
            ),
            NodeKind::Choice { tag, variants } => {
                let label = format!("Choice(tag={})", render_reference(engine, tag));
                let children = variants
                    .iter()
                    .map(|(lit, kids)| {
                        let variant_label = match lit {
                            Literal::IntLit(v) => format!("Variant({v})"),
                            Literal::StrLit(s) => format!("Variant(\"{s}\")"),
                        };
                        ChildEntry::Virtual {
                            label: variant_label,
                            children: kids.clone(),
                        }
                    })
                    .collect();
                (label, children)
            }
            NodeKind::Hole { expected_kind } => {
                let label = match expected_kind {
                    Some(hint) => format!("Hole(expected={hint:?})"),
                    None => "Hole".to_owned(),
                };
                (label, Vec::new())
            }
        };
        Some(NodeInfo {
            node_id,
            label,
            children,
        })
    }
}

/// Get a short display name for a node (just the name, no variant prefix).
///
/// Returns the variable name for named nodes (Scalar/Array/Matrix),
/// or the kind name for structural nodes (Tuple/Repeat/etc.).
/// External crates use this instead of matching on `NodeKind` directly.
#[must_use]
pub fn node_display_name(engine: &AstEngine, node_id: NodeId) -> String {
    engine.structure.get(node_id).map_or_else(
        || format!("#{}", node_id.value()),
        |node| match node.kind() {
            NodeKind::Scalar { name }
            | NodeKind::Array { name, .. }
            | NodeKind::Matrix { name, .. } => name.as_str().to_owned(),
            NodeKind::Tuple { .. } => "Tuple".to_owned(),
            NodeKind::Repeat { .. } => "Repeat".to_owned(),
            NodeKind::Section { .. } => "Section".to_owned(),
            NodeKind::Sequence { .. } => "Sequence".to_owned(),
            NodeKind::Choice { .. } => "Choice".to_owned(),
            NodeKind::Hole { .. } => "Hole".to_owned(),
        },
    )
}
