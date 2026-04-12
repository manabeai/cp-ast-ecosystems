use super::engine::AstEngine;
use super::error::OperationError;
use super::result::ApplyResult;
use super::types::{ConstraintDef, ConstraintDefKind, VarType};
use crate::constraint::Expression;
use crate::constraint::{Constraint, ConstraintId, ExpectedType};
use crate::structure::{Ident, NodeId, Reference};

impl AstEngine {
    /// Add a constraint to a node.
    ///
    /// # Errors
    /// Returns `OperationError` if the target node doesn't exist.
    pub(crate) fn add_constraint_op(
        &mut self,
        target: NodeId,
        constraint_def: &ConstraintDef,
    ) -> Result<ApplyResult, OperationError> {
        // 1. Verify target exists
        if !self.structure.contains(target) {
            return Err(OperationError::NodeNotFound { node: target });
        }

        // 2. Convert ConstraintDefKind → Constraint
        let constraint = convert_def_to_constraint(target, &constraint_def.kind);

        // 3. Add to ConstraintSet
        let cid = self.constraints.add(Some(target), constraint);

        Ok(ApplyResult {
            created_nodes: vec![],
            removed_nodes: vec![],
            created_constraints: vec![cid],
            affected_constraints: vec![],
        })
    }

    /// Remove a constraint by ID.
    ///
    /// # Errors
    /// Returns `OperationError` if the constraint doesn't exist.
    pub(crate) fn remove_constraint_op(
        &mut self,
        constraint_id: ConstraintId,
    ) -> Result<ApplyResult, OperationError> {
        // 1. Verify constraint exists
        if self.constraints.get(constraint_id).is_none() {
            return Err(OperationError::InvalidOperation {
                action: "RemoveConstraint".to_owned(),
                reason: format!("Constraint {constraint_id:?} not found"),
            });
        }

        // 2. Remove
        self.constraints.remove(constraint_id);

        Ok(ApplyResult {
            created_nodes: vec![],
            removed_nodes: vec![],
            created_constraints: vec![],
            affected_constraints: vec![constraint_id],
        })
    }
}

fn convert_def_to_constraint(target: NodeId, kind: &ConstraintDefKind) -> Constraint {
    let target_ref = Reference::VariableRef(target);
    match kind {
        ConstraintDefKind::Range { lower, upper } => Constraint::Range {
            target: target_ref,
            lower: parse_expression(lower),
            upper: parse_expression(upper),
        },
        ConstraintDefKind::TypeDecl { typ } => Constraint::TypeDecl {
            target: target_ref,
            expected: match typ {
                VarType::Int => ExpectedType::Int,
                VarType::Str => ExpectedType::Str,
                VarType::Char => ExpectedType::Char,
            },
        },
        ConstraintDefKind::Relation { op, rhs } => Constraint::Relation {
            lhs: Expression::Var(target_ref),
            op: *op,
            rhs: parse_expression(rhs),
        },
        ConstraintDefKind::Distinct => Constraint::Distinct {
            elements: target_ref,
            unit: crate::constraint::DistinctUnit::Element,
        },
        ConstraintDefKind::Sorted { order } => Constraint::Sorted {
            elements: target_ref,
            order: *order,
        },
        ConstraintDefKind::Property { tag } => Constraint::Property {
            target: target_ref,
            tag: crate::constraint::PropertyTag::Custom(tag.clone()),
        },
        ConstraintDefKind::SumBound { upper, .. } => Constraint::SumBound {
            variable: target_ref,
            upper: parse_expression(upper),
        },
        ConstraintDefKind::Guarantee { description } => Constraint::Guarantee {
            description: description.clone(),
            predicate: None,
        },
    }
}

/// Simple expression parser: try to parse as i64, otherwise treat as unresolved reference.
pub(super) fn parse_expression(s: &str) -> Expression {
    if let Ok(n) = s.parse::<i64>() {
        Expression::Lit(n)
    } else {
        Expression::Var(Reference::Unresolved(Ident::new(s)))
    }
}
