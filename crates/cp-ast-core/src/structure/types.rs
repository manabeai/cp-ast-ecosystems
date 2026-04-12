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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKindHint {
    AnyScalar,
    AnyArray,
    AnyMatrix,
    AnyTuple,
    AnyRepeat,
    AnySection,
    AnyChoice,
    Any,
}
