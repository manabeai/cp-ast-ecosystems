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
    /// Variable reference, such as `N` or `A[i]`.
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

/// Parse a compact numeric expression string used by editor actions.
///
/// This parser intentionally supports the small expression language accepted by
/// the UI: integer literals, variable names, binary arithmetic, power, and
/// simple function calls such as `min(N,5)`.
#[must_use]
pub fn parse_expression_str(input: &str) -> Expression {
    let s = input.trim();
    if let Ok(n) = s.parse::<i64>() {
        return Expression::Lit(n);
    }

    if let Some((pos, op)) = find_top_level_operator(s, &['+', '-']) {
        return parse_binary(s, pos, op);
    }
    if let Some((pos, op)) = find_top_level_operator(s, &['*', '/']) {
        return parse_binary(s, pos, op);
    }
    if let Some((pos, _)) = find_top_level_operator(s, &['^']) {
        return Expression::Pow {
            base: Box::new(parse_expression_str(&s[..pos])),
            exp: Box::new(parse_expression_str(&s[pos + 1..])),
        };
    }

    if let Some((name, args)) = parse_fn_call(s) {
        return Expression::FnCall {
            name: Ident::new(name),
            args: args.iter().map(|arg| parse_expression_str(arg)).collect(),
        };
    }

    Expression::Var(Reference::Unresolved(Ident::new(s)))
}

fn parse_binary(s: &str, pos: usize, op: char) -> Expression {
    Expression::BinOp {
        op: match op {
            '+' => ArithOp::Add,
            '-' => ArithOp::Sub,
            '*' => ArithOp::Mul,
            '/' => ArithOp::Div,
            _ => unreachable!("operator is filtered by caller"),
        },
        lhs: Box::new(parse_expression_str(&s[..pos])),
        rhs: Box::new(parse_expression_str(&s[pos + 1..])),
    }
}

fn find_top_level_operator(s: &str, ops: &[char]) -> Option<(usize, char)> {
    let mut depth = 0usize;
    for (pos, ch) in s.char_indices().rev() {
        match ch {
            ')' => depth = depth.saturating_add(1),
            '(' => depth = depth.saturating_sub(1),
            _ if depth == 0 && ops.contains(&ch) && pos > 0 => return Some((pos, ch)),
            _ => {}
        }
    }
    None
}

fn parse_fn_call(s: &str) -> Option<(&str, Vec<String>)> {
    let open = s.find('(')?;
    if !s.ends_with(')') || open == 0 {
        return None;
    }
    let name = s[..open].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }

    let inner = &s[open + 1..s.len() - 1];
    let args = split_top_level_args(inner)?;
    Some((name, args))
}

fn split_top_level_args(s: &str) -> Option<Vec<String>> {
    if s.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (pos, ch) in s.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => {
                depth = depth.checked_sub(1)?;
            }
            ',' if depth == 0 => {
                let arg = s[start..pos].trim();
                if arg.is_empty() {
                    return None;
                }
                args.push(arg.to_owned());
                start = pos + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }

    let arg = s[start..].trim();
    if arg.is_empty() {
        return None;
    }
    args.push(arg.to_owned());
    Some(args)
}
