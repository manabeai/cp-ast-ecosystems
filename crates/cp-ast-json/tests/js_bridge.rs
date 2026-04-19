//! JS bridge roundtrip test.
//!
//! Verifies that JSON survives a JS parse/stringify cycle without loss.
//! Requires Node.js to be available on PATH.

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};
use std::process::Command;

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn js_parse_stringify_roundtrip() {
    if !node_available() {
        eprintln!("SKIP: Node.js not available");
        return;
    }

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

    let json_from_rust = cp_ast_json::serialize_ast(&engine).unwrap();

    // Pass through Node.js: JSON.parse then JSON.stringify with same formatting
    let output = Command::new("node")
        .arg("-e")
        .arg("const fs=require('fs');const d=fs.readFileSync('/dev/stdin','utf8');const o=JSON.parse(d);process.stdout.write(JSON.stringify(o,null,2));")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .take()
                .unwrap()
                .write_all(json_from_rust.as_bytes())
                .unwrap();
            child.wait_with_output()
        })
        .expect("failed to run Node.js");

    assert!(
        output.status.success(),
        "Node.js failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json_from_js = String::from_utf8(output.stdout).unwrap();

    // Rust can parse the JS output
    let restored = cp_ast_json::deserialize_ast(&json_from_js).unwrap();
    let json_roundtripped = cp_ast_json::serialize_ast(&restored).unwrap();

    // Full cycle identity
    assert_eq!(
        json_from_rust, json_roundtripped,
        "Rust → JSON → JS parse → JS stringify → JSON → Rust → JSON should be identical"
    );
}
