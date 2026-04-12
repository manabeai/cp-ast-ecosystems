use cp_ast_core::constraint::{ExpectedType, Expression};

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
