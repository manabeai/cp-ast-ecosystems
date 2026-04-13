//! 複数テストケース問題の AST 例 (Section)
//!
//! 入力形式:
//! ```text
//! T
//! (各テストケース)
//! N
//! A_1 A_2 ... A_N
//! ```
//!
//! 使用 `NodeKind`: `Scalar`, `Array`, `Section`, `Repeat`, `Sequence`
//!
//! 制約:
//! - 1 ≤ T ≤ 100
//! - 1 ≤ N ≤ 10^5
//! - 0 ≤ `A_i` ≤ 10^9
//! - T, N, `A_i` は整数
//! - 全テストケースの N の和 ≤ 2 × 10^5

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};
use cp_ast_tree::{
    render_combined_tree, render_constraint_tree, render_structure_tree, TreeOptions,
};

fn main() {
    let mut engine = AstEngine::new();

    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });

    let section = engine.structure.add_node(NodeKind::Section {
        header: None,
        body: vec![n_id, a_id],
    });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(t_id)),
        index_var: Some(Ident::new("tc")),
        body: vec![section],
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![t_id, repeat],
        });

    engine.constraints.add(
        Some(t_id),
        Constraint::Range {
            target: Reference::VariableRef(t_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );
    engine.constraints.add(
        Some(t_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(t_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(5)),
            },
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(0),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(9)),
            },
        },
    );
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "Sum of N over all test cases ≤ 2 × 10^5".to_owned(),
            predicate: None,
        },
    );

    let opts = TreeOptions::default();
    println!("=== Structure ===");
    print!("{}", render_structure_tree(&engine, &opts));
    println!("\n=== Constraints ===");
    print!("{}", render_constraint_tree(&engine, &opts));
    println!("\n=== Combined ===");
    print!("{}", render_combined_tree(&engine, &opts));
}
