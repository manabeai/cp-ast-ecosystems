use std::fmt::Write;

use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind};

use super::{render_expression, render_reference};

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
            let length_str = render_expression(engine, length);
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
            render_tuple_elements(engine, elements, false, output);
            output.push('\n');
        }
        NodeKind::Repeat { body, .. } => {
            if body.len() == 1 {
                if let Some(body_node) = engine.structure.get(body[0]) {
                    if let NodeKind::Tuple { elements } = body_node.kind() {
                        render_tuple_elements(engine, elements, true, output);
                        output.push('\n');
                        return;
                    }
                }
            }
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
        NodeKind::Choice { tag, variants } => {
            render_choice(engine, tag, variants, output);
        }
    }
}

/// Render tuple elements inline, optionally with repeat-body subscript (`_i`).
fn render_tuple_elements(
    engine: &AstEngine,
    elements: &[NodeId],
    indexed: bool,
    output: &mut String,
) {
    for (i, &element_id) in elements.iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }
        let Some(element_node) = engine.structure.get(element_id) else {
            output.push_str("<?>");
            continue;
        };
        match element_node.kind() {
            NodeKind::Scalar { name } if indexed => {
                write!(output, "{}_i", name.as_str()).unwrap();
            }
            NodeKind::Scalar { name } => {
                output.push_str(name.as_str());
            }
            NodeKind::Array { name, length } => {
                let len = render_expression(engine, length);
                if indexed {
                    write!(
                        output,
                        "{n}_{{i,1}} {n}_{{i,2}} … {n}_{{i,{len}_i}}",
                        n = name.as_str(),
                    )
                    .unwrap();
                } else {
                    write!(output, "{n}_1 {n}_2 … {n}_{len}", n = name.as_str()).unwrap();
                }
            }
            _ => {
                output.push_str("<?>");
            }
        }
    }
}

fn render_choice(
    engine: &AstEngine,
    tag: &crate::structure::Reference,
    variants: &[(crate::structure::Literal, Vec<NodeId>)],
    output: &mut String,
) {
    let tag_str = render_reference(engine, tag);
    for (literal, children) in variants {
        let lit_str = match literal {
            crate::structure::Literal::IntLit(v) => v.to_string(),
            crate::structure::Literal::StrLit(s) => format!("\"{s}\""),
        };
        let mut child_names = Vec::new();
        for &child_id in children {
            if let Some(child_node) = engine.structure.get(child_id) {
                if let NodeKind::Scalar { name } = child_node.kind() {
                    child_names.push(name.as_str().to_string());
                } else {
                    child_names.push("<?>".to_string());
                }
            }
        }
        writeln!(
            output,
            "If {} = {}: {}",
            tag_str,
            lit_str,
            child_names.join(" ")
        )
        .unwrap();
    }
}
