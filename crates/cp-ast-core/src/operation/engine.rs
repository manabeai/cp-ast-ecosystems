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
            Action::FillHole { .. } => todo!("T-08"),
            Action::AddConstraint { .. } => todo!("T-09"),
            Action::RemoveConstraint { .. } => todo!("T-09"),
            Action::ReplaceNode { .. } => todo!("T-09"),
            Action::AddSlotElement { .. } => todo!("T-09"),
            Action::RemoveSlotElement { .. } => todo!("T-09"),
            Action::IntroduceMultiTestCase { .. } => todo!("T-09"),
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
