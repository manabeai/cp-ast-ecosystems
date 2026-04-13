//! 文字列問題の AST 例
//!
//! 入力形式:
//! ```text
//! N
//! S_1
//! ...
//! S_N
//! ```
//!
//! 使用 `NodeKind`: `Scalar`, `Repeat`, `Sequence`

use cp_ast_core::constraint::{CharSetSpec, Constraint, ExpectedType, Expression};
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
    let s_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("S"),
    });

    let repeat = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: Some(Ident::new("i")),
        body: vec![s_id],
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat],
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
        Some(s_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(s_id),
            expected: ExpectedType::Str,
        },
    );
    engine.constraints.add(
        Some(s_id),
        Constraint::CharSet {
            target: Reference::VariableRef(s_id),
            charset: CharSetSpec::LowerAlpha,
        },
    );
    engine.constraints.add(
        Some(s_id),
        Constraint::StringLength {
            target: Reference::VariableRef(s_id),
            min: Expression::Lit(1),
            max: Expression::Pow {
                base: Box::new(Expression::Lit(10)),
                exp: Box::new(Expression::Lit(5)),
            },
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
