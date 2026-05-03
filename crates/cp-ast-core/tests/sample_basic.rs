//! Tests for sample generation: dependency graph, topological sort, generators, and output.

use cp_ast_core::constraint::*;
use cp_ast_core::operation::AstEngine;
use cp_ast_core::sample::*;
use cp_ast_core::structure::*;

/// Helper: build a simple "N + Array A of length N" engine.
fn build_n_plus_array_engine() -> AstEngine {
    let mut engine = AstEngine::new();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header, a_id],
        });
    }

    // N: Int, 1 ≤ N ≤ 10
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
            lower: Expression::Lit(1),
            upper: Expression::Lit(10),
        },
    );

    // A elements: Int, 0 ≤ A_i ≤ 100
    engine.constraints.add(
        Some(a_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(a_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(0),
            upper: Expression::Lit(100),
        },
    );

    engine
}

#[test]
fn dependency_graph_simple() {
    let engine = build_n_plus_array_engine();
    let graph = DependencyGraph::build(&engine);

    // Find the array node (id 2, since root=0, N=1, A=2)
    let all = graph.all_nodes();
    assert!(all.len() >= 4); // root, N, A, header (Tuple)

    // Array A should depend on scalar N (its length reference)
    let n_id = NodeId::from_raw(1);
    let a_id = NodeId::from_raw(2);
    let a_deps = graph.dependencies_of(a_id);
    assert!(
        a_deps.contains(&n_id),
        "Array A should depend on scalar N for length"
    );
}

#[test]
fn topo_sort_linear() {
    let engine = build_n_plus_array_engine();
    let graph = DependencyGraph::build(&engine);
    let order = graph.topological_sort().expect("should not have cycle");

    // N must come before A in generation order
    let n_id = NodeId::from_raw(1);
    let a_id = NodeId::from_raw(2);
    let n_pos = order.iter().position(|id| *id == n_id);
    let a_pos = order.iter().position(|id| *id == a_id);

    assert!(
        n_pos.is_some() && a_pos.is_some(),
        "Both N and A must be in the sort order"
    );
    assert!(
        n_pos.unwrap() < a_pos.unwrap(),
        "N must come before A (N at {n_pos:?}, A at {a_pos:?})",
    );
}

#[test]
fn topo_sort_detects_cycle() {
    // Create a synthetic cycle by making two nodes depend on each other
    // through array length references.
    let mut engine = AstEngine::new();

    // Node A references B's length, and B references A's length → cycle
    let a_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("A"),
    });
    let b_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("B"),
    });

    // Make a_arr depend on b_id for length
    let a_arr = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("AA"),
        length: Expression::Var(Reference::VariableRef(b_id)),
    });
    // Make b_arr depend on a_arr for length (creates a cycle a_arr→b_id, but not directly)
    // Instead, use a Repeat whose count is a_arr, with b_id in its body
    // Actually, we need a real structural cycle.

    // Simpler: Create arrays that reference each other's scalars where the scalars
    // are children of containers that depend on the arrays.
    // Let's test with the graph directly by verifying a clean graph passes
    // and then checking that cycles in graphs are detected.

    // Build a graph where both arrays are children of root sequence,
    // but a_arr depends on b_id and b_arr depends on a_id.
    let b_arr = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("BB"),
        length: Expression::Var(Reference::VariableRef(a_id)),
    });

    // Make a_id a child of a container that depends on b_arr
    // (Tuple containing a_id, which is a child → depends on parent)
    let t1 = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![a_id],
    });
    // t1 depends on b_arr (put it as child of a sequence that includes b_arr first)
    // Actually the cycle is: a_arr depends on b_id, and b_arr depends on a_id.
    // There's no cycle there unless a_id depends on a_arr or b_id depends on b_arr.
    // Let's create the cycle explicitly:
    // Make root → sequence containing a_arr and b_arr
    // a_arr depends on b_id (length ref) AND b_id is a child of b_arr's container
    // Actually, let's just make a_id depend on b_arr through a Tuple inside b_arr's body
    // This is getting complex. Let me use a Repeat instead:

    // Simplest cycle: Repeat whose count references a node inside its own body
    let inner_scalar = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let repeat_node = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(inner_scalar)),
        index_var: None,
        body: vec![inner_scalar],
    });

    // Connect root
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![repeat_node, a_arr, b_arr, t1],
        });
    }

    let graph = DependencyGraph::build(&engine);
    let result = graph.topological_sort();

    // The repeat creates a cycle: repeat depends on inner_scalar (count ref),
    // inner_scalar depends on repeat (body child depends on parent).
    assert!(
        result.is_err(),
        "Should detect cycle: repeat depends on inner_scalar, inner_scalar depends on repeat"
    );
}

#[test]
fn generate_range_within_bounds() {
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
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(10),
            upper: Expression::Lit(20),
        },
    );

    // Property test: run with many seeds, all values should be in [10, 20]
    for seed in 0..100 {
        let sample = generate(&engine, seed).unwrap();
        let value = sample.values.get(&n_id).expect("N should have a value");
        if let SampleValue::Int(v) = value {
            assert!(
                (10..=20).contains(v),
                "seed {seed}: value {v} not in [10, 20]"
            );
        } else {
            panic!("Expected Int value for N");
        }
    }
}

#[test]
fn generate_array_correct_length() {
    let engine = build_n_plus_array_engine();

    for seed in 0..20 {
        let sample = generate(&engine, seed).unwrap();

        let n_id = NodeId::from_raw(1);
        let a_id = NodeId::from_raw(2);

        let n_val = match sample.values.get(&n_id) {
            Some(SampleValue::Int(v)) => *v,
            _ => panic!("N should be Int"),
        };

        let Some(SampleValue::Array(a_val)) = sample.values.get(&a_id) else {
            panic!("A should be Array")
        };

        assert_eq!(
            a_val.len(),
            usize::try_from(n_val).unwrap(),
            "seed {seed}: array length should equal N={n_val}"
        );
    }
}

#[test]
fn generate_distinct_all_unique() {
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
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(5),
            upper: Expression::Lit(5), // fixed N=5
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::VariableRef(a_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Distinct {
            elements: Reference::VariableRef(a_id),
            unit: DistinctUnit::Element,
        },
    );

    for seed in 0..50 {
        let sample = generate(&engine, seed).unwrap();
        let Some(SampleValue::Array(arr)) = sample.values.get(&a_id) else {
            panic!("A should be Array")
        };

        let ints: Vec<i64> = arr
            .iter()
            .map(|v| match v {
                SampleValue::Int(i) => *i,
                _ => panic!("expected Int"),
            })
            .collect();

        let mut unique = ints.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            ints.len(),
            "seed {seed}: all elements should be distinct, got {ints:?}"
        );
    }
}

#[test]
fn generate_sorted_is_sorted() {
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
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(5),
            upper: Expression::Lit(10),
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::VariableRef(a_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(1000),
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Sorted {
            elements: Reference::VariableRef(a_id),
            order: SortOrder::Ascending,
        },
    );

    for seed in 0..30 {
        let sample = generate(&engine, seed).unwrap();
        let Some(SampleValue::Array(arr)) = sample.values.get(&a_id) else {
            panic!("A should be Array")
        };

        let ints: Vec<i64> = arr
            .iter()
            .map(|v| match v {
                SampleValue::Int(i) => *i,
                _ => panic!("expected Int"),
            })
            .collect();

        for w in ints.windows(2) {
            assert!(
                w[0] <= w[1],
                "seed {seed}: array should be sorted ascending, got {ints:?}"
            );
        }
    }
}

#[test]
fn sample_to_text_n_plus_array() {
    let engine = build_n_plus_array_engine();
    let sample = generate(&engine, 42).unwrap();

    let text = sample_to_text(&engine, &sample);

    // Text should have at least 2 lines: header line with N, then array line
    let lines: Vec<&str> = text.trim().lines().collect();
    assert!(lines.len() >= 2, "Expected at least 2 lines, got: {text:?}");

    // First line should be N (an integer)
    let n: i64 = lines[0]
        .trim()
        .parse()
        .expect("First line should be an integer");
    assert!((1..=10).contains(&n), "N should be in [1, 10], got {n}");

    // Second line should have N space-separated integers
    let elements: Vec<&str> = lines[1].split_whitespace().collect();
    assert_eq!(
        elements.len(),
        usize::try_from(n).unwrap(),
        "Array should have {n} elements, got {}",
        elements.len()
    );

    // All elements should be valid integers in [0, 100]
    for elem in &elements {
        let v: i64 = elem.parse().expect("Array element should be integer");
        assert!((0..=100).contains(&v), "Array element {v} not in [0, 100]");
    }
}

#[test]
fn generate_deterministic_with_same_seed() {
    let engine = build_n_plus_array_engine();

    let sample1 = generate(&engine, 123).unwrap();
    let sample2 = generate(&engine, 123).unwrap();

    // Same seed should produce identical results
    assert_eq!(sample1.values.len(), sample2.values.len());
    for (id, v1) in &sample1.values {
        let v2 = sample2.values.get(id).expect("same keys");
        assert_eq!(v1, v2, "Values for {id:?} should match with same seed");
    }
}

#[test]
fn generate_with_expression_bounds() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }

    // 1 ≤ N ≤ 2×10^5
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

    let sample = generate(&engine, 99).unwrap();
    let value = sample.values.get(&n_id).expect("N should have value");
    if let SampleValue::Int(v) = value {
        assert!(
            (1..=200_000).contains(v),
            "N should be in [1, 200000], got {v}"
        );
    } else {
        panic!("Expected Int value");
    }
}

#[test]
fn generate_hole_is_skipped() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyArray),
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let sample = generate(&engine, 0).unwrap();
    // Hole nodes are silently skipped — no value generated
    assert!(
        !sample.values.contains_key(&hole_id),
        "Hole node should not have a generated value"
    );
}

#[test]
fn generate_repeat_expansion() {
    // N=3, then repeat 3 times: each iteration has scalar X in [1, 100]
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: None,
        body: vec![x_id],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });
    }

    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(3),
            upper: Expression::Lit(3),
        },
    );
    engine.constraints.add(
        Some(x_id),
        Constraint::Range {
            target: Reference::VariableRef(x_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let sample = generate(&engine, 42).unwrap();

    // X should NOT be in top-level values (it's inside repeat)
    assert!(
        !sample.values.contains_key(&x_id),
        "X should not be in top-level values"
    );

    // repeat_instances should have 3 iterations
    let instances = sample
        .repeat_instances
        .get(&repeat_id)
        .expect("repeat_instances should contain repeat node");
    assert_eq!(instances.len(), 3, "Should have 3 iterations");

    for (i, iteration) in instances.iter().enumerate() {
        let val = iteration
            .get(&x_id)
            .unwrap_or_else(|| panic!("Iteration {i} should have X"));
        if let SampleValue::Int(v) = val {
            assert!(
                (1..=100).contains(v),
                "Iteration {i}: X={v} not in [1, 100]"
            );
        } else {
            panic!("Expected Int for X in iteration {i}");
        }
    }
}

#[test]
fn generate_repeat_body_child_inter_reference() {
    // N=5, repeat N times: X in [1, 10], Y in [1, X]
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: None,
        body: vec![x_id, y_id],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });
    }

    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(5),
            upper: Expression::Lit(5),
        },
    );
    engine.constraints.add(
        Some(x_id),
        Constraint::Range {
            target: Reference::VariableRef(x_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(10),
        },
    );
    engine.constraints.add(
        Some(y_id),
        Constraint::Range {
            target: Reference::VariableRef(y_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(x_id)),
        },
    );

    for seed in 0..50 {
        let sample = generate(&engine, seed).unwrap();
        let instances = sample
            .repeat_instances
            .get(&repeat_id)
            .expect("should have repeat instances");
        assert_eq!(instances.len(), 5);

        for (i, iteration) in instances.iter().enumerate() {
            let x_val = match iteration.get(&x_id) {
                Some(SampleValue::Int(v)) => *v,
                _ => panic!("seed {seed}, iter {i}: X should be Int"),
            };
            let y_val = match iteration.get(&y_id) {
                Some(SampleValue::Int(v)) => *v,
                _ => panic!("seed {seed}, iter {i}: Y should be Int"),
            };
            assert!(
                (1..=x_val).contains(&y_val),
                "seed {seed}, iter {i}: Y={y_val} not in [1, X={x_val}]"
            );
        }
    }
}

#[test]
fn generate_repeat_zero_count() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: None,
        body: vec![x_id],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });
    }

    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(0),
            upper: Expression::Lit(0),
        },
    );

    let sample = generate(&engine, 42).unwrap();
    let instances = sample.repeat_instances.get(&repeat_id).unwrap();
    assert!(
        instances.is_empty(),
        "Repeat with count=0 should have 0 iterations"
    );
}

#[test]
fn generate_repeat_count_exceeds_limit() {
    use cp_ast_core::sample::{GenerationConfig, GenerationError, generate_with_config};

    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: None,
        body: vec![x_id],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });
    }

    // N = 100, but max_repeat_count = 10
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(100),
            upper: Expression::Lit(100),
        },
    );

    let config = GenerationConfig {
        max_retries: 100,
        max_repeat_count: 10,
        ..Default::default()
    };
    let result = generate_with_config(&engine, 42, config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, GenerationError::InvalidStructure(_)),
        "Expected InvalidStructure for repeat limit, got: {err}"
    );
}

#[test]
fn sample_to_text_with_repeat() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: None,
        body: vec![a_id],
    });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header, repeat_id],
        });
    }

    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(3),
            upper: Expression::Lit(3),
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::VariableRef(a_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let sample = generate(&engine, 42).unwrap();
    let text = sample_to_text(&engine, &sample);
    let lines: Vec<&str> = text.trim().lines().collect();

    // Line 0: N=3
    assert_eq!(lines[0].trim(), "3", "First line should be N=3");
    // Lines 1-3: one array per line (each array has 3 elements)
    assert_eq!(
        lines.len(),
        4,
        "Should have 4 lines: N + 3 repeat iterations"
    );
    for (i, line) in lines[1..].iter().enumerate() {
        let elements: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(
            elements.len(),
            3,
            "Iteration {i}: array should have 3 elements"
        );
        for (j, elem) in elements.iter().enumerate() {
            let v: i64 = elem.parse().unwrap_or_else(|_| {
                panic!("Iteration {i}, element {j}: should be integer, got: {elem:?}")
            });
            assert!(
                (1..=100).contains(&v),
                "Iteration {i}, element {j}: value {v} not in [1, 100]"
            );
        }
    }
}

#[test]
fn generate_array_elements_bounded_by_variable() {
    // N in [3, 10], A_i in [1, N] (variable upper bound)
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
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(3),
            upper: Expression::Lit(10),
        },
    );
    // A_i in [1, N] — variable upper bound
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );

    for seed in 0..100 {
        let sample = generate(&engine, seed).unwrap();
        let n_val = match sample.values.get(&n_id) {
            Some(SampleValue::Int(v)) => *v,
            _ => panic!("N should be Int"),
        };
        let Some(SampleValue::Array(arr)) = sample.values.get(&a_id) else {
            panic!("A should be Array")
        };
        for elem in arr {
            if let SampleValue::Int(v) = elem {
                assert!(
                    (1..=n_val).contains(v),
                    "seed {seed}: element {v} not in [1, {n_val}]"
                );
            }
        }
    }
}

#[test]
fn generate_scalar_with_binop_variable_bound() {
    // N in [1, 10], M in [1, 2*N]
    // Put M inside a Repeat with count=one (scalar set to 1) to establish dependency order
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let one = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("one"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(one)),
        index_var: None,
        body: vec![m_id],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, one, repeat_id],
        });
    }

    // one = 1 (fixed)
    engine.constraints.add(
        Some(one),
        Constraint::Range {
            target: Reference::VariableRef(one),
            lower: Expression::Lit(1),
            upper: Expression::Lit(1),
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(10),
        },
    );
    engine.constraints.add(
        Some(m_id),
        Constraint::Range {
            target: Reference::VariableRef(m_id),
            lower: Expression::Lit(1),
            upper: Expression::BinOp {
                op: ArithOp::Mul,
                lhs: Box::new(Expression::Lit(2)),
                rhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            },
        },
    );

    for seed in 0..100 {
        let sample = generate(&engine, seed).unwrap();
        let n_val = match sample.values.get(&n_id) {
            Some(SampleValue::Int(v)) => *v,
            _ => panic!("N should be Int"),
        };
        // M is inside repeat body, so access via repeat_instances
        let instances = sample
            .repeat_instances
            .get(&repeat_id)
            .expect("should have repeat instances");
        assert_eq!(
            instances.len(),
            1,
            "Repeat with count=1 should have 1 iteration"
        );
        let m_val = match instances[0].get(&m_id) {
            Some(SampleValue::Int(v)) => *v,
            _ => panic!("M should be Int"),
        };
        assert!(
            (1..=2 * n_val).contains(&m_val),
            "seed {seed}: M={m_val} not in [1, {}]",
            2 * n_val
        );
    }
}

#[test]
fn generate_unresolved_reference_returns_error() {
    use cp_ast_core::sample::GenerationError;

    // Constraint expression referencing non-existent variable
    let mut engine = AstEngine::new();
    let unknown_id = NodeId::from_raw(9999);
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }

    // N in [1, unknown_variable] — expression references non-existent node
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(unknown_id)),
        },
    );

    let result = generate(&engine, 42);
    assert!(result.is_err(), "Should fail with unresolved reference");
    let err = result.unwrap_err();
    assert!(
        matches!(err, GenerationError::UnresolvedReference(_)),
        "Expected UnresolvedReference, got: {err}"
    );
}

#[test]
fn generate_range_empty_returns_error() {
    use cp_ast_core::sample::GenerationError;

    // N in [10, 5] — invalid range
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
            lower: Expression::Lit(10),
            upper: Expression::Lit(5),
        },
    );

    let result = generate(&engine, 42);
    assert!(result.is_err(), "Should fail with empty range");
    let err = result.unwrap_err();
    assert!(
        matches!(err, GenerationError::RangeEmpty { min: 10, max: 5 }),
        "Expected RangeEmpty, got: {err}"
    );
}

#[test]
fn generate_choice_branching() {
    // Choice with tag T, variants: (1, [X]), (2, [Y])
    let mut engine = AstEngine::new();
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id]),
        ],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![choice_id],
        });
    }

    engine.constraints.add(
        Some(x_id),
        Constraint::Range {
            target: Reference::VariableRef(x_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(y_id),
        Constraint::Range {
            target: Reference::VariableRef(y_id),
            lower: Expression::Lit(200),
            upper: Expression::Lit(300),
        },
    );

    let sample = generate(&engine, 42).unwrap();

    // Tag should be 1 or 2
    let tag_val = match sample.values.get(&t_id) {
        Some(SampleValue::Int(v)) => *v,
        _ => panic!("Tag T should be Int"),
    };
    assert!(
        tag_val == 1 || tag_val == 2,
        "Tag should be 1 or 2, got {tag_val}"
    );

    if tag_val == 1 {
        assert!(
            sample.values.contains_key(&x_id),
            "Variant 1: X should exist"
        );
        assert!(
            !sample.values.contains_key(&y_id),
            "Variant 1: Y should NOT exist"
        );
    } else {
        assert!(
            !sample.values.contains_key(&x_id),
            "Variant 2: X should NOT exist"
        );
        assert!(
            sample.values.contains_key(&y_id),
            "Variant 2: Y should exist"
        );
    }
}

#[test]
fn generate_choice_all_variants_reachable() {
    let mut engine = AstEngine::new();
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id]),
        ],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![choice_id],
        });
    }

    engine.constraints.add(
        Some(x_id),
        Constraint::Range {
            target: Reference::VariableRef(x_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(y_id),
        Constraint::Range {
            target: Reference::VariableRef(y_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let mut saw_variant_1 = false;
    let mut saw_variant_2 = false;

    for seed in 0..200 {
        let sample = generate(&engine, seed).unwrap();
        match sample.values.get(&t_id) {
            Some(SampleValue::Int(1)) => saw_variant_1 = true,
            Some(SampleValue::Int(2)) => saw_variant_2 = true,
            other => panic!("Unexpected tag value: {other:?}"),
        }
        if saw_variant_1 && saw_variant_2 {
            break;
        }
    }

    assert!(saw_variant_1, "Variant 1 was never selected in 200 seeds");
    assert!(saw_variant_2, "Variant 2 was never selected in 200 seeds");
}

#[test]
fn generate_choice_empty_variants_error() {
    use cp_ast_core::sample::GenerationError;

    let mut engine = AstEngine::new();
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![choice_id],
        });
    }

    let result = generate(&engine, 42);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), GenerationError::InvalidStructure(_)),
        "Expected InvalidStructure for empty variants"
    );
}

#[test]
fn generate_deterministic_with_config() {
    use cp_ast_core::sample::{GenerationConfig, generate_with_config};

    let engine = build_n_plus_array_engine();
    let config = GenerationConfig {
        max_retries: 50,
        max_repeat_count: 1000,
        ..Default::default()
    };

    let sample1 = generate_with_config(&engine, 42, config.clone()).unwrap();
    let sample2 = generate_with_config(&engine, 42, config).unwrap();

    assert_eq!(sample1.values.len(), sample2.values.len());
    for (id, v1) in &sample1.values {
        let v2 = sample2.values.get(id).expect("same keys");
        assert_eq!(
            v1, v2,
            "Values for {id:?} should match with same seed+config"
        );
    }
}

#[test]
fn generate_cycle_returns_error() {
    use cp_ast_core::sample::GenerationError;

    let mut engine = AstEngine::new();
    let inner = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(inner)),
        index_var: None,
        body: vec![inner],
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![repeat_id],
        });
    }

    let result = generate(&engine, 42);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), GenerationError::CycleDetected(_)),
        "Expected CycleDetected error"
    );
}

#[test]
fn repeat_with_expression_count_n_minus_1() {
    let mut engine = AstEngine::default();

    // N = 5
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

    // u_i scalar (body of repeat)
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

    // Repeat N-1 times
    let count_expr = Expression::BinOp {
        op: ArithOp::Sub,
        lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
        rhs: Box::new(Expression::Lit(1)),
    };
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: count_expr,
        index_var: None,
        body: vec![u_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    // N=5, so N-1=4 edges
    assert_eq!(sample.repeat_instances[&repeat_id].len(), 4);

    // Each iteration should have a value for u_id in range [1, 5]
    for instance in &sample.repeat_instances[&repeat_id] {
        if let Some(SampleValue::Int(v)) = instance.get(&u_id) {
            assert!((1..=5).contains(v), "u should be in [1, 5], got {v}");
        } else {
            panic!("Expected Int value for u_id in iteration");
        }
    }
}

#[test]
fn repeat_with_literal_count() {
    let mut engine = AstEngine::default();
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
            upper: Expression::Lit(100),
        },
    );

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Lit(3),
        index_var: None,
        body: vec![x_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    assert_eq!(sample.repeat_instances[&repeat_id].len(), 3);
}

#[test]
fn repeat_with_loop_variable_basic() {
    // Triangular pattern: row i has (i+1) elements
    let mut engine = AstEngine::default();

    // N = 3
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
            lower: Expression::Lit(3),
            upper: Expression::Lit(3),
        },
    );

    // Array C with length = i + 1 (loop variable)
    let c_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("C"),
        length: Expression::BinOp {
            op: ArithOp::Add,
            lhs: Box::new(Expression::Var(Reference::Unresolved(Ident::new("i")))),
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
                indices: vec![Ident::new("j")],
            },
            lower: Expression::Lit(1),
            upper: Expression::Lit(9),
        },
    );

    // Repeat N times with index_var "i"
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: Some(Ident::new("i")),
        body: vec![c_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();

    // 3 iterations
    let instances = &sample.repeat_instances[&repeat_id];
    assert_eq!(instances.len(), 3);

    // Row 0: i=0, length = 0+1 = 1 element
    // Row 1: i=1, length = 1+1 = 2 elements
    // Row 2: i=2, length = 2+1 = 3 elements
    for (i, iteration) in instances.iter().enumerate() {
        if let Some(SampleValue::Array(elements)) = iteration.get(&c_id) {
            assert_eq!(
                elements.len(),
                i + 1,
                "row {i} should have {} elements, got {}",
                i + 1,
                elements.len()
            );
        } else {
            panic!("row {i} missing array value for c_id");
        }
    }
}

#[test]
fn repeat_with_loop_variable_decreasing() {
    // Decreasing pattern: row i has (N-i) elements
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
            lower: Expression::Lit(3),
            upper: Expression::Lit(3),
        },
    );

    // Array with length = N - i
    let c_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("C"),
        length: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Var(Reference::Unresolved(Ident::new("i")))),
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
                indices: vec![Ident::new("j")],
            },
            lower: Expression::Lit(0),
            upper: Expression::Lit(9),
        },
    );

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: Some(Ident::new("i")),
        body: vec![c_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let instances = &sample.repeat_instances[&repeat_id];
    assert_eq!(instances.len(), 3);

    // Row 0: N-0=3 elements, Row 1: N-1=2 elements, Row 2: N-2=1 element
    for (i, iteration) in instances.iter().enumerate() {
        if let Some(SampleValue::Array(elements)) = iteration.get(&c_id) {
            let expected = 3 - i;
            assert_eq!(
                elements.len(),
                expected,
                "row {i} should have {expected} elements, got {}",
                elements.len()
            );
        } else {
            panic!("row {i} missing array value for c_id");
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn choice_in_repeat_generates_independently() {
    // Query pattern: Q queries, each with tag T choosing variant
    let mut engine = AstEngine::default();

    // Q = 10
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
            lower: Expression::Lit(10),
            upper: Expression::Lit(10),
        },
    );

    // Tag T (set by Choice)
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });

    // Variant 1: single X scalar
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
            upper: Expression::Lit(100),
        },
    );

    // Variant 2: two scalars Y, Z
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
            upper: Expression::Lit(100),
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
            upper: Expression::Lit(100),
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

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![q_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let instances = &sample.repeat_instances[&repeat_id];
    assert_eq!(instances.len(), 10);

    // Check that at least one variant 1 and one variant 2 were chosen
    let mut saw_variant1 = false;
    let mut saw_variant2 = false;
    for iteration in instances {
        if let Some(SampleValue::Int(tag)) = iteration.get(&t_id) {
            match tag {
                1 => saw_variant1 = true,
                2 => saw_variant2 = true,
                _ => panic!("unexpected tag value: {tag}"),
            }
        }
    }
    assert!(saw_variant1, "should see at least one variant 1");
    assert!(saw_variant2, "should see at least one variant 2");
}

/// Test for cross-variable constraint dependencies: `1 ≤ n ≤ 4`, `1 ≤ A ≤ 10^n`
#[test]
fn generate_cross_variable_constraint_pow() {
    let mut engine = AstEngine::new();

    // Create scalar n
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("n"),
    });

    // Create array A with length = n
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });

    // Set up structure: root -> sequence[n, A]
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });
    }

    // n: Int, 1 ≤ n ≤ 4
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
            lower: Expression::Lit(1),
            upper: Expression::Lit(4),
        },
    );

    // A elements: Int, 1 ≤ A_i ≤ 10^n (cross-variable constraint)
    engine.constraints.add(
        Some(a_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(a_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(1),
            // upper = 10^n (references n)
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            },
        },
    );

    // Generate samples with multiple seeds to ensure it doesn't fail
    for seed in 0..20 {
        let sample = generate(&engine, seed).expect("should generate successfully");

        // Verify n is in valid range
        let n_val = match sample.values.get(&n_id) {
            Some(SampleValue::Int(v)) => *v,
            _ => panic!("seed {seed}: n should be Int"),
        };
        assert!(
            (1..=4).contains(&n_val),
            "seed {seed}: n={n_val} should be in [1, 4]"
        );

        // Verify A elements are in valid range [1, 10^n]
        let max_a = 10_i64.pow(u32::try_from(n_val).unwrap());
        let Some(SampleValue::Array(a_val)) = sample.values.get(&a_id) else {
            panic!("seed {seed}: A should be Array")
        };

        for (i, elem) in a_val.iter().enumerate() {
            if let SampleValue::Int(v) = elem {
                assert!(
                    (1..=max_a).contains(v),
                    "seed {seed}: A[{i}]={v} should be in [1, {max_a}]"
                );
            } else {
                panic!("seed {seed}: A[{i}] should be Int");
            }
        }
    }
}

/// Test for cross-variable constraint dependencies: `1 ≤ N ≤ 100`, `1 ≤ A ≤ N`
#[test]
fn generate_cross_variable_constraint_simple() {
    let mut engine = AstEngine::new();

    // Create scalar N
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    // Create array A with length = N
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });

    // Set up structure: root -> sequence[N, A]
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });
    }

    // N: Int, 1 ≤ N ≤ 100
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
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    // A elements: Int, 1 ≤ A_i ≤ N (cross-variable constraint)
    engine.constraints.add(
        Some(a_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(a_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(1),
            // upper = N (references N)
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );

    // Generate samples with multiple seeds to ensure it doesn't fail
    for seed in 0..20 {
        let sample = generate(&engine, seed).expect("should generate successfully");

        // Verify N is in valid range
        let n_val = match sample.values.get(&n_id) {
            Some(SampleValue::Int(v)) => *v,
            _ => panic!("seed {seed}: N should be Int"),
        };
        assert!(
            (1..=100).contains(&n_val),
            "seed {seed}: N={n_val} should be in [1, 100]"
        );

        // Verify A elements are in valid range [1, N]
        let Some(SampleValue::Array(a_val)) = sample.values.get(&a_id) else {
            panic!("seed {seed}: A should be Array")
        };

        for (i, elem) in a_val.iter().enumerate() {
            if let SampleValue::Int(v) = elem {
                assert!(
                    (1..=n_val).contains(v),
                    "seed {seed}: A[{i}]={v} should be in [1, {n_val}]"
                );
            } else {
                panic!("seed {seed}: A[{i}] should be Int");
            }
        }
    }
}

/// Test that constraint dependencies are correctly added to the dependency graph
#[test]
fn dependency_graph_includes_constraint_refs() {
    let mut engine = AstEngine::new();

    // Create scalar N
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    // Create scalar A (no structural dependency on N)
    let a_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("A"),
    });

    // Set up structure: root -> sequence[N, A]
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });
    }

    // A has constraint: 1 ≤ A ≤ N (references N)
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::VariableRef(a_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );

    let graph = DependencyGraph::build(&engine);

    // A should depend on N due to the constraint reference
    let a_deps = graph.dependencies_of(a_id);
    assert!(
        a_deps.contains(&n_id),
        "A should depend on N due to constraint reference; deps = {a_deps:?}"
    );

    // Topo sort should put N before A
    let order = graph.topological_sort().expect("should not have cycle");
    let n_pos = order.iter().position(|id| *id == n_id);
    let a_pos = order.iter().position(|id| *id == a_id);
    assert!(
        n_pos.is_some() && a_pos.is_some(),
        "Both N and A must be in sort order"
    );
    assert!(
        n_pos.unwrap() < a_pos.unwrap(),
        "N must come before A due to constraint dependency"
    );
}

#[test]
fn generate_char_matrix_respects_custom_charset() {
    let mut engine = AstEngine::new();

    let h_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("H"),
    });
    let w_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("W"),
    });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![h_id, w_id],
    });
    let s_id = engine.structure.add_node(NodeKind::Matrix {
        name: Ident::new("S"),
        rows: Reference::VariableRef(h_id),
        cols: Reference::VariableRef(w_id),
    });

    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![header, s_id],
        });
    }

    for (node_id, value) in [(h_id, 3), (w_id, 4)] {
        engine.constraints.add(
            Some(node_id),
            Constraint::TypeDecl {
                target: Reference::VariableRef(node_id),
                expected: ExpectedType::Int,
            },
        );
        engine.constraints.add(
            Some(node_id),
            Constraint::Range {
                target: Reference::VariableRef(node_id),
                lower: Expression::Lit(value),
                upper: Expression::Lit(value),
            },
        );
    }
    engine.constraints.add(
        Some(s_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(s_id),
            expected: ExpectedType::Char,
        },
    );
    engine.constraints.add(
        Some(s_id),
        Constraint::CharSet {
            target: Reference::VariableRef(s_id),
            charset: CharSetSpec::Custom(vec!['.', '#']),
        },
    );

    let sample = generate(&engine, 42).expect("sample should generate");
    let Some(SampleValue::Grid(rows)) = sample.values.get(&s_id) else {
        panic!("S should be a grid");
    };

    assert_eq!(rows.len(), 3);
    for row in rows {
        assert_eq!(row.len(), 4);
        for value in row {
            let SampleValue::Str(cell) = value else {
                panic!("grid cell should be a character string");
            };
            assert!(
                cell == "." || cell == "#",
                "cell should respect custom charset, got {cell}"
            );
        }
    }

    let text = sample_to_text(&engine, &sample);
    for line in text.lines().skip(1) {
        assert_eq!(line.len(), 4);
        assert!(line.chars().all(|c| c == '.' || c == '#'));
    }
}

#[test]
fn constraint_expression_parser_handles_simple_var_arithmetic_for_samples() {
    use cp_ast_core::operation::action::Action;
    use cp_ast_core::operation::types::{ConstraintDef, ConstraintDefKind, FillContent, VarType};

    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    let n = *engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .last()
        .unwrap();
    let m = engine
        .apply(&Action::AddSibling {
            target: n,
            element: FillContent::Scalar {
                name: "M".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap()
        .created_nodes
        .iter()
        .find(|id| {
            engine.structure.get(**id).is_some_and(
                |node| matches!(node.kind(), NodeKind::Scalar { name } if name.as_str() == "M"),
            )
        })
        .copied()
        .unwrap();

    engine
        .apply(&Action::AddConstraint {
            target: n,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "3".to_owned(),
                },
            },
        })
        .unwrap();
    engine
        .apply(&Action::AddConstraint {
            target: m,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "N+2".to_owned(),
                },
            },
        })
        .unwrap();

    assert!(generate(&engine, 0).is_ok());
}
