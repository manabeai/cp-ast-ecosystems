//! Golden file tests.
//!
//! These tests serialize a known AST and compare against saved JSON fixtures.
//! To regenerate: `UPDATE_GOLDEN=1 cargo test -p cp-ast-json --test golden`

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};
use std::path::PathBuf;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn assert_golden(name: &str, json: &str) {
    let path = golden_dir().join(name);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(golden_dir()).unwrap();
        std::fs::write(&path, json).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "Golden file not found: {}. Run with UPDATE_GOLDEN=1 to create.",
            path.display()
        )
    });
    assert_eq!(json, expected, "Golden file mismatch: {name}");
}

#[test]
fn golden_basic_array() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });
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

    let json = cp_ast_json::serialize_ast(&engine).unwrap();
    assert_golden("basic_array.json", &json);

    // Also verify roundtrip from golden file
    let restored = cp_ast_json::deserialize_ast(&json).unwrap();
    let json2 = cp_ast_json::serialize_ast(&restored).unwrap();
    assert_eq!(json, json2);
}
