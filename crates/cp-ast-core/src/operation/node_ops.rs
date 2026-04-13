use super::engine::AstEngine;
use super::error::OperationError;
use super::result::ApplyResult;
use super::types::FillContent;
use crate::structure::{NodeId, NodeKind};

impl AstEngine {
    /// Replace an existing node with new content.
    ///
    /// # Errors
    /// Returns `OperationError` if:
    /// - Target node doesn't exist
    /// - Target node is a Hole (use `FillHole` instead)
    /// - Node has existing constraints (unsafe to replace)
    pub(crate) fn replace_node(
        &mut self,
        target: NodeId,
        replacement: &FillContent,
    ) -> Result<ApplyResult, OperationError> {
        // 1. Verify target exists
        let node = self
            .structure
            .get(target)
            .ok_or(OperationError::NodeNotFound { node: target })?;

        // 2. Verify it's NOT a Hole (replacing holes should use FillHole)
        if matches!(node.kind(), NodeKind::Hole { .. }) {
            return Err(OperationError::InvalidOperation {
                action: "ReplaceNode".to_owned(),
                reason: "Use FillHole for Hole nodes".to_owned(),
            });
        }

        // 3. Check for constraints referencing this node
        let existing_constraints = self.constraints.for_node(target);
        if !existing_constraints.is_empty() {
            return Err(OperationError::InvalidOperation {
                action: "ReplaceNode".to_owned(),
                reason: format!(
                    "Node has {} existing constraints; remove them first",
                    existing_constraints.len()
                ),
            });
        }

        // 4. Expand replacement FillContent using existing helper
        let mut created_nodes = Vec::new();
        let new_kind = self.expand_fill_content(replacement, &mut created_nodes);

        // 5. Replace the node's kind
        if let Some(node) = self.structure.get_mut(target) {
            node.set_kind(new_kind);
        }

        Ok(ApplyResult {
            created_nodes,
            removed_nodes: vec![],
            created_constraints: vec![],
            affected_constraints: vec![],
        })
    }

    /// Add an element to a parent node's slot.
    ///
    /// # Errors
    /// Returns `OperationError` if:
    /// - Parent node doesn't exist
    /// - Parent node doesn't have the specified slot
    pub(crate) fn add_slot_element(
        &mut self,
        parent: NodeId,
        slot_name: &str,
        element: &FillContent,
    ) -> Result<ApplyResult, OperationError> {
        // 1. Verify parent exists
        if !self.structure.contains(parent) {
            return Err(OperationError::NodeNotFound { node: parent });
        }

        // 2. Expand element FillContent first (needs &mut self)
        let mut created_nodes = Vec::new();
        let new_kind = self.expand_fill_content(element, &mut created_nodes);

        // 3. Create the new node
        let new_node_id = self.structure.add_node(new_kind);
        created_nodes.push(new_node_id);

        // 4. Get parent mutably and add to the correct slot
        let parent_node = self
            .structure
            .get_mut(parent)
            .ok_or(OperationError::NodeNotFound { node: parent })?;

        match parent_node.kind() {
            NodeKind::Sequence { children: _ } => {
                if slot_name == "children" {
                    if let NodeKind::Sequence { children } = parent_node.kind() {
                        // Clone children to avoid borrow checker issues
                        let mut new_children = children.clone();
                        new_children.push(new_node_id);
                        parent_node.set_kind(NodeKind::Sequence {
                            children: new_children,
                        });
                    }
                } else {
                    return Err(OperationError::InvalidOperation {
                        action: "AddSlotElement".to_owned(),
                        reason: format!("Sequence node does not have slot '{slot_name}'"),
                    });
                }
            }
            NodeKind::Section { header, body } => {
                if slot_name == "body" {
                    let mut new_body = body.clone();
                    new_body.push(new_node_id);
                    parent_node.set_kind(NodeKind::Section {
                        header: *header,
                        body: new_body,
                    });
                } else {
                    return Err(OperationError::InvalidOperation {
                        action: "AddSlotElement".to_owned(),
                        reason: format!("Section node does not have slot '{slot_name}'"),
                    });
                }
            }
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                if slot_name == "body" {
                    let mut new_body = body.clone();
                    new_body.push(new_node_id);
                    parent_node.set_kind(NodeKind::Repeat {
                        count: count.clone(),
                        index_var: index_var.clone(),
                        body: new_body,
                    });
                } else {
                    return Err(OperationError::InvalidOperation {
                        action: "AddSlotElement".to_owned(),
                        reason: format!("Repeat node does not have slot '{slot_name}'"),
                    });
                }
            }
            NodeKind::Tuple { elements } => {
                if slot_name == "elements" {
                    let mut new_elements = elements.clone();
                    new_elements.push(new_node_id);
                    parent_node.set_kind(NodeKind::Tuple {
                        elements: new_elements,
                    });
                } else {
                    return Err(OperationError::InvalidOperation {
                        action: "AddSlotElement".to_owned(),
                        reason: format!("Tuple node does not have slot '{slot_name}'"),
                    });
                }
            }
            _ => {
                return Err(OperationError::InvalidOperation {
                    action: "AddSlotElement".to_owned(),
                    reason: format!("Node does not have slot '{slot_name}'"),
                });
            }
        }

        Ok(ApplyResult {
            created_nodes,
            removed_nodes: vec![],
            created_constraints: vec![],
            affected_constraints: vec![],
        })
    }

    /// Remove an element from a parent node's slot.
    ///
    /// # Errors
    /// Returns `OperationError` if:
    /// - Parent or child node doesn't exist
    /// - Parent node doesn't have the specified slot
    /// - Child is not in the specified slot
    #[allow(clippy::too_many_lines)]
    pub(crate) fn remove_slot_element(
        &mut self,
        parent: NodeId,
        slot_name: &str,
        child: NodeId,
    ) -> Result<ApplyResult, OperationError> {
        // 1. Verify parent and child both exist
        if !self.structure.contains(parent) {
            return Err(OperationError::NodeNotFound { node: parent });
        }
        if !self.structure.contains(child) {
            return Err(OperationError::NodeNotFound { node: child });
        }

        // 2. Get parent's NodeKind and find the slot
        let parent_node = self
            .structure
            .get(parent)
            .ok_or(OperationError::NodeNotFound { node: parent })?;

        let (slot_exists, child_in_slot) = match parent_node.kind() {
            NodeKind::Sequence { children } => {
                if slot_name == "children" {
                    (true, children.contains(&child))
                } else {
                    (false, false)
                }
            }
            NodeKind::Section { body, .. } | NodeKind::Repeat { body, .. } => {
                if slot_name == "body" {
                    (true, body.contains(&child))
                } else {
                    (false, false)
                }
            }
            NodeKind::Tuple { elements } => {
                if slot_name == "elements" {
                    (true, elements.contains(&child))
                } else {
                    (false, false)
                }
            }
            _ => (false, false),
        };

        if !slot_exists {
            return Err(OperationError::InvalidOperation {
                action: "RemoveSlotElement".to_owned(),
                reason: format!("Node does not have slot '{slot_name}'"),
            });
        }

        if !child_in_slot {
            return Err(OperationError::InvalidOperation {
                action: "RemoveSlotElement".to_owned(),
                reason: format!("Child {child:?} not found in slot '{slot_name}'"),
            });
        }

        // 3. Remove child from the slot's Vec<NodeId>
        let parent_node = self
            .structure
            .get_mut(parent)
            .ok_or(OperationError::NodeNotFound { node: parent })?;

        match parent_node.kind() {
            NodeKind::Sequence { children } => {
                let mut new_children = children.clone();
                new_children.retain(|&id| id != child);
                parent_node.set_kind(NodeKind::Sequence {
                    children: new_children,
                });
            }
            NodeKind::Section { header, body } => {
                let mut new_body = body.clone();
                new_body.retain(|&id| id != child);
                parent_node.set_kind(NodeKind::Section {
                    header: *header,
                    body: new_body,
                });
            }
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                let mut new_body = body.clone();
                new_body.retain(|&id| id != child);
                parent_node.set_kind(NodeKind::Repeat {
                    count: count.clone(),
                    index_var: index_var.clone(),
                    body: new_body,
                });
            }
            NodeKind::Tuple { elements } => {
                let mut new_elements = elements.clone();
                new_elements.retain(|&id| id != child);
                parent_node.set_kind(NodeKind::Tuple {
                    elements: new_elements,
                });
            }
            _ => unreachable!("We checked this above"),
        }

        // 4. Remove the child node from the arena
        self.structure.remove(child);

        // 5. Remove any constraints referencing the child node
        let child_constraints = self.constraints.for_node(child);
        for cid in &child_constraints {
            self.constraints.remove(*cid);
        }

        Ok(ApplyResult {
            created_nodes: vec![],
            removed_nodes: vec![child],
            created_constraints: vec![],
            affected_constraints: child_constraints,
        })
    }
}
