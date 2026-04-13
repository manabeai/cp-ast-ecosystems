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
use cp_ast_core::operation::{
    Action, AstEngine, ConstraintDef, ConstraintDefKind, FillContent, LengthSpec, VarType,
};
use cp_ast_core::projection::ProjectionAPI;
use cp_ast_core::render::{render_constraints, render_input};
use cp_ast_core::sample::{generate, sample_to_text, SampleValue};
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

/// Build N + Array A of length N entirely via operation Actions, then run the
/// full pipeline: completeness → render → generate → verify constraints.
#[test]
#[allow(clippy::too_many_lines)]
fn e2e_n_plus_array_via_operations() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // ── Build structure via operations ───────────────────────────────────
    // Create a Tuple header for N (Scalar needs Tuple wrapper for line separation)
    let header = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![] });

    // Add scalar N to header Tuple via AddSlotElement
    let n_id = engine
        .apply(&Action::AddSlotElement {
            parent: header,
            slot_name: "elements".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .copied()
        .unwrap();

    // Add array A of length N to root Sequence
    let a_id = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Array {
                name: "A".to_owned(),
                element_type: VarType::Int,
                length: LengthSpec::RefVar(n_id),
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .copied()
        .unwrap();

    // Wire root: Sequence → [header Tuple, Array A]
    if let Some(root_node) = engine.structure.get_mut(root) {
        root_node.set_kind(NodeKind::Sequence {
            children: vec![header, a_id],
        });
    }

    // ── AddConstraint for all bounds ────────────────────────────────────
    // 1 ≤ N ≤ 2×10^5 (as literal "200000")
    engine
        .apply(&Action::AddConstraint {
            target: n_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "200000".to_owned(),
                },
            },
        })
        .unwrap();

    // 0 ≤ A_i ≤ 10^9
    engine
        .apply(&Action::AddConstraint {
            target: a_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "0".to_owned(),
                    upper: "1000000000".to_owned(),
                },
            },
        })
        .unwrap();

    // Global guarantee
    engine
        .apply(&Action::AddConstraint {
            target: root,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Guarantee {
                    description: "All values are integers".to_owned(),
                },
            },
        })
        .unwrap();

    // ── Verify completeness ─────────────────────────────────────────────
    let summary = engine.completeness();
    assert_eq!(summary.total_holes, 0, "No holes should remain");
    assert!(summary.is_complete, "AST should be complete");

    // ── Render ──────────────────────────────────────────────────────────
    let input_text = render_input(&engine);
    assert!(
        input_text.contains('N'),
        "Input format should mention N, got: {input_text}"
    );
    assert!(
        input_text.contains('A'),
        "Input format should mention A, got: {input_text}"
    );

    let constraint_text = render_constraints(&engine);
    assert!(
        constraint_text.contains("1 ≤ N"),
        "Should show N lower bound, got: {constraint_text}"
    );

    // ── Generate 5 samples & verify constraints ─────────────────────────
    for seed in 0..5 {
        let sample = generate(&engine, seed).unwrap();

        // N: Int in [1, 200_000]
        let n_val = match sample.values.get(&n_id) {
            Some(SampleValue::Int(v)) => {
                assert!(
                    (1..=200_000).contains(v),
                    "seed {seed}: N={v} not in [1, 200000]"
                );
                *v
            }
            other => panic!("seed {seed}: expected Int for N, got {other:?}"),
        };

        // A: Array of length N, elements in [0, 10^9]
        match sample.values.get(&a_id) {
            Some(SampleValue::Array(arr)) => {
                assert_eq!(
                    arr.len(),
                    usize::try_from(n_val).unwrap(),
                    "seed {seed}: array len should equal N={n_val}"
                );
                for (i, elem) in arr.iter().enumerate() {
                    if let SampleValue::Int(v) = elem {
                        assert!(
                            (0..=1_000_000_000).contains(v),
                            "seed {seed}: A[{i}]={v} not in [0, 10^9]"
                        );
                    } else {
                        panic!("seed {seed}: A[{i}] should be Int");
                    }
                }
            }
            other => panic!("seed {seed}: expected Array for A, got {other:?}"),
        }

        // Rendered text should be parseable
        let text = sample_to_text(&engine, &sample);
        let lines: Vec<&str> = text.trim().lines().collect();
        assert!(
            lines.len() >= 2,
            "seed {seed}: should have ≥2 lines, got: {text:?}"
        );
        let parsed_n: i64 = lines[0]
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("seed {seed}: N should parse, got '{}'", lines[0]));
        assert_eq!(
            parsed_n, n_val,
            "seed {seed}: rendered N should match generated"
        );

        // Second line elements count should match N
        let elems: Vec<&str> = lines[1].split_whitespace().collect();
        assert_eq!(
            elems.len(),
            usize::try_from(n_val).unwrap(),
            "seed {seed}: array line should have {n_val} elements"
        );
    }
}
