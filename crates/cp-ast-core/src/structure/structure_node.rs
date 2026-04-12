use super::node_id::NodeId;
use super::node_kind::NodeKind;

/// A node in the structure AST.
///
/// Rev.1: Simplified to just id + kind. Name and structural data
/// are embedded in `NodeKind` variants. Parent-child relationships
/// are managed by the `StructureAst` arena.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureNode {
    id: NodeId,
    kind: NodeKind,
}

impl StructureNode {
    /// Create a new structure node.
    #[must_use]
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self { id, kind }
    }

    /// Returns the unique ID of this node.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Returns a reference to the kind of this node.
    #[must_use]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Replace the kind of this node (used by `FillHole`).
    pub fn set_kind(&mut self, kind: NodeKind) {
        self.kind = kind;
    }
}
