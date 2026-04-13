use std::fmt::Write;

use crate::constraint::Expression;
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind};

use super::render_reference;

/// Render the input format for competitive programming problems.
///
/// Walks the `StructureAst` in Sequence.children order and produces
/// competitive-programming-style input format text.
#[must_use]
pub fn render_input(engine: &AstEngine) -> String {
    let mut output = String::new();
    let root = engine.structure.root();
    render_node(engine, root, &mut output);
    output
}

fn render_node(engine: &AstEngine, node_id: NodeId, output: &mut String) {
    let Some(node) = engine.structure.get(node_id) else {
        output.push_str("<?>\n");
        return;
    };

    match node.kind() {
        NodeKind::Scalar { name } => {
            output.push_str(name.as_str());
            output.push('\n');
        }
        NodeKind::Array { name, length } => {
            let length_str = match length {
                Expression::Var(r) => render_reference(engine, r),
                _ => format!("{length:?}"),
            };
            writeln!(
                output,
                "{}_1 {}_2 … {}_{}",
                name.as_str(),
                name.as_str(),
                name.as_str(),
                length_str
            )
            .unwrap();
        }
        NodeKind::Matrix {
            name,
            rows: _,
            cols: _,
        } => {
            // For now, just show the pattern - in a real implementation we'd need
            // to resolve the actual dimensions
            writeln!(output, "{}_{{{},{}}}", name.as_str(), 1, 1).unwrap();
        }
        NodeKind::Tuple { elements } => {
            for (i, &element_id) in elements.iter().enumerate() {
                if i > 0 {
                    output.push(' ');
                }
                if let Some(element_node) = engine.structure.get(element_id) {
                    if let NodeKind::Scalar { name } = element_node.kind() {
                        output.push_str(name.as_str());
                    } else {
                        output.push_str("<?>");
                    }
                } else {
                    output.push_str("<?>");
                }
            }
            output.push('\n');
        }
        NodeKind::Repeat { body, .. } => {
            // Check if body contains a single Tuple - if so, use indexed form
            if body.len() == 1 {
                if let Some(body_node) = engine.structure.get(body[0]) {
                    if let NodeKind::Tuple { elements } = body_node.kind() {
                        // Render in indexed form: u_i v_i (showing the pattern)
                        for (i, &element_id) in elements.iter().enumerate() {
                            if i > 0 {
                                output.push(' ');
                            }
                            if let Some(element_node) = engine.structure.get(element_id) {
                                if let NodeKind::Scalar { name } = element_node.kind() {
                                    write!(output, "{}_i", name.as_str()).unwrap();
                                } else {
                                    output.push_str("<?>");
                                }
                            } else {
                                output.push_str("<?>");
                            }
                        }
                        output.push('\n');
                        return;
                    }
                }
            }

            // Default: render body recursively
            for &child_id in body {
                render_node(engine, child_id, output);
            }
        }
        NodeKind::Section { header: _, body } => {
            // Render body recursively
            for &child_id in body {
                render_node(engine, child_id, output);
            }
        }
        NodeKind::Sequence { children } => {
            // Render each child in order
            for &child_id in children {
                render_node(engine, child_id, output);
            }
        }
        NodeKind::Hole { .. } => {
            output.push_str("[ ]\n");
        }
        NodeKind::Choice { .. } => {
            output.push_str("(choice)\n");
        }
    }
}
