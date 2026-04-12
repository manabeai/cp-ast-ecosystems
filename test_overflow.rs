use cp_ast_core::constraint::*;

#[test]
fn test_overflow() {
    let e = Expression::BinOp {
        op: ArithOp::Add,
        lhs: Box::new(Expression::Lit(i64::MAX)),
        rhs: Box::new(Expression::Lit(1)),
    };
    println!("Result: {:?}", e.evaluate_constant());
}
