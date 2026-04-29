//! One-way conversion from `FullProjection` → `FullProjectionDto`.

use cp_ast_core::constraint::CharSetSpec;
use cp_ast_core::projection::types::{
    CandidateField, CompletedConstraint, CompletenessSummary, ConstraintEditProjection,
    ConstraintItem, ConstraintItemStatus, DraftConstraint, ExprCandidate, FullProjection,
    HoleCandidateDetail, Hotspot, HotspotAction, HotspotActionKind, HotspotDirection,
    NodeEditProjection, ProjectedConstraints, ProjectedNode, StructureLine,
};

use crate::dto::{
    CandidateFieldDto, CharSetSpecDto, CompletedConstraintDto, CompletenessSummaryDto,
    ConstraintEditProjectionDto, ConstraintItemDto, DraftConstraintDto, ExprCandidateDto,
    FullProjectionDto, HoleCandidateDetailDto, HotspotActionDto, HotspotDto, NodeEditProjectionDto,
    ProjectedConstraintsDto, ProjectedNodeDto, StructureLineDto,
};
use crate::error::ConversionError;

/// Serialize a [`FullProjection`] to a JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_projection(proj: &FullProjection) -> Result<String, ConversionError> {
    let dto = projection_to_dto(proj);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Convert a [`FullProjection`] to its DTO representation.
#[must_use]
pub fn projection_to_dto(proj: &FullProjection) -> FullProjectionDto {
    FullProjectionDto {
        nodes: proj.nodes.iter().map(projected_node_to_dto).collect(),
        structure_lines: proj
            .structure_lines
            .iter()
            .map(structure_line_to_dto)
            .collect(),
        hotspots: proj.hotspots.iter().map(hotspot_to_dto).collect(),
        constraints: projected_constraints_to_dto(&proj.constraints),
        available_vars: proj
            .available_vars
            .iter()
            .map(expr_candidate_to_dto)
            .collect(),
        completeness: completeness_to_dto(&proj.completeness),
    }
}

fn structure_line_to_dto(line: &StructureLine) -> StructureLineDto {
    StructureLineDto {
        depth: line.depth,
        nodes: line.nodes.iter().map(projected_node_to_dto).collect(),
    }
}

fn projected_node_to_dto(node: &ProjectedNode) -> ProjectedNodeDto {
    ProjectedNodeDto {
        id: node.id.value().to_string(),
        label: node.label.clone(),
        depth: node.depth,
        is_hole: node.is_hole,
        edit: node.edit.as_ref().map(node_edit_to_dto),
    }
}

fn node_edit_to_dto(edit: &NodeEditProjection) -> NodeEditProjectionDto {
    NodeEditProjectionDto {
        kind: edit.kind.clone(),
        name: edit.name.clone(),
        value_type: edit.value_type.clone(),
        length_expr: edit.length_expr.clone(),
        allowed_kinds: edit.allowed_kinds.clone(),
        allowed_types: edit.allowed_types.clone(),
    }
}

fn hotspot_to_dto(hs: &Hotspot) -> HotspotDto {
    HotspotDto {
        parent_id: hs.parent_id.value().to_string(),
        direction: hotspot_direction_str(hs.direction),
        candidates: hs.candidates.clone(),
        candidate_details: hs
            .candidate_details
            .iter()
            .map(candidate_detail_to_dto)
            .collect(),
        action: hotspot_action_to_dto(&hs.action),
    }
}

fn hotspot_action_to_dto(action: &HotspotAction) -> HotspotActionDto {
    HotspotActionDto {
        kind: hotspot_action_kind_str(action.kind),
        target_id: action.target_id.value().to_string(),
        slot_name: action.slot_name.clone(),
    }
}

fn hotspot_action_kind_str(kind: HotspotActionKind) -> String {
    match kind {
        HotspotActionKind::AddSlotElement => "add_slot_element",
        HotspotActionKind::AddSibling => "add_sibling",
        HotspotActionKind::FillHole => "fill_hole",
        HotspotActionKind::AddChoiceVariant => "add_choice_variant",
    }
    .to_owned()
}

fn candidate_detail_to_dto(candidate: &HoleCandidateDetail) -> HoleCandidateDetailDto {
    HoleCandidateDetailDto {
        kind: candidate.kind.clone(),
        label: candidate.label.clone(),
        fields: candidate
            .fields
            .iter()
            .map(candidate_field_to_dto)
            .collect(),
    }
}

fn candidate_field_to_dto(field: &CandidateField) -> CandidateFieldDto {
    CandidateFieldDto {
        name: field.name.clone(),
        field_type: field.field_type.clone(),
        label: field.label.clone(),
        required: field.required,
        options: field.options.clone(),
        default_value: field.default_value.clone(),
    }
}

fn hotspot_direction_str(d: HotspotDirection) -> String {
    match d {
        HotspotDirection::Below => "below".to_owned(),
        HotspotDirection::Right => "right".to_owned(),
        HotspotDirection::Inside => "inside".to_owned(),
        HotspotDirection::Variant => "variant".to_owned(),
    }
}

fn projected_constraints_to_dto(pc: &ProjectedConstraints) -> ProjectedConstraintsDto {
    ProjectedConstraintsDto {
        items: pc.items.iter().map(constraint_item_to_dto).collect(),
        drafts: pc.drafts.iter().map(draft_constraint_to_dto).collect(),
        completed: pc
            .completed
            .iter()
            .map(completed_constraint_to_dto)
            .collect(),
    }
}

fn constraint_item_to_dto(item: &ConstraintItem) -> ConstraintItemDto {
    ConstraintItemDto {
        index: item.index,
        status: match item.status {
            ConstraintItemStatus::Draft => "draft",
            ConstraintItemStatus::Completed => "completed",
        }
        .to_owned(),
        target_id: item.target_id.value().to_string(),
        target_name: item.target_name.clone(),
        display: item.display.clone(),
        template: item.template.clone(),
        constraint_id: item.constraint_id.clone(),
        draft_index: item.draft_index,
        completed_index: item.completed_index,
        edit: item.edit.as_ref().map(constraint_edit_to_dto),
    }
}

fn constraint_edit_to_dto(edit: &ConstraintEditProjection) -> ConstraintEditProjectionDto {
    match edit {
        ConstraintEditProjection::Range {
            lower,
            upper,
            constraint_id,
        } => ConstraintEditProjectionDto::Range {
            lower: lower.clone(),
            upper: upper.clone(),
            constraint_id: constraint_id.clone(),
        },
        ConstraintEditProjection::CharSet {
            charset,
            constraint_id,
        } => ConstraintEditProjectionDto::CharSet {
            charset: charset_spec_to_dto(charset),
            constraint_id: constraint_id.clone(),
        },
        ConstraintEditProjection::StringLength {
            min,
            max,
            constraint_id,
        } => ConstraintEditProjectionDto::StringLength {
            min: min.clone(),
            max: max.clone(),
            constraint_id: constraint_id.clone(),
        },
    }
}

fn draft_constraint_to_dto(dc: &DraftConstraint) -> DraftConstraintDto {
    DraftConstraintDto {
        index: dc.index,
        target_id: dc.target_id.value().to_string(),
        target_name: dc.target_name.clone(),
        display: dc.display.clone(),
        template: dc.template.clone(),
    }
}

fn completed_constraint_to_dto(cc: &CompletedConstraint) -> CompletedConstraintDto {
    CompletedConstraintDto {
        index: cc.index,
        constraint_id: cc.constraint_id.clone(),
        display: cc.display.clone(),
    }
}

fn expr_candidate_to_dto(ec: &ExprCandidate) -> ExprCandidateDto {
    ExprCandidateDto {
        name: ec.name.clone(),
        node_id: ec.node_id.value().to_string(),
        value_type: ec.value_type.clone(),
        node_kind: ec.node_kind.clone(),
    }
}

fn charset_spec_to_dto(charset: &CharSetSpec) -> CharSetSpecDto {
    match charset {
        CharSetSpec::LowerAlpha => CharSetSpecDto::LowerAlpha,
        CharSetSpec::UpperAlpha => CharSetSpecDto::UpperAlpha,
        CharSetSpec::Alpha => CharSetSpecDto::Alpha,
        CharSetSpec::Digit => CharSetSpecDto::Digit,
        CharSetSpec::AlphaNumeric => CharSetSpecDto::AlphaNumeric,
        CharSetSpec::Custom(chars) => CharSetSpecDto::Custom {
            chars: chars.clone(),
        },
        CharSetSpec::Range(from, to) => CharSetSpecDto::Range {
            from: *from,
            to: *to,
        },
    }
}

fn completeness_to_dto(cs: &CompletenessSummary) -> CompletenessSummaryDto {
    CompletenessSummaryDto {
        total_holes: cs.total_holes,
        filled_slots: cs.filled_slots,
        unsatisfied_constraints: cs.unsatisfied_constraints,
        is_complete: cs.is_complete,
    }
}
