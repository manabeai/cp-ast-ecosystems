use cp_ast_core::constraint::{
    ArithOp, CharSetSpec, ConstraintId, DistinctUnit, PropertyTag, RelationOp,
    RenderHintKind, Separator, SortOrder,
};

#[test]
fn constraint_id_from_raw_and_value() {
    let id = ConstraintId::from_raw(5);
    assert_eq!(id.value(), 5);
}

#[test]
fn constraint_id_copy_and_eq() {
    let a = ConstraintId::from_raw(1);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn constraint_id_hash_usable() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(ConstraintId::from_raw(1));
    set.insert(ConstraintId::from_raw(1));
    assert_eq!(set.len(), 1);
}

#[test]
fn relation_op_all_variants() {
    let ops = [
        RelationOp::Lt,
        RelationOp::Le,
        RelationOp::Gt,
        RelationOp::Ge,
        RelationOp::Eq,
        RelationOp::Ne,
    ];
    assert_eq!(ops.len(), 6);
}

#[test]
fn relation_op_copy() {
    let op = RelationOp::Lt;
    let copied = op;
    assert_eq!(op, copied);
}

#[test]
fn arith_op_all_variants() {
    let ops = [ArithOp::Add, ArithOp::Sub, ArithOp::Mul, ArithOp::Div];
    assert_eq!(ops.len(), 4);
}

#[test]
fn distinct_unit_variants() {
    assert_ne!(DistinctUnit::Element, DistinctUnit::Tuple);
}

#[test]
fn property_tag_predefined_and_custom() {
    let tags = [
        PropertyTag::Simple,
        PropertyTag::Connected,
        PropertyTag::Tree,
        PropertyTag::Permutation,
        PropertyTag::Binary,
        PropertyTag::Odd,
        PropertyTag::Even,
        PropertyTag::Custom("Bipartite".to_owned()),
    ];
    assert_eq!(tags.len(), 8);
}

#[test]
fn sort_order_variants() {
    let orders = [
        SortOrder::Ascending,
        SortOrder::NonDecreasing,
        SortOrder::Descending,
        SortOrder::NonIncreasing,
    ];
    assert_eq!(orders.len(), 4);
}

#[test]
fn charset_spec_predefined() {
    assert_ne!(CharSetSpec::LowerAlpha, CharSetSpec::UpperAlpha);
}

#[test]
fn charset_spec_custom() {
    let cs = CharSetSpec::Custom(vec!['a', 'b', 'c']);
    assert!(matches!(cs, CharSetSpec::Custom(_)));
}

#[test]
fn charset_spec_range() {
    let cs = CharSetSpec::Range('a', 'z');
    assert!(matches!(cs, CharSetSpec::Range(_, _)));
}

#[test]
fn render_hint_kind_separator() {
    let hint = RenderHintKind::Separator(Separator::Space);
    assert!(matches!(hint, RenderHintKind::Separator(Separator::Space)));
}

#[test]
fn separator_variants() {
    assert_ne!(Separator::Space, Separator::None);
}
