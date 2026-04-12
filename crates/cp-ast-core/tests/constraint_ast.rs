use cp_ast_core::constraint::*;
use cp_ast_core::structure::*;

// --- Expression tests ---

#[test]
fn expression_lit() {
    let e = Expression::Lit(42);
    assert_eq!(e, Expression::Lit(42));
}

#[test]
fn expression_var() {
    let e = Expression::Var(Reference::VariableRef(NodeId::from_raw(1)));
    assert!(matches!(e, Expression::Var(_)));
}

#[test]
fn expression_binop_add() {
    let e = Expression::BinOp {
        op: ArithOp::Add,
        lhs: Box::new(Expression::Lit(1)),
        rhs: Box::new(Expression::Lit(2)),
    };
    if let Expression::BinOp { op, lhs, rhs } = &e {
        assert_eq!(*op, ArithOp::Add);
        assert_eq!(**lhs, Expression::Lit(1));
        assert_eq!(**rhs, Expression::Lit(2));
    } else {
        panic!("Expected BinOp variant");
    }
}

#[test]
fn expression_pow() {
    let e = Expression::Pow {
        base: Box::new(Expression::Lit(10)),
        exp: Box::new(Expression::Lit(9)),
    };
    if let Expression::Pow { base, exp } = &e {
        assert_eq!(**base, Expression::Lit(10));
        assert_eq!(**exp, Expression::Lit(9));
    } else {
        panic!("Expected Pow variant");
    }
}

#[test]
fn expression_fncall() {
    let e = Expression::FnCall {
        name: Ident::new("max"),
        args: vec![Expression::Lit(1), Expression::Lit(2)],
    };
    if let Expression::FnCall { name, args } = &e {
        assert_eq!(name.as_str(), "max");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], Expression::Lit(1));
        assert_eq!(args[1], Expression::Lit(2));
    } else {
        panic!("Expected FnCall variant");
    }
}

#[test]
fn expression_evaluate_lit() {
    assert_eq!(Expression::Lit(42).evaluate_constant(), Some(42));
}

#[test]
fn expression_evaluate_pow() {
    let e = Expression::Pow {
        base: Box::new(Expression::Lit(10)),
        exp: Box::new(Expression::Lit(9)),
    };
    assert_eq!(e.evaluate_constant(), Some(1_000_000_000));
}

#[test]
fn expression_evaluate_binop_mul() {
    let e = Expression::BinOp {
        op: ArithOp::Mul,
        lhs: Box::new(Expression::Lit(2)),
        rhs: Box::new(Expression::Pow {
            base: Box::new(Expression::Lit(10)),
            exp: Box::new(Expression::Lit(5)),
        }),
    };
    assert_eq!(e.evaluate_constant(), Some(200_000));
}

#[test]
fn expression_evaluate_binop_add() {
    let e = Expression::BinOp {
        op: ArithOp::Add,
        lhs: Box::new(Expression::Lit(3)),
        rhs: Box::new(Expression::Lit(7)),
    };
    assert_eq!(e.evaluate_constant(), Some(10));
}

#[test]
fn expression_evaluate_binop_sub() {
    let e = Expression::BinOp {
        op: ArithOp::Sub,
        lhs: Box::new(Expression::Lit(10)),
        rhs: Box::new(Expression::Lit(3)),
    };
    assert_eq!(e.evaluate_constant(), Some(7));
}

#[test]
fn expression_evaluate_binop_div() {
    let e = Expression::BinOp {
        op: ArithOp::Div,
        lhs: Box::new(Expression::Lit(10)),
        rhs: Box::new(Expression::Lit(3)),
    };
    assert_eq!(e.evaluate_constant(), Some(3));
}

#[test]
fn expression_evaluate_div_by_zero() {
    let e = Expression::BinOp {
        op: ArithOp::Div,
        lhs: Box::new(Expression::Lit(10)),
        rhs: Box::new(Expression::Lit(0)),
    };
    assert_eq!(e.evaluate_constant(), None);
}

#[test]
fn expression_evaluate_overflow_returns_none() {
    let e = Expression::BinOp {
        op: ArithOp::Add,
        lhs: Box::new(Expression::Lit(i64::MAX)),
        rhs: Box::new(Expression::Lit(1)),
    };
    assert_eq!(e.evaluate_constant(), None);
}

#[test]
fn expression_evaluate_pow_overflow_returns_none() {
    let e = Expression::Pow {
        base: Box::new(Expression::Lit(10)),
        exp: Box::new(Expression::Lit(19)),
    };
    assert_eq!(e.evaluate_constant(), None);
}

#[test]
fn expression_evaluate_var_returns_none() {
    let e = Expression::Var(Reference::VariableRef(NodeId::from_raw(1)));
    assert_eq!(e.evaluate_constant(), None);
}

#[test]
fn expression_evaluate_fncall_returns_none() {
    let e = Expression::FnCall {
        name: Ident::new("max"),
        args: vec![Expression::Lit(1)],
    };
    assert_eq!(e.evaluate_constant(), None);
}

// --- ExpectedType tests ---

#[test]
fn expected_type_equality() {
    assert_eq!(ExpectedType::Int, ExpectedType::Int);
}

#[test]
fn expected_type_all_variants() {
    let types = [ExpectedType::Int, ExpectedType::Str, ExpectedType::Char];
    assert_eq!(types.len(), 3);
}

// --- Constraint tests (all 12 variants) ---

#[test]
fn constraint_range() {
    let n = NodeId::from_raw(1);
    let c = Constraint::Range {
        target: Reference::VariableRef(n),
        lower: Expression::Lit(1),
        upper: Expression::Lit(100),
    };
    if let Constraint::Range {
        target,
        lower,
        upper,
    } = &c
    {
        assert_eq!(*target, Reference::VariableRef(n));
        assert_eq!(*lower, Expression::Lit(1));
        assert_eq!(*upper, Expression::Lit(100));
    } else {
        panic!("Expected Range variant");
    }
}

#[test]
fn constraint_type_decl() {
    let n = NodeId::from_raw(1);
    let c = Constraint::TypeDecl {
        target: Reference::VariableRef(n),
        expected: ExpectedType::Int,
    };
    if let Constraint::TypeDecl { target, expected } = &c {
        assert_eq!(*target, Reference::VariableRef(n));
        assert_eq!(*expected, ExpectedType::Int);
    } else {
        panic!("Expected TypeDecl variant");
    }
}

#[test]
fn constraint_length_relation() {
    let c = Constraint::LengthRelation {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        length: Expression::Var(Reference::VariableRef(NodeId::from_raw(2))),
    };
    assert!(matches!(c, Constraint::LengthRelation { .. }));
}

#[test]
fn constraint_relation() {
    let c = Constraint::Relation {
        lhs: Expression::Var(Reference::VariableRef(NodeId::from_raw(1))),
        op: RelationOp::Le,
        rhs: Expression::Var(Reference::VariableRef(NodeId::from_raw(2))),
    };
    if let Constraint::Relation { lhs, op, rhs } = &c {
        assert_eq!(*op, RelationOp::Le);
        assert!(matches!(lhs, Expression::Var(Reference::VariableRef(_))));
        assert!(matches!(rhs, Expression::Var(Reference::VariableRef(_))));
    } else {
        panic!("Expected Relation variant");
    }
}

#[test]
fn constraint_distinct() {
    let c = Constraint::Distinct {
        elements: Reference::VariableRef(NodeId::from_raw(1)),
        unit: DistinctUnit::Element,
    };
    assert!(matches!(c, Constraint::Distinct { .. }));
}

#[test]
fn constraint_property() {
    let c = Constraint::Property {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        tag: PropertyTag::Simple,
    };
    assert!(matches!(c, Constraint::Property { .. }));
}

#[test]
fn constraint_sum_bound() {
    let c = Constraint::SumBound {
        variable: Reference::VariableRef(NodeId::from_raw(1)),
        upper: Expression::Lit(200_000),
    };
    assert!(matches!(c, Constraint::SumBound { .. }));
}

#[test]
fn constraint_sorted() {
    let c = Constraint::Sorted {
        elements: Reference::VariableRef(NodeId::from_raw(1)),
        order: SortOrder::Ascending,
    };
    assert!(matches!(c, Constraint::Sorted { .. }));
}

#[test]
fn constraint_guarantee() {
    let c = Constraint::Guarantee {
        description: "Input is valid".to_owned(),
        predicate: None,
    };
    assert!(matches!(c, Constraint::Guarantee { .. }));
}

#[test]
fn constraint_charset() {
    let c = Constraint::CharSet {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        charset: CharSetSpec::LowerAlpha,
    };
    assert!(matches!(c, Constraint::CharSet { .. }));
}

#[test]
fn constraint_string_length() {
    let c = Constraint::StringLength {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        min: Expression::Lit(1),
        max: Expression::Lit(100),
    };
    assert!(matches!(c, Constraint::StringLength { .. }));
}

#[test]
fn constraint_render_hint() {
    let c = Constraint::RenderHint {
        target: Reference::VariableRef(NodeId::from_raw(1)),
        hint: RenderHintKind::Separator(Separator::Space),
    };
    assert!(matches!(c, Constraint::RenderHint { .. }));
}
