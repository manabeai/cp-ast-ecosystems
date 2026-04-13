use cp_ast_core::constraint::Expression;
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{
    ChildEntry, DefaultTreeVisitor, Ident, Literal, NodeKind, Reference, TreeVisitor,
};

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
fn visitor_scalar_label() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, n_id).unwrap();
    assert_eq!(info.label, "Scalar(N)");
    assert!(info.children.is_empty());
}

#[test]
fn visitor_array_label() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, a_id).unwrap();
    assert_eq!(info.label, "Array(A, len=N)");
    assert!(info.children.is_empty());
}

#[test]
fn visitor_tuple_children() {
    let engine = setup_graph_engine();
    let visitor = DefaultTreeVisitor;
    let root_info = visitor.node_info(&engine, engine.structure.root()).unwrap();
    assert_eq!(root_info.label, "Sequence");
    assert_eq!(root_info.children.len(), 2);
}

#[test]
fn visitor_repeat_label_with_index() {
    let engine = setup_graph_engine();
    let visitor = DefaultTreeVisitor;
    // The repeat node is the second child of root Sequence
    let root_info = visitor.node_info(&engine, engine.structure.root()).unwrap();
    if let ChildEntry::Node(repeat_id) = &root_info.children[1] {
        let repeat_info = visitor.node_info(&engine, *repeat_id).unwrap();
        assert_eq!(repeat_info.label, "Repeat(count=M, i)");
        assert_eq!(repeat_info.children.len(), 1);
    } else {
        panic!("expected Node entry");
    }
}

#[test]
fn visitor_choice_virtual_children() {
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
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, choice).unwrap();
    assert_eq!(info.label, "Choice(tag=T)");
    assert_eq!(info.children.len(), 2);
    match &info.children[0] {
        ChildEntry::Virtual { label, children } => {
            assert_eq!(label, "Variant(1)");
            assert_eq!(children.len(), 1);
        }
        ChildEntry::Node(_) => panic!("expected Virtual entry"),
    }
}

#[test]
fn visitor_hole_label() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(cp_ast_core::structure::NodeKindHint::AnyArray),
    });
    let visitor = DefaultTreeVisitor;
    let info = visitor.node_info(&engine, hole_id).unwrap();
    assert_eq!(info.label, "Hole(expected=AnyArray)");
}

#[test]
fn visitor_nonexistent_node_returns_none() {
    let engine = AstEngine::new();
    let visitor = DefaultTreeVisitor;
    let fake_id = cp_ast_core::structure::NodeId::from_raw(999);
    assert!(visitor.node_info(&engine, fake_id).is_none());
}
