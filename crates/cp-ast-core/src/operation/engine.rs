use super::action::Action;
use super::error::OperationError;
use super::result::{ApplyResult, PreviewResult};
use crate::constraint::ConstraintSet;
use crate::structure::{NodeKind, StructureAst};

/// The main AST engine that owns both Structure and Constraint data.
///
/// Provides `apply()` to execute actions and `preview()` to dry-run them.
#[derive(Debug, Clone)]
pub struct AstEngine {
    /// The structure AST.
    pub structure: StructureAst,
    /// The constraint set.
    pub constraints: ConstraintSet,
}

impl AstEngine {
    /// Create a new engine with empty structure and constraints.
    #[must_use]
    pub fn new() -> Self {
        Self {
            structure: StructureAst::new(),
            constraints: ConstraintSet::new(),
        }
    }

    /// Apply an action to the AST, returning the result or an error.
    ///
    /// # Errors
    /// Returns `OperationError` if the action cannot be applied.
    pub fn apply(&mut self, action: &Action) -> Result<ApplyResult, OperationError> {
        match action {
            Action::FillHole { target, fill } => self.fill_hole(*target, fill),
            Action::AddConstraint { target, constraint } => {
                self.add_constraint_op(*target, constraint)
            }
            Action::RemoveConstraint { constraint_id } => self.remove_constraint_op(*constraint_id),
            Action::ReplaceNode {
                target,
                replacement,
            } => self.replace_node(*target, replacement),
            Action::AddSlotElement {
                parent,
                slot_name,
                element,
            } => self.add_slot_element(*parent, slot_name, element),
            Action::RemoveSlotElement {
                parent,
                slot_name,
                child,
            } => self.remove_slot_element(*parent, slot_name, *child),
            Action::IntroduceMultiTestCase {
                count_var_name,
                sum_bound,
            } => self.introduce_multi_test_case(count_var_name, sum_bound.as_ref()),
        }
    }

    /// Preview an action without applying it (dry-run).
    ///
    /// Clones `self`, applies the action on the clone, and derives what
    /// *would* happen — new holes created and constraints affected —
    /// without mutating the original engine.
    ///
    /// # Errors
    /// Returns `OperationError` if the action is invalid.
    pub fn preview(&self, action: &Action) -> Result<PreviewResult, OperationError> {
        let mut clone = self.clone();
        let result = clone.apply(action)?;

        // Holes created: nodes that were created AND are Hole kind in the clone.
        let new_holes_created = result
            .created_nodes
            .iter()
            .copied()
            .filter(|&id| {
                clone
                    .structure
                    .get(id)
                    .is_some_and(|n| matches!(n.kind(), NodeKind::Hole { .. }))
            })
            .collect();

        // Constraints affected: union of created + affected from ApplyResult.
        let mut constraints_affected = result.created_constraints;
        constraints_affected.extend(result.affected_constraints);

        Ok(PreviewResult {
            new_holes_created,
            constraints_affected,
        })
    }
}

impl Default for AstEngine {
    fn default() -> Self {
        Self::new()
    }
}
