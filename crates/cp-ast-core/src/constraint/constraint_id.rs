/// Unique identifier for a constraint in the `ConstraintSet`.
///
/// Used by `RemoveConstraint` and `ViolationDetail` for precise constraint addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintId(u64);

impl ConstraintId {
    /// Create a `ConstraintId` from a raw value.
    #[must_use]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw numeric value.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}
