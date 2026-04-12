/// Variable name or index name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(String);

impl Ident {
    /// Create a new identifier.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Ident {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Literal value in AST expressions and Choice variant tags.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal.
    IntLit(i64),
    /// String literal.
    StrLit(String),
}

/// Hint for what kind of node is expected at a Hole position.
/// Used by UI to suggest candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKindHint {
    /// Expects any scalar-like node (single value).
    AnyScalar,
    /// Expects any array-like node (1D sequence).
    AnyArray,
    /// Expects any matrix-like node (2D grid).
    AnyMatrix,
    /// Expects any tuple-like node (fixed-size heterogeneous group).
    AnyTuple,
    /// Expects any repeat-like node (dynamic repetition).
    AnyRepeat,
    /// Expects any section-like node (input/output block).
    AnySection,
    /// Expects any choice-like node (branching format).
    AnyChoice,
    /// Any node kind is acceptable.
    Any,
}
