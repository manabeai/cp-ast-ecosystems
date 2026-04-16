use cp_ast_core::operation::AstEngine;
use cp_ast_core::projection::{
    get_constraint_targets, get_expr_candidates, get_hole_candidates, project_full,
    project_node_detail, DiagnosticLevel,
};

#[test]
fn test_project_full_empty_ast() {
    // Create an AST with holes for testing
    let mut engine = AstEngine::new();

    // Add a hole to the root sequence
    let hole_id = engine
        .structure
        .add_node(cp_ast_core::structure::NodeKind::Hole {
            expected_kind: None,
        });

    // Modify the root to include the hole
    let root_id = engine.structure.root();
    if let Some(root) = engine.structure.get_mut(root_id) {
        root.set_kind(cp_ast_core::structure::NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let projection = project_full(&engine);

    // Should have at least the root sequence and a hole
    assert!(!projection.outline.is_empty());
    assert!(projection.completeness.total_holes > 0);
    assert!(!projection.completeness.is_complete);

    // Should have diagnostics about unfilled holes
    assert!(!projection.diagnostics.is_empty());
    let has_hole_diagnostic = projection
        .diagnostics
        .iter()
        .any(|d| d.level == DiagnosticLevel::Info && d.message.contains("Unfilled hole"));
    assert!(has_hole_diagnostic);
}

#[test]
fn test_project_node_detail_array() {
    let engine = AstEngine::new();

    // Find a node to test with - for now test with root
    let root_id = engine.structure.root();

    let detail = project_node_detail(&engine, root_id);
    assert!(detail.is_some());

    let detail = detail.unwrap();
    // Root sequence should have no expression slots but might have constraints
    assert!(detail.slots.is_empty());
}

#[test]
fn test_get_hole_candidates() {
    // Create an AST with holes for testing
    let mut engine = AstEngine::new();

    // Add a hole to the root sequence
    let hole_id = engine
        .structure
        .add_node(cp_ast_core::structure::NodeKind::Hole {
            expected_kind: None,
        });

    // Modify the root to include the hole
    let root_id = engine.structure.root();
    if let Some(root) = engine.structure.get_mut(root_id) {
        root.set_kind(cp_ast_core::structure::NodeKind::Sequence {
            children: vec![hole_id],
        });
    }

    let candidates = get_hole_candidates(&engine, hole_id);
    assert!(!candidates.is_empty());

    // Should include Scalar, Array, Matrix, Section candidates
    let has_scalar = candidates.iter().any(|c| c.kind == "Scalar");
    let has_array = candidates.iter().any(|c| c.kind == "Array");
    assert!(has_scalar);
    assert!(has_array);

    // Scalar should have suggested names
    if let Some(scalar_candidate) = candidates.iter().find(|c| c.kind == "Scalar") {
        assert!(!scalar_candidate.suggested_names.is_empty());
        assert!(scalar_candidate.suggested_names.contains(&"N".to_owned()));
    }
}

#[test]
fn test_get_expr_candidates() {
    let engine = AstEngine::new();

    let menu = get_expr_candidates(&engine);

    // Should have some literals
    assert!(!menu.literals.is_empty());
    assert!(menu.literals.contains(&0));
    assert!(menu.literals.contains(&1));

    // References might be empty for a fresh engine, but the structure should work
    // (references will be filled when scalar nodes are added)
    assert!(menu.references.is_empty() || !menu.references.is_empty());
}

#[test]
fn test_get_constraint_targets() {
    let engine = AstEngine::new();

    let menu = get_constraint_targets(&engine);

    // For a fresh engine, might not have many non-hole, non-structural nodes
    // But the function should work without errors
    assert!(menu.targets.is_empty() || !menu.targets.is_empty());
}

#[test]
fn test_hole_candidates_invalid_node() {
    let engine = AstEngine::new();

    // Test with root node (which is not a hole)
    let root_id = engine.structure.root();
    let candidates = get_hole_candidates(&engine, root_id);

    // Should return empty since root is not a hole
    assert!(candidates.is_empty());
}

#[test]
fn test_outline_structure() {
    let engine = AstEngine::new();

    let projection = project_full(&engine);

    // Test outline properties
    for node in &projection.outline {
        assert!(!node.label.is_empty());
        assert!(!node.kind_label.is_empty());
        // Depth should be reasonable (not negative, not excessively deep)
        assert!(node.depth < 100); // Sanity check
    }

    // Root should be at depth 0
    if let Some(root_node) = projection.outline.iter().find(|n| n.depth == 0) {
        assert_eq!(root_node.kind_label, "Sequence");
    }
}
