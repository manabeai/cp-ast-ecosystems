//! Deterministic sample input generation.
//!
//! The sample generator evaluates supported constraints and structure nodes to
//! produce concrete input text. Use [`crate::sample::generate`] or
//! [`crate::sample::generate_with_config`] to obtain a
//! [`crate::sample::GeneratedSample`], then [`crate::sample::sample_to_text`] to
//! render it.

/// Dependency graph helpers used by sample generation.
pub mod dependency;
/// Sample generation engine and result types.
pub mod generator;
/// Text rendering for generated samples.
pub mod output;

pub use dependency::{CycleError, DependencyGraph};
pub use generator::{
    GeneratedSample, GenerationConfig, GenerationError, SampleValue, generate, generate_with_config,
};
pub use output::sample_to_text;
