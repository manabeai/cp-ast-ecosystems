use super::engine::AstEngine;
use super::error::OperationError;
use super::result::ApplyResult;
use super::types::{FillContent, LengthSpec, VarType};
use crate::constraint::{Constraint, ExpectedType};
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
                let length_ref = length_spec_to_reference(length);
                NodeKind::Array {
                    name: Ident::new(name),
                    length: length_ref,
                }
            }
            FillContent::Grid {
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
        }
    }
}

fn var_type_to_expected(fill: &FillContent) -> Option<ExpectedType> {
    match fill {
        FillContent::Scalar { typ, .. } | FillContent::OutputSingleValue { typ, .. } => {
            Some(var_type_to_expected_type(typ))
        }
        FillContent::OutputYesNo => Some(ExpectedType::Str),
        _ => None,
    }
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
