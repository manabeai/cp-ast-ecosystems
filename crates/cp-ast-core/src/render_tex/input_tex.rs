//! Input format TeX rendering.

use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind};

use super::tex_helpers::{expression_to_tex, ident_to_tex, reference_to_tex};
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
            render_tuple_line(engine, elements, lines, warnings, options);
        }
        NodeKind::Array { name, length } => {
            let name_str = ident_to_tex(name);
            let length_str = expression_to_tex(engine, length, warnings);
            lines.push(format!(
                "{name_str}_1 \\ {name_str}_2 \\ \\cdots \\ {name_str}_{{{length_str}}}"
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
            let count_str = expression_to_tex(engine, count, warnings);
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
        NodeKind::Choice { tag, variants } => {
            let tag_str = reference_to_tex(engine, tag, warnings);
            lines.push("\\begin{cases}".to_owned());
            for (i, (literal, children)) in variants.iter().enumerate() {
                let mut variant_lines = Vec::new();
                for &child_id in children {
                    collect_lines(engine, child_id, &mut variant_lines, warnings, options);
                }
                let body_str = variant_lines.join(" \\ ");
                let lit_str = match literal {
                    crate::structure::Literal::IntLit(v) => v.to_string(),
                    crate::structure::Literal::StrLit(s) => format!("\\text{{{s}}}"),
                };
                let separator = if i + 1 < variants.len() { " \\\\" } else { "" };
                lines.push(format!(
                    "{body_str} & \\text{{if }} {tag_str} = {lit_str}{separator}"
                ));
            }
            lines.push("\\end{cases}".to_owned());
        }
    }
}

/// Render a Tuple node as a single line with inline Array expansion.
fn render_tuple_line(
    engine: &AstEngine,
    elements: &[NodeId],
    lines: &mut Vec<String>,
    warnings: &mut Vec<TexWarning>,
    options: &TexOptions,
) {
    let parts: Vec<String> = elements
        .iter()
        .map(|&eid| {
            if let Some(enode) = engine.structure.get(eid) {
                match enode.kind() {
                    NodeKind::Scalar { name } => ident_to_tex(name),
                    NodeKind::Array { name, length } => {
                        let n = ident_to_tex(name);
                        let l = expression_to_tex(engine, length, warnings);
                        format!("{n}_1 \\ {n}_2 \\ \\cdots \\ {n}_{{{l}}}")
                    }
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
                let elems = collect_tuple_elements(engine, elements, warnings);
                if !elems.is_empty() {
                    render_repeat_tuple_rows(count_str, &elems, lines);
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

/// A Tuple element in repeat context: either a Scalar or an inline Array.
enum RepeatTupleElem {
    Scalar { name: String },
    Array { name: String, length_tex: String },
}

impl RepeatTupleElem {
    /// Render this element for the given row index string.
    fn render_row(&self, row: &str) -> String {
        match self {
            Self::Scalar { name } => format!("{name}_{row}"),
            Self::Array { name, length_tex } => {
                format!(
                    "{name}_{{{row},1}} \\ {name}_{{{row},2}} \\ \\cdots \\ {name}_{{{row},{length_tex}_{row}}}"
                )
            }
        }
    }
}

/// Collect Tuple elements as `RepeatTupleElem` for repeat-body rendering.
fn collect_tuple_elements(
    engine: &AstEngine,
    elements: &[NodeId],
    warnings: &mut Vec<TexWarning>,
) -> Vec<RepeatTupleElem> {
    elements
        .iter()
        .filter_map(|&eid| {
            engine.structure.get(eid).and_then(|n| match n.kind() {
                NodeKind::Scalar { name } => Some(RepeatTupleElem::Scalar {
                    name: ident_to_tex(name),
                }),
                NodeKind::Array { name, length } => Some(RepeatTupleElem::Array {
                    name: ident_to_tex(name),
                    length_tex: expression_to_tex(engine, length, warnings),
                }),
                _ => None,
            })
        })
        .collect()
}

/// Render repeat tuple rows: first, second, vdots, last.
fn render_repeat_tuple_rows(count_str: &str, elems: &[RepeatTupleElem], lines: &mut Vec<String>) {
    for row in &["1", "2"] {
        let parts: Vec<String> = elems.iter().map(|e| e.render_row(row)).collect();
        lines.push(parts.join(" \\ "));
    }
    lines.push("\\vdots".to_owned());
    let last_parts: Vec<String> = elems.iter().map(|e| e.render_row(count_str)).collect();
    lines.push(last_parts.join(" \\ "));
}
