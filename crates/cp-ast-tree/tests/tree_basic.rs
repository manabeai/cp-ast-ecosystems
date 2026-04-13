use cp_ast_core::constraint::Expression;
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, Literal, NodeKind, Reference};
use cp_ast_tree::{render_structure_tree, TreeOptions};

fn setup_graph_engine() -> AstEngine {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let tuple_header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    let tuple_edge = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(m_id)),
        index_var: Some(Ident::new("i")),
        body: vec![tuple_edge],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tuple_header, repeat],
        });
    }
    engine
}

#[test]
fn structure_tree_graph_problem() {
    let engine = setup_graph_engine();
    let output = render_structure_tree(&engine, &TreeOptions::default());
    let expected = "\
Sequence
├── Tuple
│   ├── Scalar(N)
│   └── Scalar(M)
└── Repeat(count=M, i)
    └── Tuple
        ├── Scalar(u)
        └── Scalar(v)
";
    assert_eq!(output, expected);
}

#[test]
fn structure_tree_with_node_ids() {
    let engine = setup_graph_engine();
    let options = TreeOptions {
        show_node_ids: true,
        ..TreeOptions::default()
    };
    let output = render_structure_tree(&engine, &options);
    // Root is #0, N is #1, M is #2, tuple_header is #3, etc.
    assert!(output.contains("#0 Sequence"));
    assert!(output.contains("#1 Scalar(N)"));
}

#[test]
fn structure_tree_empty_ast() {
    let engine = AstEngine::new();
    let output = render_structure_tree(&engine, &TreeOptions::default());
    // Empty AST has only the root Sequence with no children
    assert_eq!(output, "Sequence\n");
}

#[test]
fn structure_tree_single_scalar() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    let output = render_structure_tree(&engine, &TreeOptions::default());
    assert_eq!(output, "Sequence\n└── Scalar(N)\n");
}

#[test]
fn structure_tree_choice() {
    let mut engine = AstEngine::new();
    let tag_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let choice = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(tag_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id]),
        ],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tag_id, choice],
        });
    }
    let output = render_structure_tree(&engine, &TreeOptions::default());
    let expected = "\
Sequence
├── Scalar(T)
└── Choice(tag=T)
    ├── Variant(1)
    │   └── Scalar(X)
    └── Variant(2)
        └── Scalar(Y)
";
    assert_eq!(output, expected);
}
