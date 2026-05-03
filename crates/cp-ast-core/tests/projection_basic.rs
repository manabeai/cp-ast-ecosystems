use cp_ast_core::operation::AstEngine;
use cp_ast_core::projection::*;
use cp_ast_core::structure::*;

#[test]
fn nodes_returns_dfs_order() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id, hole_id],
        });
    }
    let nodes = engine.nodes();
    assert_eq!(nodes.len(), 2); // N + hole (root Sequence is hidden from UI projection)
    assert_eq!(nodes[0].label, "N");
    assert_eq!(nodes[0].depth, 0);
    assert!(nodes[1].is_hole);
    assert_eq!(nodes[1].depth, 0);
}

#[test]
fn empty_root_sequence_is_hidden_from_nodes_projection() {
    let engine = AstEngine::new();
    let nodes = engine.nodes();
    assert!(nodes.is_empty());
}

#[test]
fn children_of_sequence() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    if let Some(root) = engine.structure.get_mut(engine.structure.root()) {
        root.set_kind(NodeKind::Sequence {
            children: vec![n_id],
        });
    }
    let children = engine.children(engine.structure.root());
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "children");
    assert_eq!(children[0].child, n_id);
}

#[test]
fn inspect_scalar_node() {
    let mut engine = AstEngine::new();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let detail = engine.inspect(n_id).unwrap();
    assert_eq!(detail.kind_label, "N");
    assert!(detail.constraints.is_empty());
}

#[test]
fn hole_candidates_for_hole() {
    let mut engine = AstEngine::new();
    let hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    let candidates = engine.hole_candidates(hole_id);
    assert!(!candidates.is_empty());
    assert!(
        candidates
            .iter()
            .any(|c| matches!(c, CandidateKind::IntroduceScalar { .. }))
    );
}

#[test]
fn completeness_with_holes() {
    let mut engine = AstEngine::new();
    let _hole_id = engine.structure.add_node(NodeKind::Hole {
        expected_kind: None,
    });
    let summary = engine.completeness();
    assert!(summary.total_holes > 0);
    assert!(!summary.is_complete);
}

#[test]
fn completeness_all_filled() {
    let engine = AstEngine::new(); // Just root Sequence, no holes
    let summary = engine.completeness();
    assert_eq!(summary.total_holes, 0);
    assert!(summary.is_complete);
}

#[test]
fn why_not_editable_root() {
    let engine = AstEngine::new();
    let reason = engine.why_not_editable(engine.structure.root());
    assert!(matches!(reason, Some(NotEditableReason::IsRoot)));
}
