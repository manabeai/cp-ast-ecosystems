use cp_ast_core::constraint::{ArithOp, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render_tex::{render_constraints_tex, render_input_tex, SectionMode, TexOptions};
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
