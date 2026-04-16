use super::types::{
    CompletenessInfo, ConstraintSummary, ConstraintTargetMenu, Diagnostic, DiagnosticLevel,
    ExprCandidateMenu, FullProjection, HoleCandidate, NodeDetailProjection, OutlineNode,
    ReferenceCandidate, SlotInfo,
};
use crate::constraint::{Constraint, Expression};
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind};

/// Project full AST outline for the editor UI.
#[must_use]
pub fn project_full(engine: &AstEngine) -> FullProjection {
    let mut outline = Vec::new();
    let mut total_holes = 0;
    let mut diagnostics = Vec::new();

    // DFS traversal to build outline
    let mut stack = vec![(engine.structure.root(), 0)]; // (node_id, depth)

    while let Some((node_id, depth)) = stack.pop() {
        if let Some(node) = engine.structure.get(node_id) {
            let label = make_node_label(node.kind());
            let kind_label = make_kind_label(node.kind());
            let is_hole = matches!(node.kind(), NodeKind::Hole { .. });
            let child_ids = extract_child_ids(node.kind());

            if is_hole {
                total_holes += 1;
                // Create diagnostic for unfilled hole
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Info,
                    message: "Unfilled hole".to_owned(),
                    node_id: Some(node_id),
                    constraint_id: None,
                });
            } else {
                // Check if node has any constraints
                let constraint_ids = engine.constraints.for_node(node_id);
                if constraint_ids.is_empty() && !matches!(node.kind(), NodeKind::Sequence { .. }) {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        message: "No constraints defined".to_owned(),
                        node_id: Some(node_id),
                        constraint_id: None,
                    });
                }
            }

            outline.push(OutlineNode {
                id: node_id,
                label,
                kind_label,
                depth,
                is_hole,
                child_ids: child_ids.clone(),
            });

            // Add children in reverse order for DFS (stack is LIFO)
            for &child_id in child_ids.iter().rev() {
                stack.push((child_id, depth + 1));
            }
        }
    }

    let is_complete = total_holes == 0;
    let missing_constraints = if is_complete {
        Vec::new()
    } else {
        vec!["Fill remaining holes".to_owned()]
    };

    FullProjection {
        outline,
        diagnostics,
        completeness: CompletenessInfo {
            total_holes,
            is_complete,
            missing_constraints,
        },
    }
}

/// Project detail for a specific node.
#[must_use]
pub fn project_node_detail(engine: &AstEngine, node: NodeId) -> Option<NodeDetailProjection> {
    let node_data = engine.structure.get(node)?;

    // Get expression slots based on NodeKind
    let slots = extract_slots(node_data.kind());

    // Get constraints that reference this node
    let constraint_ids = engine.constraints.for_node(node);
    let mut related_constraints = Vec::new();

    for constraint_id in constraint_ids {
        if let Some(constraint) = engine.constraints.get(constraint_id) {
            related_constraints.push(ConstraintSummary {
                id: constraint_id,
                label: format_constraint_summary(constraint),
                kind_label: get_constraint_kind_label(constraint),
            });
        }
    }

    Some(NodeDetailProjection {
        slots,
        related_constraints,
    })
}

/// Get candidates for filling a hole.
#[must_use]
pub fn get_hole_candidates(engine: &AstEngine, hole: NodeId) -> Vec<HoleCandidate> {
    // Verify the node is a hole
    if let Some(node) = engine.structure.get(hole) {
        if !matches!(node.kind(), NodeKind::Hole { .. }) {
            return Vec::new();
        }
    } else {
        return Vec::new();
    }

    vec![
        HoleCandidate {
            kind: "Scalar".to_owned(),
            suggested_names: vec!["N".to_owned(), "M".to_owned(), "K".to_owned()],
        },
        HoleCandidate {
            kind: "Array".to_owned(),
            suggested_names: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
        },
        HoleCandidate {
            kind: "Matrix".to_owned(),
            suggested_names: vec!["Grid".to_owned(), "Matrix".to_owned()],
        },
        HoleCandidate {
            kind: "Section".to_owned(),
            suggested_names: vec!["Section".to_owned()],
        },
    ]
}

/// Get expression candidates for a slot.
#[must_use]
pub fn get_expr_candidates(engine: &AstEngine) -> ExprCandidateMenu {
    let mut references = Vec::new();

    // Walk all structure nodes, collect Scalar nodes as reference candidates
    for node in engine.structure.iter() {
        if let NodeKind::Scalar { name } = node.kind() {
            references.push(ReferenceCandidate {
                node_id: node.id(),
                label: name.as_str().to_owned(),
            });
        }
    }

    let literals = vec![0, 1, 2, 100_000, 1_000_000_000];

    ExprCandidateMenu {
        references,
        literals,
    }
}

/// Get nodes that can be targets for new constraints.
#[must_use]
pub fn get_constraint_targets(engine: &AstEngine) -> ConstraintTargetMenu {
    let mut targets = Vec::new();

    // Walk all structure nodes, exclude Holes and structural nodes
    for node in engine.structure.iter() {
        match node.kind() {
            NodeKind::Hole { .. } | NodeKind::Section { .. } | NodeKind::Sequence { .. } => {
                // Skip these structural nodes
            }
            _ => {
                let label = make_node_label(node.kind());
                targets.push(ReferenceCandidate {
                    node_id: node.id(),
                    label,
                });
            }
        }
    }

    ConstraintTargetMenu { targets }
}

/// Helper function to create node labels (reusing existing logic).
fn make_node_label(kind: &NodeKind) -> String {
    match kind {
        NodeKind::Scalar { name } => name.as_str().to_owned(),
        NodeKind::Array { name, .. } => format!("{}[]", name.as_str()),
        NodeKind::Matrix { name, .. } => format!("{}[][]", name.as_str()),
        NodeKind::Tuple { .. } => "Tuple".to_owned(),
        NodeKind::Repeat { .. } => "Repeat".to_owned(),
        NodeKind::Section { .. } => "Section".to_owned(),
        NodeKind::Sequence { .. } => "Sequence".to_owned(),
        NodeKind::Choice { .. } => "Choice".to_owned(),
        NodeKind::Hole { .. } => "[ ]".to_owned(),
    }
}

/// Helper function to create kind labels.
fn make_kind_label(kind: &NodeKind) -> String {
    match kind {
        NodeKind::Scalar { .. } => "Scalar".to_owned(),
        NodeKind::Array { .. } => "Array".to_owned(),
        NodeKind::Matrix { .. } => "Matrix".to_owned(),
        NodeKind::Tuple { .. } => "Tuple".to_owned(),
        NodeKind::Repeat { .. } => "Repeat".to_owned(),
        NodeKind::Section { .. } => "Section".to_owned(),
        NodeKind::Sequence { .. } => "Sequence".to_owned(),
        NodeKind::Choice { .. } => "Choice".to_owned(),
        NodeKind::Hole { .. } => "Hole".to_owned(),
    }
}

/// Extract child node IDs from a `NodeKind`.
fn extract_child_ids(kind: &NodeKind) -> Vec<NodeId> {
    match kind {
        NodeKind::Sequence { children } => children.clone(),
        NodeKind::Section { header, body } => {
            header.iter().copied().chain(body.iter().copied()).collect()
        }
        NodeKind::Repeat { body, .. } => body.clone(),
        NodeKind::Tuple { elements } => elements.clone(),
        NodeKind::Choice { variants, .. } => variants
            .iter()
            .flat_map(|(_, v)| v.iter())
            .copied()
            .collect(),
        _ => Vec::new(),
    }
}

/// Extract expression slots from a node kind.
fn extract_slots(kind: &NodeKind) -> Vec<SlotInfo> {
    match kind {
        NodeKind::Array { length, .. } => {
            vec![SlotInfo {
                kind: crate::operation::SlotKind::ArrayLength,
                current_expr: Some(format_expression(length)),
                is_editable: true,
            }]
        }
        NodeKind::Repeat { count, .. } => {
            vec![SlotInfo {
                kind: crate::operation::SlotKind::RepeatCount,
                current_expr: Some(format_expression(count)),
                is_editable: true,
            }]
        }
        _ => Vec::new(),
    }
}

/// Format an expression as a string.
fn format_expression(expr: &Expression) -> String {
    match expr {
        Expression::Lit(n) => n.to_string(),
        Expression::Var(r) => format!("{r:?}"),
        Expression::BinOp { op, lhs, rhs } => {
            format!(
                "({} {:?} {})",
                format_expression(lhs),
                op,
                format_expression(rhs)
            )
        }
        Expression::Pow { base, exp } => {
            format!("({} ^ {})", format_expression(base), format_expression(exp))
        }
        Expression::FnCall { name, args } => {
            let arg_strs: Vec<_> = args.iter().map(format_expression).collect();
            format!("{}({})", name.as_str(), arg_strs.join(", "))
        }
    }
}

/// Create a summary label for a constraint.
fn format_constraint_summary(constraint: &Constraint) -> String {
    match constraint {
        Constraint::Range { lower, upper, .. } => {
            format!(
                "Range: {} to {}",
                format_expression(lower),
                format_expression(upper)
            )
        }
        Constraint::TypeDecl { expected, .. } => format!("Type: {expected:?}"),
        Constraint::LengthRelation { length, .. } => {
            format!("Length: {}", format_expression(length))
        }
        Constraint::Relation { lhs, op, rhs } => {
            format!(
                "Relation: {} {:?} {}",
                format_expression(lhs),
                op,
                format_expression(rhs)
            )
        }
        Constraint::Distinct { unit, .. } => format!("Distinct: {unit:?}"),
        Constraint::Property { tag, .. } => format!("Property: {tag:?}"),
        Constraint::SumBound { upper, .. } => format!("SumBound: {}", format_expression(upper)),
        Constraint::Sorted { order, .. } => format!("Sorted: {order:?}"),
        Constraint::Guarantee { description, .. } => format!("Guarantee: {description}"),
        Constraint::CharSet { charset, .. } => format!("CharSet: {charset:?}"),
        Constraint::StringLength { min, max, .. } => {
            format!(
                "StringLength: {} to {}",
                format_expression(min),
                format_expression(max)
            )
        }
        Constraint::RenderHint { hint, .. } => format!("RenderHint: {hint:?}"),
    }
}

/// Get the kind label for a constraint type.
fn get_constraint_kind_label(constraint: &Constraint) -> String {
    match constraint {
        Constraint::Range { .. } => "Range".to_owned(),
        Constraint::TypeDecl { .. } => "TypeDecl".to_owned(),
        Constraint::LengthRelation { .. } => "LengthRelation".to_owned(),
        Constraint::Relation { .. } => "Relation".to_owned(),
        Constraint::Distinct { .. } => "Distinct".to_owned(),
        Constraint::Property { .. } => "Property".to_owned(),
        Constraint::SumBound { .. } => "SumBound".to_owned(),
        Constraint::Sorted { .. } => "Sorted".to_owned(),
        Constraint::Guarantee { .. } => "Guarantee".to_owned(),
        Constraint::CharSet { .. } => "CharSet".to_owned(),
        Constraint::StringLength { .. } => "StringLength".to_owned(),
        Constraint::RenderHint { .. } => "RenderHint".to_owned(),
    }
}
