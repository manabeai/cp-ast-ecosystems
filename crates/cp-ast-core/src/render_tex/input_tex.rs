//! Input format TeX rendering.

use crate::constraint::Expression;
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind};

use super::tex_helpers::{ident_to_tex, reference_to_tex};
use super::{TexOptions, TexOutput, TexWarning};

/// Render input format as TeX array layout.
pub(crate) fn render_input_tex_impl(engine: &AstEngine, options: &TexOptions) -> TexOutput {
    let mut warnings = Vec::new();
    let root = engine.structure.root();

    let mut lines: Vec<String> = Vec::new();
    collect_lines(engine, root, &mut lines, &mut warnings, options);

    if lines.is_empty() {
        return TexOutput {
            tex: String::new(),
            warnings,
        };
    }

    let mut tex = String::from("\\[\n\\begin{array}{l}\n");
    for (i, line) in lines.iter().enumerate() {
        tex.push_str(line);
        if i < lines.len() - 1 {
            tex.push_str(" \\\\");
        }
        tex.push('\n');
    }
    tex.push_str("\\end{array}\n\\]\n");

    TexOutput { tex, warnings }
}

/// Collect TeX lines from the AST recursively.
fn collect_lines(
    engine: &AstEngine,
    node_id: NodeId,
    lines: &mut Vec<String>,
    warnings: &mut Vec<TexWarning>,
    options: &TexOptions,
) {
    let Some(node) = engine.structure.get(node_id) else {
        lines.push("\\texttt{<?>}".to_owned());
        return;
    };

    match node.kind() {
        NodeKind::Scalar { name } => {
            lines.push(ident_to_tex(name));
        }
        NodeKind::Tuple { elements } => {
            let parts: Vec<String> = elements
                .iter()
                .map(|&eid| {
                    if let Some(enode) = engine.structure.get(eid) {
                        match enode.kind() {
                            NodeKind::Scalar { name } => ident_to_tex(name),
                            NodeKind::Hole { .. } => {
                                if options.include_holes {
                                    warnings.push(TexWarning::HoleEncountered { node_id: eid });
                                    "\\texttt{<hole>}".to_owned()
                                } else {
                                    String::new()
                                }
                            }
                            _ => "\\texttt{<?>}".to_owned(),
                        }
                    } else {
                        "\\texttt{<?>}".to_owned()
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();
            if !parts.is_empty() {
                lines.push(parts.join(" \\ "));
            }
        }
        NodeKind::Array { name, length } => {
            let name_str = ident_to_tex(name);
            let length_str = match length {
                Expression::Var(r) => reference_to_tex(engine, r, warnings),
                _ => format!("{length:?}"),
            };
            lines.push(format!(
                "{name_str}_1 \\ {name_str}_2 \\ \\cdots \\ {name_str}_{length_str}"
            ));
        }
        NodeKind::Matrix { name, rows, cols } => {
            let name_str = ident_to_tex(name);
            let rows_str = reference_to_tex(engine, rows, warnings);
            let cols_str = reference_to_tex(engine, cols, warnings);
            // First row
            lines.push(format!(
                "{name_str}_{{1,1}} \\ {name_str}_{{1,2}} \\ \\cdots \\ {name_str}_{{1,{cols_str}}}"
            ));
            // Second row
            lines.push(format!(
                "{name_str}_{{2,1}} \\ {name_str}_{{2,2}} \\ \\cdots \\ {name_str}_{{2,{cols_str}}}"
            ));
            // Vdots
            lines.push("\\vdots".to_owned());
            // Last row
            lines.push(format!(
                "{name_str}_{{{rows_str},1}} \\ {name_str}_{{{rows_str},2}} \\ \\cdots \\ {name_str}_{{{rows_str},{cols_str}}}"
            ));
        }
        NodeKind::Repeat { count, body, .. } => {
            let count_str = match count {
                Expression::Var(r) => reference_to_tex(engine, r, warnings),
                _ => format!("{count:?}"),
            };
            render_repeat_lines(engine, &count_str, body, lines, warnings, options);
        }
        NodeKind::Section { body, .. } => {
            for &child_id in body {
                collect_lines(engine, child_id, lines, warnings, options);
            }
        }
        NodeKind::Sequence { children } => {
            for &child_id in children {
                collect_lines(engine, child_id, lines, warnings, options);
            }
        }
        NodeKind::Hole { .. } => {
            if options.include_holes {
                warnings.push(TexWarning::HoleEncountered { node_id });
                lines.push("\\texttt{<hole>}".to_owned());
            }
        }
        NodeKind::Choice { .. } => {
            lines.push("\\texttt{(choice)}".to_owned());
        }
    }
}

/// Render Repeat node as subscripted vertical expansion.
fn render_repeat_lines(
    engine: &AstEngine,
    count_str: &str,
    body: &[NodeId],
    lines: &mut Vec<String>,
    warnings: &mut Vec<TexWarning>,
    options: &TexOptions,
) {
    // Check if body is a single Scalar
    if body.len() == 1 {
        if let Some(body_node) = engine.structure.get(body[0]) {
            if let NodeKind::Scalar { name } = body_node.kind() {
                let name_str = ident_to_tex(name);
                lines.push(format!("{name_str}_1"));
                lines.push(format!("{name_str}_2"));
                lines.push("\\vdots".to_owned());
                lines.push(format!("{name_str}_{count_str}"));
                return;
            }
            // Check if body is a single Tuple
            if let NodeKind::Tuple { elements } = body_node.kind() {
                let names: Vec<String> = elements
                    .iter()
                    .filter_map(|&eid| {
                        engine.structure.get(eid).and_then(|n| {
                            if let NodeKind::Scalar { name } = n.kind() {
                                Some(ident_to_tex(name))
                            } else {
                                None
                            }
                        })
                    })
                    .collect();
                if !names.is_empty() {
                    // First line: u_1 \ v_1
                    let first: Vec<String> = names.iter().map(|n| format!("{n}_1")).collect();
                    lines.push(first.join(" \\ "));
                    // Second line: u_2 \ v_2
                    let second: Vec<String> = names.iter().map(|n| format!("{n}_2")).collect();
                    lines.push(second.join(" \\ "));
                    // Vdots
                    lines.push("\\vdots".to_owned());
                    // Last line: u_M \ v_M
                    let last: Vec<String> =
                        names.iter().map(|n| format!("{n}_{count_str}")).collect();
                    lines.push(last.join(" \\ "));
                    return;
                }
            }
        }
    }

    // Fallback: render body children recursively
    for &child_id in body {
        collect_lines(engine, child_id, lines, warnings, options);
    }
}
