//! Constraint TeX rendering.

use crate::constraint::{Constraint, Expression, RelationOp, SortOrder};
use crate::operation::AstEngine;

use super::tex_helpers::{IndexAllocator, expression_to_tex, reference_to_tex, resolve_array_info};
use super::{TexOptions, TexOutput, TexWarning};

/// Render all constraints as TeX itemize list.
pub(crate) fn render_constraints_tex_impl(engine: &AstEngine, _options: &TexOptions) -> TexOutput {
    let mut warnings = Vec::new();
    let mut alloc = IndexAllocator::new();

    // Group constraints by type in display order
    let mut groups: [Vec<String>; 9] = Default::default();

    for (_, constraint) in engine.constraints.iter() {
        let type_index = match constraint {
            Constraint::Range { .. } => 0,
            Constraint::TypeDecl { .. } | Constraint::RenderHint { .. } => continue,
            Constraint::LengthRelation { .. } => 1,
            Constraint::Relation { .. } => 2,
            Constraint::Distinct { .. } => 3,
            Constraint::Property { .. } => 4,
            Constraint::Sorted { .. } => 5,
            Constraint::SumBound { .. } => 6,
            Constraint::Guarantee { .. }
            | Constraint::CharSet { .. }
            | Constraint::StringLength { .. } => 7,
        };
        let rendered = render_constraint_tex(engine, constraint, &mut alloc, &mut warnings);
        groups[type_index].push(rendered);
    }

    let items: Vec<String> = groups.iter().flat_map(|g| g.iter().cloned()).collect();

    if items.is_empty() {
        return TexOutput {
            tex: String::new(),
            warnings,
        };
    }

    let mut tex = String::from("\\begin{itemize}\n");
    for item in &items {
        tex.push_str("  \\item ");
        tex.push_str(item);
        tex.push('\n');
    }
    tex.push_str("\\end{itemize}\n");

    TexOutput { tex, warnings }
}

#[allow(clippy::too_many_lines)]
fn render_constraint_tex(
    engine: &AstEngine,
    constraint: &Constraint,
    alloc: &mut IndexAllocator,
    warnings: &mut Vec<TexWarning>,
) -> String {
    match constraint {
        Constraint::Range {
            target,
            lower,
            upper,
        } => {
            let lower_str = expression_to_tex(engine, lower, warnings);
            let upper_str = expression_to_tex(engine, upper, warnings);

            if let Some((name, length_expr)) = resolve_array_info(engine, target) {
                let idx = alloc.allocate();
                let length_str = match &length_expr {
                    Expression::Var(r) => reference_to_tex(engine, r, warnings),
                    _ => format!("{length_expr:?}"),
                };
                format!(
                    "${lower_str} \\le {name}_{idx} \\le {upper_str} \\ (1 \\le {idx} \\le {length_str})$"
                )
            } else {
                let target_str = reference_to_tex(engine, target, warnings);
                format!("${lower_str} \\le {target_str} \\le {upper_str}$")
            }
        }
        Constraint::LengthRelation { target, length } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let length_str = expression_to_tex(engine, length, warnings);
            format!("$|{target_str}| = {length_str}$")
        }
        Constraint::Relation { lhs, op, rhs } => {
            let lhs_str = expression_to_tex(engine, lhs, warnings);
            let rhs_str = expression_to_tex(engine, rhs, warnings);
            let op_str = match op {
                RelationOp::Le => "\\le",
                RelationOp::Lt => "<",
                RelationOp::Ge => "\\ge",
                RelationOp::Gt => ">",
                RelationOp::Eq => "=",
                RelationOp::Ne => "\\neq",
            };
            format!("${lhs_str} {op_str} {rhs_str}$")
        }
        Constraint::Distinct { elements, .. } => {
            if let Some((name, _)) = resolve_array_info(engine, elements) {
                format!("${name}_i \\neq {name}_j \\ (i \\neq j)$")
            } else {
                let elem_str = reference_to_tex(engine, elements, warnings);
                format!("${elem_str}$ are pairwise distinct")
            }
        }
        Constraint::Sorted { elements, order } => {
            if let Some((name, length_expr)) = resolve_array_info(engine, elements) {
                let length_str = match &length_expr {
                    Expression::Var(r) => reference_to_tex(engine, r, warnings),
                    _ => format!("{length_expr:?}"),
                };
                let op = match order {
                    SortOrder::Ascending | SortOrder::NonDecreasing => "\\le",
                    SortOrder::Descending | SortOrder::NonIncreasing => "\\ge",
                };
                format!("${name}_1 {op} {name}_2 {op} \\cdots {op} {name}_{length_str}$")
            } else {
                let elem_str = reference_to_tex(engine, elements, warnings);
                let order_str = match order {
                    SortOrder::Ascending | SortOrder::NonDecreasing => "ascending",
                    SortOrder::Descending | SortOrder::NonIncreasing => "descending",
                };
                format!("${elem_str}$ sorted {order_str}")
            }
        }
        Constraint::SumBound { variable, upper } => {
            let var_str = reference_to_tex(engine, variable, warnings);
            let upper_str = expression_to_tex(engine, upper, warnings);
            format!("$\\sum {var_str} \\le {upper_str}$")
        }
        Constraint::Guarantee { description, .. } => description.clone(),
        Constraint::CharSet { target, charset } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let charset_desc = match charset {
                crate::constraint::CharSetSpec::LowerAlpha => "は英小文字からなる",
                crate::constraint::CharSetSpec::UpperAlpha => "は英大文字からなる",
                crate::constraint::CharSetSpec::Alpha => "は英字からなる",
                crate::constraint::CharSetSpec::Digit => "は数字からなる",
                crate::constraint::CharSetSpec::AlphaNumeric => "は英数字からなる",
                crate::constraint::CharSetSpec::Custom(_)
                | crate::constraint::CharSetSpec::Range(_, _) => {
                    return format!("${target_str}$: charset constraint");
                }
            };
            format!("${target_str}$ {charset_desc}")
        }
        Constraint::StringLength { target, min, max } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let min_str = expression_to_tex(engine, min, warnings);
            let max_str = expression_to_tex(engine, max, warnings);
            format!("${min_str} \\le |{target_str}| \\le {max_str}$")
        }
        Constraint::Property { target, tag } => {
            let target_str = reference_to_tex(engine, target, warnings);
            let tag_desc = match tag {
                crate::constraint::PropertyTag::Simple => "is a simple graph",
                crate::constraint::PropertyTag::Connected => "is connected",
                crate::constraint::PropertyTag::Tree => "is a tree",
                crate::constraint::PropertyTag::Permutation => "is a permutation",
                crate::constraint::PropertyTag::Binary => "is binary",
                crate::constraint::PropertyTag::Odd => "is odd",
                crate::constraint::PropertyTag::Even => "is even",
                crate::constraint::PropertyTag::Custom(s) => {
                    return format!("${target_str}$: {s}");
                }
            };
            format!("${target_str}$ {tag_desc}")
        }
        Constraint::TypeDecl { .. } | Constraint::RenderHint { .. } => {
            // Should never reach here due to `continue` in caller
            String::new()
        }
    }
}
