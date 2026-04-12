//! TeX conversion helpers for expressions, references, identifiers.

use crate::constraint::Expression;
use crate::operation::AstEngine;
use crate::structure::{Ident, NodeKind, Reference};

use super::TexWarning;

/// Convert an `Expression` to its TeX representation.
#[allow(dead_code)]
pub(crate) fn expression_to_tex(
    engine: &AstEngine,
    expr: &Expression,
    warnings: &mut Vec<TexWarning>,
) -> String {
    // Stub — implemented in Task T-02
    let _ = (engine, warnings);
    format!("{expr:?}")
}

/// Convert a `Reference` to its TeX representation.
#[allow(dead_code)]
pub(crate) fn reference_to_tex(
    engine: &AstEngine,
    reference: &Reference,
    warnings: &mut Vec<TexWarning>,
) -> String {
    // Stub — implemented in Task T-02
    let _ = warnings;
    match reference {
        Reference::VariableRef(node_id) => {
            if let Some(node) = engine.structure.get(*node_id) {
                match node.kind() {
                    NodeKind::Scalar { name } | NodeKind::Array { name, .. } => ident_to_tex(name),
                    _ => format!("?{node_id:?}"),
                }
            } else {
                format!("?{node_id:?}")
            }
        }
        Reference::IndexedRef { target, indices } => {
            let base = if let Some(node) = engine.structure.get(*target) {
                match node.kind() {
                    NodeKind::Scalar { name } | NodeKind::Array { name, .. } => ident_to_tex(name),
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
            warnings.push(TexWarning::UnresolvedReference { name: name.clone() });
            ident_to_tex(ident)
        }
    }
}

/// Convert an `Ident` to its TeX representation.
/// Single character → as-is; multiple characters → `\mathrm{}`.
#[allow(dead_code)]
pub(crate) fn ident_to_tex(ident: &Ident) -> String {
    let s = ident.as_str();
    if s.len() <= 1 {
        s.to_owned()
    } else {
        format!("\\mathrm{{{s}}}")
    }
}

/// Index variable allocator for array/repeat subscripts.
/// Allocates `i`, `j`, `k`, `l`, ... deterministically.
#[allow(dead_code)]
pub(crate) struct IndexAllocator {
    next: u8,
}

#[allow(dead_code)]
impl IndexAllocator {
    pub(crate) fn new() -> Self {
        Self { next: b'i' }
    }

    pub(crate) fn allocate(&mut self) -> char {
        let c = self.next as char;
        self.next += 1;
        c
    }
}
