use cp_ast_core::structure::NodeId;

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
