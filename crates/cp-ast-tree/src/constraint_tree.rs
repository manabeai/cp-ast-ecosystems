//! Constraint tree rendering — constraints grouped by target node.

use std::fmt::Write;

use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::render_single_constraint;
use cp_ast_core::structure::node_display_name;

use crate::TreeOptions;

/// Render constraints grouped by target node as an ASCII tree.
///
/// Output format:
/// ```text
/// Constraints
/// ├── N
/// │   ├── 1 ≤ N ≤ 100000
/// │   └── N is integer
/// └── (global)
///     └── Property: Simple graph
/// ```
#[must_use]
pub fn render_constraint_tree(engine: &AstEngine, options: &TreeOptions) -> String {
    let mut output = String::from("Constraints\n");

    let mut groups: Vec<GroupEntry> = Vec::new();

    // Collect per-node constraint groups
    for (node_id, constraint_ids) in engine.constraints.nodes_with_constraints() {
        let node_label = node_display_name(engine, node_id);

        let mut items: Vec<ConstraintItem> = Vec::new();
        for &cid in constraint_ids {
            if let Some(constraint) = engine.constraints.get(cid) {
                let text = render_single_constraint(engine, constraint);
                if text.is_empty() {
                    continue;
                }
                items.push(ConstraintItem {
                    text,
                    constraint_id: if options.show_constraint_ids {
                        Some(cid.value())
                    } else {
                        None
                    },
                });
            }
        }
        if !items.is_empty() {
            groups.push(GroupEntry {
                label: node_label,
                items,
            });
        }
    }

    // Collect global constraints
    let global_ids = engine.constraints.global();
    if !global_ids.is_empty() {
        let mut items: Vec<ConstraintItem> = Vec::new();
        for &cid in global_ids {
            if let Some(constraint) = engine.constraints.get(cid) {
                let text = render_single_constraint(engine, constraint);
                if text.is_empty() {
                    continue;
                }
                items.push(ConstraintItem {
                    text,
                    constraint_id: if options.show_constraint_ids {
                        Some(cid.value())
                    } else {
                        None
                    },
                });
            }
        }
        if !items.is_empty() {
            groups.push(GroupEntry {
                label: "(global)".to_owned(),
                items,
            });
        }
    }

    // Render groups as a tree
    for (gi, group) in groups.iter().enumerate() {
        let is_last_group = gi + 1 == groups.len();
        let group_connector = if is_last_group {
            "└── "
        } else {
            "├── "
        };
        let group_continuation = if is_last_group { "    " } else { "│   " };

        let _ = writeln!(output, "{group_connector}{}", group.label);

        for (ci, item) in group.items.iter().enumerate() {
            let is_last_item = ci + 1 == group.items.len();
            let item_connector = if is_last_item {
                "└── "
            } else {
                "├── "
            };

            let label = match item.constraint_id {
                Some(id) => format!("[C{}] {}", id, item.text),
                None => item.text.clone(),
            };
            let _ = writeln!(output, "{group_continuation}{item_connector}{label}");
        }
    }

    output
}

struct GroupEntry {
    label: String,
    items: Vec<ConstraintItem>,
}

struct ConstraintItem {
    text: String,
    constraint_id: Option<u64>,
}
