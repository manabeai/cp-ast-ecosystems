use cp_ast_core::constraint::Expression;
use cp_ast_core::operation::{Action, SlotId, SlotKind};
use cp_ast_core::structure::NodeId;
use std::collections::HashSet;

#[test]
fn test_slot_kind_as_str() {
    assert_eq!(SlotKind::ArrayLength.as_str(), "ArrayLength");
    assert_eq!(SlotKind::RepeatCount.as_str(), "RepeatCount");
    assert_eq!(SlotKind::RangeLower.as_str(), "RangeLower");
    assert_eq!(SlotKind::RangeUpper.as_str(), "RangeUpper");
    assert_eq!(SlotKind::RelationLhs.as_str(), "RelationLhs");
    assert_eq!(SlotKind::RelationRhs.as_str(), "RelationRhs");
    assert_eq!(SlotKind::LengthLength.as_str(), "LengthLength");
}

#[test]
fn test_slot_kind_all_distinct() {
    let all_variants = [
        SlotKind::ArrayLength,
        SlotKind::RepeatCount,
        SlotKind::RangeLower,
        SlotKind::RangeUpper,
        SlotKind::RelationLhs,
        SlotKind::RelationRhs,
        SlotKind::LengthLength,
    ];

    let strings: HashSet<&str> = all_variants.iter().map(|k| k.as_str()).collect();
    assert_eq!(
        strings.len(),
        all_variants.len(),
        "All SlotKind variants should have distinct as_str() values"
    );
}

#[test]
fn test_slot_kind_display() {
    assert_eq!(SlotKind::ArrayLength.to_string(), "ArrayLength");
    assert_eq!(SlotKind::RepeatCount.to_string(), "RepeatCount");
    assert_eq!(format!("{}", SlotKind::RangeLower), "RangeLower");
}

#[test]
fn test_slot_id_construction() {
    let node_id = NodeId::new();
    let slot_id = SlotId {
        owner: node_id,
        kind: SlotKind::ArrayLength,
    };

    assert_eq!(slot_id.owner, node_id);
    assert_eq!(slot_id.kind, SlotKind::ArrayLength);
}

#[test]
fn test_slot_id_comparison() {
    let node_id1 = NodeId::new();
    let node_id2 = NodeId::new();

    let slot1 = SlotId {
        owner: node_id1,
        kind: SlotKind::ArrayLength,
    };
    let slot2 = SlotId {
        owner: node_id1,
        kind: SlotKind::ArrayLength,
    };
    let slot3 = SlotId {
        owner: node_id2,
        kind: SlotKind::ArrayLength,
    };
    let slot4 = SlotId {
        owner: node_id1,
        kind: SlotKind::RepeatCount,
    };

    assert_eq!(slot1, slot2);
    assert_ne!(slot1, slot3);
    assert_ne!(slot1, slot4);
}

#[test]
fn test_action_set_expr() {
    let node_id = NodeId::new();
    let slot_id = SlotId {
        owner: node_id,
        kind: SlotKind::ArrayLength,
    };
    let expr = Expression::Lit(42);

    let action = Action::SetExpr {
        slot: slot_id.clone(),
        expr: expr.clone(),
    };

    match action {
        Action::SetExpr {
            slot,
            expr: actual_expr,
        } => {
            assert_eq!(slot, slot_id);
            assert_eq!(actual_expr, expr);
        }
        _ => panic!("Expected SetExpr variant"),
    }
}
