use super::api::ProjectionAPI;
use super::types::{
    AvailableAction, CandidateKind, CompletenessSummary, NodeDetail, NotEditableReason,
    ProjectedNode, SlotEntry,
};
use crate::constraint::{Constraint, Expression};
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind};

impl ProjectionAPI for AstEngine {
    fn nodes(&self) -> Vec<ProjectedNode> {
        let mut result = Vec::new();
        let mut stack = vec![(self.structure.root(), 0)]; // (node_id, depth)

        while let Some((node_id, depth)) = stack.pop() {
            if let Some(node) = self.structure.get(node_id) {
                let label = make_label(node.kind());
                let is_hole = matches!(node.kind(), NodeKind::Hole { .. });

                result.push(ProjectedNode {
                    id: node_id,
                    label,
                    depth,
                    is_hole,
                });

                // Add children in reverse order for DFS (stack is LIFO)
                let children = children_of(node.kind());
                for &child_id in children.iter().rev() {
                    stack.push((child_id, depth + 1));
                }
            }
        }

        result
    }

    fn children(&self, node: NodeId) -> Vec<SlotEntry> {
        if let Some(node) = self.structure.get(node) {
            slot_entries_for(node.kind())
        } else {
            Vec::new()
        }
    }

    fn inspect(&self, node: NodeId) -> Option<NodeDetail> {
        let node = self.structure.get(node)?;
        let kind_label = make_label(node.kind());

        let constraint_ids = self.constraints.for_node(node.id());
        let constraints = constraint_ids
            .iter()
            .filter_map(|&id| self.constraints.get(id))
            .map(format_constraint)
            .collect();

        Some(NodeDetail {
            id: node.id(),
            kind_label,
            constraints,
        })
    }

    fn hole_candidates(&self, hole: NodeId) -> Vec<CandidateKind> {
        if let Some(node) = self.structure.get(hole) {
            if matches!(node.kind(), NodeKind::Hole { .. }) {
                vec![
                    CandidateKind::IntroduceScalar {
                        suggested_names: vec!["N".into(), "M".into(), "K".into()],
                    },
                    CandidateKind::IntroduceArray {
                        suggested_names: vec!["A".into(), "B".into()],
                    },
                    CandidateKind::IntroduceMatrix,
                    CandidateKind::IntroduceSection,
                ]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    fn available_actions(&self) -> Vec<AvailableAction> {
        let mut actions = Vec::new();

        for node in self.structure.iter() {
            let node_id = node.id();

            match node.kind() {
                NodeKind::Hole { .. } => {
                    actions.push(AvailableAction {
                        target: node_id,
                        description: "Fill hole".to_owned(),
                    });
                }
                _ => {
                    // Non-hole, non-root nodes can potentially be replaced or removed
                    if node_id != self.structure.root() {
                        actions.push(AvailableAction {
                            target: node_id,
                            description: "Replace node".to_owned(),
                        });

                        // Only allow removal if no constraints
                        let constraints = self.constraints.for_node(node_id);
                        if constraints.is_empty() {
                            actions.push(AvailableAction {
                                target: node_id,
                                description: "Remove from parent".to_owned(),
                            });
                        }
                    }
                }
            }
        }

        actions
    }

    fn why_not_editable(&self, node: NodeId) -> Option<NotEditableReason> {
        if node == self.structure.root() {
            return Some(NotEditableReason::IsRoot);
        }

        let constraint_ids = self.constraints.for_node(node);
        if !constraint_ids.is_empty() {
            return Some(NotEditableReason::HasDependents {
                dependents: vec![node], // Simplified - just return the node itself
            });
        }

        None
    }

    fn completeness(&self) -> CompletenessSummary {
        let mut total_nodes: usize = 0;
        let mut total_holes: usize = 0;

        for node in self.structure.iter() {
            total_nodes += 1;
            if matches!(node.kind(), NodeKind::Hole { .. }) {
                total_holes += 1;
            }
        }

        let filled_slots = total_nodes.saturating_sub(total_holes);
        let unsatisfied_constraints = 0; // Future work - constraint satisfaction checking
        let is_complete = total_holes == 0;

        CompletenessSummary {
            total_holes,
            filled_slots,
            unsatisfied_constraints,
            is_complete,
        }
    }
}

/// Generate display label for a node kind.
fn make_label(kind: &NodeKind) -> String {
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

/// Get child node IDs for DFS traversal.
fn children_of(kind: &NodeKind) -> Vec<NodeId> {
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

/// Get named slot entries for a node.
fn slot_entries_for(kind: &NodeKind) -> Vec<SlotEntry> {
    match kind {
        NodeKind::Sequence { children } => children
            .iter()
            .map(|&child| SlotEntry {
                name: "children".to_owned(),
                child,
            })
            .collect(),
        NodeKind::Section { header, body } => {
            let mut entries = Vec::new();
            if let Some(header_id) = header {
                entries.push(SlotEntry {
                    name: "header".to_owned(),
                    child: *header_id,
                });
            }
            for &body_id in body {
                entries.push(SlotEntry {
                    name: "body".to_owned(),
                    child: body_id,
                });
            }
            entries
        }
        NodeKind::Repeat { body, .. } => body
            .iter()
            .map(|&child| SlotEntry {
                name: "body".to_owned(),
                child,
            })
            .collect(),
        NodeKind::Tuple { elements } => elements
            .iter()
            .map(|&child| SlotEntry {
                name: "elements".to_owned(),
                child,
            })
            .collect(),
        NodeKind::Choice { variants, .. } => variants
            .iter()
            .flat_map(|(_, v)| v.iter())
            .map(|&child| SlotEntry {
                name: "variant".to_owned(),
                child,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Format a constraint as a human-readable string.
fn format_constraint(c: &Constraint) -> String {
    match c {
        Constraint::Range { lower, upper, .. } => {
            format!("Range: {} to {}", format_expr(lower), format_expr(upper))
        }
        Constraint::TypeDecl { expected, .. } => format!("Type: {expected:?}"),
        Constraint::LengthRelation { length, .. } => format!("Length: {}", format_expr(length)),
        Constraint::Relation { lhs, op, rhs } => {
            format!(
                "Relation: {} {:?} {}",
                format_expr(lhs),
                op,
                format_expr(rhs)
            )
        }
        Constraint::Distinct { unit, .. } => format!("Distinct: {unit:?}"),
        Constraint::Property { tag, .. } => format!("Property: {tag:?}"),
        Constraint::SumBound { upper, .. } => format!("SumBound: {}", format_expr(upper)),
        Constraint::Sorted { order, .. } => format!("Sorted: {order:?}"),
        Constraint::Guarantee { description, .. } => format!("Guarantee: {description}"),
        Constraint::CharSet { charset, .. } => format!("CharSet: {charset:?}"),
        Constraint::StringLength { min, max, .. } => {
            format!("StringLength: {} to {}", format_expr(min), format_expr(max))
        }
        Constraint::RenderHint { hint, .. } => format!("RenderHint: {hint:?}"),
    }
}

/// Format an expression as a simple string.
fn format_expr(e: &Expression) -> String {
    match e {
        Expression::Lit(n) => n.to_string(),
        Expression::Var(r) => format!("{r:?}"),
        Expression::BinOp { op, lhs, rhs } => {
            format!("({} {:?} {})", format_expr(lhs), op, format_expr(rhs))
        }
        Expression::Pow { base, exp } => {
            format!("({} ^ {})", format_expr(base), format_expr(exp))
        }
        Expression::FnCall { name, args } => {
            let arg_strs: Vec<_> = args.iter().map(format_expr).collect();
            format!("{}({})", name.as_str(), arg_strs.join(", "))
        }
    }
}
