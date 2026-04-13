//! 行列問題の AST 例
//!
//! 入力形式:
//! ```text
//! H W
//! A_{1,1} ... A_{1,W}
//! ...
//! A_{H,1} ... A_{H,W}
//! ```
//!
//! 使用 `NodeKind`: `Scalar`, `Matrix`, `Tuple`, `Sequence`
//! 使用 `Constraint`: `Range`, `TypeDecl`, `Guarantee`

use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, Reference};
use cp_ast_tree::{
    render_combined_tree, render_constraint_tree, render_structure_tree, TreeOptions,
};

fn main() {
    let mut engine = AstEngine::new();

    let h_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("H"),
    });
    let w_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("W"),
    });
    let a_id = engine.structure.add_node(NodeKind::Matrix {
        name: Ident::new("A"),
        rows: Reference::VariableRef(h_id),
        cols: Reference::VariableRef(w_id),
    });

    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![h_id, w_id],
    });
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![header, a_id],
        });

    engine.constraints.add(
        Some(h_id),
        Constraint::Range {
            target: Reference::VariableRef(h_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(1000),
        },
    );
    engine.constraints.add(
        Some(h_id),
        Constraint::TypeDecl {
            target: Reference::VariableRef(h_id),
            expected: ExpectedType::Int,
        },
    );
    engine.constraints.add(
        Some(w_id),
        Constraint::Range {
            target: Reference::VariableRef(w_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(1000),
        },
    );
    engine.constraints.add(
        Some(a_id),
        Constraint::Range {
            target: Reference::IndexedRef {
                target: a_id,
                indices: vec![Ident::new("i"), Ident::new("j")],
            },
            lower: Expression::Lit(0),
            upper: Expression::Lit(9),
        },
    );
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "H * W ≤ 10^6".to_owned(),
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
