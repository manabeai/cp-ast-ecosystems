use cp_ast_core::structure::{NodeKind, Slot, SlotValue, StructureNode};

#[test]
fn hole_has_unique_id() {
    let slot1 = Slot::hole("a");
    let slot2 = Slot::hole("b");

    let id1 = match slot1.value() {
        SlotValue::Hole(ref h) => h.id(),
        SlotValue::Filled(_) => panic!("expected hole"),
    };
    let id2 = match slot2.value() {
        SlotValue::Hole(ref h) => h.id(),
        SlotValue::Filled(_) => panic!("expected hole"),
    };
    assert_ne!(id1, id2);
}

#[test]
fn collect_holes_from_tree() {
    let child = StructureNode::new(NodeKind::Scalar).with_name("N");
    let node = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::filled("defined", child))
        .with_slot(Slot::hole("undefined"));

    let holes: Vec<_> = node.slots().iter().filter(|s| s.is_hole()).collect();
    assert_eq!(holes.len(), 1);
    assert_eq!(holes[0].name(), "undefined");
}
