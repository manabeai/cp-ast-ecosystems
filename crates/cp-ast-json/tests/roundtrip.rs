use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, NodeKindHint, Reference};

/// Build a simple array problem: N then `A_1..A_N`.
fn build_array_engine() -> AstEngine {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });

    // Wire children to root Sequence
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });

    // Constraints
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(5)),
            },
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );
    engine
}

#[test]
fn basic_roundtrip_json_identity() {
    let engine = build_array_engine();
    let json1 = cp_ast_json::serialize_ast(&engine).unwrap();
    let engine2 = cp_ast_json::deserialize_ast(&json1).unwrap();
    let json2 = cp_ast_json::serialize_ast(&engine2).unwrap();
    assert_eq!(
        json1, json2,
        "JSON roundtrip should produce identical output"
    );
}

#[test]
fn basic_roundtrip_structure_preserved() {
    let engine = build_array_engine();
    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    // Root preserved
    assert_eq!(
        restored.structure.root().value(),
        engine.structure.root().value()
    );
    // next_id preserved
    assert_eq!(restored.structure.next_id(), engine.structure.next_id());
    // Node count preserved
    assert_eq!(restored.structure.len(), engine.structure.len());
    // Constraint count preserved
    assert_eq!(restored.constraints.len(), engine.constraints.len());
    assert_eq!(restored.constraints.next_id(), engine.constraints.next_id());
}

#[test]
fn schema_version_present() {
    let engine = build_array_engine();
    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["schema_version"], 1);
}

#[test]
fn ids_are_decimal_strings() {
    let engine = build_array_engine();
    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let root = &parsed["document"]["structure"]["root"];
    assert!(root.is_string(), "root should be a string, got: {root}");
    assert_eq!(root.as_str().unwrap(), "0");
}

#[test]
fn tombstone_preservation() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    engine.structure.remove(m_id);

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    assert!(restored.structure.get(m_id).is_none());
    assert!(restored.structure.get(n_id).is_some());
    assert_eq!(
        restored.structure.arena_raw().len(),
        engine.structure.arena_raw().len()
    );
    assert!(restored.structure.arena_raw()[usize::try_from(m_id.value()).unwrap()].is_none());
}

#[test]
fn constraint_tombstone_preservation() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    let c0 = engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );
    let c1 = engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.remove(c0);

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    assert!(restored.constraints.get(c0).is_none());
    assert!(restored.constraints.get(c1).is_some());
    assert_eq!(
        restored.constraints.arena_raw().len(),
        engine.constraints.arena_raw().len()
    );
}

#[test]
fn id_preservation() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    let n_restored = restored.structure.get(n_id).unwrap();
    assert_eq!(n_restored.id().value(), n_id.value());
    let a_restored = restored.structure.get(a_id).unwrap();
    assert_eq!(a_restored.id().value(), a_id.value());
    assert_eq!(restored.structure.next_id(), engine.structure.next_id());
    assert_eq!(restored.constraints.next_id(), engine.constraints.next_id());
}

#[test]
fn child_ordering_preserved() {
    let mut engine = AstEngine::new();
    let a = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("A"),
    });
    let b = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("B"),
    });
    let c = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("C"),
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![c, a, b],
        });

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    if let NodeKind::Sequence { children } = restored
        .structure
        .get(restored.structure.root())
        .unwrap()
        .kind()
    {
        assert_eq!(children, &[c, a, b]);
    } else {
        panic!("root should be Sequence");
    }
}

#[test]
fn constraint_ordering_preserved() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    let c0 = engine.constraints.add(
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );
    let c1 = engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    let cids = restored.constraints.for_node(n_id);
    assert_eq!(cids, vec![c0, c1]);
}

#[test]
fn hole_preservation() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyArray),
    });
    let hole_none_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![hole_id, hole_none_id],
        });

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    match restored.structure.get(hole_id).unwrap().kind() {
        NodeKind::Hole { expected_kind } => {
            assert_eq!(*expected_kind, Some(NodeKindHint::AnyArray));
        }
        _ => panic!("expected Hole"),
    }
    match restored.structure.get(hole_none_id).unwrap().kind() {
        NodeKind::Hole { expected_kind } => {
            assert_eq!(*expected_kind, None);
        }
        _ => panic!("expected Hole"),
    }
}

#[test]
fn unresolved_reference_preservation() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::Unresolved(Ident::new("unknown_var")),
            lower: Expression::Lit(0),
            upper: Expression::Lit(100),
        },
    );

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();

    let cids = restored.constraints.for_node(n_id);
    assert_eq!(cids.len(), 1);
    match restored.constraints.get(cids[0]).unwrap() {
        Constraint::Range { target, .. } => match target {
            Reference::Unresolved(name) => assert_eq!(name.as_str(), "unknown_var"),
            _ => panic!("expected Unresolved reference"),
        },
        _ => panic!("expected Range constraint"),
    }
}
