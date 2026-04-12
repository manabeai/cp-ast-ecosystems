//! Integration test: express a typical `AtCoder` ABC problem as AST.
//!
//! Problem: Given N and array A of length N, (typical ABC B-C level).
//! Input format:
//!   N
//!   `A_1` `A_2` ... `A_N`
//! Constraints:
//!   1 <= N <= 2 * 10^5
//!   0 <= `A_i` <= 10^9

use cp_ast_core::constraint::{Constraint, ConstraintSet, ExpectedType, Expression};
use cp_ast_core::structure::{NodeKind, Slot, StructureNode};

#[test]
fn express_n_plus_array_problem() {
    // --- Build StructureAST ---

    // Scalar variable N
    let n_node = StructureNode::new(NodeKind::Scalar).with_name("N");
    let n_id = n_node.id();

    // Array variable A
    let a_node = StructureNode::new(NodeKind::Array).with_name("A");
    let a_id = a_node.id();

    // Input block containing N then A
    let input = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::filled("line_1", n_node))
        .with_slot(Slot::filled("line_2", a_node));

    // Verify structure
    assert_eq!(input.kind(), NodeKind::InputBlock);
    assert_eq!(input.slots().len(), 2);
    assert_eq!(input.slots()[0].name(), "line_1");
    assert_eq!(input.slots()[1].name(), "line_2");

    // --- Build ConstraintAST ---

    let mut constraints = ConstraintSet::new();

    // N: Int
    constraints.add(Constraint::expected_type(n_id, ExpectedType::Int));

    // 1 <= N <= 2 * 10^5
    constraints.add(Constraint::range(
        n_id,
        Expression::Literal(1),
        Expression::Mul(
            Box::new(Expression::Literal(2)),
            Box::new(Expression::Power(10, 5)),
        ),
    ));

    // A: Array(Int)
    constraints.add(Constraint::expected_type(
        a_id,
        ExpectedType::Array(Box::new(ExpectedType::Int)),
    ));

    // len(A) = N
    constraints.add(Constraint::length(a_id, n_id));

    // 0 <= A[i] <= 10^9
    constraints.add(Constraint::element(
        a_id,
        Constraint::range(
            a_id, // element constraint references the array
            Expression::Literal(0),
            Expression::Power(10, 9),
        ),
    ));

    // Verify constraints
    assert_eq!(constraints.len(), 5);

    // N has 2 constraints (type + range)
    let n_constraints: Vec<_> = constraints.for_target(n_id).collect();
    assert_eq!(n_constraints.len(), 2);

    // A has 3 constraints (type + length + element)
    let a_constraints: Vec<_> = constraints.for_target(a_id).collect();
    assert_eq!(a_constraints.len(), 3);

    // Verify range upper bound evaluates correctly
    let range_upper = Expression::Mul(
        Box::new(Expression::Literal(2)),
        Box::new(Expression::Power(10, 5)),
    );
    assert_eq!(range_upper.evaluate_constant(), Some(200_000));
}

#[test]
fn express_problem_with_hole() {
    // A problem being built incrementally:
    // InputBlock with N defined but second variable still a hole

    let n_node = StructureNode::new(NodeKind::Scalar).with_name("N");

    let input = StructureNode::new(NodeKind::InputBlock)
        .with_slot(Slot::filled("line_1", n_node))
        .with_slot(Slot::hole("line_2")); // Not yet defined

    assert_eq!(input.slots().len(), 2);
    assert!(input.slots()[0].is_filled());
    assert!(input.slots()[1].is_hole());
}
