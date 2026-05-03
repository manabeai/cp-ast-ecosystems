use super::engine::AstEngine;
use super::error::OperationError;
use super::result::ApplyResult;
use super::types::{FillContent, LengthSpec, VarType};
use crate::constraint::{Constraint, ExpectedType, Expression};
use crate::structure::{Ident, NodeId, NodeKind, Reference};

impl AstEngine {
    /// Fill a hole with concrete content.
    ///
    /// # Errors
    /// Returns `OperationError` if the target node doesn't exist or is not a hole.
    pub(crate) fn fill_hole(
        &mut self,
        target: NodeId,
        fill: &FillContent,
    ) -> Result<ApplyResult, OperationError> {
        // 1. Verify target exists
        let node = self
            .structure
            .get(target)
            .ok_or(OperationError::NodeNotFound { node: target })?;

        // 2. Verify it's a Hole
        if !matches!(node.kind(), NodeKind::Hole { .. }) {
            return Err(OperationError::InvalidOperation {
                action: "FillHole".to_owned(),
                reason: format!("Node {target:?} is not a Hole"),
            });
        }

        // 3. Expand FillContent to NodeKind + possibly child nodes
        let mut created_nodes = Vec::new();
        let new_kind = self.expand_fill_content(fill, &mut created_nodes);

        // 4. Replace the hole with the new kind
        if let Some(node) = self.structure.get_mut(target) {
            node.set_kind(new_kind);
        }

        // 4.5. Resolve Unresolved variable references in structure expressions
        self.resolve_structure_references(target);
        for &child_id in &created_nodes {
            self.resolve_structure_references(child_id);
        }

        // 5. Auto-add TypeDecl constraint if applicable
        let mut created_constraints = Vec::new();
        if let Some(expected_type) = var_type_to_expected(fill) {
            let cid = self.constraints.add(
                Some(target),
                Constraint::TypeDecl {
                    target: Reference::VariableRef(target),
                    expected: expected_type,
                },
            );
            created_constraints.push(cid);
        }

        Ok(ApplyResult {
            created_nodes,
            removed_nodes: vec![],
            created_constraints,
            affected_constraints: vec![],
        })
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn expand_fill_content(
        &mut self,
        fill: &FillContent,
        created: &mut Vec<NodeId>,
    ) -> NodeKind {
        match fill {
            FillContent::Scalar { name, .. } => NodeKind::Scalar {
                name: Ident::new(name),
            },
            FillContent::Array { name, length, .. } => {
                let length_expr = length_spec_to_expression(length);
                NodeKind::Array {
                    name: Ident::new(name),
                    length: length_expr,
                }
            }
            FillContent::Repeat { count } | FillContent::MultiTestCaseTemplate { count } => {
                let hole = self.structure.add_node(NodeKind::Hole {
                    expected_kind: None,
                });
                created.push(hole);
                NodeKind::Repeat {
                    count: length_spec_to_expression(count),
                    index_var: None,
                    body: vec![hole],
                }
            }
            FillContent::Grid {
                name, rows, cols, ..
            }
            | FillContent::GridTemplate {
                name, rows, cols, ..
            } => {
                let rows_ref = length_spec_to_reference(rows);
                let cols_ref = length_spec_to_reference(cols);
                NodeKind::Matrix {
                    name: Ident::new(name),
                    rows: rows_ref,
                    cols: cols_ref,
                }
            }
            FillContent::Section { label: _ } => {
                // Create a hole for the body
                let body_hole = self.structure.add_node(NodeKind::Hole {
                    expected_kind: None,
                });
                created.push(body_hole);
                NodeKind::Section {
                    header: None,
                    body: vec![body_hole],
                }
            }
            FillContent::OutputSingleValue { .. } | FillContent::OutputYesNo => NodeKind::Scalar {
                name: Ident::new("ans"),
            },
            FillContent::EdgeList { edge_count } => {
                let u = self.structure.add_node(NodeKind::Scalar {
                    name: Ident::new("u"),
                });
                let v = self.structure.add_node(NodeKind::Scalar {
                    name: Ident::new("v"),
                });
                let tuple = self.structure.add_node(NodeKind::Tuple {
                    elements: vec![u, v],
                });
                created.push(u);
                created.push(v);
                created.push(tuple);
                NodeKind::Repeat {
                    count: length_spec_to_expression(edge_count),
                    index_var: None,
                    body: vec![tuple],
                }
            }
            FillContent::WeightedEdgeList {
                edge_count,
                weight_name,
                ..
            } => {
                let u = self.structure.add_node(NodeKind::Scalar {
                    name: Ident::new("u"),
                });
                let v = self.structure.add_node(NodeKind::Scalar {
                    name: Ident::new("v"),
                });
                let w = self.structure.add_node(NodeKind::Scalar {
                    name: Ident::new(weight_name),
                });
                let tuple = self.structure.add_node(NodeKind::Tuple {
                    elements: vec![u, v, w],
                });
                created.push(u);
                created.push(v);
                created.push(w);
                created.push(tuple);
                NodeKind::Repeat {
                    count: length_spec_to_expression(edge_count),
                    index_var: None,
                    body: vec![tuple],
                }
            }
            FillContent::QueryList { query_count } => {
                let choice = self.structure.add_node(NodeKind::Choice {
                    tag: Reference::Unresolved(Ident::new("type")),
                    variants: vec![],
                });
                created.push(choice);
                NodeKind::Repeat {
                    count: length_spec_to_expression(query_count),
                    index_var: None,
                    body: vec![choice],
                }
            }
        }
    }
}

fn var_type_to_expected(fill: &FillContent) -> Option<ExpectedType> {
    match fill {
        FillContent::Scalar { typ, .. }
        | FillContent::Array {
            element_type: typ, ..
        }
        | FillContent::OutputSingleValue { typ, .. } => Some(var_type_to_expected_type(typ)),
        FillContent::OutputYesNo => Some(ExpectedType::Str),
        FillContent::WeightedEdgeList { weight_type, .. } => {
            Some(var_type_to_expected_type(weight_type))
        }
        FillContent::GridTemplate { cell_type, .. } => Some(var_type_to_expected_type(cell_type)),
        _ => None,
    }
}

/// `pub(crate)` wrapper around `var_type_to_expected` for use by other operation modules.
pub(crate) fn var_type_to_expected_from_fill(fill: &FillContent) -> Option<ExpectedType> {
    var_type_to_expected(fill)
}

fn var_type_to_expected_type(vt: &VarType) -> ExpectedType {
    match vt {
        VarType::Int => ExpectedType::Int,
        VarType::Str => ExpectedType::Str,
        VarType::Char => ExpectedType::Char,
    }
}

fn length_spec_to_reference(spec: &LengthSpec) -> Reference {
    match spec {
        LengthSpec::Fixed(n) => Reference::Unresolved(Ident::new(&format!("{n}"))),
        LengthSpec::RefVar(id) => Reference::VariableRef(*id),
        LengthSpec::Expr(s) => Reference::Unresolved(Ident::new(s)),
    }
}

fn length_spec_to_expression(spec: &LengthSpec) -> Expression {
    match spec {
        #[allow(clippy::cast_possible_wrap)]
        LengthSpec::Fixed(n) => Expression::Lit(*n as i64),
        LengthSpec::RefVar(id) => Expression::Var(Reference::VariableRef(*id)),
        LengthSpec::Expr(s) => parse_length_expr(s),
    }
}

/// Parse a length expression string into an `Expression`.
///
/// Handles patterns like "N-1", "N+1", "2*N", or falls back to
/// a simple literal or unresolved variable reference.
pub(super) fn parse_length_expr(s: &str) -> Expression {
    crate::constraint::parse_expression_str(s)
}
