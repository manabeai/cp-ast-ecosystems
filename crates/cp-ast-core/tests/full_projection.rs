use cp_ast_core::operation::action::Action;
use cp_ast_core::operation::engine::AstEngine;
use cp_ast_core::operation::types::{FillContent, LengthSpec, VarType};
use cp_ast_core::projection::types::{HotspotActionKind, HotspotDirection};
use cp_ast_core::structure::Reference;
use cp_ast_core::structure::{Ident, NodeKind};

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
fn char_scalar_generates_charset_draft() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "c".to_owned(),
                typ: VarType::Char,
            },
        })
        .unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    assert_eq!(proj.constraints.items.len(), 1);
    assert_eq!(proj.constraints.drafts[0].template, "CharSet");
    assert!(proj.constraints.items[0].display.contains("charset(c)"));
}

#[test]
fn string_scalar_generates_charset_and_length_drafts() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "S".to_owned(),
                typ: VarType::Str,
            },
        })
        .unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    let templates: Vec<_> = proj
        .constraints
        .drafts
        .iter()
        .map(|draft| draft.template.as_str())
        .collect();
    assert_eq!(templates, vec!["CharSet", "StringLength"]);
}

#[test]
fn scalar_has_right_hotspot() {
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
    let n = *result.created_nodes.last().unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    let right = proj
        .hotspots
        .iter()
        .find(|h| h.direction == HotspotDirection::Right);
    assert!(
        right.is_some(),
        "Scalar in Sequence should have Right hotspot"
    );
    let right = right.unwrap();
    assert!(right.candidates.iter().any(|c| c == "scalar"));
    assert!(right.candidates.iter().any(|c| c == "array"));
    assert_eq!(right.action.kind, HotspotActionKind::AddSibling);
    assert_eq!(right.action.target_id, n);
    assert!(right.action.slot_name.is_none());
    assert!(
        right
            .candidate_details
            .iter()
            .any(|candidate| candidate.kind == "array"
                && candidate.fields.iter().any(|field| field.name == "length"
                    && field.required
                    && field.field_type == "length"))
    );
}

#[test]
fn tuple_projects_as_single_structure_line() {
    let mut engine = AstEngine::new();
    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: cp_ast_core::constraint::Expression::Var(
            cp_ast_core::structure::Reference::VariableRef(n),
        ),
    });
    let tuple = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![n, a],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![tuple],
        });

    let proj = cp_ast_core::projection::project_full(&engine);
    assert_eq!(proj.structure_lines.len(), 1);
    let labels: Vec<&str> = proj.structure_lines[0]
        .nodes
        .iter()
        .map(|node| node.label.as_str())
        .collect();
    assert_eq!(labels, vec!["N", "A[]"]);

    let n_right = proj
        .hotspots
        .iter()
        .find(|h| h.parent_id == n && h.direction == HotspotDirection::Right);
    let a_right = proj
        .hotspots
        .iter()
        .find(|h| h.parent_id == a && h.direction == HotspotDirection::Right);
    assert!(n_right.is_none());
    let a_right = a_right.unwrap();
    assert_eq!(a_right.action.kind, HotspotActionKind::AddSibling);
    assert_eq!(a_right.action.target_id, a);
}

#[test]
fn below_and_inside_hotspots_project_action_targets_without_label_inference() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    let repeat = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Repeat {
                count: LengthSpec::Expr("T".to_owned()),
            },
        })
        .unwrap()
        .created_nodes
        .into_iter()
        .find(|id| {
            matches!(
                engine.structure.get(*id).unwrap().kind(),
                NodeKind::Repeat { .. }
            )
        })
        .unwrap();

    let hole = match engine.structure.get(repeat).unwrap().kind() {
        NodeKind::Repeat { body, .. } => body[0],
        _ => unreachable!(),
    };

    let proj = cp_ast_core::projection::project_full(&engine);
    let inside = proj
        .hotspots
        .iter()
        .find(|h| h.parent_id == repeat && h.direction == HotspotDirection::Inside)
        .unwrap();
    assert_eq!(inside.action.kind, HotspotActionKind::FillHole);
    assert_eq!(inside.action.target_id, hole);

    engine
        .apply(&Action::FillHole {
            target: hole,
            fill: FillContent::Scalar {
                name: "N".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    let proj = cp_ast_core::projection::project_full(&engine);
    let below = proj
        .hotspots
        .iter()
        .find(|h| h.parent_id == repeat && h.direction == HotspotDirection::Below)
        .unwrap();
    assert_eq!(below.action.kind, HotspotActionKind::AddSlotElement);
    assert_eq!(below.action.target_id, repeat);
    assert_eq!(below.action.slot_name.as_deref(), Some("body"));
}

#[test]
fn projected_node_contains_edit_metadata_without_parsing_label() {
    let mut engine = AstEngine::new();
    let n = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let a = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("A"),
        length: cp_ast_core::constraint::Expression::Var(Reference::VariableRef(n)),
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n, a],
        });

    let proj = cp_ast_core::projection::project_full(&engine);
    let a_node = proj.nodes.iter().find(|node| node.id == a).unwrap();
    let edit = a_node.edit.as_ref().unwrap();
    assert_eq!(edit.kind, "array");
    assert_eq!(edit.name, "A");
    assert_eq!(edit.value_type, "number");
    assert_eq!(edit.length_expr.as_deref(), Some("N"));
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
