//! Integration test: express a typical `AtCoder` ABC problem with Rev.1 types.
//!
//! Problem: N + array A of length N.
//! Input:
//!   N
//!   `A_1` `A_2` ... `A_N`
//! Constraints:
//!   1 ≤ N ≤ 2×10^5
//!   0 ≤ `A_i` ≤ 10^9
//!   All values are integers

use cp_ast_core::constraint::*;
use cp_ast_core::structure::*;

#[test]
fn express_n_plus_array_rev1() {
    // --- Build StructureAST ---
    let mut ast = StructureAst::new();

    // Scalar N
    let n_id = ast.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    // Array A with length referencing N
    let a_id = ast.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Reference::VariableRef(n_id),
    });

    // Tuple for header line: (N) — single element on first line
    let header_id = ast.add_node(NodeKind::Tuple {
        elements: vec![n_id],
    });

    // Connect root Sequence → [header, A]
    if let Some(root) = ast.get_mut(ast.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header_id, a_id],
        });
    }

    // Verify structure
    assert_eq!(ast.len(), 4); // root + N + A + header
    assert!(ast.contains(n_id));
    assert!(ast.contains(a_id));

    // --- Build ConstraintAST ---
    let mut constraints = ConstraintSet::new();

    // N: Int
    constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );

    // 1 ≤ N ≤ 2×10^5
    constraints.add(
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

    // A elements: Int
    constraints.add(
        Some(a_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(a_id),
            expected: ExpectedType::Int,
        },
    );

    // 0 ≤ A_i ≤ 10^9
    constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(0),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(9)),
            },
        },
    );

    // All values are integers (global guarantee)
    constraints.add(
        None,
        Constraint::Guarantee {
            description: "All values are integers".to_owned(),
            predicate: None,
        },
    );

    assert_eq!(constraints.len(), 5);
    assert_eq!(constraints.for_node(n_id).len(), 2);
    assert_eq!(constraints.for_node(a_id).len(), 2);
    assert_eq!(constraints.global().len(), 1);
}

#[test]
fn express_problem_with_holes_rev1() {
    let mut ast = StructureAst::new();

    let n_id = ast.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let hole_id = ast.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyArray),
    });

    if let Some(root) = ast.get_mut(ast.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, hole_id],
        });
    }

    // Verify hole exists
    let hole_node = ast.get(hole_id).unwrap();
    assert!(matches!(hole_node.kind(), NodeKind::Hole { .. }));

    // Count holes
    let hole_count = ast
        .iter()
        .filter(|n| matches!(n.kind(), NodeKind::Hole { .. }))
        .count();
    assert_eq!(hole_count, 1);
}
