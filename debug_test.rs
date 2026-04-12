use cp_ast_core::constraint::{Constraint, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render_tex::{render_constraints_tex, TexOptions};
use cp_ast_core::structure::{Ident, NodeKind, Reference};

fn main() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar { name: Ident::new("N") });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence { children: vec![n_id] });
    }
    // Add in reverse order — Guarantee first, then Range
    engine.constraints.add(
        None,
        Constraint::Guarantee {
            description: "answer exists".to_owned(),
            predicate: None,
        },
    );
    engine.constraints.add(
        Some(n_id),
        Constraint::Range {
            target: Reference::VariableRef(n_id),
            lower: Expression::Lit(1),
            upper: Expression::Lit(100),
        },
    );

    let result = render_constraints_tex(&engine, &TexOptions::default());
    println!("Generated TeX:");
    println!("{}", result.tex);
    println!("Lines:");
    for (i, line) in result.tex.lines().enumerate() {
        println!("{}: {}", i, line);
    }
}
