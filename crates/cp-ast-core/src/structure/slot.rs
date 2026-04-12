use super::node_id::NodeId;
use super::structure_node::StructureNode;

/// Information about an unfilled position in the AST.
///
/// A hole represents a position that has not yet been filled.
/// The semantic expectations for this position (expected type, allowed
/// node kinds) are held by the `ConstraintAST`, not here.
#[derive(Debug, Clone)]
pub struct HoleInfo {
    id: NodeId,
}

impl HoleInfo {
    /// Create a new hole with a unique ID.
    #[must_use]
    pub fn new() -> Self {
        Self { id: NodeId::new() }
    }

    /// Returns the unique ID of this hole.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }
}

impl Default for HoleInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// The value held by a slot: either a filled node or a hole.
#[derive(Debug, Clone)]
pub enum SlotValue {
    /// A filled position containing a structure node.
    Filled(StructureNode),
    /// An unfilled position (hole).
    Hole(HoleInfo),
}

/// A named position within a structure node.
///
/// Slots give semantic meaning to parent-child relationships.
/// For example, an `Array` node might have slots `"element_type"` and `"length"`.
#[derive(Debug, Clone)]
pub struct Slot {
    name: String,
    value: SlotValue,
}

impl Slot {
    /// Create a slot filled with a structure node.
    #[must_use]
    pub fn filled(name: &str, node: StructureNode) -> Self {
        Self {
            name: name.to_owned(),
            value: SlotValue::Filled(node),
        }
    }

    /// Create a slot with a hole (unfilled position).
    #[must_use]
    pub fn hole(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: SlotValue::Hole(HoleInfo::new()),
        }
    }

    /// Returns the name of this slot.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the slot's value.
    #[must_use]
    pub fn value(&self) -> &SlotValue {
        &self.value
    }

    /// Returns `true` if this slot is filled.
    #[must_use]
    pub fn is_filled(&self) -> bool {
        matches!(self.value, SlotValue::Filled(_))
    }

    /// Returns `true` if this slot is a hole.
    #[must_use]
    pub fn is_hole(&self) -> bool {
        matches!(self.value, SlotValue::Hole(_))
    }
}
