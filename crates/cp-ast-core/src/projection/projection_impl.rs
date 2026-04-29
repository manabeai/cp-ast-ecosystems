use super::api::ProjectionAPI;
use super::types::{
    AvailableAction, CandidateKind, CompletenessSummary, NodeDetail, NodeEditProjection,
    NotEditableReason, ProjectedNode, SlotEntry,
};
use crate::constraint::{ArithOp, Constraint, ExpectedType, Expression};
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind, Reference};

impl ProjectionAPI for AstEngine {
    fn nodes(&self) -> Vec<ProjectedNode> {
        let mut result = Vec::new();
        let root_id = self.structure.root();
        let mut stack = Vec::new();

        if let Some(root) = self.structure.get(root_id) {
            if is_hidden_root(root.kind()) {
                for &child_id in children_of(root.kind()).iter().rev() {
                    stack.push((child_id, 0));
                }
            } else {
                stack.push((root_id, 0));
            }
        }

        while let Some((node_id, depth)) = stack.pop() {
            if let Some(node) = self.structure.get(node_id) {
                let label = make_label(node.kind());
                let is_hole = matches!(node.kind(), NodeKind::Hole { .. });

                result.push(ProjectedNode {
                    id: node_id,
                    label,
                    depth,
                    is_hole,
                    edit: node_edit_projection(self, node_id),
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

fn node_edit_projection(engine: &AstEngine, node_id: NodeId) -> Option<NodeEditProjection> {
    let node = engine.structure.get(node_id)?;
    let value_type = expected_type(engine, node_id)
        .map_or("number", |typ| match typ {
            ExpectedType::Int => "number",
            ExpectedType::Str => "string",
            ExpectedType::Char => "char",
        })
        .to_owned();

    match node.kind() {
        NodeKind::Scalar { name } => Some(NodeEditProjection {
            kind: "scalar".to_owned(),
            name: name.as_str().to_owned(),
            value_type,
            length_expr: None,
            allowed_kinds: vec!["scalar".to_owned(), "array".to_owned()],
            allowed_types: vec!["number".to_owned(), "char".to_owned(), "string".to_owned()],
        }),
        NodeKind::Array { name, length } => Some(NodeEditProjection {
            kind: "array".to_owned(),
            name: name.as_str().to_owned(),
            value_type,
            length_expr: Some(format_expr_with_names(length, engine)),
            allowed_kinds: vec!["scalar".to_owned(), "array".to_owned()],
            allowed_types: vec!["number".to_owned(), "char".to_owned(), "string".to_owned()],
        }),
        _ => None,
    }
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

fn format_expr_with_names(expr: &Expression, engine: &AstEngine) -> String {
    match expr {
        Expression::Lit(n) => n.to_string(),
        Expression::Var(r) => ref_to_name(r, engine),
        Expression::BinOp { op, lhs, rhs } => {
            let symbol = match op {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "*",
                ArithOp::Div => "/",
            };
            format!(
                "{}{}{}",
                format_expr_with_names(lhs, engine),
                symbol,
                format_expr_with_names(rhs, engine)
            )
        }
        Expression::Pow { base, exp } => {
            format!(
                "{}^{}",
                format_expr_with_names(base, engine),
                format_expr_with_names(exp, engine)
            )
        }
        Expression::FnCall { name, args } => {
            let arg_strs: Vec<_> = args
                .iter()
                .map(|arg| format_expr_with_names(arg, engine))
                .collect();
            format!("{}({})", name.as_str(), arg_strs.join(","))
        }
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

/// Generate display label for a node kind.
pub(super) fn make_label(kind: &NodeKind) -> String {
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

fn is_hidden_root(kind: &NodeKind) -> bool {
    matches!(kind, NodeKind::Sequence { .. })
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
