//! Lossless JSON serialization for `cp-ast-core` AST.
//!
//! Provides full-snapshot JSON roundtrip that preserves arena structure,
//! tombstones, IDs, counters, and ordering through Rust → JS → Rust cycles.

pub mod dto;
pub mod error;
mod from_dto;
mod to_dto;

pub use dto::AstDocumentEnvelope;
pub use error::ConversionError;
