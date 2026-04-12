//! TeX rendering for competitive programming AST structures.
//!
//! Produces deterministic, diff-stable TeX fragments for:
//! - Constraint notation (itemize lists)
//! - Input format notation (array layouts)

pub mod constraint_tex;
pub mod input_tex;
pub mod tex_helpers;

use crate::operation::AstEngine;
use crate::structure::NodeId;

/// TeX generation output.
#[derive(Debug, Clone)]
pub struct TexOutput {
    /// The generated TeX string.
    pub tex: String,
    /// Warnings encountered during generation.
    pub warnings: Vec<TexWarning>,
}

/// Options for TeX generation.
#[derive(Debug, Clone)]
pub struct TexOptions {
    /// Whether to include section headers (`\paragraph{}` wrappers).
    pub section_mode: SectionMode,
    /// Whether to render Hole nodes (if false, holes are silently skipped).
    pub include_holes: bool,
}

impl Default for TexOptions {
    fn default() -> Self {
        Self {
            section_mode: SectionMode::Fragment,
            include_holes: true,
        }
    }
}

/// Section mode for TeX output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionMode {
    /// TeX fragments only.
    Fragment,
    /// With section headers.
    Standalone,
}

/// Warning emitted during TeX generation.
#[derive(Debug, Clone, PartialEq)]
pub enum TexWarning {
    /// A Hole node was encountered in the AST.
    HoleEncountered {
        /// The ID of the Hole node.
        node_id: NodeId,
    },
    /// A constraint type is not supported for TeX rendering.
    UnsupportedConstraint {
        /// Description of the unsupported constraint.
        description: String,
    },
    /// A Reference could not be resolved to a named node.
    UnresolvedReference {
        /// The unresolvable reference name.
        name: String,
    },
}

/// Render constraint notation as TeX.
#[must_use]
pub fn render_constraints_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput {
    constraint_tex::render_constraints_tex_impl(engine, options)
}

/// Render input format as TeX.
#[must_use]
pub fn render_input_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput {
    input_tex::render_input_tex_impl(engine, options)
}

/// Render both input and constraint notation as a combined TeX fragment.
#[must_use]
pub fn render_full_tex(engine: &AstEngine, options: &TexOptions) -> TexOutput {
    let input = render_input_tex(engine, options);
    let constraints = render_constraints_tex(engine, options);

    let mut warnings = input.warnings;
    warnings.extend(constraints.warnings);

    let tex = match options.section_mode {
        SectionMode::Standalone => {
            let mut out = String::new();
            if !input.tex.is_empty() {
                out.push_str("\\paragraph{入力}\n");
                out.push_str(&input.tex);
            }
            if !constraints.tex.is_empty() {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str("\\paragraph{制約}\n");
                out.push_str(&constraints.tex);
            }
            out
        }
        SectionMode::Fragment => {
            let mut out = String::new();
            if !input.tex.is_empty() {
                out.push_str(&input.tex);
            }
            if !constraints.tex.is_empty() {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&constraints.tex);
            }
            out
        }
    };

    TexOutput { tex, warnings }
}
