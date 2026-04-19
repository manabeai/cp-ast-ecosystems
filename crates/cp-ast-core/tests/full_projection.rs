use cp_ast_core::operation::action::Action;
use cp_ast_core::operation::engine::AstEngine;
use cp_ast_core::operation::types::{FillContent, VarType};
use cp_ast_core::projection::types::HotspotDirection;

#[test]
fn empty_engine_has_below_hotspot() {
    let engine = AstEngine::new();
    let proj = cp_ast_core::projection::project_full(&engine);
    assert!(
        !proj.hotspots.is_empty(),
        "Empty engine should have a below hotspot"
    );
    assert_eq!(proj.hotspots[0].direction, HotspotDirection::Below);
}

#[test]
fn scalar_generates_draft_range() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    assert_eq!(
        proj.constraints.drafts.len(),
        1,
        "Scalar Int should generate 1 draft Range"
    );
    assert!(proj.constraints.drafts[0].display.contains('N'));
}

#[test]
fn scalar_has_right_hotspot() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    let right = proj
        .hotspots
        .iter()
        .find(|h| h.direction == HotspotDirection::Right);
    assert!(
        right.is_some(),
        "Scalar in Sequence should have Right hotspot"
    );
}

#[test]
fn available_vars_includes_scalar() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    assert!(proj.available_vars.iter().any(|v| v.name == "N"));
}

#[test]
fn existing_range_suppresses_draft() {
    use cp_ast_core::operation::types::{ConstraintDef, ConstraintDefKind};

    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    let n_id = *result.created_nodes.last().unwrap();
    engine
        .apply(&Action::AddConstraint {
            target: n_id,
            constraint: ConstraintDef {
                kind: ConstraintDefKind::Range {
                    lower: "1".to_owned(),
                    upper: "100".to_owned(),
                },
            },
        })
        .unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    // Draft should be empty since real Range exists
    assert_eq!(proj.constraints.drafts.len(), 0);
    // Should have at least the Range + TypeDecl completed constraints
    assert!(!proj.constraints.completed.is_empty());
}
