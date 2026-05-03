//! Combined tree rendering — structure tree with inline constraint annotations.

use std::fmt::Write;

use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::render_single_constraint;
use cp_ast_core::structure::{DefaultTreeVisitor, NodeId};

use crate::TreeOptions;
use crate::drawing::draw_tree;

/// Render the structure tree with constraints annotated on each node.
///
/// Each node shows its constraints in `[...]` brackets. Global constraints
/// are listed at the bottom.
#[must_use]
pub fn render_combined_tree(engine: &AstEngine, options: &TreeOptions) -> String {
    let visitor = DefaultTreeVisitor;
    let root = engine.structure.root();
    let mut output = String::new();

    let label_fn = |node_id: NodeId, label: &str| -> String {
        let base = if options.show_node_ids {
            format!("#{} {}", node_id.value(), label)
        } else {
            label.to_owned()
        };

        let annotation = build_constraint_annotation(engine, node_id, options);
        if annotation.is_empty() {
            base
        } else {
            format!("{base}  [{annotation}]")
        }
    };

    draw_tree(engine, &visitor, root, &label_fn, &mut output);

    // Append global constraints
    let global_ids = engine.constraints.global();
    let global_lines: Vec<String> = global_ids
        .iter()
        .filter_map(|&cid| engine.constraints.get(cid))
        .map(|c| render_single_constraint(engine, c))
        .filter(|s| !s.is_empty())
        .collect();

    if !global_lines.is_empty() {
        let _ = writeln!(output, "(global) {}", global_lines.join(", "));
    }

    output
}

fn build_constraint_annotation(
    engine: &AstEngine,
    node_id: NodeId,
    options: &TreeOptions,
) -> String {
    let constraint_ids = engine.constraints.for_node(node_id);
    let parts: Vec<String> = constraint_ids
        .iter()
        .filter_map(|&cid| {
            let constraint = engine.constraints.get(cid)?;
            let text = render_single_constraint(engine, constraint);
            if text.is_empty() {
                return None;
            }
            if options.show_constraint_ids {
                Some(format!("C{}:{text}", cid.value()))
            } else {
                Some(text)
            }
        })
        .collect();
    parts.join(", ")
}
