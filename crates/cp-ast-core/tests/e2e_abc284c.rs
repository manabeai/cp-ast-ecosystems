//! End-to-end integration tests for full pipeline verification.
//!
//! Covers two `AtCoder` problems built via operation-based workflow:
//!
//! **ABC284-C (Graph / Connected Components)**
//!   Input: N M, then M lines of `u_i` `v_i`
//!   Constraints: 1 ≤ N ≤ 100, 0 ≤ M ≤ `N(N-1)/2`, 1 ≤ `u_i`,`v_i` ≤ N, simple graph
//!
//! **ABC300-A (Simple Arithmetic)**
//!   Input: A B (single line)
//!   Constraints: 1 ≤ A,B ≤ 100

use cp_ast_core::constraint::Expression;
use cp_ast_core::operation::{
    Action, AstEngine, ConstraintDef, ConstraintDefKind, FillContent, VarType,
};
use cp_ast_core::projection::ProjectionAPI;
use cp_ast_core::render::{render_constraints, render_input};
use cp_ast_core::sample::{SampleValue, generate, sample_to_text};
use cp_ast_core::structure::{NodeKind, Reference};

// ---------------------------------------------------------------------------
// ABC284-C: Graph — Build via operations → render → generate → verify
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::too_many_lines)]
fn e2e_abc284c_full_pipeline() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // ── Step 1: Build structure via operations ──────────────────────────
    // Create container nodes manually (FillContent doesn't support Tuple/Repeat),
    // then populate them with AddSlotElement operations.

    // Header tuple (N M) — start empty, fill via operations
    let tuple_header = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![] });

    // Edge tuple (u_i v_i) — start empty, fill via operations
    let tuple_edge = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![] });

    // Add N to header via AddSlotElement
    let n_id = engine
        .apply(&Action::AddSlotElement {
            parent: tuple_header,
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

    // Add M to header via AddSlotElement
    let m_id = engine
        .apply(&Action::AddSlotElement {
            parent: tuple_header,
            slot_name: "elements".to_owned(),
            element: FillContent::Scalar {
                name: "M".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .copied()
        .unwrap();

    // Add u to edge tuple
    let u_id = engine
        .apply(&Action::AddSlotElement {
            parent: tuple_edge,
            slot_name: "elements".to_owned(),
            element: FillContent::Scalar {
                name: "u".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .copied()
        .unwrap();

    // Add v to edge tuple
    let v_id = engine
        .apply(&Action::AddSlotElement {
            parent: tuple_edge,
            slot_name: "elements".to_owned(),
            element: FillContent::Scalar {
                name: "v".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .copied()
        .unwrap();

    // Repeat M times: edge tuple (manual — no FillContent for Repeat)
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(m_id)),
        index_var: None,
        body: vec![tuple_edge],
    });

    // Wire root Sequence → [tuple_header, repeat]
    if let Some(root_node) = engine.structure.get_mut(root) {
        root_node.set_kind(NodeKind::Sequence {
            children: vec![tuple_header, repeat],
        });
    }

    // ── Step 2: AddConstraint for all bounds ────────────────────────────

    // 1 ≤ N ≤ 100
    engine
        .apply(&Action::AddConstraint {
            target: n_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "100".to_owned(),
                },
            },
        })
        .unwrap();

    // 0 ≤ M ≤ N(N-1)/2  (use expression string for upper bound)
    engine
        .apply(&Action::AddConstraint {
            target: m_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "0".to_owned(),
                    upper: "N*(N-1)/2".to_owned(),
                },
            },
        })
        .unwrap();

    // 1 ≤ u_i ≤ N
    engine
        .apply(&Action::AddConstraint {
            target: u_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "N".to_owned(),
                },
            },
        })
        .unwrap();

    // 1 ≤ v_i ≤ N
    engine
        .apply(&Action::AddConstraint {
            target: v_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "N".to_owned(),
                },
            },
        })
        .unwrap();

    // Simple graph property
    engine
        .apply(&Action::AddConstraint {
            target: root,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Property {
                    tag: "simple_graph".to_owned(),
                },
            },
        })
        .unwrap();

    // ── Step 3: Verify completeness ─────────────────────────────────────
    let summary = engine.completeness();
    assert_eq!(summary.total_holes, 0, "No holes should remain");
    assert!(summary.is_complete, "AST should be complete");

    // ── Step 4: Render ──────────────────────────────────────────────────
    let input_text = render_input(&engine);
    assert!(
        input_text.contains("N M"),
        "Input format should show N M header, got: {input_text}"
    );
    assert!(
        input_text.contains('u') && input_text.contains('v'),
        "Input format should mention u/v edge vars, got: {input_text}"
    );

    let constraint_text = render_constraints(&engine);
    assert!(
        constraint_text.contains("1 ≤ N ≤ 100"),
        "Should show N constraint, got: {constraint_text}"
    );

    // ── Step 5: Generation with unresolved expression ──────────────────
    // The upper bound "N*(N-1)/2" is stored as an Unresolved name (the expression
    // parser doesn't yet support complex expressions). The new Result-based API
    // correctly reports this as an error instead of silently defaulting.
    let result = generate(&engine, 0);
    assert!(
        result.is_err(),
        "generate() should fail with unresolved expression in constraint"
    );
}

// ---------------------------------------------------------------------------
// ABC300-A: Simple Arithmetic — Scalar pair via operations, full pipeline
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::too_many_lines)]
fn e2e_abc300a_simple_arithmetic() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // ── Step 1: Build structure ─────────────────────────────────────────
    // Single-line pair: A B — use a Tuple with two AddSlotElement operations
    let tuple = engine
        .structure
        .add_node(NodeKind::Tuple { elements: vec![] });

    let a_id = engine
        .apply(&Action::AddSlotElement {
            parent: tuple,
            slot_name: "elements".to_owned(),
            element: FillContent::Scalar {
                name: "A".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .copied()
        .unwrap();

    let b_id = engine
        .apply(&Action::AddSlotElement {
            parent: tuple,
            slot_name: "elements".to_owned(),
            element: FillContent::Scalar {
                name: "B".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .copied()
        .unwrap();

    // Root → Sequence[tuple]
    if let Some(root_node) = engine.structure.get_mut(root) {
        root_node.set_kind(NodeKind::Sequence {
            children: vec![tuple],
        });
    }

    // ── Step 2: AddConstraint ───────────────────────────────────────────
    // 1 ≤ A ≤ 100
    engine
        .apply(&Action::AddConstraint {
            target: a_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "100".to_owned(),
                },
            },
        })
        .unwrap();

    // 1 ≤ B ≤ 100
    engine
        .apply(&Action::AddConstraint {
            target: b_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "100".to_owned(),
                },
            },
        })
        .unwrap();

    // Guarantee: "All values are integers"
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

    // ── Step 3: Completeness ────────────────────────────────────────────
    let summary = engine.completeness();
    assert_eq!(summary.total_holes, 0);
    assert!(summary.is_complete);

    // ── Step 4: Render ──────────────────────────────────────────────────
    let input_text = render_input(&engine);
    assert!(
        input_text.contains('A') && input_text.contains('B'),
        "Should render A B, got: {input_text}"
    );

    let constraint_text = render_constraints(&engine);
    assert!(
        constraint_text.contains("1 ≤ A ≤ 100"),
        "Should show A constraint, got: {constraint_text}"
    );
    assert!(
        constraint_text.contains("1 ≤ B ≤ 100"),
        "Should show B constraint, got: {constraint_text}"
    );

    // ── Step 5: Generate 5 samples & verify ─────────────────────────────
    for seed in 0..5 {
        let sample = generate(&engine, seed).unwrap();

        // A in [1, 100]
        let a_val = match sample.values.get(&a_id) {
            Some(SampleValue::Int(v)) => {
                assert!((1..=100).contains(v), "seed {seed}: A={v} not in [1, 100]");
                *v
            }
            other => panic!("seed {seed}: expected Int for A, got {other:?}"),
        };

        // B in [1, 100]
        let b_val = match sample.values.get(&b_id) {
            Some(SampleValue::Int(v)) => {
                assert!((1..=100).contains(v), "seed {seed}: B={v} not in [1, 100]");
                *v
            }
            other => panic!("seed {seed}: expected Int for B, got {other:?}"),
        };

        // Render and verify parseable
        let text = sample_to_text(&engine, &sample);
        let tokens: Vec<&str> = text.split_whitespace().collect();
        assert!(
            tokens.len() >= 2,
            "seed {seed}: should have ≥2 tokens, got: {text:?}"
        );
        let parsed_a: i64 = tokens[0]
            .parse()
            .unwrap_or_else(|_| panic!("seed {seed}: A should parse, got '{}'", tokens[0]));
        let parsed_b: i64 = tokens[1]
            .parse()
            .unwrap_or_else(|_| panic!("seed {seed}: B should parse, got '{}'", tokens[1]));
        assert_eq!(parsed_a, a_val, "seed {seed}: rendered A mismatch");
        assert_eq!(parsed_b, b_val, "seed {seed}: rendered B mismatch");
    }
}
