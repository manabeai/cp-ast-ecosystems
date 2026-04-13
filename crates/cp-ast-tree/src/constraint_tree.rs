//! Constraint tree rendering — constraints grouped by target node.

use std::fmt::Write;

use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::{constraint_to_tree, ConstraintNode};
use cp_ast_core::structure::node_display_name;

use crate::TreeOptions;

struct Group {
    label: String,
    nodes: Vec<(ConstraintNode, Option<u64>)>,
}

/// Render constraints grouped by target node as an ASCII tree.
///
/// Each constraint is expanded as a proper AST subtree rather than a flat
/// text string. Expressions with sub-structure (e.g. `Pow`, `BinOp`) are
/// shown as nested nodes; simple leaves (`Lit`, single `Var`) are inlined.
///
/// Output format:
/// ```text
/// Constraints
/// ├── N
/// │   └── Range
/// │       ├── target: N
/// │       ├── lower: 1
/// │       └── upper
/// │           └── Pow
/// │               ├── 10
/// │               └── 5
/// └── (global)
///     └── Guarantee: Simple graph
/// ```
#[must_use]
pub fn render_constraint_tree(engine: &AstEngine, options: &TreeOptions) -> String {
    let mut output = String::from("Constraints\n");
    let groups = collect_groups(engine, options);

    for (gi, group) in groups.iter().enumerate() {
        let is_last_group = gi + 1 == groups.len();
        let gconn = if is_last_group {
            "└── "
        } else {
            "├── "
        };
        let gpad = if is_last_group { "    " } else { "│   " };

        let _ = writeln!(output, "{gconn}{}", group.label);

        for (ci, (node, cid)) in group.nodes.iter().enumerate() {
            let is_last = ci + 1 == group.nodes.len();
            let iconn = if is_last { "└── " } else { "├── " };
            let ipad = if is_last { "    " } else { "│   " };

            let label = match cid {
                Some(id) => format!("[C{id}] {}", node.label),
                None => node.label.clone(),
            };

            let _ = writeln!(output, "{gpad}{iconn}{label}");
            render_children(&node.children, &format!("{gpad}{ipad}"), &mut output);
        }
    }

    output
}

fn collect_groups(engine: &AstEngine, options: &TreeOptions) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();

    for (node_id, constraint_ids) in engine.constraints.nodes_with_constraints() {
        let label = node_display_name(engine, node_id);
        let nodes = build_nodes(engine, options, constraint_ids);
        if !nodes.is_empty() {
            groups.push(Group { label, nodes });
        }
    }

    let global_ids = engine.constraints.global();
    if !global_ids.is_empty() {
        let nodes = build_nodes(engine, options, global_ids);
        if !nodes.is_empty() {
            groups.push(Group {
                label: "(global)".to_owned(),
                nodes,
            });
        }
    }

    groups
}

fn build_nodes(
    engine: &AstEngine,
    options: &TreeOptions,
    ids: &[cp_ast_core::constraint::ConstraintId],
) -> Vec<(ConstraintNode, Option<u64>)> {
    let mut nodes = Vec::new();
    for &cid in ids {
        if let Some(constraint) = engine.constraints.get(cid) {
            if let Some(tree) = constraint_to_tree(engine, constraint) {
                let id = options.show_constraint_ids.then(|| cid.value());
                nodes.push((tree, id));
            }
        }
    }
    nodes
}

/// Recursively render child [`ConstraintNode`]s as an ASCII subtree.
fn render_children(children: &[ConstraintNode], prefix: &str, output: &mut String) {
    for (i, child) in children.iter().enumerate() {
        let is_last = i + 1 == children.len();
        let branch = if is_last { "└── " } else { "├── " };
        let indent = if is_last { "    " } else { "│   " };
        let _ = writeln!(output, "{prefix}{branch}{}", child.label);
        render_children(&child.children, &format!("{prefix}{indent}"), output);
    }
}
