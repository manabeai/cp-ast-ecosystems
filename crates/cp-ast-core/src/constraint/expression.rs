use crate::structure::NodeId;

/// An expression used in constraint bounds.
///
/// Expressions represent values that appear in constraints like
/// `1 <= N <= 2 * 10^5`. They can be literal constants, references
/// to other nodes (variables), or arithmetic combinations.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal integer value (e.g., `1`, `42`)
    Literal(i64),
    /// A power expression (base^exp), e.g., `10^9`
    Power(i64, u32),
    /// A reference to another node's value (e.g., `N`)
    Ref(NodeId),
    /// Multiplication of two expressions (e.g., `2 * 10^5`)
    Mul(Box<Expression>, Box<Expression>),
}

impl Expression {
    /// Evaluate this expression to a constant value, if possible.
    ///
    /// Returns `None` if the expression contains node references
    /// that cannot be resolved without runtime context.
    #[must_use]
    pub fn evaluate_constant(&self) -> Option<i64> {
        match self {
            Self::Literal(v) => Some(*v),
            Self::Power(base, exp) => Some(base.pow(*exp)),
            Self::Ref(_) => None,
            Self::Mul(lhs, rhs) => {
                let l = lhs.evaluate_constant()?;
                let r = rhs.evaluate_constant()?;
                Some(l * r)
            }
        }
    }
}
