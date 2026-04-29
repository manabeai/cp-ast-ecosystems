//! One-way conversion from `FullProjection` → `FullProjectionDto`.

use cp_ast_core::projection::types::{
    CompletedConstraint, CompletenessSummary, DraftConstraint, ExprCandidate, FullProjection,
    Hotspot, HotspotDirection, ProjectedConstraints, ProjectedNode, StructureLine,
};

use crate::dto::{
    CompletedConstraintDto, CompletenessSummaryDto, DraftConstraintDto, ExprCandidateDto,
    FullProjectionDto, HotspotDto, ProjectedConstraintsDto, ProjectedNodeDto, StructureLineDto,
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
    }
}

fn hotspot_to_dto(hs: &Hotspot) -> HotspotDto {
    HotspotDto {
        parent_id: hs.parent_id.value().to_string(),
        direction: hotspot_direction_str(hs.direction),
        candidates: hs.candidates.clone(),
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
        drafts: pc.drafts.iter().map(draft_constraint_to_dto).collect(),
        completed: pc
            .completed
            .iter()
            .map(completed_constraint_to_dto)
            .collect(),
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
