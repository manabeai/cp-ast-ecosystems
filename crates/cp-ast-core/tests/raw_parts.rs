#![allow(clippy::cast_possible_truncation)]

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};

#[test]
fn structure_ast_raw_parts_roundtrip() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });

    // Remove a_id to create a tombstone
    engine.structure.remove(a_id);

    let root = engine.structure.root();
    let next_id = engine.structure.next_id();
    let arena = engine.structure.arena_raw();

    // Verify tombstone exists
    assert!(arena[a_id.value() as usize].is_none());
    assert!(arena[n_id.value() as usize].is_some());

    // Reconstruct
    let reconstructed =
        cp_ast_core::structure::StructureAst::from_raw_parts(root, arena.to_vec(), next_id);
    assert_eq!(reconstructed.root(), root);
    assert_eq!(reconstructed.next_id(), next_id);
    assert!(reconstructed.get(n_id).is_some());
    assert!(reconstructed.get(a_id).is_none()); // tombstone preserved
}

#[test]
fn constraint_set_raw_parts_roundtrip() {
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
    let c2 = engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "test".to_owned(),
            predicate: None,
        },
    );

    // Remove c0 to create tombstone
    engine.constraints.remove(c0);

    let next_id = engine.constraints.next_id();
    let arena = engine.constraints.arena_raw();
    let by_node = engine.constraints.by_node_raw();
    let global = engine.constraints.global();

    // Verify tombstone
    assert!(arena[c0.value() as usize].is_none());
    assert!(arena[c1.value() as usize].is_some());

    // Reconstruct
    let reconstructed = cp_ast_core::constraint::ConstraintSet::from_raw_parts(
        arena.to_vec(),
        by_node.to_vec(),
        global.to_vec(),
        next_id,
    );
    assert_eq!(reconstructed.next_id(), next_id);
    assert!(reconstructed.get(c0).is_none());
    assert!(reconstructed.get(c1).is_some());
    assert_eq!(reconstructed.global(), &[c2]);
}
