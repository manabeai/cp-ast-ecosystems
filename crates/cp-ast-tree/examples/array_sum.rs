//! 配列問題の AST 例
//!
//! 入力形式:
//! ```text
//! N
//! A_1 A_2 ... A_N
//! ```
//!
//! 使用 `NodeKind`: `Scalar`, `Array`, `Sequence`
//!
//! 制約:
//! - 1 ≤ N ≤ 10^5
//! - 1 ≤ `A_i` ≤ 10^9
//! - N, `A_i` は整数

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};
use cp_ast_tree::{
    TreeOptions, render_combined_tree, render_constraint_tree, render_structure_tree,
};

fn main() {
    let mut engine = AstEngine::new();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: Expression::Var(Reference::VariableRef(n_id)),
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, a_id],
        });

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
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(9)),
            },
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::TypeDecl {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i")],
            },
            expected: ExpectedType::Int,
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
