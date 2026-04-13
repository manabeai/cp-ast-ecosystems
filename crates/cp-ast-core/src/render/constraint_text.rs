use crate::constraint::{ArithOp, Constraint, ExpectedType, Expression, RelationOp, SortOrder};
use crate::operation::AstEngine;

use super::render_reference;

/// Render human-readable constraint text for competitive programming problems.
///
/// Takes an `AstEngine`, produces human-readable constraint text sorted by type.
#[must_use]
pub fn render_constraints(engine: &AstEngine) -> String {
    let mut constraints_by_type: [Vec<String>; 9] = Default::default();

    // Group constraints by type in display order
    for (_, constraint) in engine.constraints.iter() {
        let rendered = render_single_constraint(engine, constraint);
        let type_index = match constraint {
            Constraint::Range { .. } => 0,
            Constraint::TypeDecl { .. } => 1,
            Constraint::LengthRelation { .. } => 2,
            Constraint::Relation { .. } => 3,
            Constraint::Distinct { .. } => 4,
            Constraint::Property { .. } => 5,
            Constraint::Sorted { .. } => 6,
            Constraint::SumBound { .. } => 7,
            Constraint::Guarantee { .. }
            | Constraint::CharSet { .. }
            | Constraint::StringLength { .. } => 8,
            Constraint::RenderHint { .. } => continue, // Skip RenderHint
        };
        constraints_by_type[type_index].push(rendered);
    }

    let mut output = String::new();
    for type_constraints in &constraints_by_type {
        for constraint_text in type_constraints {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(constraint_text);
        }
    }

    output
}

/// Render a single constraint to a human-readable string.
#[must_use]
pub fn render_single_constraint(engine: &AstEngine, constraint: &Constraint) -> String {
    match constraint {
        Constraint::Range {
            target,
            lower,
            upper,
        } => {
            let target_str = render_reference(engine, target);
            let lower_str = render_expression(engine, lower);
            let upper_str = render_expression(engine, upper);
            format!("{lower_str} ≤ {target_str} ≤ {upper_str}")
        }
        Constraint::TypeDecl { target, expected } => {
            let target_str = render_reference(engine, target);
            let type_str = match expected {
                ExpectedType::Int => "integer",
                ExpectedType::Str => "string",
                ExpectedType::Char => "character",
            };
            format!("{target_str} is {type_str}")
        }
        Constraint::LengthRelation { target, length } => {
            let target_str = render_reference(engine, target);
            let length_str = render_expression(engine, length);
            format!("|{target_str}| = {length_str}")
        }
        Constraint::Relation { lhs, op, rhs } => {
            let lhs_str = render_expression(engine, lhs);
            let rhs_str = render_expression(engine, rhs);
            let op_str = match op {
                RelationOp::Le => "≤",
                RelationOp::Lt => "<",
                RelationOp::Ge => "≥",
                RelationOp::Gt => ">",
                RelationOp::Eq => "=",
                RelationOp::Ne => "≠",
            };
            format!("{lhs_str} {op_str} {rhs_str}")
        }
        Constraint::Distinct { elements, .. } => {
            let elements_str = render_reference(engine, elements);
            format!("{elements_str} are pairwise distinct")
        }
        Constraint::Property { target, tag } => {
            let target_str = render_reference(engine, target);
            format!("{target_str}: {tag:?}")
        }
        Constraint::Sorted { elements, order } => {
            let elements_str = render_reference(engine, elements);
            let order_str = match order {
                SortOrder::Ascending | SortOrder::NonDecreasing => "ascending",
                SortOrder::Descending | SortOrder::NonIncreasing => "descending",
            };
            format!("{elements_str} sorted {order_str}")
        }
        Constraint::SumBound { variable, upper } => {
            let variable_str = render_reference(engine, variable);
            let upper_str = render_expression(engine, upper);
            format!("∑{variable_str} ≤ {upper_str}")
        }
        Constraint::Guarantee { description, .. } => description.clone(),
        Constraint::CharSet { target, charset } => {
            let target_str = render_reference(engine, target);
            format!("{target_str} ∈ {charset}")
        }
        Constraint::StringLength { target, min, max } => {
            let target_str = render_reference(engine, target);
            let min_str = render_expression(engine, min);
            let max_str = render_expression(engine, max);
            format!("{min_str} ≤ |{target_str}| ≤ {max_str}")
        }
        Constraint::RenderHint { .. } => String::new(), // Should be skipped
    }
}

fn render_expression(engine: &AstEngine, expr: &Expression) -> String {
    match expr {
        Expression::Lit(n) => n.to_string(),
        Expression::Var(reference) => render_reference(engine, reference),
        Expression::BinOp { op, lhs, rhs } => {
            let lhs_str = render_expression(engine, lhs);
            let rhs_str = render_expression(engine, rhs);
            let op_str = match op {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "×",
                ArithOp::Div => "÷",
            };
            format!("{lhs_str} {op_str} {rhs_str}")
        }
        Expression::Pow { base, exp } => {
            let base_str = render_expression(engine, base);
            let exp_str = render_expression(engine, exp);
            format!("{base_str}^{exp_str}")
        }
        Expression::FnCall { name, args } => {
            let args_str = args
                .iter()
                .map(|arg| render_expression(engine, arg))
                .collect::<Vec<_>>()
                .join(",");
            format!("{}({})", name.as_str(), args_str)
        }
    }
}
