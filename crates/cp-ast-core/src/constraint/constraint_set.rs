use super::constraint::Constraint;
use super::constraint_id::ConstraintId;
use crate::structure::NodeId;

/// Arena-based constraint set with `ConstraintId` addressing.
///
/// Rev.1 S-2: Constraints are identified by `ConstraintId` for precise
/// `RemoveConstraint` operations. Supports per-node and global constraints.
#[derive(Debug, Clone, Default)]
pub struct ConstraintSet {
    arena: Vec<Option<Constraint>>,
    by_node: Vec<(NodeId, Vec<ConstraintId>)>,
    global: Vec<ConstraintId>,
    next_id: u64,
}

impl ConstraintSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a constraint. If `target` is Some, it's per-node; if None, it's global.
    /// Returns the assigned `ConstraintId`.
    #[allow(clippy::cast_possible_truncation)]
    pub fn add(&mut self, target: Option<NodeId>, constraint: Constraint) -> ConstraintId {
        let id = ConstraintId::from_raw(self.next_id);
        self.next_id += 1;
        let idx = id.value() as usize;
        if idx >= self.arena.len() {
            self.arena.resize_with(idx + 1, || None);
        }
        self.arena[idx] = Some(constraint);
        match target {
            Some(node_id) => {
                if let Some(entry) = self.by_node.iter_mut().find(|(n, _)| *n == node_id) {
                    entry.1.push(id);
                } else {
                    self.by_node.push((node_id, vec![id]));
                }
            }
            None => {
                self.global.push(id);
            }
        }
        id
    }

    /// Remove a constraint by ID.
    #[allow(clippy::cast_possible_truncation)]
    pub fn remove(&mut self, id: ConstraintId) -> Option<Constraint> {
        let constraint = self.arena.get_mut(id.value() as usize)?.take()?;
        // Remove from by_node index
        for (_, ids) in &mut self.by_node {
            ids.retain(|cid| *cid != id);
        }
        // Remove from global
        self.global.retain(|cid| *cid != id);
        Some(constraint)
    }

    /// Get a constraint by ID.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get(&self, id: ConstraintId) -> Option<&Constraint> {
        self.arena.get(id.value() as usize)?.as_ref()
    }

    /// Get constraint IDs for a specific node.
    #[must_use]
    pub fn for_node(&self, node: NodeId) -> Vec<ConstraintId> {
        self.by_node
            .iter()
            .find(|(n, _)| *n == node)
            .map_or_else(Vec::new, |(_, ids)| ids.clone())
    }

    /// Get global constraint IDs.
    #[must_use]
    pub fn global(&self) -> &[ConstraintId] {
        &self.global
    }

    /// Iterate over (`NodeId`, `&[ConstraintId]`) pairs for all nodes with constraints.
    pub fn nodes_with_constraints(&self) -> impl Iterator<Item = (NodeId, &[ConstraintId])> {
        self.by_node
            .iter()
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(node_id, ids)| (*node_id, ids.as_slice()))
    }

    /// Count of live constraints.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arena.iter().filter(|c| c.is_some()).count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all live (`ConstraintId`, &Constraint) pairs.
    ///
    /// # Panics
    ///
    /// Panics if the arena index cannot be converted to u64 (only possible on systems
    /// where usize > u64, which is extremely unlikely).
    pub fn iter(&self) -> impl Iterator<Item = (ConstraintId, &Constraint)> {
        self.arena.iter().enumerate().filter_map(|(i, slot)| {
            slot.as_ref().map(|c| {
                (
                    ConstraintId::from_raw(u64::try_from(i).expect("index fits u64")),
                    c,
                )
            })
        })
    }

    /// Returns the next ID that will be assigned.
    #[must_use]
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    /// Returns a raw view of the arena including tombstone (`None`) slots.
    #[must_use]
    pub fn arena_raw(&self) -> &[Option<Constraint>] {
        &self.arena
    }

    /// Returns the raw per-node constraint index.
    #[must_use]
    pub fn by_node_raw(&self) -> &[(NodeId, Vec<ConstraintId>)] {
        &self.by_node
    }

    /// Reconstruct a `ConstraintSet` from raw parts.
    ///
    /// Used by deserialization layers for lossless arena restoration.
    #[must_use]
    pub fn from_raw_parts(
        arena: Vec<Option<Constraint>>,
        by_node: Vec<(NodeId, Vec<ConstraintId>)>,
        global: Vec<ConstraintId>,
        next_id: u64,
    ) -> Self {
        Self {
            arena,
            by_node,
            global,
            next_id,
        }
    }
}
