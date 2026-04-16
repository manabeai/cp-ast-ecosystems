use crate::structure::NodeId;

/// Expression slot type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotKind {
    /// Array.length
    ArrayLength,
    /// Repeat.count
    RepeatCount,
    /// Range.lower
    RangeLower,
    /// Range.upper
    RangeUpper,
    /// Relation.lhs
    RelationLhs,
    /// Relation.rhs
    RelationRhs,
    /// LengthRelation.length
    LengthLength,
}

impl SlotKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArrayLength => "ArrayLength",
            Self::RepeatCount => "RepeatCount",
            Self::RangeLower => "RangeLower",
            Self::RangeUpper => "RangeUpper",
            Self::RelationLhs => "RelationLhs",
            Self::RelationRhs => "RelationRhs",
            Self::LengthLength => "LengthLength",
        }
    }
}

impl std::fmt::Display for SlotKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Identifies a specific expression slot on a node or constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlotId {
    /// The owning node ID.
    pub owner: NodeId,
    /// Which slot within the owner.
    pub kind: SlotKind,
}
