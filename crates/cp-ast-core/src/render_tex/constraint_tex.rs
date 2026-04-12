//! Constraint TeX rendering.

use super::{TexOptions, TexOutput};
use crate::operation::AstEngine;

/// Render all constraints as TeX itemize list.
pub(crate) fn render_constraints_tex_impl(engine: &AstEngine, _options: &TexOptions) -> TexOutput {
    // Stub — implemented in Task T-03
    let _ = engine;
    TexOutput {
        tex: String::new(),
        warnings: Vec::new(),
    }
}
