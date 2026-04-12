use cp_ast_core::structure::{NodeId, NodeKind};

#[test]
fn node_id_unique() {
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    assert_ne!(id1, id2);
}

#[test]
fn node_id_copy_equality() {
    let id = NodeId::new();
    let id_copy = id;
    assert_eq!(id, id_copy);
}

#[test]
fn node_id_debug_format() {
    let id = NodeId::new();
    let debug = format!("{id:?}");
    assert!(debug.contains("NodeId"));
}

#[test]
fn node_kind_equality() {
    assert_eq!(NodeKind::Scalar, NodeKind::Scalar);
    assert_ne!(NodeKind::Scalar, NodeKind::Array);
}

#[test]
fn node_kind_copy() {
    let kind = NodeKind::Array;
    let copied = kind;
    assert_eq!(kind, copied);
}

#[test]
fn node_kind_all_variants_exist() {
    // Verify all expected variants compile
    let variants = [
        NodeKind::Scalar,
        NodeKind::Array,
        NodeKind::Matrix,
        NodeKind::MultiTestCase,
        NodeKind::Query,
        NodeKind::InputBlock,
        NodeKind::OutputBlock,
    ];
    // Verify we have the expected number of variants
    assert_eq!(variants.len(), 7);
}
