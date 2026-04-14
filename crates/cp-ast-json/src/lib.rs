//! Lossless JSON serialization for `cp-ast-core` AST.
//!
//! Provides full-snapshot JSON roundtrip that preserves arena structure,
//! tombstones, IDs, counters, and ordering through Rust → JS → Rust cycles.

pub mod dto;
pub mod error;
mod from_dto;
pub mod to_dto;

pub use dto::AstDocumentEnvelope;
pub use error::ConversionError;
pub use from_dto::envelope_to_engine;
pub use to_dto::engine_to_envelope;
