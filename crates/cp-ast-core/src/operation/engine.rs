use super::action::Action;
use super::error::OperationError;
use super::result::{ApplyResult, PreviewResult};
use crate::constraint::ConstraintSet;
use crate::structure::StructureAst;

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

    /// Preview an action without applying it.
    ///
    /// # Errors
    /// Returns `OperationError` if the action is invalid.
    pub fn preview(&self, action: &Action) -> Result<PreviewResult, OperationError> {
        let _ = action;
        todo!("T-10")
    }
}

impl Default for AstEngine {
    fn default() -> Self {
        Self::new()
    }
}
