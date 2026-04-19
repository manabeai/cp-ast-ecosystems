use super::engine::AstEngine;
use super::error::OperationError;
use super::result::ApplyResult;
use super::types::{ConstraintDef, ConstraintDefKind, VarType};
use crate::constraint::Expression;
use crate::constraint::{CharSetSpec, Constraint, ConstraintId, ExpectedType};
use crate::structure::{Ident, NodeId, NodeKind, Reference, StructureAst};

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
        let mut constraint = convert_def_to_constraint(target, &constraint_def.kind);

        // 3. Resolve any Unresolved variable names against the structure
        resolve_constraint_references(&self.structure, &mut constraint);

        // 4. Add to ConstraintSet
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
        ConstraintDefKind::CharSet { spec } => Constraint::CharSet {
            target: target_ref,
            charset: parse_charset_spec(spec),
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

// ── Reference resolution ───────────────────────────────────────────

/// Resolve `Unresolved` variable names in a constraint against the structure.
fn resolve_constraint_references(structure: &StructureAst, constraint: &mut Constraint) {
    match constraint {
        Constraint::Range { lower, upper, .. } => {
            resolve_expression_references(structure, lower);
            resolve_expression_references(structure, upper);
        }
        Constraint::SumBound { upper, .. } => {
            resolve_expression_references(structure, upper);
        }
        Constraint::Relation { lhs, rhs, .. } => {
            resolve_expression_references(structure, lhs);
            resolve_expression_references(structure, rhs);
        }
        _ => {}
    }
}

/// Recursively resolve `Unresolved` variable names in an expression.
fn resolve_expression_references(structure: &StructureAst, expr: &mut Expression) {
    match expr {
        Expression::Var(Reference::Unresolved(name)) => {
            if let Some(node_id) = find_node_by_name(structure, name.as_str()) {
                *expr = Expression::Var(Reference::VariableRef(node_id));
            }
        }
        Expression::BinOp { lhs, rhs, .. } => {
            resolve_expression_references(structure, lhs);
            resolve_expression_references(structure, rhs);
        }
        Expression::Pow { base, exp } => {
            resolve_expression_references(structure, base);
            resolve_expression_references(structure, exp);
        }
        Expression::FnCall { args, .. } => {
            for arg in args {
                resolve_expression_references(structure, arg);
            }
        }
        _ => {}
    }
}

/// Find a structure node by its variable name (Scalar, Array, or Matrix).
fn find_node_by_name(structure: &StructureAst, name: &str) -> Option<NodeId> {
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

/// Parse a charset spec string into `CharSetSpec`.
fn parse_charset_spec(spec: &str) -> CharSetSpec {
    match spec {
        "LowerAlpha" => CharSetSpec::LowerAlpha,
        "UpperAlpha" => CharSetSpec::UpperAlpha,
        "Alpha" => CharSetSpec::Alpha,
        "Digit" => CharSetSpec::Digit,
        "AlphaNumeric" => CharSetSpec::AlphaNumeric,
        _ => {
            // Handle "Custom:abc" format from frontend
            if let Some(chars) = spec.strip_prefix("Custom:") {
                CharSetSpec::Custom(chars.chars().collect())
            } else {
                CharSetSpec::Custom(spec.chars().collect())
            }
        }
    }
}
