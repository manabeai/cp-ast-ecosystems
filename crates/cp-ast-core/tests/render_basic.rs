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

#[test]
fn render_expression_count_repeat() {
    use cp_ast_core::constraint::ArithOp;

    let mut engine = AstEngine::default();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Lit(1)),
        },
        index_var: None,
        body: vec![tuple_id],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let text = render_input(&engine);
    assert!(text.contains("u_i v_i"), "got: {text}");
}

#[test]
fn render_choice_plain_text() {
    use cp_ast_core::structure::Literal;

    let mut engine = AstEngine::default();
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let z_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Z"),
    });
    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id, z_id]),
        ],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![choice_id],
        });

    let text = render_input(&engine);
    assert!(text.contains("If T = 1: X"), "got: {text}");
    assert!(text.contains("If T = 2: Y Z"), "got: {text}");
}

#[test]
fn render_tuple_with_inline_array() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let c_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("C"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(c_id)),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, a_id, m_id],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![tuple_id],
        });

    let text = render_input(&engine);
    assert_eq!(text, "N A_1 A_2 … A_C M\n");
}

#[test]
fn render_repeat_tuple_with_inline_array() {
    let mut engine = AstEngine::new();
    let q_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let c_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("C"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(c_id)),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, a_id, m_id],
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(q_id)),
        index_var: None,
        body: vec![tuple_id],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![q_id, repeat_id],
        });

    let text = render_input(&engine);
    assert!(
        text.contains("N_i A_{i,1} A_{i,2} … A_{i,C_i} M_i"),
        "got: {text}"
    );
}
