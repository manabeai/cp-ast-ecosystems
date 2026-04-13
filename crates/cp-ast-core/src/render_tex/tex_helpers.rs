//! TeX conversion helpers for expressions, references, identifiers.

use crate::constraint::{ArithOp, Expression};
use crate::operation::AstEngine;
use crate::structure::{Ident, NodeKind, Reference};

use super::TexWarning;

/// Convert an `Expression` to its TeX representation.
pub fn expression_to_tex(
    engine: &AstEngine,
    expr: &Expression,
    warnings: &mut Vec<TexWarning>,
) -> String {
    match expr {
        Expression::Lit(n) => lit_to_tex(*n),
        Expression::Var(reference) => reference_to_tex(engine, reference, warnings),
        Expression::BinOp { op, lhs, rhs } => {
            let lhs_str = expression_to_tex(engine, lhs, warnings);
            let rhs_str = expression_to_tex(engine, rhs, warnings);
            let op_str = match op {
                ArithOp::Add => " + ",
                ArithOp::Sub => " - ",
                ArithOp::Mul => " \\times ",
                ArithOp::Div => " \\div ",
            };
            format!("{lhs_str}{op_str}{rhs_str}")
        }
        Expression::Pow { base, exp } => {
            let base_str = expression_to_tex(engine, base, warnings);
            let exp_str = expression_to_tex(engine, exp, warnings);
            format!("{base_str}^{{{exp_str}}}")
        }
        Expression::FnCall { name, args } => {
            let name_str = name.as_str();
            let tex_name = match name_str {
                "min" | "max" | "gcd" | "lcm" | "log" => format!("\\{name_str}"),
                _ => ident_to_tex(name),
            };
            let args_str = args
                .iter()
                .map(|a| expression_to_tex(engine, a, warnings))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{tex_name}({args_str})")
        }
    }
}

/// Convert a literal integer to TeX, auto-decomposing large numbers.
fn lit_to_tex(n: i64) -> String {
    if n < 0 {
        let pos = lit_to_tex(-n);
        return format!("-{pos}");
    }
    if let Some(tex) = decompose_large_number(n) {
        return tex;
    }
    n.to_string()
}

/// Try to decompose n into `a × 10^k` form for TeX.
/// Returns `Some(tex_string)` if decomposable, None otherwise.
fn decompose_large_number(n: i64) -> Option<String> {
    if n <= 0 {
        return None;
    }
    let mut k: u32 = 0;
    let mut remaining = n;
    while remaining % 10 == 0 {
        remaining /= 10;
        k += 1;
    }
    if k < 2 {
        return None;
    }
    if remaining == 1 {
        Some(format!("10^{{{k}}}"))
    } else if remaining < 10 {
        Some(format!("{remaining} \\times 10^{{{k}}}"))
    } else {
        None
    }
}

/// Convert a `Reference` to its TeX representation.
pub fn reference_to_tex(
    engine: &AstEngine,
    reference: &Reference,
    warnings: &mut Vec<TexWarning>,
) -> String {
    match reference {
        Reference::VariableRef(node_id) => {
            if let Some(node) = engine.structure.get(*node_id) {
                match node.kind() {
                    NodeKind::Scalar { name }
                    | NodeKind::Array { name, .. }
                    | NodeKind::Matrix { name, .. } => ident_to_tex(name),
                    _ => format!("?{node_id:?}"),
                }
            } else {
                format!("?{node_id:?}")
            }
        }
        Reference::IndexedRef { target, indices } => {
            let base = if let Some(node) = engine.structure.get(*target) {
                match node.kind() {
                    NodeKind::Scalar { name }
                    | NodeKind::Array { name, .. }
                    | NodeKind::Matrix { name, .. } => ident_to_tex(name),
                    _ => format!("?{target:?}"),
                }
            } else {
                format!("?{target:?}")
            };
            let idx_str = indices
                .iter()
                .map(Ident::as_str)
                .collect::<Vec<_>>()
                .join(",");
            format!("{base}_{{{idx_str}}}")
        }
        Reference::Unresolved(ident) => {
            let name = ident.as_str().to_owned();
            warnings.push(TexWarning::UnresolvedReference { name });
            ident_to_tex(ident)
        }
    }
}

/// Convert an `Ident` to its TeX representation.
/// Single character → as-is (math italic); multiple characters → `\mathrm{}`.
#[must_use]
pub fn ident_to_tex(ident: &Ident) -> String {
    let s = ident.as_str();
    if s.len() <= 1 {
        s.to_owned()
    } else {
        format!("\\mathrm{{{s}}}")
    }
}

/// Index variable allocator for array/repeat subscripts.
/// Allocates `i`, `j`, `k`, `l`, ... deterministically.
#[derive(Debug, Clone)]
pub struct IndexAllocator {
    next: u8,
}

impl Default for IndexAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexAllocator {
    /// Create a new allocator starting at 'i'.
    #[must_use]
    pub fn new() -> Self {
        Self { next: b'i' }
    }

    /// Allocate the next index variable character.
    pub fn allocate(&mut self) -> char {
        let c = self.next as char;
        self.next += 1;
        c
    }
}

/// Check if a Reference target is an Array node, and if so return (name, `length_ref`).
#[must_use]
pub fn resolve_array_info(
    engine: &AstEngine,
    reference: &Reference,
) -> Option<(String, Expression)> {
    if let Reference::VariableRef(node_id) = reference {
        if let Some(node) = engine.structure.get(*node_id) {
            if let NodeKind::Array { name, length } = node.kind() {
                return Some((ident_to_tex(name), length.clone()));
            }
        }
    }
    None
}
