use super::node_id::NodeId;
use super::node_kind::NodeKind;
use super::structure_node::StructureNode;

/// Arena-based structure AST.
///
/// Nodes are stored in a Vec indexed by `NodeId`. Insertion order is preserved
/// for deterministic canonical rendering (Rev.1 M-1).
#[derive(Debug, Clone)]
pub struct StructureAst {
    root: NodeId,
    arena: Vec<Option<StructureNode>>,
    next_id: u64,
}

impl StructureAst {
    /// Create a new AST with an empty Sequence as root.
    #[must_use]
    pub fn new() -> Self {
        let root_id = NodeId::from_raw(0);
        let root_node = StructureNode::new(
            root_id,
            NodeKind::Sequence {
                children: Vec::new(),
            },
        );
        Self {
            root: root_id,
            arena: vec![Some(root_node)],
            next_id: 1,
        }
    }

    /// Add a node to the arena and return its assigned `NodeId`.
    #[allow(clippy::cast_possible_truncation)] // NodeId values are controlled and expected to fit in usize
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId::from_raw(self.next_id);
        self.next_id += 1;
        let node = StructureNode::new(id, kind);
        let idx = id.value() as usize;
        if idx >= self.arena.len() {
            self.arena.resize_with(idx + 1, || None);
        }
        self.arena[idx] = Some(node);
        id
    }

    /// Get a reference to a node by ID.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // NodeId values are controlled and expected to fit in usize
    pub fn get(&self, id: NodeId) -> Option<&StructureNode> {
        self.arena
            .get(id.value() as usize)
            .and_then(|slot| slot.as_ref())
    }

    /// Get a mutable reference to a node by ID.
    #[allow(clippy::cast_possible_truncation)] // NodeId values are controlled and expected to fit in usize
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut StructureNode> {
        self.arena
            .get_mut(id.value() as usize)
            .and_then(|slot| slot.as_mut())
    }

    /// Remove a node from the arena, returning it if it existed.
    #[allow(clippy::cast_possible_truncation)] // NodeId values are controlled and expected to fit in usize
    pub fn remove(&mut self, id: NodeId) -> Option<StructureNode> {
        self.arena
            .get_mut(id.value() as usize)
            .and_then(Option::take)
    }

    /// Returns the root node ID.
    #[must_use]
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Set a new root node ID.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = id;
    }

    /// Check if a node exists in the arena.
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.get(id).is_some()
    }

    /// Returns the count of live (non-removed) nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arena.iter().filter(|s| s.is_some()).count()
    }

    /// Returns true if the arena has no live nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all live nodes in arena order.
    pub fn iter(&self) -> impl Iterator<Item = &StructureNode> {
        self.arena.iter().filter_map(|s| s.as_ref())
    }

    /// Returns the next ID that will be assigned.
    #[must_use]
    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

impl Default for StructureAst {
    fn default() -> Self {
        Self::new()
    }
}
