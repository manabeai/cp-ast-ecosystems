use std::fmt;

/// Expected type for `TypeDecl` constraints.
///
/// Rev.1: Simplified to 3 variants. Array/Tuple/Float removed.
/// Complex type info is expressed via constraint composition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedType {
    Int,
    Str,
    Char,
}

impl fmt::Display for ExpectedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "integer"),
            Self::Str => write!(f, "string"),
            Self::Char => write!(f, "character"),
        }
    }
}
