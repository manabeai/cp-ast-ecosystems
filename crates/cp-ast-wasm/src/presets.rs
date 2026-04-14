//! Preset AST definitions for the AST viewer frontend.
//!
//! Each preset builds a complete `AstEngine` with both structure nodes and
//! constraints, representing a typical competitive programming input pattern.

use cp_ast_core::constraint::{
    CharSetSpec, Constraint, DistinctUnit, ExpectedType, Expression, PropertyTag, SortOrder,
};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::structure::{Ident, Literal, NodeKind, NodeKindHint, Reference};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
}

/// Returns the list of all available presets.
#[must_use]
pub fn list() -> Vec<PresetInfo> {
    vec![
        PresetInfo {
            name: "scalar_only".to_owned(),
            description: "Single integer N with range constraint".to_owned(),
        },
        PresetInfo {
            name: "scalar_array".to_owned(),
            description: "N followed by array A of length N".to_owned(),
        },
        PresetInfo {
            name: "tuple_repeat".to_owned(),
            description: "N followed by N pairs (A_i, B_i)".to_owned(),
        },
        PresetInfo {
            name: "matrix".to_owned(),
            description: "H W followed by H×W matrix".to_owned(),
        },
        PresetInfo {
            name: "choice".to_owned(),
            description: "Query-type with tag-dependent branching".to_owned(),
        },
        PresetInfo {
            name: "graph_simple".to_owned(),
            description: "N M followed by M edges (u_i, v_i), simple connected".to_owned(),
        },
        PresetInfo {
            name: "sorted_distinct".to_owned(),
            description: "Sorted ascending + distinct array".to_owned(),
        },
        PresetInfo {
            name: "string_problem".to_owned(),
            description: "N strings with charset and length constraints".to_owned(),
        },
        PresetInfo {
            name: "hole_structure".to_owned(),
            description: "Incomplete AST with Hole placeholders".to_owned(),
        },
    ]
}

/// Build a preset AST by name.
#[must_use]
pub fn build(name: &str) -> Option<AstEngine> {
    match name {
        "scalar_only" => Some(scalar_only()),
        "scalar_array" => Some(scalar_array()),
        "tuple_repeat" => Some(tuple_repeat()),
        "matrix" => Some(matrix()),
        "choice" => Some(choice()),
        "graph_simple" => Some(graph_simple()),
        "sorted_distinct" => Some(sorted_distinct()),
        "string_problem" => Some(string_problem()),
        "hole_structure" => Some(hole_structure()),
        _ => None,
    }
}

// ── helpers ──────────────────────────────────────────────────────────

fn pow10(exp: i64) -> Expression {
    Expression::Pow {
        base: Box::new(Expression::Lit(10)),
        exp: Box::new(Expression::Lit(exp)),
    }
}

fn var_ref(id: cp_ast_core::structure::NodeId) -> Reference {
    Reference::VariableRef(id)
}

fn var_expr(id: cp_ast_core::structure::NodeId) -> Expression {
    Expression::Var(var_ref(id))
}

fn indexed_ref(id: cp_ast_core::structure::NodeId, idx: &str) -> Reference {
    Reference::IndexedRef {
        target: id,
        indices: vec![Ident::new(idx)],
    }
}

fn add_int_type(engine: &mut AstEngine, id: cp_ast_core::structure::NodeId) {
    engine.constraints.add(
        Some(id),
        Constraint::TypeDecl {
            target: var_ref(id),
            expected: ExpectedType::Int,
        },
    );
}

fn add_range(
    engine: &mut AstEngine,
    id: cp_ast_core::structure::NodeId,
    target: Reference,
    lower: Expression,
    upper: Expression,
) {
    engine.constraints.add(
        Some(id),
        Constraint::Range {
            target,
            lower,
            upper,
        },
    );
}

fn set_root_sequence(engine: &mut AstEngine, children: Vec<cp_ast_core::structure::NodeId>) {
    engine
        .structure
        .get_mut(engine.structure.root())
        .unwrap()
        .set_kind(NodeKind::Sequence { children });
}

// ── preset builders ─────────────────────────────────────────────────

/// Just `N`. Constraints: 1 ≤ N ≤ 10^9, Int.
fn scalar_only() -> AstEngine {
    let mut engine = AstEngine::new();

    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    set_root_sequence(&mut engine, vec![n]);

    add_int_type(&mut engine, n);
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(9));

    engine
}

/// `N` then `A_1..A_N`. Constraints: 1 ≤ N ≤ 10^5, 1 ≤ `A_i` ≤ 10^9, Int.
fn scalar_array() -> AstEngine {
    let mut engine = AstEngine::new();

    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: var_expr(n),
    });
    set_root_sequence(&mut engine, vec![n, a]);

    add_int_type(&mut engine, n);
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, a);
    add_range(
        &mut engine,
        a,
        indexed_ref(a, "i"),
        Expression::Lit(1),
        pow10(9),
    );

    engine
}

/// `N` then N lines of `(A_i, B_i)`. Uses Repeat+Tuple.
fn tuple_repeat() -> AstEngine {
    let mut engine = AstEngine::new();

    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("A"),
    });
    let b = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("B"),
    });
    let tup = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![a, b],
    });
    let rep = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(n),
        index_var: Some(Ident::new("i")),
        body: vec![tup],
    });
    set_root_sequence(&mut engine, vec![n, rep]);

    add_int_type(&mut engine, n);
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, a);
    add_range(&mut engine, a, var_ref(a), Expression::Lit(1), pow10(9));
    add_int_type(&mut engine, b);
    add_range(&mut engine, b, var_ref(b), Expression::Lit(1), pow10(9));

    engine
}

/// `H W` then H×W matrix C. Uses Matrix node.
fn matrix() -> AstEngine {
    let mut engine = AstEngine::new();

    let h = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("H"),
    });
    let w = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("W"),
    });
    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![h, w],
    });
    let c = engine.structure.add_node(NodeKind::Matrix {
        name: Ident::new("C"),
        rows: var_ref(h),
        cols: var_ref(w),
    });
    set_root_sequence(&mut engine, vec![header, c]);

    add_int_type(&mut engine, h);
    add_range(&mut engine, h, var_ref(h), Expression::Lit(1), pow10(3));
    add_int_type(&mut engine, w);
    add_range(&mut engine, w, var_ref(w), Expression::Lit(1), pow10(3));
    add_int_type(&mut engine, c);
    add_range(
        &mut engine,
        c,
        indexed_ref(c, "i"),
        Expression::Lit(0),
        pow10(9),
    );

    engine
}

/// Query-type with tag branching. Choice node with 2 variants.
///
/// ```text
/// Q
/// for each query:
///   t_i x_i          (when t_i = 1)
///   t_i l_i r_i      (when t_i = 2)
/// ```
#[allow(clippy::many_single_char_names)]
fn choice() -> AstEngine {
    let mut engine = AstEngine::new();

    let q = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    let t = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("t"),
    });

    // Variant 1: t=1 → x
    let x = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("x"),
    });
    // Variant 2: t=2 → l r
    let l = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("l"),
    });
    let r = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("r"),
    });

    let choice_node = engine.structure.add_node(NodeKind::Choice {
        tag: var_ref(t),
        variants: vec![
            (Literal::IntLit(1), vec![x]),
            (Literal::IntLit(2), vec![l, r]),
        ],
    });
    let rep = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(q),
        index_var: Some(Ident::new("i")),
        body: vec![choice_node],
    });
    set_root_sequence(&mut engine, vec![q, rep]);

    add_int_type(&mut engine, q);
    add_range(&mut engine, q, var_ref(q), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, t);
    add_range(
        &mut engine,
        t,
        var_ref(t),
        Expression::Lit(1),
        Expression::Lit(2),
    );
    add_int_type(&mut engine, x);
    add_range(&mut engine, x, var_ref(x), Expression::Lit(1), pow10(9));
    add_int_type(&mut engine, l);
    add_int_type(&mut engine, r);
    add_range(&mut engine, l, var_ref(l), Expression::Lit(1), pow10(9));
    add_range(&mut engine, r, var_ref(r), Expression::Lit(1), pow10(9));

    engine
}

/// `N M` then M edges `(u_i, v_i)`. Property: Simple, Connected.
fn graph_simple() -> AstEngine {
    let mut engine = AstEngine::new();

    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let m = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("M"),
    });
    let u = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });

    let header = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n, m],
    });
    let edge = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u, v],
    });
    let rep = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(m),
        index_var: Some(Ident::new("i")),
        body: vec![edge],
    });
    set_root_sequence(&mut engine, vec![header, rep]);

    add_int_type(&mut engine, n);
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, m);
    add_range(&mut engine, m, var_ref(m), Expression::Lit(1), var_expr(n));
    add_int_type(&mut engine, u);
    add_range(&mut engine, u, var_ref(u), Expression::Lit(1), var_expr(n));
    add_int_type(&mut engine, v);
    add_range(&mut engine, v, var_ref(v), Expression::Lit(1), var_expr(n));
    engine.constraints.add(
        None,
        Constraint::Property {
            target: var_ref(n),
            tag: PropertyTag::Simple,
        },
    );
    engine.constraints.add(
        None,
        Constraint::Property {
            target: var_ref(n),
            tag: PropertyTag::Connected,
        },
    );

    engine
}

/// Sorted ascending + distinct array.
fn sorted_distinct() -> AstEngine {
    let mut engine = AstEngine::new();

    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: var_expr(n),
    });
    set_root_sequence(&mut engine, vec![n, a]);

    add_int_type(&mut engine, n);
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    add_int_type(&mut engine, a);
    add_range(
        &mut engine,
        a,
        indexed_ref(a, "i"),
        Expression::Lit(1),
        pow10(9),
    );
    engine.constraints.add(
        Some(a),
        Constraint::Sorted {
            elements: var_ref(a),
            order: SortOrder::Ascending,
        },
    );
    engine.constraints.add(
        Some(a),
        Constraint::Distinct {
            elements: var_ref(a),
            unit: DistinctUnit::Element,
        },
    );

    engine
}

/// N strings with `CharSet(LowerAlpha)` + `StringLength` constraints.
fn string_problem() -> AstEngine {
    let mut engine = AstEngine::new();

    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let s = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("S"),
    });
    let rep = engine.structure.add_node(NodeKind::Repeat {
        count: var_expr(n),
        index_var: Some(Ident::new("i")),
        body: vec![s],
    });
    set_root_sequence(&mut engine, vec![n, rep]);

    add_int_type(&mut engine, n);
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));
    engine.constraints.add(
        Some(s),
        Constraint::TypeDecl {
            target: var_ref(s),
            expected: ExpectedType::Str,
        },
    );
    engine.constraints.add(
        Some(s),
        Constraint::CharSet {
            target: var_ref(s),
            charset: CharSetSpec::LowerAlpha,
        },
    );
    engine.constraints.add(
        Some(s),
        Constraint::StringLength {
            target: var_ref(s),
            min: Expression::Lit(1),
            max: pow10(5),
        },
    );

    engine
}

/// Incomplete AST with Hole nodes for UI demonstration.
///
/// ```text
/// N
/// <Hole: array expected>
/// <Hole: any>
/// ```
fn hole_structure() -> AstEngine {
    let mut engine = AstEngine::new();

    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let hole1 = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::AnyArray),
    });
    let hole2 = engine.structure.add_node(NodeKind::Hole {
        expected_kind: Some(NodeKindHint::Any),
    });
    set_root_sequence(&mut engine, vec![n, hole1, hole2]);

    add_int_type(&mut engine, n);
    add_range(&mut engine, n, var_ref(n), Expression::Lit(1), pow10(5));

    engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_presets_build_and_serialize() {
        for info in list() {
            let engine =
                build(&info.name).unwrap_or_else(|| panic!("preset '{}' not found", info.name));
            let json = cp_ast_json::serialize_ast(&engine)
                .unwrap_or_else(|e| panic!("preset '{}' failed to serialize: {e}", info.name));
            assert!(
                !json.is_empty(),
                "preset '{}' produced empty JSON",
                info.name
            );
            let _ = cp_ast_json::deserialize_ast(&json)
                .unwrap_or_else(|e| panic!("preset '{}' failed roundtrip: {e}", info.name));
        }
    }

    #[test]
    fn all_presets_render() {
        for info in list() {
            let engine = build(&info.name).unwrap();
            let _ = cp_ast_core::render::render_input(&engine);
            let _ = cp_ast_core::render::render_constraints(&engine);
            let _ =
                cp_ast_tree::render_structure_tree(&engine, &cp_ast_tree::TreeOptions::default());
            let _ =
                cp_ast_tree::render_constraint_tree(&engine, &cp_ast_tree::TreeOptions::default());
        }
    }

    #[test]
    fn unknown_preset_returns_none() {
        assert!(build("nonexistent").is_none());
    }

    #[test]
    fn preset_list_has_9_entries() {
        assert_eq!(list().len(), 9);
    }
}
