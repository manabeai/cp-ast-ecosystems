//! End-to-end integration tests for Gap Resolution (A, B, H, D).
//!
//! Tests real-world competitive programming input patterns that were
//! previously impossible to express with the AST.

use cp_ast_core::constraint::{ArithOp, Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::render_input;
use cp_ast_core::render_tex::{render_input_tex, TexOptions};
use cp_ast_core::sample::generator::generate;
use cp_ast_core::sample::output::sample_to_text;
use cp_ast_core::structure::{Ident, Literal, NodeKind, Reference};

/// Gap A: Graph problem — N nodes, N-1 edges (tree input).
#[test]
fn e2e_graph_tree_n_minus_1_edges() {
    let mut engine = AstEngine::default();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(5),
            upper: Expression::Lit(5),
        },
    );

    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    engine.constraints.add(
        Some(u_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(u_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(u_id),
        Constraint::Range {
            target: Reference::VariableRef(u_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );

    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    engine.constraints.add(
        Some(v_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(v_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(v_id),
        Constraint::Range {
            target: Reference::VariableRef(v_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );

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

    let header_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![header_id, repeat_id],
        });

    // Sample generation
    let sample = generate(&engine, 42).unwrap();
    let output = sample_to_text(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 5, "expected 5 lines, got: {lines:?}"); // N + N-1 edges
    assert_eq!(lines[0], "5");

    for line in &lines[1..] {
        let parts: Vec<i64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(parts.len(), 2);
        assert!((1..=5).contains(&parts[0]));
        assert!((1..=5).contains(&parts[1]));
    }

    // TeX rendering
    let tex_result = render_input_tex(&engine, &TexOptions::default());
    assert!(
        tex_result.tex.contains("N - 1"),
        "TeX should show N - 1 count: {}",
        tex_result.tex
    );

    // Plain text rendering
    let text = render_input(&engine);
    assert!(text.contains("u_i v_i"), "plain text: {text}");
}

/// Gap H + D: Triangular matrix — row i has N-i-1 elements.
#[test]
fn e2e_triangular_matrix_via_repeat_loop_var() {
    let mut engine = AstEngine::default();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(4),
            upper: Expression::Lit(4),
        },
    );

    // Array C with length = N - i - 1
    let c_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("C"),
        length: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::BinOp {
                op: ArithOp::Sub,
                lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
                rhs: Box::new(Expression::Var(Reference::Unresolved(Ident::new("i")))),
            }),
            rhs: Box::new(Expression::Lit(1)),
        },
    });
    engine.constraints.add(
        Some(c_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(c_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(c_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: c_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(0),
            upper: Expression::Lit(99),
        },
    );

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Lit(1)),
        },
        index_var: Some(Ident::new("i")),
        body: vec![c_id],
    });

    let header_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![header_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let output = sample_to_text(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();

    // N=4, N-1=3 rows: i=0 → 3 elements, i=1 → 2, i=2 → 1
    assert_eq!(lines[0], "4");
    assert_eq!(lines.len(), 4); // N + 3 rows
    assert_eq!(lines[1].split_whitespace().count(), 3); // N-0-1=3
    assert_eq!(lines[2].split_whitespace().count(), 2); // N-1-1=2
    assert_eq!(lines[3].split_whitespace().count(), 1); // N-2-1=1
}

/// Gap B: Query problem — Q queries with tag-dependent variants.
#[test]
#[allow(clippy::too_many_lines)]
fn e2e_query_problem_choice_in_repeat() {
    let mut engine = AstEngine::default();

    let q_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    engine.constraints.add(
        Some(q_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(q_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(q_id),
        Constraint::Range {
            target: Reference::VariableRef(q_id),
            lower: Expression::Lit(20),
            upper: Expression::Lit(20),
        },
    );

    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });

    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    engine.constraints.add(
        Some(x_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(x_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(x_id),
        Constraint::Range {
            target: Reference::VariableRef(x_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(1000),
        },
    );

    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    engine.constraints.add(
        Some(y_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(y_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(y_id),
        Constraint::Range {
            target: Reference::VariableRef(y_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(1000),
        },
    );

    let z_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Z"),
    });
    engine.constraints.add(
        Some(z_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(z_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(z_id),
        Constraint::Range {
            target: Reference::VariableRef(z_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(1000),
        },
    );

    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id, z_id]),
        ],
    });

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(q_id)),
        index_var: None,
        body: vec![choice_id],
    });

    let header_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![q_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![header_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let output = sample_to_text(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();

    assert_eq!(lines[0], "20");
    assert_eq!(lines.len(), 21); // Q + 20 query lines

    let mut type1_count = 0;
    let mut type2_count = 0;
    for line in &lines[1..] {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "1" => {
                assert_eq!(parts.len(), 2, "type 1 should have tag + X");
                type1_count += 1;
            }
            "2" => {
                assert_eq!(parts.len(), 3, "type 2 should have tag + Y + Z");
                type2_count += 1;
            }
            other => panic!("unexpected tag: {other}"),
        }
    }
    assert!(type1_count > 0, "should have at least one type-1 query");
    assert!(type2_count > 0, "should have at least one type-2 query");

    // TeX rendering should use cases environment
    let tex_result = render_input_tex(&engine, &TexOptions::default());
    assert!(
        tex_result.tex.contains("\\begin{cases}"),
        "TeX should use cases: {}",
        tex_result.tex
    );

    // Plain text should show If T = k: ...
    let text = render_input(&engine);
    assert!(text.contains("If T = 1:"), "plain text: {text}");
    assert!(text.contains("If T = 2:"), "plain text: {text}");
}
