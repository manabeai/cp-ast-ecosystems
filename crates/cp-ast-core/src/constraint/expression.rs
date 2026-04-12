use super::types::ArithOp;
use crate::structure::{Ident, Reference};

/// Expression in constraints — represents numeric formulas.
///
/// Rev.1: 5 variants replacing the old 4.
/// Lit/Var/BinOp/Pow/FnCall.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal: 1, 42, 1000000007.
    Lit(i64),
    /// Variable reference: N, A[i].
    Var(Reference),
    /// Binary arithmetic: lhs op rhs.
    BinOp {
        op: ArithOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    /// Power: base^exp (e.g., 10^9, 2^30).
    Pow {
        base: Box<Expression>,
        exp: Box<Expression>,
    },
    /// Function call: min(a,b), max(a,b), abs(x), len(arr).
    FnCall { name: Ident, args: Vec<Expression> },
}

impl Expression {
    /// Evaluate to a constant if possible (no variable references).
    #[must_use]
    pub fn evaluate_constant(&self) -> Option<i64> {
        match self {
            Self::Lit(v) => Some(*v),
            Self::Var(_) | Self::FnCall { .. } => None,
            Self::BinOp { op, lhs, rhs } => {
                let l = lhs.evaluate_constant()?;
                let r = rhs.evaluate_constant()?;
                match op {
                    ArithOp::Add => l.checked_add(r),
                    ArithOp::Sub => l.checked_sub(r),
                    ArithOp::Mul => l.checked_mul(r),
                    ArithOp::Div => {
                        if r == 0 {
                            None
                        } else {
                            l.checked_div(r)
                        }
                    }
                }
            }
            Self::Pow { base, exp } => {
                let b = base.evaluate_constant()?;
                let e = exp.evaluate_constant()?;
                let e_u32 = u32::try_from(e).ok()?;
                b.checked_pow(e_u32)
            }
        }
    }
}
