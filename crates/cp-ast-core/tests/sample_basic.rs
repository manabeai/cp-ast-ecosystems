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
        length: Reference::VariableRef(n_id),
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
        length: Reference::VariableRef(b_id),
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
        length: Reference::VariableRef(a_id),
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
        count: Reference::VariableRef(inner_scalar),
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
        length: Reference::VariableRef(n_id),
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
        length: Reference::VariableRef(n_id),
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
