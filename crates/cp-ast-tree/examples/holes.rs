//! Hole を含む未完成 AST 例
//!
//! Hole は AST のまだ埋まっていないスロット。
//! 設計中・解析途中の AST を表現するのに使う。
//!
//! 使用 `NodeKind`: `Scalar`, `Hole`, `Sequence`
//! 使用 `Constraint`: なし

use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, NodeKind, NodeKindHint};
use cp_ast_tree::{TreeOptions, render_structure_tree};

fn main() {
    let mut engine = AstEngine::new();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });

    // まだ何が来るか分かっていないスロット (ヒントなし)
    let hole1 = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });

    // 配列か行列が来ると推測されるスロット (ヒントあり)
    let hole2 = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyArray),
    });

    // スカラーが来ると推測されるスロット
    let hole3 = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyScalar),
    });

    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, hole1, hole2, hole3],
        });

    let opts = TreeOptions::default();
    println!("=== Structure (with holes) ===");
    print!("{}", render_structure_tree(&engine, &opts));

    println!("\n=== with node IDs ===");
    let opts_with_ids = TreeOptions {
        show_node_ids: true,
        ..TreeOptions::default()
    };
    print!("{}", render_structure_tree(&engine, &opts_with_ids));
}
