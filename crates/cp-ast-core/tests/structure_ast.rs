use cp_ast_core::constraint::Expression;
use cp_ast_core::structure::{
    Ident, Literal, NodeId, NodeKind, NodeKindHint, Reference, StructureAst, StructureNode,
};

// --- NodeKind tests ---

#[test]
fn node_kind_scalar() {
    let kind = NodeKind::Scalar {
        name: Ident::new("N"),
    };
    assert!(matches!(kind, NodeKind::Scalar { .. }));
}

#[test]
fn node_kind_array() {
    let kind = NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::Unresolved(Ident::new("N"))),
    };
    assert!(matches!(kind, NodeKind::Array { .. }));
}

#[test]
fn node_kind_matrix() {
    let kind = NodeKind::Matrix {
        name: Ident::new("C"),
        rows: Reference::Unresolved(Ident::new("H")),
        cols: Reference::Unresolved(Ident::new("W")),
    };
    assert!(matches!(kind, NodeKind::Matrix { .. }));
}

#[test]
fn node_kind_tuple() {
    let kind = NodeKind::Tuple {
        elements: vec![NodeId::from_raw(1), NodeId::from_raw(2)],
    };
    assert!(matches!(kind, NodeKind::Tuple { .. }));
}

#[test]
fn node_kind_repeat() {
    let kind = NodeKind::Repeat {
        count: Expression::Var(Reference::Unresolved(Ident::new("M"))),
        index_var: None,
        body: vec![NodeId::from_raw(5)],
    };
    assert!(matches!(kind, NodeKind::Repeat { .. }));
}

#[test]
fn node_kind_section() {
    let kind = NodeKind::Section {
        header: Some(NodeId::from_raw(1)),
        body: vec![NodeId::from_raw(2)],
    };
    assert!(matches!(kind, NodeKind::Section { .. }));
}

#[test]
fn node_kind_sequence() {
    let kind = NodeKind::Sequence {
        children: vec![NodeId::from_raw(1), NodeId::from_raw(2)],
    };
    assert!(matches!(kind, NodeKind::Sequence { .. }));
}

#[test]
fn node_kind_choice() {
    let kind = NodeKind::Choice {
        tag: Reference::Unresolved(Ident::new("type")),
        variants: vec![
            (Literal::IntLit(1), vec![NodeId::from_raw(10)]),
            (Literal::IntLit(2), vec![NodeId::from_raw(20)]),
        ],
    };
    assert!(matches!(kind, NodeKind::Choice { .. }));
}

#[test]
fn node_kind_hole() {
    let kind = NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyScalar),
    };
    assert!(matches!(kind, NodeKind::Hole { .. }));
}

#[test]
fn node_kind_hole_no_hint() {
    let kind = NodeKind::Hole {
        expected_kind: None,
    };
    assert!(matches!(
        kind,
        NodeKind::Hole {
            expected_kind: None
        }
    ));
}

// --- StructureNode tests ---

#[test]
fn structure_node_creation() {
    let node = StructureNode::new(
        NodeId::from_raw(1),
        NodeKind::Scalar {
            name: Ident::new("N"),
        },
    );
    assert_eq!(node.id(), NodeId::from_raw(1));
    assert!(matches!(node.kind(), &NodeKind::Scalar { .. }));
}

// --- StructureAst tests ---

#[test]
fn ast_new_has_sequence_root() {
    let ast = StructureAst::new();
    let root = ast.get(ast.root()).unwrap();
    assert!(matches!(root.kind(), &NodeKind::Sequence { .. }));
}

#[test]
fn ast_add_and_get_node() {
    let mut ast = StructureAst::new();
    let id = ast.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let node = ast.get(id).unwrap();
    assert!(matches!(node.kind(), &NodeKind::Scalar { .. }));
    assert_eq!(node.id(), id);
}

#[test]
fn ast_add_multiple_nodes() {
    let mut ast = StructureAst::new();
    let id1 = ast.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let id2 = ast.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    assert_ne!(id1, id2);
    assert!(ast.contains(id1));
    assert!(ast.contains(id2));
}

#[test]
fn ast_remove_node() {
    let mut ast = StructureAst::new();
    let id = ast.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    assert!(ast.contains(id));
    let removed = ast.remove(id);
    assert!(removed.is_some());
    assert!(!ast.contains(id));
}

#[test]
fn ast_get_nonexistent_returns_none() {
    let ast = StructureAst::new();
    assert!(ast.get(NodeId::from_raw(999)).is_none());
}

#[test]
fn ast_get_mut_and_modify() {
    let mut ast = StructureAst::new();
    let id = ast.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    ast.get_mut(id).unwrap().set_kind(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let node = ast.get(id).unwrap();
    assert!(matches!(node.kind(), &NodeKind::Scalar { .. }));
}

#[test]
fn ast_len_counts_only_live_nodes() {
    let mut ast = StructureAst::new();
    assert_eq!(ast.len(), 1); // root Sequence
    let id = ast.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    assert_eq!(ast.len(), 2);
    ast.remove(id);
    assert_eq!(ast.len(), 1);
}

#[test]
fn ast_iter_yields_live_nodes() {
    let mut ast = StructureAst::new();
    ast.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    ast.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let nodes: Vec<_> = ast.iter().collect();
    assert_eq!(nodes.len(), 3); // root + N + M
}

#[test]
fn ast_add_hole_node() {
    let mut ast = StructureAst::new();
    let hole_id = ast.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyScalar),
    });
    let node = ast.get(hole_id).unwrap();
    assert!(matches!(node.kind(), &NodeKind::Hole { .. }));
}
