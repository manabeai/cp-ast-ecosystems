/// The kind of structure node in a competitive programming problem specification.
///
/// Each variant represents a structural concept recognized by competitive
/// programming problem authors and readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// A single value (e.g., `N`, `M`, `K`)
    Scalar,
    /// A one-dimensional sequence (e.g., `A_1`, `A_2`, ..., `A_N`)
    Array,
    /// A two-dimensional grid (e.g., H×W matrix)
    Matrix,
    /// A wrapper for T test cases
    MultiTestCase,
    /// A query structure with type-dependent sub-formats
    Query,
    /// A block describing input format
    InputBlock,
    /// A block describing output format
    OutputBlock,
}
