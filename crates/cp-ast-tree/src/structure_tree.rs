//! Structure AST tree rendering.

use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::DefaultTreeVisitor;

use crate::drawing::draw_tree;
use crate::TreeOptions;

/// Render the structure AST as an ASCII tree.
#[must_use]
pub fn render_structure_tree(engine: &AstEngine, options: &TreeOptions) -> String {
    let visitor = DefaultTreeVisitor;
    let root = engine.structure.root();
    let mut output = String::new();

    let label_fn = |node_id: cp_ast_core::structure::NodeId, label: &str| {
        if options.show_node_ids {
            format!("#{} {}", node_id.value(), label)
        } else {
            label.to_owned()
        }
    };

    draw_tree(engine, &visitor, root, &label_fn, &mut output);
    output
}
