use super::constraint_ops::parse_expression;
use super::engine::AstEngine;
use super::error::OperationError;
use super::result::ApplyResult;
use super::types::SumBoundDef;
use crate::constraint::Constraint;
use crate::structure::{Ident, NodeKind, Reference};

impl AstEngine {
    /// Introduce multi-test-case wrapper around the current structure.
    ///
    /// This operation:
    /// 1. Creates a count variable with the specified name
    /// 2. Wraps the current root's children in a Repeat node
    /// 3. Sets root to a new Sequence with [`count_var`, `repeat_node`]
    /// 4. Optionally adds a `SumBound` constraint
    ///
    /// # Errors
    /// Returns `OperationError` if:
    /// - A Repeat node already exists in the structure (multi-test-case already exists)
    pub(crate) fn introduce_multi_test_case(
        &mut self,
        count_var_name: &str,
        sum_bound: Option<&SumBoundDef>,
    ) -> Result<ApplyResult, OperationError> {
        // 1. Check if a Repeat node already exists (indicating multi-test-case wrapper exists)
        for node in self.structure.iter() {
            if matches!(node.kind(), NodeKind::Repeat { .. }) {
                return Err(OperationError::InvalidOperation {
                    action: "IntroduceMultiTestCase".to_owned(),
                    reason: "Multi-test-case wrapper already exists".to_owned(),
                });
            }
        }

        // 2. Create count variable as a Scalar node
        let count_scalar_id = self.structure.add_node(NodeKind::Scalar {
            name: Ident::new(count_var_name),
        });

        // 3. Get current root's children (root is always a Sequence)
        let current_root = self.structure.root();
        let current_children = if let Some(root_node) = self.structure.get(current_root) {
            if let NodeKind::Sequence { children } = root_node.kind() {
                children.clone()
            } else {
                // Fallback: wrap current root as a single child
                vec![current_root]
            }
        } else {
            vec![]
        };

        // 4. Create a new Repeat node with the current children as its body
        let repeat_id = self.structure.add_node(NodeKind::Repeat {
            count: Reference::VariableRef(count_scalar_id),
            body: current_children,
        });

        // 5. Set root to a new Sequence with [count_scalar_id, repeat_id]
        if let Some(root_node) = self.structure.get_mut(current_root) {
            root_node.set_kind(NodeKind::Sequence {
                children: vec![count_scalar_id, repeat_id],
            });
        }

        let created_nodes = vec![count_scalar_id, repeat_id];
        let mut created_constraints = vec![];

        // 6. Optionally add SumBound constraint
        if let Some(sum_bound_def) = sum_bound {
            let constraint = Constraint::SumBound {
                variable: Reference::Unresolved(Ident::new(&sum_bound_def.bound_var)),
                upper: parse_expression(&sum_bound_def.upper),
            };
            let cid = self.constraints.add(None, constraint);
            created_constraints.push(cid);
        }

        Ok(ApplyResult {
            created_nodes,
            removed_nodes: vec![],
            created_constraints,
            affected_constraints: vec![],
        })
    }
}
