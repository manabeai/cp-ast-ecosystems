//! クエリ型問題の AST 例 (Choice)
//!
//! 入力形式:
//! ```text
//! N Q
//! query_1
//! ...
//! query_Q
//!
//! 各 query は:
//!   1 x    → 配列に x を追加
//!   2 k    → k 番目の要素を出力
//!   3      → 最小値を出力
//! ```
//!
//! 使用 `NodeKind`: `Scalar`, `Tuple`, `Choice`, `Repeat`, `Sequence`

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, Literal, NodeKind, Reference};
use cp_ast_tree::{
    render_combined_tree, render_constraint_tree, render_structure_tree, TreeOptions,
};

#[allow(clippy::too_many_lines)]
fn main() {
    let mut engine = AstEngine::new();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let q_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("t"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("x"),
    });
    let k_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("k"),
    });

    // type1: "1 x"
    let type1 = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![t_id, x_id],
    });
    // type2: "2 k"
    let type2 = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![t_id, k_id],
    });
    // type3: "3"  (tag only)
    let type3 = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![t_id],
    });

    let choice = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![type1]),
            (Literal::IntLit(2), vec![type2]),
            (Literal::IntLit(3), vec![type3]),
        ],
    });

    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, q_id],
    });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(q_id)),
        index_var: None,
        body: vec![choice],
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![header, repeat],
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
        Some(q_id),
        Constraint::Range {
            target: Reference::VariableRef(q_id),
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(5)),
            },
        },
    );
    engine.constraints.add(
        Some(t_id),
        Constraint::Range {
            target: Reference::VariableRef(t_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(3),
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
        Some(x_id),
        Constraint::Range {
            target: Reference::VariableRef(x_id),
            lower: Expression::Lit(1),
            upper: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(9)),
            },
        },
    );
    engine.constraints.add(
        Some(k_id),
        Constraint::Range {
            target: Reference::VariableRef(k_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
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
