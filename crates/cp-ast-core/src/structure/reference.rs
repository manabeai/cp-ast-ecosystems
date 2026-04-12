use super::node_id::NodeId;
use super::types::Ident;

/// Reference to a variable or indexed element.
#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
    /// Direct reference to a variable node.
    VariableRef(NodeId),
    /// Indexed reference: A[i], C[i][j].
    IndexedRef { target: NodeId, indices: Vec<Ident> },
    /// Unresolved reference (name only, used during construction).
    Unresolved(Ident),
}
