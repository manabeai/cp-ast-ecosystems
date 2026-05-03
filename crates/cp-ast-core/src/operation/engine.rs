use super::action::Action;
use super::error::OperationError;
use super::result::{ApplyResult, PreviewResult};
use crate::constraint::{Constraint, ConstraintSet, Expression};
use crate::structure::{NodeId, NodeKind, Reference, StructureAst, StructureNode};

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
            Action::AddSibling { target, element } => self.add_sibling(*target, element),
            Action::AddChoiceVariant {
                choice,
                tag_value,
                first_element,
            } => self.add_choice_variant(*choice, tag_value, first_element),
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

impl AstEngine {
    /// Resolve `Unresolved` variable name references in a structure node's expressions.
    ///
    /// Looks up names like "N" in the structure and replaces them with `VariableRef(node_id)`.
    /// Handles expressions in Array length, Repeat count, and references in Matrix rows/cols.
    pub(crate) fn resolve_structure_references(&mut self, node_id: NodeId) {
        let Some(node) = self.structure.get(node_id) else {
            return;
        };
        let kind = node.kind().clone();
        match kind {
            NodeKind::Array { name, mut length } => {
                Self::resolve_expr_refs(&self.structure, node_id, &mut length);
                if let Some(n) = self.structure.get_mut(node_id) {
                    n.set_kind(NodeKind::Array { name, length });
                }
            }
            NodeKind::Matrix {
                name,
                mut rows,
                mut cols,
            } => {
                Self::resolve_ref(&self.structure, node_id, &mut rows);
                Self::resolve_ref(&self.structure, node_id, &mut cols);
                if let Some(n) = self.structure.get_mut(node_id) {
                    n.set_kind(NodeKind::Matrix { name, rows, cols });
                }
            }
            NodeKind::Repeat {
                mut count,
                index_var,
                body,
            } => {
                Self::resolve_expr_refs(&self.structure, node_id, &mut count);
                if let Some(n) = self.structure.get_mut(node_id) {
                    n.set_kind(NodeKind::Repeat {
                        count,
                        index_var,
                        body,
                    });
                }
            }
            _ => {}
        }
    }

    /// Resolve all unresolved references in structure expressions and constraints.
    ///
    /// This is useful after deserialization, where historical share links may
    /// contain symbolic expressions that predate structured `FnCall` encoding.
    pub fn resolve_all_references(&mut self) {
        let node_ids: Vec<NodeId> = self.structure.iter().map(StructureNode::id).collect();
        for node_id in node_ids {
            self.resolve_structure_references(node_id);
        }

        let structure = &self.structure;
        for (_, constraint) in self.constraints.iter_mut() {
            Self::resolve_constraint_refs(structure, constraint);
        }
    }

    /// Resolve Unresolved names in a `Reference` against the structure.
    fn resolve_ref(structure: &StructureAst, _owner: NodeId, reference: &mut Reference) {
        if let Reference::Unresolved(name) = reference {
            if let Some(target_id) = Self::find_node_by_name_static(structure, name.as_str()) {
                *reference = Reference::VariableRef(target_id);
            }
        }
    }

    /// Resolve Unresolved names in an `Expression` against the structure.
    fn resolve_expr_refs(structure: &StructureAst, owner: NodeId, expr: &mut Expression) {
        match expr {
            Expression::Var(reference) => {
                Self::resolve_ref(structure, owner, reference);
            }
            Expression::BinOp { lhs, rhs, .. } => {
                Self::resolve_expr_refs(structure, owner, lhs);
                Self::resolve_expr_refs(structure, owner, rhs);
            }
            Expression::Pow { base, exp } => {
                Self::resolve_expr_refs(structure, owner, base);
                Self::resolve_expr_refs(structure, owner, exp);
            }
            Expression::FnCall { args, .. } => {
                for arg in args {
                    Self::resolve_expr_refs(structure, owner, arg);
                }
            }
            Expression::Lit(_) => {}
        }
    }

    fn resolve_constraint_refs(structure: &StructureAst, constraint: &mut Constraint) {
        match constraint {
            Constraint::Range { lower, upper, .. } => {
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), lower);
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), upper);
            }
            Constraint::SumBound { upper, .. } => {
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), upper);
            }
            Constraint::LengthRelation { length, .. } => {
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), length);
            }
            Constraint::Relation { lhs, rhs, .. } => {
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), lhs);
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), rhs);
            }
            Constraint::StringLength { min, max, .. } => {
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), min);
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), max);
            }
            Constraint::Guarantee {
                predicate: Some(expr),
                ..
            } => {
                Self::resolve_expr_refs(structure, NodeId::from_raw(0), expr);
            }
            Constraint::TypeDecl { .. }
            | Constraint::Distinct { .. }
            | Constraint::Property { .. }
            | Constraint::Sorted { .. }
            | Constraint::CharSet { .. }
            | Constraint::RenderHint { .. }
            | Constraint::Guarantee {
                predicate: None, ..
            } => {}
        }
    }

    /// Find a structure node by its variable name.
    fn find_node_by_name_static(structure: &StructureAst, name: &str) -> Option<NodeId> {
        for node in structure.iter() {
            let node_name = match node.kind() {
                NodeKind::Scalar { name }
                | NodeKind::Array { name, .. }
                | NodeKind::Matrix { name, .. } => Some(name.as_str()),
                _ => None,
            };
            if node_name == Some(name) {
                return Some(node.id());
            }
        }
        None
    }
}
