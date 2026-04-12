use super::node_id::NodeId;
use super::node_kind::NodeKind;
use super::slot::Slot;

/// A node in the structure AST representing a part of a problem specification.
///
/// Each node has a unique ID, a kind, an optional name, and zero or more
/// named slots that hold child nodes or holes.
#[derive(Debug, Clone)]
pub struct StructureNode {
    id: NodeId,
    kind: NodeKind,
    name: Option<String>,
    slots: Vec<Slot>,
}

impl StructureNode {
    /// Create a new structure node of the given kind with a unique ID.
    #[must_use]
    pub fn new(kind: NodeKind) -> Self {
        Self {
            id: NodeId::new(),
            kind,
            name: None,
            slots: Vec::new(),
        }
    }

    /// Set the name of this node (builder pattern).
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Add a slot to this node (builder pattern).
    #[must_use]
    pub fn with_slot(mut self, slot: Slot) -> Self {
        self.slots.push(slot);
        self
    }

    /// Returns the unique ID of this node.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Returns the kind of this node.
    #[must_use]
    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns the name of this node, if set.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns a slice of this node's slots.
    #[must_use]
    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }
}
