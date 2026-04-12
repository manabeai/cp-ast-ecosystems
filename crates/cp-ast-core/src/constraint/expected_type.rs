/// The expected type for a position in the structure AST.
///
/// This represents what kind of value a hole or slot should accept.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedType {
    /// Integer value (e.g., N, M)
    Int,
    /// String value
    String,
    /// Floating-point value
    Float,
    /// Array of elements with a given type (e.g., `A_1..A_N` of Int)
    Array(Box<ExpectedType>),
    /// Tuple of heterogeneous types
    Tuple(Vec<ExpectedType>),
}
