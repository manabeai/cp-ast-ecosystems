//! Core AST model for competitive-programming problem specifications.
//!
//! `cp-ast-core` models an input format as two cooperating trees:
//!
//! - [`structure`] describes the shape of the input: scalars, arrays, matrices,
//!   tuples, repeated blocks, choices, sections, and holes.
//! - [`constraint`] attaches semantic facts to those nodes: numeric ranges,
//!   expected value types, distinctness, sortedness, string character sets,
//!   rendering hints, and similar problem-statement constraints.
//!
//! Most applications use [`operation::AstEngine`] as the main entry point. It
//! stores both trees and applies high-level [`operation::Action`] values that
//! are convenient for editors and other interactive tools.
//!
//! # Public API Map
//!
//! - [`structure`] - AST node IDs, node kinds, references, literals, and the
//!   arena-backed [`structure::StructureAst`].
//! - [`constraint`] - constraint expressions, IDs, and
//!   [`constraint::ConstraintSet`].
//! - [`operation`] - mutation API centered on [`operation::AstEngine`].
//! - [`projection`] - UI-friendly read models, including
//!   [`projection::project_full`].
//! - [`render`] - deterministic text rendering for input formats and
//!   constraints.
//! - [`render_tex`] - deterministic TeX/KaTeX rendering.
//! - [`sample`] - deterministic sample input generation from an AST.
//!
//! # Example
//!
//! Build a tiny input format for `N` followed by an array `A` of length `N`:
//!
//! ```
//! use cp_ast_core::constraint::{Constraint, ExpectedType, Expression};
//! use cp_ast_core::operation::AstEngine;
//! use cp_ast_core::render::render_input;
//! use cp_ast_core::structure::{Ident, NodeKind, Reference};
//!
//! let mut engine = AstEngine::new();
//! let n = engine.structure.add_node(NodeKind::Scalar {
//!     name: Ident::new("N"),
//! });
//! let a = engine.structure.add_node(NodeKind::Array {
//!     name: Ident::new("A"),
//!     length: Expression::Var(Reference::VariableRef(n)),
//! });
//!
//! let root = engine.structure.add_node(NodeKind::Sequence {
//!     children: vec![n, a],
//! });
//! engine.structure.set_root(root);
//! engine.constraints.add(
//!     Some(n),
//!     Constraint::TypeDecl {
//!         target: Reference::VariableRef(n),
//!         expected: ExpectedType::Int,
//!     },
//! );
//!
//! let rendered = render_input(&engine);
//! assert!(rendered.starts_with("N\nA_1"));
//! ```
//!
//! For JSON roundtrips and browser integration, pair this crate with
//! `cp-ast-json` and `cp-ast-wasm`.

/// Constraint AST types and storage.
pub mod constraint;
/// High-level mutation API for building and editing AST documents.
pub mod operation;
/// UI-friendly read projections derived from AST documents.
pub mod projection;
/// Plain-text rendering helpers.
pub mod render;
/// TeX and KaTeX rendering helpers.
pub mod render_tex;
/// Deterministic sample input generation.
pub mod sample;
/// Structure AST types and storage.
pub mod structure;
