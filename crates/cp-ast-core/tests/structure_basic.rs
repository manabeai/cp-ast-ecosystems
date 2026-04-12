use cp_ast_core::structure::{NodeId, NodeKind, Slot, StructureNode};

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

#[test]
fn structure_node_with_filled_slots() {
    let child = StructureNode::new(NodeKind::Scalar).with_name("N");

    let parent =
        StructureNode::new(NodeKind::InputBlock).with_slot(Slot::filled("first_var", child));

    assert_eq!(parent.kind(), NodeKind::InputBlock);
    assert_eq!(parent.slots().len(), 1);
    assert_eq!(parent.slots()[0].name(), "first_var");
    assert!(parent.slots()[0].is_filled());
}

#[test]
fn structure_node_with_hole_slot() {
    let node = StructureNode::new(NodeKind::InputBlock).with_slot(Slot::hole("missing_var"));

    assert_eq!(node.slots().len(), 1);
    assert!(node.slots()[0].is_hole());
}

#[test]
fn structure_node_name() {
    let node = StructureNode::new(NodeKind::Scalar).with_name("N");
    assert_eq!(node.name(), Some("N"));

    let unnamed = StructureNode::new(NodeKind::InputBlock);
    assert_eq!(unnamed.name(), None);
}
