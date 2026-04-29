use super::engine::AstEngine;
use super::error::OperationError;
use super::fill_hole::var_type_to_expected_from_fill;
use super::result::ApplyResult;
use super::types::FillContent;
use crate::constraint::{Constraint, ExpectedType};
use crate::structure::{Literal, NodeId, NodeKind, Reference};

fn constraint_is_compatible_with_replacement(
    constraint: &Constraint,
    expected_type: Option<&ExpectedType>,
) -> bool {
    match constraint {
        Constraint::TypeDecl { .. } => false,
        Constraint::Range { .. } => matches!(expected_type, Some(ExpectedType::Int) | None),
        Constraint::CharSet { .. } => {
            matches!(expected_type, Some(ExpectedType::Char | ExpectedType::Str))
        }
        Constraint::StringLength { .. } => matches!(expected_type, Some(ExpectedType::Str)),
        _ => true,
    }
}

impl AstEngine {
    /// Replace an existing node with new content.
    ///
    /// # Errors
    /// Returns `OperationError` if:
    /// - Target node doesn't exist
    /// - Target node is a Hole (use `FillHole` instead)
    /// - Constraints incompatible with the replacement are removed
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

        // 3. Remove constraints that cannot survive the new node shape/type.
        let existing_constraints = self.constraints.for_node(target);
        let expected_type = var_type_to_expected_from_fill(replacement);
        let mut affected_constraints = Vec::new();
        for cid in &existing_constraints {
            let Some(constraint) = self.constraints.get(*cid) else {
                continue;
            };
            if !constraint_is_compatible_with_replacement(constraint, expected_type.as_ref()) {
                self.constraints.remove(*cid);
                affected_constraints.push(*cid);
            }
        }

        // 4. Expand replacement FillContent using existing helper
        let mut created_nodes = Vec::new();
        let new_kind = self.expand_fill_content(replacement, &mut created_nodes);

        // 5. Replace the node's kind
        if let Some(node) = self.structure.get_mut(target) {
            node.set_kind(new_kind);
        }

        // 5.5. Resolve Unresolved variable references in structure expressions
        self.resolve_structure_references(target);
        for &child_id in &created_nodes {
            self.resolve_structure_references(child_id);
        }

        let mut created_constraints = Vec::new();
        if let Some(expected) = expected_type {
            let cid = self.constraints.add(
                Some(target),
                Constraint::TypeDecl {
                    target: Reference::VariableRef(target),
                    expected,
                },
            );
            created_constraints.push(cid);
        }

        Ok(ApplyResult {
            created_nodes,
            removed_nodes: vec![],
            created_constraints,
            affected_constraints,
        })
    }

    /// Add an element to a parent node's slot.
    ///
    /// # Errors
    /// Returns `OperationError` if:
    /// - Parent node doesn't exist
    /// - Parent node doesn't have the specified slot
    #[allow(clippy::too_many_lines)]
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

        // 3.5. Resolve Unresolved variable references in structure expressions
        self.resolve_structure_references(new_node_id);
        for &child_id in &created_nodes {
            self.resolve_structure_references(child_id);
        }

        // 4. Auto-add TypeDecl constraint if applicable
        let mut created_constraints = Vec::new();
        if let Some(expected_type) = var_type_to_expected_from_fill(element) {
            let cid = self.constraints.add(
                Some(new_node_id),
                Constraint::TypeDecl {
                    target: Reference::VariableRef(new_node_id),
                    expected: expected_type,
                },
            );
            created_constraints.push(cid);
        }

        // 5. Get parent mutably and add to the correct slot
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
            created_constraints,
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
            NodeKind::Sequence { children } if slot_name == "children" => {
                (true, children.contains(&child))
            }
            NodeKind::Section { body, .. } | NodeKind::Repeat { body, .. }
                if slot_name == "body" =>
            {
                (true, body.contains(&child))
            }
            NodeKind::Tuple { elements } if slot_name == "elements" => {
                (true, elements.contains(&child))
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

    // ── AddSibling / AddChoiceVariant ──────────────────────────────────

    /// Add a sibling element next to a target node.
    ///
    /// If the target is inside a Tuple, the new element is appended to that Tuple.
    /// If the target is a direct child of a Sequence, Section body, or Repeat body,
    /// a new Tuple is created containing the target and the new element, and that
    /// Tuple replaces the target in the parent's slot.
    ///
    /// # Errors
    /// Returns `OperationError` if the target node doesn't exist or has no parent slot.
    pub(crate) fn add_sibling(
        &mut self,
        target: NodeId,
        element: &FillContent,
    ) -> Result<ApplyResult, OperationError> {
        if !self.structure.contains(target) {
            return Err(OperationError::NodeNotFound { node: target });
        }

        let (parent_id, slot) =
            self.find_parent(target)
                .ok_or(OperationError::InvalidOperation {
                    action: "AddSibling".to_owned(),
                    reason: "Target node has no parent slot".to_owned(),
                })?;

        // Expand the fill content into a new node
        let mut created_nodes = Vec::new();
        let new_kind = self.expand_fill_content(element, &mut created_nodes);
        let new_node_id = self.structure.add_node(new_kind);
        created_nodes.push(new_node_id);

        // Resolve Unresolved variable references in structure expressions
        self.resolve_structure_references(new_node_id);
        for &child_id in &created_nodes {
            self.resolve_structure_references(child_id);
        }

        // Auto-add TypeDecl constraint if applicable
        let mut created_constraints = Vec::new();
        if let Some(expected_type) = var_type_to_expected_from_fill(element) {
            let cid = self.constraints.add(
                Some(new_node_id),
                Constraint::TypeDecl {
                    target: Reference::VariableRef(new_node_id),
                    expected: expected_type,
                },
            );
            created_constraints.push(cid);
        }

        if let ParentSlot::Tuple = slot {
            // Target is already inside a Tuple — append new element
            let parent_node = self
                .structure
                .get(parent_id)
                .ok_or(OperationError::NodeNotFound { node: parent_id })?;
            if let NodeKind::Tuple { elements } = parent_node.kind() {
                let mut new_elements = elements.clone();
                new_elements.push(new_node_id);
                let parent_node = self
                    .structure
                    .get_mut(parent_id)
                    .ok_or(OperationError::NodeNotFound { node: parent_id })?;
                parent_node.set_kind(NodeKind::Tuple {
                    elements: new_elements,
                });
            } else {
                return Err(OperationError::InvalidOperation {
                    action: "AddSibling".to_owned(),
                    reason: "Parent slot/kind invariant violation".to_owned(),
                });
            }
        } else {
            // Target is a direct child of Sequence/Section/Repeat —
            // create a Tuple[target, new_element] and replace target in parent
            let tuple_id = self.structure.add_node(NodeKind::Tuple {
                elements: vec![target, new_node_id],
            });
            created_nodes.push(tuple_id);
            self.replace_child_in_slot(parent_id, &slot, target, tuple_id)?;
        }

        Ok(ApplyResult {
            created_nodes,
            removed_nodes: vec![],
            created_constraints,
            affected_constraints: vec![],
        })
    }

    /// Add a variant to a Choice node.
    ///
    /// # Errors
    /// Returns `OperationError` if the target node doesn't exist or is not a Choice.
    pub(crate) fn add_choice_variant(
        &mut self,
        choice: NodeId,
        tag_value: &Literal,
        first_element: &FillContent,
    ) -> Result<ApplyResult, OperationError> {
        if !self.structure.contains(choice) {
            return Err(OperationError::NodeNotFound { node: choice });
        }

        // Verify it's a Choice
        {
            let node = self
                .structure
                .get(choice)
                .ok_or(OperationError::NodeNotFound { node: choice })?;
            if !matches!(node.kind(), NodeKind::Choice { .. }) {
                return Err(OperationError::InvalidOperation {
                    action: "AddChoiceVariant".to_owned(),
                    reason: format!("Node {choice:?} is not a Choice"),
                });
            }
        }

        // Expand the fill content
        let mut created_nodes = Vec::new();
        let new_kind = self.expand_fill_content(first_element, &mut created_nodes);
        let new_node_id = self.structure.add_node(new_kind);
        created_nodes.push(new_node_id);

        // Resolve Unresolved variable references in structure expressions
        self.resolve_structure_references(new_node_id);
        for &child_id in &created_nodes {
            self.resolve_structure_references(child_id);
        }

        // Auto-add TypeDecl constraint if applicable
        let mut created_constraints = Vec::new();
        if let Some(expected_type) = var_type_to_expected_from_fill(first_element) {
            let cid = self.constraints.add(
                Some(new_node_id),
                Constraint::TypeDecl {
                    target: Reference::VariableRef(new_node_id),
                    expected: expected_type,
                },
            );
            created_constraints.push(cid);
        }

        // Append the variant
        // expand_fill_content only creates new nodes — it never mutates existing ones,
        // so the choice node kind is unchanged here.
        let node = self
            .structure
            .get(choice)
            .ok_or(OperationError::NodeNotFound { node: choice })?;
        if let NodeKind::Choice { tag, variants } = node.kind() {
            let tag_cloned = tag.clone();
            let mut new_variants = variants.clone();
            new_variants.push((tag_value.clone(), vec![new_node_id]));
            let node = self
                .structure
                .get_mut(choice)
                .ok_or(OperationError::NodeNotFound { node: choice })?;
            node.set_kind(NodeKind::Choice {
                tag: tag_cloned,
                variants: new_variants,
            });
        } else {
            return Err(OperationError::InvalidOperation {
                action: "AddChoiceVariant".to_owned(),
                reason: "Choice node kind invariant violation".to_owned(),
            });
        }

        Ok(ApplyResult {
            created_nodes,
            removed_nodes: vec![],
            created_constraints,
            affected_constraints: vec![],
        })
    }

    /// Find which node contains `target` in its children/body/elements slot.
    ///
    /// Note: does not search inside Choice variant bodies.
    fn find_parent(&self, target: NodeId) -> Option<(NodeId, ParentSlot)> {
        for node in self.structure.iter() {
            let parent_id = node.id();
            match node.kind() {
                NodeKind::Sequence { children } if children.contains(&target) => {
                    return Some((parent_id, ParentSlot::Sequence));
                }
                NodeKind::Section { body, .. } if body.contains(&target) => {
                    return Some((parent_id, ParentSlot::SectionBody));
                }
                NodeKind::Repeat { body, .. } if body.contains(&target) => {
                    return Some((parent_id, ParentSlot::RepeatBody));
                }
                NodeKind::Tuple { elements } if elements.contains(&target) => {
                    return Some((parent_id, ParentSlot::Tuple));
                }
                _ => {}
            }
        }
        None
    }

    /// Replace `old_child` with `new_child` in the specified slot of a parent node.
    fn replace_child_in_slot(
        &mut self,
        parent: NodeId,
        slot: &ParentSlot,
        old_child: NodeId,
        new_child: NodeId,
    ) -> Result<(), OperationError> {
        let parent_node = self
            .structure
            .get(parent)
            .ok_or(OperationError::NodeNotFound { node: parent })?;

        let new_kind = match (slot, parent_node.kind()) {
            (ParentSlot::Sequence, NodeKind::Sequence { children }) => {
                let new_children = children
                    .iter()
                    .map(|&id| if id == old_child { new_child } else { id })
                    .collect();
                NodeKind::Sequence {
                    children: new_children,
                }
            }
            (ParentSlot::SectionBody, NodeKind::Section { header, body }) => {
                let new_body = body
                    .iter()
                    .map(|&id| if id == old_child { new_child } else { id })
                    .collect();
                NodeKind::Section {
                    header: *header,
                    body: new_body,
                }
            }
            (
                ParentSlot::RepeatBody,
                NodeKind::Repeat {
                    count,
                    index_var,
                    body,
                },
            ) => {
                let new_body = body
                    .iter()
                    .map(|&id| if id == old_child { new_child } else { id })
                    .collect();
                NodeKind::Repeat {
                    count: count.clone(),
                    index_var: index_var.clone(),
                    body: new_body,
                }
            }
            _ => {
                return Err(OperationError::InvalidOperation {
                    action: "AddSibling".to_owned(),
                    reason: "Parent slot mismatch".to_owned(),
                });
            }
        };

        let parent_node = self
            .structure
            .get_mut(parent)
            .ok_or(OperationError::NodeNotFound { node: parent })?;
        parent_node.set_kind(new_kind);
        Ok(())
    }
}

/// Describes which slot of a parent node contains the target child.
enum ParentSlot {
    Sequence,
    SectionBody,
    RepeatBody,
    Tuple,
}
