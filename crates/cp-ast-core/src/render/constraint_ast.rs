//! AST-structured representation of constraints for tree rendering.
//!
//! Unlike [`render_single_constraint`] which flattens to a text string,
//! [`constraint_to_tree`] preserves the internal tree structure so renderers
//! can display it as a proper ASCII tree.
//!
//! [`render_single_constraint`]: super::render_single_constraint

use crate::constraint::{ArithOp, Constraint, ExpectedType, Expression, RelationOp, SortOrder};
use crate::operation::AstEngine;

use super::{render_expression, render_reference};

/// A node in the constraint AST tree for rendering purposes.
///
/// Leaf nodes (no children) are displayed inline. Inner nodes display their
/// label and then recurse into children with box-drawing connectors.
#[derive(Debug, Clone)]
pub struct ConstraintNode {
    /// Display label for this node.
    pub label: String,
    /// Sub-nodes (empty for leaf nodes).
    pub children: Vec<ConstraintNode>,
}

impl ConstraintNode {
    fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: vec![],
        }
    }

    fn node(label: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            label: label.into(),
            children,
        }
    }
}

/// Convert a [`Constraint`] into a [`ConstraintNode`] tree for display.
///
/// Returns `None` for constraints that should not appear in the tree
/// (currently only [`Constraint::RenderHint`]).
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn constraint_to_tree(engine: &AstEngine, constraint: &Constraint) -> Option<ConstraintNode> {
    match constraint {
        Constraint::RenderHint { .. } => None,

        Constraint::Range {
            target,
            lower,
            upper,
        } => {
            let target_str = render_reference(engine, target);
            Some(ConstraintNode::node(
                "Range",
                vec![
                    ConstraintNode::leaf(format!("target: {target_str}")),
                    expr_field(engine, "lower", lower),
                    expr_field(engine, "upper", upper),
                ],
            ))
        }

        Constraint::TypeDecl { target, expected } => {
            let target_str = render_reference(engine, target);
            let type_str = match expected {
                ExpectedType::Int => "Int",
                ExpectedType::Str => "Str",
                ExpectedType::Char => "Char",
            };
            Some(ConstraintNode::node(
                "TypeDecl",
                vec![
                    ConstraintNode::leaf(format!("target: {target_str}")),
                    ConstraintNode::leaf(format!("type: {type_str}")),
                ],
            ))
        }

        Constraint::LengthRelation { target, length } => {
            let target_str = render_reference(engine, target);
            Some(ConstraintNode::node(
                "LengthRelation",
                vec![
                    ConstraintNode::leaf(format!("target: {target_str}")),
                    expr_field(engine, "length", length),
                ],
            ))
        }

        Constraint::Relation { lhs, op, rhs } => {
            let op_str = match op {
                RelationOp::Le => "≤",
                RelationOp::Lt => "<",
                RelationOp::Ge => "≥",
                RelationOp::Gt => ">",
                RelationOp::Eq => "=",
                RelationOp::Ne => "≠",
            };
            Some(ConstraintNode::node(
                format!("Relation({op_str})"),
                vec![
                    expr_field(engine, "lhs", lhs),
                    expr_field(engine, "rhs", rhs),
                ],
            ))
        }

        Constraint::Distinct { elements, unit } => {
            let elements_str = render_reference(engine, elements);
            Some(ConstraintNode::node(
                "Distinct",
                vec![
                    ConstraintNode::leaf(format!("elements: {elements_str}")),
                    ConstraintNode::leaf(format!("unit: {unit:?}")),
                ],
            ))
        }

        Constraint::Property { target, tag } => {
            let target_str = render_reference(engine, target);
            Some(ConstraintNode::node(
                "Property",
                vec![
                    ConstraintNode::leaf(format!("target: {target_str}")),
                    ConstraintNode::leaf(format!("tag: {tag:?}")),
                ],
            ))
        }

        Constraint::Sorted { elements, order } => {
            let elements_str = render_reference(engine, elements);
            let order_str = match order {
                SortOrder::Ascending => "Ascending",
                SortOrder::Descending => "Descending",
                SortOrder::NonDecreasing => "NonDecreasing",
                SortOrder::NonIncreasing => "NonIncreasing",
            };
            Some(ConstraintNode::node(
                "Sorted",
                vec![
                    ConstraintNode::leaf(format!("elements: {elements_str}")),
                    ConstraintNode::leaf(format!("order: {order_str}")),
                ],
            ))
        }

        Constraint::SumBound { variable, upper } => {
            let variable_str = render_reference(engine, variable);
            Some(ConstraintNode::node(
                "SumBound",
                vec![
                    ConstraintNode::leaf(format!("variable: {variable_str}")),
                    expr_field(engine, "upper", upper),
                ],
            ))
        }

        Constraint::Guarantee { description, .. } => {
            Some(ConstraintNode::leaf(format!("Guarantee: {description}")))
        }

        Constraint::CharSet { target, charset } => {
            let target_str = render_reference(engine, target);
            Some(ConstraintNode::node(
                "CharSet",
                vec![
                    ConstraintNode::leaf(format!("target: {target_str}")),
                    ConstraintNode::leaf(format!("chars: {charset}")),
                ],
            ))
        }

        Constraint::StringLength { target, min, max } => {
            let target_str = render_reference(engine, target);
            Some(ConstraintNode::node(
                "StringLength",
                vec![
                    ConstraintNode::leaf(format!("target: {target_str}")),
                    expr_field(engine, "min", min),
                    expr_field(engine, "max", max),
                ],
            ))
        }
    }
}

/// Convert an `Expression` into a `ConstraintNode` subtree.
fn expression_to_tree(engine: &AstEngine, expr: &Expression) -> ConstraintNode {
    match expr {
        Expression::Lit(n) => ConstraintNode::leaf(n.to_string()),
        Expression::Var(r) => ConstraintNode::leaf(render_reference(engine, r)),
        Expression::BinOp { op, lhs, rhs } => {
            let op_str = match op {
                ArithOp::Add => "Add",
                ArithOp::Sub => "Sub",
                ArithOp::Mul => "Mul",
                ArithOp::Div => "Div",
            };
            ConstraintNode::node(
                op_str,
                vec![
                    expression_to_tree(engine, lhs),
                    expression_to_tree(engine, rhs),
                ],
            )
        }
        Expression::Pow { base, exp } => ConstraintNode::node(
            "Pow",
            vec![
                expression_to_tree(engine, base),
                expression_to_tree(engine, exp),
            ],
        ),
        Expression::FnCall { name, args } => ConstraintNode::node(
            name.as_str().to_string(),
            args.iter().map(|a| expression_to_tree(engine, a)).collect(),
        ),
    }
}

/// Create a named field node for an expression.
///
/// Simple expressions (literal or single variable) are inlined into the label
/// for readability. Complex expressions (binary ops, pow, fn calls) are shown
/// as a nested subtree.
fn expr_field(engine: &AstEngine, field_name: &str, expr: &Expression) -> ConstraintNode {
    if is_simple_expr(expr) {
        ConstraintNode::leaf(format!("{field_name}: {}", render_expression(engine, expr)))
    } else {
        ConstraintNode::node(
            field_name.to_string(),
            vec![expression_to_tree(engine, expr)],
        )
    }
}

/// Returns `true` for expressions simple enough to render inline (no subtree).
fn is_simple_expr(expr: &Expression) -> bool {
    matches!(expr, Expression::Lit(_) | Expression::Var(_))
}
