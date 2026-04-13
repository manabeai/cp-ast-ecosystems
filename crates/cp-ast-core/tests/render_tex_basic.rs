use cp_ast_core::constraint::{ArithOp, Constraint, DistinctUnit, Expression, SortOrder};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render_tex::{
    render_constraints_tex, render_full_tex, render_input_tex, SectionMode, TexOptions,
};
use cp_ast_core::render_tex::{tex_helpers, TexWarning};
use cp_ast_core::structure::{Ident, NodeKind, Reference};

#[test]
fn empty_engine_produces_empty_tex() {
    let engine = AstEngine::new();
    let options = TexOptions::default();

    let input_result = render_input_tex(&engine, &options);
    assert!(input_result.tex.is_empty() || input_result.tex.trim().is_empty());
    assert!(input_result.warnings.is_empty());

    let constraint_result = render_constraints_tex(&engine, &options);
    assert!(constraint_result.tex.is_empty());
    assert!(constraint_result.warnings.is_empty());
}

#[test]
fn default_options() {
    let options = TexOptions::default();
    assert_eq!(options.section_mode, SectionMode::Fragment);
    assert!(options.include_holes);
}

// ---- ident_to_tex tests ----

#[test]
fn ident_single_upper() {
    assert_eq!(tex_helpers::ident_to_tex(&Ident::new("N")), "N");
}

#[test]
fn ident_single_lower() {
    assert_eq!(tex_helpers::ident_to_tex(&Ident::new("x")), "x");
}

#[test]
fn ident_multi_char() {
    assert_eq!(
        tex_helpers::ident_to_tex(&Ident::new("ans")),
        "\\mathrm{ans}"
    );
}

// ---- expression_to_tex tests ----

#[test]
fn expr_literal_small() {
    let engine = AstEngine::new();
    let mut w = vec![];
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &Expression::Lit(42), &mut w),
        "42"
    );
    assert!(w.is_empty());
}

#[test]
fn expr_literal_power_of_10() {
    let engine = AstEngine::new();
    let mut w = vec![];
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &Expression::Lit(100_000), &mut w),
        "10^{5}"
    );
}

#[test]
fn expr_literal_a_times_power_of_10() {
    let engine = AstEngine::new();
    let mut w = vec![];
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &Expression::Lit(200_000), &mut w),
        "2 \\times 10^{5}"
    );
}

#[test]
fn expr_pow() {
    let engine = AstEngine::new();
    let mut w = vec![];
    let expr = Expression::Pow {
        base: Box::new(Expression::Lit(10)),
        exp: Box::new(Expression::Lit(9)),
    };
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &expr, &mut w),
        "10^{9}"
    );
}

#[test]
fn expr_binop_mul() {
    let engine = AstEngine::new();
    let mut w = vec![];
    let expr = Expression::BinOp {
        op: ArithOp::Mul,
        lhs: Box::new(Expression::Lit(2)),
        rhs: Box::new(Expression::Pow {
            base: Box::new(Expression::Lit(10)),
            exp: Box::new(Expression::Lit(5)),
        }),
    };
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &expr, &mut w),
        "2 \\times 10^{5}"
    );
}

#[test]
fn expr_binop_add_var() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let mut w = vec![];
    let expr = Expression::BinOp {
        op: ArithOp::Add,
        lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
        rhs: Box::new(Expression::Lit(1)),
    };
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &expr, &mut w),
        "N + 1"
    );
}

#[test]
fn expr_fncall_min() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let mut w = vec![];
    let expr = Expression::FnCall {
        name: Ident::new("min"),
        args: vec![
            Expression::Var(Reference::VariableRef(n_id)),
            Expression::Var(Reference::VariableRef(m_id)),
        ],
    };
    assert_eq!(
        tex_helpers::expression_to_tex(&engine, &expr, &mut w),
        "\\min(N, M)"
    );
}

// ---- reference_to_tex tests ----

#[test]
fn ref_variable_scalar() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let mut w = vec![];
    assert_eq!(
        tex_helpers::reference_to_tex(&engine, &Reference::VariableRef(n_id), &mut w),
        "N"
    );
}

#[test]
fn ref_indexed() {
    let mut engine = AstEngine::new();
    let c_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("C"),
    });
    let mut w = vec![];
    let reference = Reference::IndexedRef {
        target: c_id,
        indices: vec![Ident::new("i"), Ident::new("j")],
    };
    assert_eq!(
        tex_helpers::reference_to_tex(&engine, &reference, &mut w),
        "C_{i,j}"
    );
}

#[test]
fn ref_unresolved_emits_warning() {
    let engine = AstEngine::new();
    let mut w = vec![];
    let result =
        tex_helpers::reference_to_tex(&engine, &Reference::Unresolved(Ident::new("X")), &mut w);
    assert_eq!(result, "X");
    assert_eq!(w.len(), 1);
    assert!(matches!(&w[0], TexWarning::UnresolvedReference { name } if name == "X"));
}

// ---- IndexAllocator tests ----

#[test]
fn index_allocator_sequential() {
    let mut alloc = tex_helpers::IndexAllocator::new();
    assert_eq!(alloc.allocate(), 'i');
    assert_eq!(alloc.allocate(), 'j');
    assert_eq!(alloc.allocate(), 'k');
}

// ---- Constraint TeX tests ----

#[test]
fn constraint_tex_scalar_range() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::BinOp {
                op: ArithOp::Mul,
                lhs: Box::new(Expression::Lit(2)),
                rhs: Box::new(Expression::Pow {
                    base: Box::new(Expression::Lit(10)),
                    exp: Box::new(Expression::Lit(5)),
                }),
            },
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $1 \\le N \\le 2 \\times 10^{5}$\n\\end{itemize}\n"
    );
    assert!(result.warnings.is_empty());
}

#[test]
fn constraint_tex_array_element_with_index_range() {
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
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::VariableRef(a_id),
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(9)),
            },
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $1 \\le A_i \\le 10^{9} \\ (1 \\le i \\le N)$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_type_decl_skipped() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: cp_ast_core::constraint::ExpectedType::Int,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert!(result.tex.is_empty());
}

#[test]
fn constraint_tex_sum_bound() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        None,
        Constraint::SumBound {
            variable: Reference::VariableRef(n_id),
            upper: Expression::BinOp {
                op: ArithOp::Mul,
                lhs: Box::new(Expression::Lit(2)),
                rhs: Box::new(Expression::Pow {
                    base: Box::new(Expression::Lit(10)),
                    exp: Box::new(Expression::Lit(5)),
                }),
            },
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $\\sum N \\le 2 \\times 10^{5}$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_distinct() {
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
    engine.constraints.add(
        Some(a_id),
        Constraint::Distinct {
            elements: Reference::VariableRef(a_id),
            unit: DistinctUnit::Element,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $A_i \\neq A_j \\ (i \\neq j)$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_sorted() {
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
    engine.constraints.add(
        Some(a_id),
        Constraint::Sorted {
            elements: Reference::VariableRef(a_id),
            order: SortOrder::Ascending,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $A_1 \\le A_2 \\le \\cdots \\le A_N$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_string_length() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let s_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("S"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, s_id],
        });
    }
    engine.constraints.add(
        Some(s_id),
        Constraint::StringLength {
            target: Reference::VariableRef(s_id),
            min: Expression::Lit(1),
            max: Expression::Var(Reference::VariableRef(n_id)),
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item $1 \\le |S| \\le N$\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_guarantee() {
    let mut engine = AstEngine::new();
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "The answer always exists.".to_owned(),
            predicate: None,
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\begin{itemize}\n  \\item The answer always exists.\n\\end{itemize}\n"
    );
}

#[test]
fn constraint_tex_ordering() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    // Add in reverse order — Guarantee first, then Range
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

    let result = render_constraints_tex(&engine, &TexOptions::default());
    // Range should come before Guarantee regardless of insertion order
    let lines: Vec<&str> = result.tex.lines().collect();
    assert!(lines[1].contains("1 \\le N \\le 10^{2}"));
    assert!(lines[2].contains("answer exists"));
}

// ---- Input Format TeX tests ----

#[test]
fn input_tex_single_scalar() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(result.tex, "\\[\n\\begin{array}{l}\nN\n\\end{array}\n\\]\n");
    assert!(result.warnings.is_empty());
}

#[test]
fn input_tex_tuple() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![tuple_id],
        });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\[\n\\begin{array}{l}\nN \\ M\n\\end{array}\n\\]\n"
    );
}

#[test]
fn input_tex_array() {
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

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\[\n\\begin{array}{l}\nN \\\\\nA_1 \\ A_2 \\ \\cdots \\ A_N\n\\end{array}\n\\]\n"
    );
}

#[test]
fn input_tex_repeat_scalar() {
    let mut engine = AstEngine::new();
    let q_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(q_id)),
        index_var: None,
        body: vec![t_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![q_id, repeat_id],
        });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\[\n\\begin{array}{l}\nQ \\\\\nT_1 \\\\\nT_2 \\\\\n\\vdots \\\\\nT_Q\n\\end{array}\n\\]\n"
    );
}

#[test]
fn input_tex_repeat_tuple() {
    let mut engine = AstEngine::new();
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
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
        count: Expression::Var(Reference::VariableRef(m_id)),
        index_var: None,
        body: vec![tuple_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![m_id, repeat_id],
        });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert_eq!(
        result.tex,
        "\\[\n\\begin{array}{l}\nM \\\\\nu_1 \\ v_1 \\\\\nu_2 \\ v_2 \\\\\n\\vdots \\\\\nu_M \\ v_M\n\\end{array}\n\\]\n"
    );
}

#[test]
fn input_tex_hole() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    assert!(result.tex.contains("\\texttt{<hole>}"));
    assert_eq!(result.warnings.len(), 1);
    assert!(matches!(
        &result.warnings[0],
        TexWarning::HoleEncountered { .. }
    ));
}

#[test]
fn input_tex_combined_n_array_repeat() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let q_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, q_id],
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("D"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(q_id)),
        index_var: None,
        body: vec![t_id],
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header, a_id, repeat_id],
        });
    }

    let result = render_input_tex(&engine, &TexOptions::default());
    let expected = "\
\\[\n\
\\begin{array}{l}\n\
N \\ Q \\\\\n\
D_1 \\ D_2 \\ \\cdots \\ D_N \\\\\n\
T_1 \\\\\n\
T_2 \\\\\n\
\\vdots \\\\\n\
T_Q\n\
\\end{array}\n\
\\]\n";
    assert_eq!(result.tex, expected);
}

// ---- render_full_tex tests ----

#[test]
fn full_tex_fragment_mode() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let result = render_full_tex(&engine, &TexOptions::default());
    // Fragment mode: input + blank line + constraints
    assert!(result.tex.contains("\\begin{array}"));
    assert!(result.tex.contains("\\begin{itemize}"));
}

#[test]
fn full_tex_standalone_mode() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let options = TexOptions {
        section_mode: SectionMode::Standalone,
        include_holes: true,
    };
    let result = render_full_tex(&engine, &options);
    assert!(result.tex.contains("\\paragraph{入力}"));
    assert!(result.tex.contains("\\paragraph{制約}"));
}

// ---- End-to-end test ----

#[test]
fn e2e_graph_problem_tex() {
    // Build a graph problem: N M header, M edges (u_i v_i), constraints
    let mut engine = AstEngine::new();

    // Variables
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });

    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    let edge_tuple = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let edges = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(m_id)),
        index_var: None,
        body: vec![edge_tuple],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header, edges],
        });
    }

    // Constraints
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(m_id),
        Constraint::Range {
            target: Reference::VariableRef(m_id),
            lower: Expression::Lit(0),
            upper: Expression::BinOp {
                op: ArithOp::Div,
                lhs: Box::new(Expression::BinOp {
                    op: ArithOp::Mul,
                    lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
                    rhs: Box::new(Expression::BinOp {
                        op: ArithOp::Sub,
                        lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
                        rhs: Box::new(Expression::Lit(1)),
                    }),
                }),
                rhs: Box::new(Expression::Lit(2)),
            },
        },
    );

    // Input TeX
    let input_result = render_input_tex(&engine, &TexOptions::default());
    let expected_input = "\
\\[\n\
\\begin{array}{l}\n\
N \\ M \\\\\n\
u_1 \\ v_1 \\\\\n\
u_2 \\ v_2 \\\\\n\
\\vdots \\\\\n\
u_M \\ v_M\n\
\\end{array}\n\
\\]\n";
    assert_eq!(input_result.tex, expected_input);

    // Constraint TeX
    let constraint_result = render_constraints_tex(&engine, &TexOptions::default());
    assert!(constraint_result.tex.contains("$1 \\le N \\le 10^{2}$"));
    assert!(constraint_result.tex.contains("$0 \\le M \\le"));

    // Full TeX
    let full_result = render_full_tex(
        &engine,
        &TexOptions {
            section_mode: SectionMode::Standalone,
            include_holes: true,
        },
    );
    assert!(full_result.tex.contains("\\paragraph{入力}"));
    assert!(full_result.tex.contains("\\paragraph{制約}"));
    assert!(full_result.warnings.is_empty());
}

#[test]
fn include_holes_false_suppresses_holes() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let options = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    let result = render_input_tex(&engine, &options);
    assert!(!result.tex.contains("hole"));
    assert!(result.warnings.is_empty());
}
