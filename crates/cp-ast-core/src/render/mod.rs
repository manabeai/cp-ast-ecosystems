//! Canonical rendering of AST structure and constraints.
//!
//! This module provides functions to render competitive-programming-style
//! input format text and human-readable constraint text.

pub mod constraint_text;
pub mod input_format;

pub use constraint_text::render_constraints;
pub use input_format::render_input;

use crate::constraint::{ArithOp, Expression};
use crate::operation::AstEngine;
use crate::structure::{NodeKind, Reference};

/// Render a `Reference` to a human-readable string, resolving node names from the AST.
pub(crate) fn render_reference(engine: &AstEngine, reference: &Reference) -> String {
    match reference {
        Reference::VariableRef(node_id) => {
            if let Some(node) = engine.structure.get(*node_id) {
                if let NodeKind::Scalar { name } = node.kind() {
                    name.as_str().to_string()
                } else {
                    format!("?{node_id:?}")
                }
            } else {
                format!("?{node_id:?}")
            }
        }
        Reference::IndexedRef { target, indices } => {
            let base = if let Some(node) = engine.structure.get(*target) {
                if let NodeKind::Scalar { name } = node.kind() {
                    name.as_str().to_string()
                } else {
                    format!("?{target:?}")
                }
            } else {
                format!("?{target:?}")
            };
            let index_str = indices
                .iter()
                .map(crate::structure::Ident::as_str)
                .collect::<Vec<_>>()
                .join("][");
            format!("{base}[{index_str}]")
        }
        Reference::Unresolved(ident) => ident.as_str().to_string(),
    }
}

/// Render an `Expression` to a human-readable string.
pub(crate) fn render_expression(engine: &AstEngine, expr: &Expression) -> String {
    match expr {
        Expression::Lit(v) => v.to_string(),
        Expression::Var(reference) => render_reference(engine, reference),
        Expression::BinOp { op, lhs, rhs } => {
            let l = render_expression(engine, lhs);
            let r = render_expression(engine, rhs);
            let op_str = match op {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "*",
                ArithOp::Div => "/",
            };
            format!("{l}{op_str}{r}")
        }
        Expression::Pow { base, exp } => {
            let b = render_expression(engine, base);
            let e = render_expression(engine, exp);
            format!("{b}^{e}")
        }
        Expression::FnCall { name, args } => {
            let args_str: Vec<String> = args.iter().map(|a| render_expression(engine, a)).collect();
            format!("{}({})", name.as_str(), args_str.join(", "))
        }
    }
}
