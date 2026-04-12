use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Stable, unique identifier for AST nodes.
///
/// Each `NodeId` is globally unique within a process lifetime.
/// Used for node identification, reference resolution, and diff comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    /// Create a new unique `NodeId`.
    #[must_use]
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Create a `NodeId` from a raw value.
    /// Used by arenas that manage their own ID allocation.
    #[must_use]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw numeric value of this ID.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}
