use std::collections::{HashMap, HashSet, VecDeque};

use crate::constraint::{Constraint, Expression};
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind, Reference};

/// Extract `NodeId` references from an `Expression`.
fn extract_var_refs(expr: &Expression) -> Vec<NodeId> {
    match expr {
        Expression::Var(Reference::VariableRef(id)) => vec![*id],
        Expression::Lit(_) | Expression::Var(_) => vec![],
        Expression::BinOp { lhs, rhs, .. } => {
            let mut refs = extract_var_refs(lhs);
            refs.extend(extract_var_refs(rhs));
            refs
        }
        Expression::Pow { base, exp } => {
            let mut refs = extract_var_refs(base);
            refs.extend(extract_var_refs(exp));
            refs
        }
        Expression::FnCall { args, .. } => args.iter().flat_map(extract_var_refs).collect(),
    }
}

/// Extract `NodeId` references from a constraint's expression fields.
///
/// Note: The `target` field identifies which node the constraint belongs to,
/// NOT a dependency. Only expression fields (lower, upper, etc.) contain
/// actual cross-references that create dependencies.
fn extract_constraint_refs(constraint: &Constraint) -> Vec<NodeId> {
    match constraint {
        Constraint::Range { lower, upper, .. } => {
            let mut refs = extract_var_refs(lower);
            refs.extend(extract_var_refs(upper));
            refs
        }
        Constraint::SumBound { upper, .. } => extract_var_refs(upper),
        Constraint::LengthRelation { length, .. } => extract_var_refs(length),
        Constraint::Relation { lhs, rhs, .. } => {
            let mut refs = extract_var_refs(lhs);
            refs.extend(extract_var_refs(rhs));
            refs
        }
        Constraint::StringLength { min, max, .. } => {
            let mut refs = extract_var_refs(min);
            refs.extend(extract_var_refs(max));
            refs
        }
        Constraint::Guarantee {
            predicate: Some(expr),
            ..
        } => extract_var_refs(expr),
        // These constraint types don't have expression fields with variable refs
        Constraint::TypeDecl { .. }
        | Constraint::Distinct { .. }
        | Constraint::Property { .. }
        | Constraint::Sorted { .. }
        | Constraint::CharSet { .. }
        | Constraint::RenderHint { .. }
        | Constraint::Guarantee {
            predicate: None, ..
        } => vec![],
    }
}

/// Error returned when the dependency graph contains a cycle.
#[derive(Debug, Clone)]
pub struct CycleError {
    /// Nodes involved in the cycle (may not be exhaustive).
    pub involved: Vec<NodeId>,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "dependency cycle detected involving {} nodes",
            self.involved.len()
        )
    }
}

impl std::error::Error for CycleError {}

/// Directed acyclic graph of generation dependencies between nodes.
///
/// An edge from A → B means "A depends on B" (B must be generated first).
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// node → set of nodes it depends on
    deps: HashMap<NodeId, Vec<NodeId>>,
    /// all nodes in the graph
    all_nodes: Vec<NodeId>,
}

impl DependencyGraph {
    /// Build a dependency graph from the engine's structure AST.
    ///
    /// Dependencies come from:
    /// 1. Reference dependencies (Array length, Matrix rows/cols, Repeat count)
    /// 2. Parent-child ordering (children depend on parent container decisions)
    #[must_use]
    pub fn build(engine: &AstEngine) -> Self {
        let mut deps: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut all_nodes = Vec::new();

        // Collect all nodes
        for node in engine.structure.iter() {
            let id = node.id();
            all_nodes.push(id);
            deps.entry(id).or_default();
        }

        // Walk structure to find dependencies
        for node in engine.structure.iter() {
            let id = node.id();
            match node.kind() {
                NodeKind::Array { length, .. } => {
                    for ref_id in extract_var_refs(length) {
                        deps.entry(id).or_default().push(ref_id);
                    }
                }
                NodeKind::Matrix { rows, cols, .. } => {
                    if let Reference::VariableRef(ref_id) = rows {
                        deps.entry(id).or_default().push(*ref_id);
                    }
                    if let Reference::VariableRef(ref_id) = cols {
                        deps.entry(id).or_default().push(*ref_id);
                    }
                }
                NodeKind::Repeat { count, body, .. } => {
                    for ref_id in extract_var_refs(count) {
                        deps.entry(id).or_default().push(ref_id);
                    }
                    // Body elements depend on the repeat node
                    for &child in body {
                        deps.entry(child).or_default().push(id);
                    }
                }
                NodeKind::Sequence { children } => {
                    for &child in children {
                        deps.entry(child).or_default().push(id);
                    }
                }
                NodeKind::Section { header, body } => {
                    if let Some(h) = header {
                        deps.entry(*h).or_default().push(id);
                    }
                    for &child in body {
                        deps.entry(child).or_default().push(id);
                    }
                }
                NodeKind::Tuple { elements } => {
                    for &child in elements {
                        deps.entry(child).or_default().push(id);
                    }
                }
                NodeKind::Choice { variants, tag } => {
                    if let Reference::VariableRef(ref_id) = tag {
                        deps.entry(id).or_default().push(*ref_id);
                    }
                    for (_, children) in variants {
                        for &child in children {
                            deps.entry(child).or_default().push(id);
                        }
                    }
                }
                NodeKind::Scalar { .. } | NodeKind::Hole { .. } => {}
            }
        }

        // Second pass: add constraint-level dependencies
        // If a node's constraint references another node via VariableRef in its
        // expressions, add a dependency edge (this node depends on the referenced node)
        let all_nodes_set: HashSet<NodeId> = all_nodes.iter().copied().collect();
        for node in engine.structure.iter() {
            let id = node.id();
            let constraint_ids = engine.constraints.for_node(id);
            for cid in constraint_ids {
                if let Some(constraint) = engine.constraints.get(cid) {
                    let refs = extract_constraint_refs(constraint);
                    for ref_id in refs {
                        // Only add edge if ref_id is a different node AND exists in structure
                        // (references to non-existent nodes will fail later during generation)
                        if ref_id != id && all_nodes_set.contains(&ref_id) {
                            deps.entry(id).or_default().push(ref_id);
                        }
                    }
                }
            }
        }

        Self { deps, all_nodes }
    }

    /// Return the direct dependencies of a node.
    #[must_use]
    pub fn dependencies_of(&self, node: NodeId) -> &[NodeId] {
        self.deps.get(&node).map_or(&[], Vec::as_slice)
    }

    /// Return all nodes in the graph.
    #[must_use]
    pub fn all_nodes(&self) -> &[NodeId] {
        &self.all_nodes
    }

    /// Topological sort using Kahn's algorithm (BFS-based).
    ///
    /// Returns nodes in generation order (dependencies first).
    ///
    /// # Errors
    /// Returns `CycleError` if the graph contains a cycle.
    pub fn topological_sort(&self) -> Result<Vec<NodeId>, CycleError> {
        // Build in-degree map
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        // Reverse adjacency: node → nodes that depend on it
        let mut reverse: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        for &node in &self.all_nodes {
            in_degree.entry(node).or_insert(0);
            reverse.entry(node).or_default();
        }

        for (&node, dep_list) in &self.deps {
            // Deduplicate dependencies for in-degree counting
            let unique_deps: HashSet<&NodeId> = dep_list.iter().collect();
            *in_degree.entry(node).or_insert(0) += unique_deps.len();
            for &dep in &unique_deps {
                reverse.entry(*dep).or_default().push(node);
            }
        }

        // Start with nodes that have no dependencies
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        for (&node, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(node);
            }
        }

        let mut sorted = Vec::with_capacity(self.all_nodes.len());

        while let Some(node) = queue.pop_front() {
            sorted.push(node);
            if let Some(dependents) = reverse.get(&node) {
                for &dependent in dependents {
                    if let Some(deg) = in_degree.get_mut(&dependent) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        if sorted.len() == self.all_nodes.len() {
            Ok(sorted)
        } else {
            // Nodes not in sorted are involved in cycles
            let sorted_set: HashSet<NodeId> = sorted.into_iter().collect();
            let involved = self
                .all_nodes
                .iter()
                .filter(|n| !sorted_set.contains(n))
                .copied()
                .collect();
            Err(CycleError { involved })
        }
    }
}
