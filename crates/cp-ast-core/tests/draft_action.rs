use std::collections::HashMap;

use cp_ast_core::constraint::CharSetSpec;
use cp_ast_core::operation::draft_action::{
    build_constraint_actions_from_draft, build_hotspot_action_from_draft,
    build_replace_action_from_draft, ConstraintDraft, HotspotDraft, NodeReplacementDraft,
    VariableCandidate,
};
use cp_ast_core::operation::{
    Action, ConstraintDefKind, FillContent, LengthSpec, VarType,
};
use cp_ast_core::projection::types::{HotspotAction, HotspotActionKind};
use cp_ast_core::constraint::ConstraintId;
use cp_ast_core::structure::NodeId;

fn vars() -> Vec<VariableCandidate> {
    vec![
        VariableCandidate {
            name: "N".to_owned(),
            node_id: NodeId::from_raw(1),
        },
        VariableCandidate {
            name: "M".to_owned(),
            node_id: NodeId::from_raw(2),
        },
    ]
}

#[test]
fn builds_replace_node_action_from_domain_draft() {
    let mut fields = HashMap::new();
    fields.insert("name".to_owned(), "B".to_owned());
    fields.insert("type".to_owned(), "char".to_owned());

    let action = build_replace_action_from_draft(&NodeReplacementDraft {
        target_id: NodeId::from_raw(4),
        candidate: "scalar".to_owned(),
        fields,
        variables: Vec::new(),
    })
    .expect("scalar replacement draft should build");

    assert_eq!(
        action,
        Action::ReplaceNode {
            target: NodeId::from_raw(4),
            replacement: FillContent::Scalar {
                name: "B".to_owned(),
                typ: VarType::Char,
            },
        }
    );
}

#[test]
fn builds_array_hotspot_action_from_domain_draft() {
    let mut fields = HashMap::new();
    fields.insert("name".to_owned(), "A".to_owned());
    fields.insert("type".to_owned(), "number".to_owned());
    fields.insert("length".to_owned(), "N".to_owned());

    let action = build_hotspot_action_from_draft(&HotspotDraft {
        route: HotspotAction {
            kind: HotspotActionKind::AddSlotElement,
            target_id: NodeId::from_raw(0),
            slot_name: Some("children".to_owned()),
        },
        candidate: "array".to_owned(),
        fields,
        variables: vars(),
    })
    .expect("array draft should build");

    assert_eq!(
        action,
        Action::AddSlotElement {
            parent: NodeId::from_raw(0),
            slot_name: "children".to_owned(),
            element: FillContent::Array {
                name: "A".to_owned(),
                element_type: VarType::Int,
                length: LengthSpec::RefVar(NodeId::from_raw(1)),
            },
        }
    );
}

#[test]
fn builds_replacement_constraint_actions_from_domain_draft() {
    let actions = build_constraint_actions_from_draft(&ConstraintDraft {
        target_id: NodeId::from_raw(3),
        template: "CharSet".to_owned(),
        existing_constraint_id: Some(ConstraintId::from_raw(9)),
        lower: None,
        upper: None,
        over_var: None,
        charset: Some(CharSetSpec::LowerAlpha),
    })
    .expect("charset draft should build");

    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions[0],
        Action::RemoveConstraint {
            constraint_id: ConstraintId::from_raw(9),
        }
    );
    assert_eq!(
        actions[1],
        Action::AddConstraint {
            target: NodeId::from_raw(3),
            constraint: cp_ast_core::operation::ConstraintDef {
                kind: ConstraintDefKind::CharSet {
                    charset: CharSetSpec::LowerAlpha,
                },
            },
        }
    );
}
