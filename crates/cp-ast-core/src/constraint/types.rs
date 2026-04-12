/// Relation operator for variable comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationOp {
    /// Less than (`<`).
    Lt,
    /// Less than or equal (`≤`).
    Le,
    /// Greater than (`>`).
    Gt,
    /// Greater than or equal (`≥`).
    Ge,
    /// Equal (`=`).
    Eq,
    /// Not equal (`≠`).
    Ne,
}

/// Arithmetic operator for expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Unit for distinctness constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DistinctUnit {
    Element,
    Tuple,
}

/// Structural property tag for graph/array properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyTag {
    Simple,
    Connected,
    Tree,
    Permutation,
    Binary,
    Odd,
    Even,
    Custom(String),
}

/// Sort order for sorted constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortOrder {
    Ascending,
    NonDecreasing,
    Descending,
    NonIncreasing,
}

/// Character set specification for string generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharSetSpec {
    LowerAlpha,
    UpperAlpha,
    Alpha,
    Digit,
    AlphaNumeric,
    Custom(Vec<char>),
    Range(char, char),
}

/// Rendering hint kind (separator info moved from `StructureAST` per S-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderHintKind {
    Separator(Separator),
}

/// Separator between elements in rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Separator {
    Space,
    None,
}
