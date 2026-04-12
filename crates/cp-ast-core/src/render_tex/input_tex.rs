//! Input format TeX rendering.

use super::{TexOptions, TexOutput};
use crate::operation::AstEngine;

/// Render input format as TeX array layout.
pub(crate) fn render_input_tex_impl(engine: &AstEngine, _options: &TexOptions) -> TexOutput {
    // Stub — implemented in Task T-04
    let _ = engine;
    TexOutput {
        tex: String::new(),
        warnings: Vec::new(),
    }
}
