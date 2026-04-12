use cp_ast_core::structure::{Ident, Literal, NodeId, NodeKindHint, Reference};

#[test]
fn ident_creation_and_equality() {
    let a = Ident::new("N");
    let b = Ident::new("N");
    assert_eq!(a, b);
    assert_eq!(a.as_str(), "N");
}

#[test]
fn ident_from_str() {
    let id: Ident = "M".into();
    assert_eq!(id.as_str(), "M");
}

#[test]
fn ident_hash_usable_in_collections() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Ident::new("N"));
    set.insert(Ident::new("N"));
    assert_eq!(set.len(), 1);
}

#[test]
fn literal_int() {
    let lit = Literal::IntLit(42);
    assert_eq!(lit, Literal::IntLit(42));
}

#[test]
fn literal_str() {
    let lit = Literal::StrLit("abc".to_owned());
    assert_eq!(lit, Literal::StrLit("abc".to_owned()));
}

#[test]
fn node_id_from_raw() {
    let id = NodeId::from_raw(42);
    assert_eq!(id.value(), 42);
}

#[test]
fn node_id_new_still_works() {
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    assert_ne!(id1, id2);
}

#[test]
fn reference_variable_ref() {
    let id = NodeId::from_raw(1);
    let r = Reference::VariableRef(id);
    assert!(matches!(r, Reference::VariableRef(_)));
}

#[test]
fn reference_indexed_ref() {
    let r = Reference::IndexedRef {
        target: NodeId::from_raw(1),
        indices: vec![Ident::new("i")],
    };
    assert!(matches!(r, Reference::IndexedRef { .. }));
}

#[test]
fn reference_unresolved() {
    let r = Reference::Unresolved(Ident::new("N"));
    assert!(matches!(r, Reference::Unresolved(_)));
}

#[test]
fn node_kind_hint_all_variants() {
    let hints = [
        NodeKindHint::AnyScalar,
        NodeKindHint::AnyArray,
        NodeKindHint::AnyMatrix,
        NodeKindHint::AnyTuple,
        NodeKindHint::AnyRepeat,
        NodeKindHint::AnySection,
        NodeKindHint::AnyChoice,
        NodeKindHint::Any,
    ];
    assert_eq!(hints.len(), 8);
}
