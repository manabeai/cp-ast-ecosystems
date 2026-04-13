//! ASCII tree drawing primitives.
//!
//! Provides `draw_tree` which renders a label+children tree structure
//! using `├──`/`└──`/`│` box-drawing connectors.

use std::fmt::Write;

use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{ChildEntry, NodeId, TreeVisitor};

/// Draw an ASCII tree starting from `node_id`, using the given visitor.
///
/// `label_fn` is called for each node to produce the final display label
/// (allowing the caller to append constraint annotations, `NodeId`s, etc.).
pub(crate) fn draw_tree(
    engine: &AstEngine,
    visitor: &impl TreeVisitor,
    node_id: NodeId,
    label_fn: &impl Fn(NodeId, &str) -> String,
    output: &mut String,
) {
    if let Some(info) = visitor.node_info(engine, node_id) {
        output.push_str(&label_fn(info.node_id, &info.label));
        output.push('\n');
        draw_children(engine, visitor, &info.children, "", label_fn, output);
    }
}

fn draw_children(
    engine: &AstEngine,
    visitor: &impl TreeVisitor,
    children: &[ChildEntry],
    prefix: &str,
    label_fn: &impl Fn(NodeId, &str) -> String,
    output: &mut String,
) {
    for (i, child) in children.iter().enumerate() {
        let is_last = i + 1 == children.len();
        let connector = if is_last { "└── " } else { "├── " };
        let continuation = if is_last { "    " } else { "│   " };

        match child {
            ChildEntry::Node(child_id) => {
                if let Some(child_info) = visitor.node_info(engine, *child_id) {
                    let _ = write!(
                        output,
                        "{prefix}{connector}{}",
                        label_fn(child_info.node_id, &child_info.label)
                    );
                    output.push('\n');
                    let new_prefix = format!("{prefix}{continuation}");
                    draw_children(
                        engine,
                        visitor,
                        &child_info.children,
                        &new_prefix,
                        label_fn,
                        output,
                    );
                }
            }
            ChildEntry::Virtual {
                label,
                children: kids,
            } => {
                let _ = write!(output, "{prefix}{connector}{label}");
                output.push('\n');
                let new_prefix = format!("{prefix}{continuation}");
                let virtual_children: Vec<ChildEntry> =
                    kids.iter().map(|id| ChildEntry::Node(*id)).collect();
                draw_children(
                    engine,
                    visitor,
                    &virtual_children,
                    &new_prefix,
                    label_fn,
                    output,
                );
            }
        }
    }
}
