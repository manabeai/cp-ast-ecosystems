use std::collections::HashMap;

use crate::constraint::{CharSetSpec, ConstraintId};
use crate::operation::action::Action;
use crate::operation::types::{
    ConstraintDef, ConstraintDefKind, FillContent, LengthSpec, VarType,
};
use crate::projection::types::{HotspotAction, HotspotActionKind};
use crate::structure::{Literal, NodeId};

/// A variable candidate visible to expression/length draft inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableCandidate {
    /// Display name used in the UI.
    pub name: String,
    /// Domain node id referenced by the name.
    pub node_id: NodeId,
}

/// User-entered draft for a structure hotspot action.
#[derive(Debug, Clone, PartialEq)]
pub struct HotspotDraft {
    /// Domain routing projected for this hotspot.
    pub route: HotspotAction,
    /// Selected candidate kind.
    pub candidate: String,
    /// Field values keyed by projected field name.
    pub fields: HashMap<String, String>,
    /// Variables that can resolve length/count field values.
    pub variables: Vec<VariableCandidate>,
}

/// User-entered draft for a constraint edit.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintDraft {
    /// Constraint target.
    pub target_id: NodeId,
    /// Projected constraint template name.
    pub template: String,
    /// Existing constraint to replace, when editing a completed row.
    pub existing_constraint_id: Option<ConstraintId>,
    /// Lower/min bound draft.
    pub lower: Option<String>,
    /// Upper/max bound draft.
    pub upper: Option<String>,
    /// SumBound variable name draft.
    pub over_var: Option<String>,
    /// CharSet draft.
    pub charset: Option<CharSetSpec>,
}

/// User-entered draft for replacing an existing projected node.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeReplacementDraft {
    /// Node to replace.
    pub target_id: NodeId,
    /// Replacement candidate kind.
    pub candidate: String,
    /// Field values keyed by projected field name.
    pub fields: HashMap<String, String>,
    /// Variables that can resolve length field values.
    pub variables: Vec<VariableCandidate>,
}

/// Build the domain action represented by a hotspot draft.
///
/// # Errors
/// Returns an error when required fields are missing or the candidate/route is unsupported.
pub fn build_hotspot_action_from_draft(draft: &HotspotDraft) -> Result<Action, String> {
    if draft.candidate == "variant" {
        let tag_value = field(draft, "tag")?
            .parse::<i64>()
            .map_err(|_| "variant tag must be an integer".to_owned())?;
        let first_name = field(draft, "name")?.to_owned();
        return Ok(Action::AddChoiceVariant {
            choice: draft.route.target_id,
            tag_value: Literal::IntLit(tag_value),
            first_element: FillContent::Scalar {
                name: first_name,
                typ: VarType::Int,
            },
        });
    }

    let fill = build_fill_content(draft)?;
    match draft.route.kind {
        HotspotActionKind::AddSlotElement => Ok(Action::AddSlotElement {
            parent: draft.route.target_id,
            slot_name: draft
                .route
                .slot_name
                .clone()
                .unwrap_or_else(|| "children".to_owned()),
            element: fill,
        }),
        HotspotActionKind::AddSibling => Ok(Action::AddSibling {
            target: draft.route.target_id,
            element: fill,
        }),
        HotspotActionKind::FillHole => Ok(Action::FillHole {
            target: draft.route.target_id,
            fill,
        }),
        HotspotActionKind::AddChoiceVariant => Err("variant route requires variant candidate".to_owned()),
    }
}

/// Build the domain action sequence represented by a constraint draft.
///
/// Existing completed constraints are represented as remove+add to keep the existing
/// operation API atomic.
///
/// # Errors
/// Returns an error when required fields are missing or the template is unsupported.
pub fn build_constraint_actions_from_draft(draft: &ConstraintDraft) -> Result<Vec<Action>, String> {
    let mut actions = Vec::new();
    if let Some(constraint_id) = draft.existing_constraint_id {
        actions.push(Action::RemoveConstraint { constraint_id });
    }

    let kind = match draft.template.as_str() {
        "Range" => ConstraintDefKind::Range {
            lower: required_option(&draft.lower, "lower")?.to_owned(),
            upper: required_option(&draft.upper, "upper")?.to_owned(),
        },
        "StringLength" => ConstraintDefKind::StringLength {
            min: required_option(&draft.lower, "min")?.to_owned(),
            max: required_option(&draft.upper, "max")?.to_owned(),
        },
        "CharSet" => ConstraintDefKind::CharSet {
            charset: draft
                .charset
                .clone()
                .ok_or_else(|| "charset is required".to_owned())?,
        },
        "SumBound" => ConstraintDefKind::SumBound {
            over_var: required_option(&draft.over_var, "over_var")?.to_owned(),
            upper: required_option(&draft.upper, "upper")?.to_owned(),
        },
        other => return Err(format!("unsupported constraint template: {other}")),
    };

    actions.push(Action::AddConstraint {
        target: draft.target_id,
        constraint: ConstraintDef { kind },
    });
    Ok(actions)
}

/// Build the domain action represented by a node replacement draft.
///
/// # Errors
/// Returns an error when required fields are missing or the candidate is unsupported.
pub fn build_replace_action_from_draft(draft: &NodeReplacementDraft) -> Result<Action, String> {
    let hotspot_like = HotspotDraft {
        route: HotspotAction {
            kind: HotspotActionKind::FillHole,
            target_id: draft.target_id,
            slot_name: None,
        },
        candidate: draft.candidate.clone(),
        fields: draft.fields.clone(),
        variables: draft.variables.clone(),
    };
    Ok(Action::ReplaceNode {
        target: draft.target_id,
        replacement: build_fill_content(&hotspot_like)?,
    })
}

fn build_fill_content(draft: &HotspotDraft) -> Result<FillContent, String> {
    match draft.candidate.as_str() {
        "scalar" => Ok(FillContent::Scalar {
            name: field(draft, "name")?.to_owned(),
            typ: var_type(field(draft, "type")?)?,
        }),
        "array" => Ok(FillContent::Array {
            name: field(draft, "name")?.to_owned(),
            element_type: var_type(field(draft, "type")?)?,
            length: length_spec(field(draft, "length")?, &draft.variables),
        }),
        "repeat" => Ok(FillContent::Repeat {
            count: length_spec(field(draft, "count")?, &draft.variables),
        }),
        "grid-template" => Ok(FillContent::GridTemplate {
            name: "S".to_owned(),
            rows: length_spec(field(draft, "rows")?, &draft.variables),
            cols: length_spec(field(draft, "cols")?, &draft.variables),
            cell_type: VarType::Char,
        }),
        "edge-list" => Ok(FillContent::EdgeList {
            edge_count: length_spec(field(draft, "count")?, &draft.variables),
        }),
        "weighted-edge-list" => Ok(FillContent::WeightedEdgeList {
            edge_count: length_spec(field(draft, "length")?, &draft.variables),
            weight_name: field(draft, "weight_name")?.to_owned(),
            weight_type: var_type(field(draft, "type")?)?,
        }),
        "query-list" => Ok(FillContent::QueryList {
            query_count: length_spec(field(draft, "length")?, &draft.variables),
        }),
        "multi-testcase" => Ok(FillContent::MultiTestCaseTemplate {
            count: length_spec(field(draft, "length")?, &draft.variables),
        }),
        other => Err(format!("unsupported candidate: {other}")),
    }
}

fn field<'a>(draft: &'a HotspotDraft, name: &str) -> Result<&'a str, String> {
    draft
        .fields
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{name} is required"))
}

fn required_option<'a>(value: &'a Option<String>, name: &str) -> Result<&'a str, String> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{name} is required"))
}

fn var_type(value: &str) -> Result<VarType, String> {
    match value {
        "number" | "Int" => Ok(VarType::Int),
        "string" | "Str" => Ok(VarType::Str),
        "char" | "Char" => Ok(VarType::Char),
        other => Err(format!("unsupported variable type: {other}")),
    }
}

fn length_spec(value: &str, variables: &[VariableCandidate]) -> LengthSpec {
    if let Some(variable) = variables.iter().find(|candidate| candidate.name == value) {
        return LengthSpec::RefVar(variable.node_id);
    }
    match value.parse::<usize>() {
        Ok(value) => LengthSpec::Fixed(value),
        Err(_) => LengthSpec::Expr(value.to_owned()),
    }
}
