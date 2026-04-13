use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::{render_constraints, render_input};
use cp_ast_core::structure::{Ident, NodeKind, Reference};

#[test]
fn render_scalar_n() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    assert_eq!(render_input(&engine), "N\n");
}

#[test]
fn render_n_plus_array() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });
    }
    assert_eq!(render_input(&engine), "N\nA_1 A_2 … A_N\n");
}

#[test]
fn render_edge_list() {
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
        index_var: None,
        body: vec![tuple_edge],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tuple_header, repeat],
        });
    }
    let output = render_input(&engine);
    assert!(output.contains("N M"));
    assert!(output.contains("u_i v_i"));
}

#[test]
fn render_constraints_range() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    let text = render_constraints(&engine);
    assert!(text.contains("1 ≤ N ≤ 100"));
}

#[test]
fn render_constraints_sorted_by_type() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    // Add constraints in reverse display order
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "answer exists".to_owned(),
            predicate: None,
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );

    let text = render_constraints(&engine);
    let lines: Vec<&str> = text.lines().collect();
    // Range should come before TypeDecl, TypeDecl before Guarantee
    let range_pos = lines.iter().position(|l| l.contains("≤")).unwrap();
    let type_pos = lines.iter().position(|l| l.contains("integer")).unwrap();
    let guarantee_pos = lines
        .iter()
        .position(|l| l.contains("answer exists"))
        .unwrap();
    assert!(range_pos < type_pos, "Range should come before TypeDecl");
    assert!(
        type_pos < guarantee_pos,
        "TypeDecl should come before Guarantee"
    );
}
