use super::constraint::Constraint;
use crate::structure::NodeId;

/// A composable set of constraints on the structure AST.
///
/// Constraints can be queried by target node to determine what
/// is allowed at each position.
#[derive(Debug, Clone, Default)]
pub struct ConstraintSet {
    constraints: Vec<Constraint>,
}

impl ConstraintSet {
    /// Create an empty constraint set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a constraint to the set.
    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Returns the number of constraints in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    /// Returns `true` if the constraint set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    /// Returns an iterator over constraints targeting the given node.
    pub fn for_target(&self, target: NodeId) -> impl Iterator<Item = &Constraint> {
        self.constraints
            .iter()
            .filter(move |c| c.target() == target)
    }

    /// Returns an iterator over all constraints.
    pub fn iter(&self) -> impl Iterator<Item = &Constraint> {
        self.constraints.iter()
    }
}
