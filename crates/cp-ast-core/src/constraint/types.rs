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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum CharSetSpec {
    LowerAlpha,
    UpperAlpha,
    Alpha,
    Digit,
    AlphaNumeric,
    Custom(Vec<char>),
    Range(char, char),
}

impl std::fmt::Display for CharSetSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LowerAlpha => write!(f, "lowercase letters"),
            Self::UpperAlpha => write!(f, "uppercase letters"),
            Self::Alpha => write!(f, "letters"),
            Self::Digit => write!(f, "digits"),
            Self::AlphaNumeric => write!(f, "alphanumeric characters"),
            Self::Custom(chars) => {
                let inner: Vec<String> = chars.iter().map(ToString::to_string).collect();
                write!(f, "{{{}}}", inner.join(", "))
            }
            Self::Range(a, z) => write!(f, "'{a}'..'{z}'"),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_lower_alpha() {
        assert_eq!(CharSetSpec::LowerAlpha.to_string(), "lowercase letters");
    }

    #[test]
    fn display_upper_alpha() {
        assert_eq!(CharSetSpec::UpperAlpha.to_string(), "uppercase letters");
    }

    #[test]
    fn display_alpha() {
        assert_eq!(CharSetSpec::Alpha.to_string(), "letters");
    }

    #[test]
    fn display_digit() {
        assert_eq!(CharSetSpec::Digit.to_string(), "digits");
    }

    #[test]
    fn display_alphanumeric() {
        assert_eq!(
            CharSetSpec::AlphaNumeric.to_string(),
            "alphanumeric characters"
        );
    }

    #[test]
    fn display_custom() {
        let spec = CharSetSpec::Custom(vec!['a', 'b', 'c']);
        assert_eq!(spec.to_string(), "{a, b, c}");
    }

    #[test]
    fn display_custom_single() {
        let spec = CharSetSpec::Custom(vec!['x']);
        assert_eq!(spec.to_string(), "{x}");
    }

    #[test]
    fn display_custom_empty() {
        let spec = CharSetSpec::Custom(vec![]);
        assert_eq!(spec.to_string(), "{}");
    }

    #[test]
    fn display_range() {
        let spec = CharSetSpec::Range('a', 'z');
        assert_eq!(spec.to_string(), "'a'..'z'");
    }
}
