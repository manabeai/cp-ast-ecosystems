//! グラフ問題の AST 例
//!
//! 入力形式:
//! ```text
//! N M
//! u_1 v_1
//! ...
//! u_M v_M
//! ```
//!
//! 使用 `NodeKind`: `Scalar`, `Tuple`, `Repeat`, `Sequence`
//!
//! 制約:
//! - 1 ≤ N ≤ 10^5
//! - 1 ≤ M ≤ N
//! - 1 ≤ `u_i`, `v_i` ≤ N
//! - N, M, `u_i`, `v_i` は整数
//! - グラフは単純連結

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};
use cp_ast_tree::{
    render_combined_tree, render_constraint_tree, render_structure_tree, TreeOptions,
};

fn main() {
    let mut engine = AstEngine::new();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });

    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n_id, m_id],
    });
    let edge = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(m_id)),
        index_var: Some(Ident::new("i")),
        body: vec![edge],
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
        Some(n_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(n_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(m_id),
        Constraint::Range {
            target: Reference::VariableRef(m_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );
    engine.constraints.add(
        Some(u_id),
        Constraint::Range {
            target: Reference::VariableRef(u_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );
    engine.constraints.add(
        Some(v_id),
        Constraint::Range {
            target: Reference::VariableRef(v_id),
            lower: Expression::Lit(1),
            upper: Expression::Var(Reference::VariableRef(n_id)),
        },
    );
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "The graph is simple and connected".to_owned(),
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
