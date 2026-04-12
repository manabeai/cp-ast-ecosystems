use cp_ast_core::constraint::{Constraint, ConstraintSet, ExpectedType, Expression};
use cp_ast_core::structure::NodeId;

#[test]
fn expected_type_equality() {
    assert_eq!(ExpectedType::Int, ExpectedType::Int);
    assert_ne!(ExpectedType::Int, ExpectedType::String);
}

#[test]
fn expected_type_array_of_int() {
    let arr = ExpectedType::Array(Box::new(ExpectedType::Int));
    assert_eq!(arr, ExpectedType::Array(Box::new(ExpectedType::Int)));
    assert_ne!(arr, ExpectedType::Array(Box::new(ExpectedType::String)));
}

#[test]
fn expression_literal() {
    let expr = Expression::Literal(42);
    assert_eq!(expr, Expression::Literal(42));
}

#[test]
fn expression_power() {
    // 10^9
    let expr = Expression::Power(10, 9);
    assert_eq!(expr, Expression::Power(10, 9));
}

#[test]
fn expression_mul() {
    // 2 * 10^5
    let expr = Expression::Mul(
        Box::new(Expression::Literal(2)),
        Box::new(Expression::Power(10, 5)),
    );
    assert!(matches!(expr, Expression::Mul(_, _)));
}

#[test]
fn expression_evaluate_literal() {
    let expr = Expression::Literal(42);
    assert_eq!(expr.evaluate_constant(), Some(42));
}

#[test]
fn expression_evaluate_power() {
    let expr = Expression::Power(10, 9);
    assert_eq!(expr.evaluate_constant(), Some(1_000_000_000));
}

#[test]
fn expression_evaluate_mul() {
    // 2 * 10^5 = 200_000
    let expr = Expression::Mul(
        Box::new(Expression::Literal(2)),
        Box::new(Expression::Power(10, 5)),
    );
    assert_eq!(expr.evaluate_constant(), Some(200_000));
}

#[test]
fn expression_evaluate_ref_returns_none() {
    use cp_ast_core::structure::NodeId;
    let expr = Expression::Ref(NodeId::new());
    assert_eq!(expr.evaluate_constant(), None);
}

#[test]
fn range_constraint_creation() {
    let target = NodeId::new();
    let c = Constraint::range(target, Expression::Literal(1), Expression::Power(10, 5));

    assert!(matches!(c, Constraint::Range { .. }));
}

#[test]
fn length_constraint_creation() {
    let array_id = NodeId::new();
    let length_id = NodeId::new();
    let c = Constraint::length(array_id, length_id);

    assert!(matches!(c, Constraint::Length { .. }));
}

#[test]
fn element_constraint_creation() {
    let array_id = NodeId::new();
    let element_c = Constraint::range(
        NodeId::new(),
        Expression::Literal(0),
        Expression::Power(10, 9),
    );
    let c = Constraint::element(array_id, element_c);

    assert!(matches!(c, Constraint::Element { .. }));
}

#[test]
fn type_constraint_creation() {
    let target = NodeId::new();
    let c = Constraint::expected_type(target, ExpectedType::Int);

    assert!(matches!(c, Constraint::Type { .. }));
}

#[test]
fn constraint_set_empty() {
    let set = ConstraintSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn constraint_set_add_and_query() {
    let n_id = NodeId::new();
    let a_id = NodeId::new();

    let mut set = ConstraintSet::new();
    set.add(Constraint::range(
        n_id,
        Expression::Literal(1),
        Expression::Power(10, 5),
    ));
    set.add(Constraint::expected_type(n_id, ExpectedType::Int));
    set.add(Constraint::length(a_id, n_id));

    assert_eq!(set.len(), 3);

    let n_constraints: Vec<_> = set.for_target(n_id).collect();
    assert_eq!(n_constraints.len(), 2);

    let a_constraints: Vec<_> = set.for_target(a_id).collect();
    assert_eq!(a_constraints.len(), 1);
}

#[test]
fn constraint_set_no_constraints_for_unknown_node() {
    let set = ConstraintSet::new();
    let unknown = NodeId::new();
    let results: Vec<_> = set.for_target(unknown).collect();
    assert!(results.is_empty());
}
