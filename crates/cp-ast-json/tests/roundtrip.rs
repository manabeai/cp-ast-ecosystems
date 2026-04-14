use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};

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
