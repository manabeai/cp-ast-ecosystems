//! Full projection: the main "View" function in the TEA architecture.
//!
//! `project_full()` returns everything the frontend needs to render the editor UI:
//! nodes, hotspots, constraints (drafts + completed), available variables, and completeness.

use std::collections::HashSet;

use super::api::ProjectionAPI;
use super::projection_impl::make_label;
use super::types::{
    CompletedConstraint, ConstraintItem, ConstraintItemStatus, DraftConstraint, ExprCandidate,
    FullProjection, Hotspot, HotspotDirection, ProjectedConstraints, ProjectedNode, StructureLine,
};
use crate::constraint::{Constraint, ExpectedType, Expression};
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind, Reference};

const BELOW_CANDIDATES: &[&str] = &[
    "scalar",
    "array",
    "repeat",
    "grid-template",
    "edge-list",
    "weighted-edge-list",
    "query-list",
    "multi-testcase",
];

const RIGHT_CANDIDATES: &[&str] = &["scalar", "array"];

/// Produce the full UI projection from the current engine state.
#[must_use]
pub fn project_full(engine: &AstEngine) -> FullProjection {
    let nodes = engine.nodes();
    let structure_lines = generate_structure_lines(engine);
    let completeness = engine.completeness();
    let hotspots = generate_hotspots(engine);
    let constraints = generate_constraints(engine);
    let available_vars = collect_available_vars(engine);

    FullProjection {
        nodes,
        structure_lines,
        hotspots,
        constraints,
        available_vars,
        completeness,
    }
}

// ---------------------------------------------------------------------------
// Hotspot generation
// ---------------------------------------------------------------------------

fn generate_hotspots(engine: &AstEngine) -> Vec<Hotspot> {
    let mut hotspots = Vec::new();
    let below: Vec<String> = BELOW_CANDIDATES.iter().map(|s| (*s).to_owned()).collect();
    let right: Vec<String> = RIGHT_CANDIDATES.iter().map(|s| (*s).to_owned()).collect();

    for node in engine.structure.iter() {
        match node.kind() {
            NodeKind::Sequence { children } => {
                hotspots.push(Hotspot {
                    parent_id: node.id(),
                    direction: HotspotDirection::Below,
                    candidates: below.clone(),
                });
                push_right_for_last_inline_child(engine, children, &right, &mut hotspots);
            }
            NodeKind::Repeat { body, .. } => {
                let has_hole = body.iter().any(|&id| {
                    engine
                        .structure
                        .get(id)
                        .is_some_and(|n| matches!(n.kind(), NodeKind::Hole { .. }))
                });
                if has_hole {
                    hotspots.push(Hotspot {
                        parent_id: node.id(),
                        direction: HotspotDirection::Inside,
                        candidates: below.clone(),
                    });
                } else {
                    hotspots.push(Hotspot {
                        parent_id: node.id(),
                        direction: HotspotDirection::Below,
                        candidates: below.clone(),
                    });
                    push_right_for_last_inline_child(engine, body, &right, &mut hotspots);
                }
            }
            NodeKind::Choice { .. } => {
                hotspots.push(Hotspot {
                    parent_id: node.id(),
                    direction: HotspotDirection::Variant,
                    candidates: below.clone(),
                });
            }
            NodeKind::Tuple { elements } => {
                push_right_for_last_inline_child(engine, elements, &right, &mut hotspots);
            }
            _ => {}
        }
    }

    hotspots
}

fn push_right_for_last_inline_child(
    engine: &AstEngine,
    children: &[NodeId],
    candidates: &[String],
    hotspots: &mut Vec<Hotspot>,
) {
    let Some(child_id) = children.iter().rev().copied().find(|&id| {
        engine
            .structure
            .get(id)
            .is_some_and(|child| is_scalar_or_array(child.kind()))
    }) else {
        return;
    };

    hotspots.push(Hotspot {
        parent_id: child_id,
        direction: HotspotDirection::Right,
        candidates: candidates.to_vec(),
    });
}

fn generate_structure_lines(engine: &AstEngine) -> Vec<StructureLine> {
    let mut lines = Vec::new();
    let root_id = engine.structure.root();
    if let Some(root) = engine.structure.get(root_id) {
        match root.kind() {
            NodeKind::Sequence { children } => {
                for &child_id in children {
                    push_structure_lines(engine, child_id, 0, &mut lines);
                }
            }
            _ => push_structure_lines(engine, root_id, 0, &mut lines),
        }
    }
    lines
}

fn push_structure_lines(
    engine: &AstEngine,
    node_id: NodeId,
    depth: usize,
    lines: &mut Vec<StructureLine>,
) {
    let Some(node) = engine.structure.get(node_id) else {
        return;
    };

    match node.kind() {
        NodeKind::Tuple { elements } => {
            let nodes = elements
                .iter()
                .filter_map(|&id| projected_node(engine, id, depth))
                .collect::<Vec<_>>();
            if !nodes.is_empty() {
                lines.push(StructureLine { depth, nodes });
            }
        }
        NodeKind::Repeat { body, .. } => {
            if let Some(node) = projected_node(engine, node_id, depth) {
                lines.push(StructureLine {
                    depth,
                    nodes: vec![node],
                });
            }
            for &child_id in body {
                push_structure_lines(engine, child_id, depth + 1, lines);
            }
        }
        NodeKind::Sequence { children } => {
            for &child_id in children {
                push_structure_lines(engine, child_id, depth, lines);
            }
        }
        NodeKind::Section { header, body } => {
            if let Some(node) = projected_node(engine, node_id, depth) {
                lines.push(StructureLine {
                    depth,
                    nodes: vec![node],
                });
            }
            if let Some(header_id) = header {
                push_structure_lines(engine, *header_id, depth + 1, lines);
            }
            for &child_id in body {
                push_structure_lines(engine, child_id, depth + 1, lines);
            }
        }
        _ => {
            if let Some(node) = projected_node(engine, node_id, depth) {
                lines.push(StructureLine {
                    depth,
                    nodes: vec![node],
                });
            }
        }
    }
}

fn projected_node(engine: &AstEngine, node_id: NodeId, depth: usize) -> Option<ProjectedNode> {
    let node = engine.structure.get(node_id)?;
    Some(ProjectedNode {
        id: node_id,
        label: make_label(node.kind()),
        depth,
        is_hole: matches!(node.kind(), NodeKind::Hole { .. }),
    })
}

fn is_scalar_or_array(kind: &NodeKind) -> bool {
    matches!(kind, NodeKind::Scalar { .. } | NodeKind::Array { .. })
}

// ---------------------------------------------------------------------------
// Constraint projection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotKind {
    Range,
    CharSet,
    StringLength,
    Other,
}

#[derive(Debug, Clone)]
struct CompletedRow {
    node_id: NodeId,
    node_name: String,
    kind: SlotKind,
    constraint: CompletedConstraint,
}

#[allow(clippy::too_many_lines)]
fn generate_constraints(engine: &AstEngine) -> ProjectedConstraints {
    let mut completed = Vec::new();
    let mut completed_rows = Vec::new();
    let mut used_completed = HashSet::new();

    for (cid, constraint) in engine.constraints.iter() {
        // TypeDecl constraints are internal bookkeeping — don't expose to the UI.
        if matches!(constraint, Constraint::TypeDecl { .. }) {
            // Still fall through to track nodes_with_range / nodes_with_charset
            // (TypeDecl won't match those arms, so just continue)
            continue;
        }

        let completed_constraint = CompletedConstraint {
            index: completed.len(),
            constraint_id: format!("c{}", cid.value()),
            display: format_constraint_display(constraint, engine),
        };

        if let Some(node_id) = constraint_target_node_id(constraint) {
            let kind = constraint_slot_kind(constraint);
            completed_rows.push(CompletedRow {
                node_id,
                node_name: ref_to_name(&Reference::VariableRef(node_id), engine),
                kind,
                constraint: completed_constraint.clone(),
            });
        }
        completed.push(completed_constraint);
    }

    let mut drafts = Vec::new();
    let mut items = Vec::new();
    for node in engine.structure.iter() {
        let node_id = node.id();
        match (node.kind(), expected_type(engine, node_id)) {
            (
                NodeKind::Scalar { name } | NodeKind::Array { name, .. },
                Some(ExpectedType::Int) | None,
            ) => {
                push_constraint_slot(
                    &completed_rows,
                    &mut used_completed,
                    &mut drafts,
                    &mut items,
                    node_id,
                    name.as_str(),
                    SlotKind::Range,
                    "? ≤ {name} ≤ ?",
                    "Range",
                );
            }
            (
                NodeKind::Scalar { name }
                | NodeKind::Array { name, .. }
                | NodeKind::Matrix { name, .. },
                Some(ExpectedType::Char),
            ) => {
                push_constraint_slot(
                    &completed_rows,
                    &mut used_completed,
                    &mut drafts,
                    &mut items,
                    node_id,
                    name.as_str(),
                    SlotKind::CharSet,
                    "charset({name}) = ?",
                    "CharSet",
                );
            }
            (
                NodeKind::Scalar { name }
                | NodeKind::Array { name, .. }
                | NodeKind::Matrix { name, .. },
                Some(ExpectedType::Str),
            ) => {
                push_constraint_slot(
                    &completed_rows,
                    &mut used_completed,
                    &mut drafts,
                    &mut items,
                    node_id,
                    name.as_str(),
                    SlotKind::CharSet,
                    "charset({name}) = ?",
                    "CharSet",
                );
                if matches!(node.kind(), NodeKind::Scalar { .. }) {
                    push_constraint_slot(
                        &completed_rows,
                        &mut used_completed,
                        &mut drafts,
                        &mut items,
                        node_id,
                        name.as_str(),
                        SlotKind::StringLength,
                        "? ≤ len({name}) ≤ ?",
                        "StringLength",
                    );
                }
            }
            _ => {}
        }
    }

    for row in &completed_rows {
        if used_completed.contains(&row.constraint.index) {
            continue;
        }
        items.push(ConstraintItem {
            index: items.len(),
            status: ConstraintItemStatus::Completed,
            target_id: row.node_id,
            target_name: row.node_name.clone(),
            display: row.constraint.display.clone(),
            template: None,
            constraint_id: Some(row.constraint.constraint_id.clone()),
            draft_index: None,
            completed_index: Some(row.constraint.index),
        });
    }

    ProjectedConstraints {
        items,
        drafts,
        completed,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_constraint_slot(
    completed_rows: &[CompletedRow],
    used_completed: &mut HashSet<usize>,
    drafts: &mut Vec<DraftConstraint>,
    items: &mut Vec<ConstraintItem>,
    node_id: NodeId,
    node_name: &str,
    kind: SlotKind,
    draft_display_template: &str,
    template: &str,
) {
    if let Some(row) = completed_rows
        .iter()
        .find(|row| row.node_id == node_id && row.kind == kind)
    {
        let completed = &row.constraint;
        used_completed.insert(completed.index);
        items.push(ConstraintItem {
            index: items.len(),
            status: ConstraintItemStatus::Completed,
            target_id: node_id,
            target_name: node_name.to_owned(),
            display: completed.display.clone(),
            template: None,
            constraint_id: Some(completed.constraint_id.clone()),
            draft_index: None,
            completed_index: Some(completed.index),
        });
        return;
    }

    let draft = DraftConstraint {
        index: drafts.len(),
        target_id: node_id,
        target_name: node_name.to_owned(),
        display: draft_display_template.replace("{name}", node_name),
        template: template.to_owned(),
    };
    drafts.push(draft.clone());
    items.push(ConstraintItem {
        index: items.len(),
        status: ConstraintItemStatus::Draft,
        target_id: node_id,
        target_name: node_name.to_owned(),
        display: draft.display,
        template: Some(draft.template),
        constraint_id: None,
        draft_index: Some(draft.index),
        completed_index: None,
    });
}

fn expected_type(engine: &AstEngine, node_id: NodeId) -> Option<ExpectedType> {
    let ids = engine.constraints.for_node(node_id);
    for cid in &ids {
        if let Some(Constraint::TypeDecl { expected, .. }) = engine.constraints.get(*cid) {
            return Some(expected.clone());
        }
    }
    None
}

fn constraint_target_node_id(constraint: &Constraint) -> Option<NodeId> {
    match constraint {
        Constraint::Range { target, .. }
        | Constraint::TypeDecl { target, .. }
        | Constraint::LengthRelation { target, .. }
        | Constraint::Distinct {
            elements: target, ..
        }
        | Constraint::Property { target, .. }
        | Constraint::SumBound {
            variable: target, ..
        }
        | Constraint::Sorted {
            elements: target, ..
        }
        | Constraint::CharSet { target, .. }
        | Constraint::StringLength { target, .. }
        | Constraint::RenderHint { target, .. } => ref_to_node_id(target),
        Constraint::Relation { .. } | Constraint::Guarantee { .. } => None,
    }
}

fn constraint_slot_kind(constraint: &Constraint) -> SlotKind {
    match constraint {
        Constraint::Range { .. } => SlotKind::Range,
        Constraint::CharSet { .. } => SlotKind::CharSet,
        Constraint::StringLength { .. } => SlotKind::StringLength,
        _ => SlotKind::Other,
    }
}

// ---------------------------------------------------------------------------
// Available variables
// ---------------------------------------------------------------------------

fn collect_available_vars(engine: &AstEngine) -> Vec<ExprCandidate> {
    let mut vars = Vec::new();
    for node in engine.structure.iter() {
        let name = match node.kind() {
            NodeKind::Scalar { name }
            | NodeKind::Array { name, .. }
            | NodeKind::Matrix { name, .. } => Some(name),
            _ => None,
        };
        if let Some(name) = name {
            vars.push(ExprCandidate {
                name: name.as_str().to_owned(),
                node_id: node.id(),
            });
        }
    }
    vars
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

fn format_constraint_display(constraint: &Constraint, engine: &AstEngine) -> String {
    match constraint {
        Constraint::Range {
            target,
            lower,
            upper,
        } => {
            let name = ref_to_name(target, engine);
            format!(
                "{} ≤ {} ≤ {}",
                format_expr_simple(lower),
                name,
                format_expr_simple(upper)
            )
        }
        Constraint::TypeDecl { target, expected } => {
            let name = ref_to_name(target, engine);
            let type_str = match expected {
                ExpectedType::Int => "int",
                ExpectedType::Str => "string",
                ExpectedType::Char => "char",
            };
            format!("{name}: {type_str}")
        }
        Constraint::Property { tag, .. } => format!("{tag:?}"),
        Constraint::SumBound { variable, upper } => {
            let name = ref_to_name(variable, engine);
            format!("Σ{} ≤ {}", name, format_expr_simple(upper))
        }
        Constraint::CharSet { target, charset } => {
            let name = ref_to_name(target, engine);
            format!("charset({name}) = {charset}")
        }
        Constraint::LengthRelation { target, length, .. } => {
            let name = ref_to_name(target, engine);
            format!("len({name}) = {}", format_expr_simple(length))
        }
        Constraint::Relation { lhs, op, rhs } => {
            format!(
                "{} {op:?} {}",
                format_expr_simple(lhs),
                format_expr_simple(rhs)
            )
        }
        Constraint::Distinct { elements, unit } => {
            let name = ref_to_name(elements, engine);
            format!("distinct({name}, {unit:?})")
        }
        Constraint::Sorted { elements, order } => {
            let name = ref_to_name(elements, engine);
            format!("sorted({name}, {order:?})")
        }
        Constraint::Guarantee { description, .. } => format!("guarantee: {description}"),
        Constraint::StringLength { target, min, max } => {
            let name = ref_to_name(target, engine);
            format!(
                "{} ≤ len({name}) ≤ {}",
                format_expr_simple(min),
                format_expr_simple(max)
            )
        }
        Constraint::RenderHint { hint, .. } => format!("hint: {hint:?}"),
    }
}

fn ref_to_name(reference: &Reference, engine: &AstEngine) -> String {
    match reference {
        Reference::VariableRef(node_id) => engine.structure.get(*node_id).map_or_else(
            || format!("?node({node_id:?})"),
            |node| match node.kind() {
                NodeKind::Scalar { name }
                | NodeKind::Array { name, .. }
                | NodeKind::Matrix { name, .. } => name.as_str().to_owned(),
                other => format!("{other:?}"),
            },
        ),
        Reference::Unresolved(ident) => ident.as_str().to_owned(),
        Reference::IndexedRef { .. } => format!("{reference:?}"),
    }
}

fn ref_to_node_id(reference: &Reference) -> Option<NodeId> {
    match reference {
        Reference::VariableRef(id) | Reference::IndexedRef { target: id, .. } => Some(*id),
        Reference::Unresolved(_) => None,
    }
}

fn format_expr_simple(expr: &Expression) -> String {
    match expr {
        Expression::Lit(n) => n.to_string(),
        Expression::Var(r) => match r {
            Reference::Unresolved(ident) => ident.as_str().to_owned(),
            Reference::VariableRef(id) => format!("var({id:?})"),
            Reference::IndexedRef { .. } => format!("{r:?}"),
        },
        Expression::BinOp { op, lhs, rhs } => {
            format!(
                "({} {op:?} {})",
                format_expr_simple(lhs),
                format_expr_simple(rhs)
            )
        }
        Expression::Pow { base, exp } => {
            format!(
                "({} ^ {})",
                format_expr_simple(base),
                format_expr_simple(exp)
            )
        }
        Expression::FnCall { name, args } => {
            let arg_strs: Vec<_> = args.iter().map(format_expr_simple).collect();
            format!("{}({})", name.as_str(), arg_strs.join(", "))
        }
    }
}
