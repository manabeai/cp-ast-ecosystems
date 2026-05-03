//! Lossless JSON serialization for `cp-ast-core` AST.
//!
//! Provides full-snapshot JSON roundtrip that preserves arena structure,
//! tombstones, IDs, counters, and ordering through Rust → JS → Rust cycles.

pub mod dto;
pub mod error;
mod from_dto;
mod to_dto;

mod action_dto;
mod projection_dto;
pub mod share_state;

pub use dto::AstDocumentEnvelope;
pub use error::ConversionError;

pub use action_dto::{deserialize_action, serialize_action};
pub use projection_dto::serialize_projection;
pub use share_state::{decode_share_state_json, deserialize_share_state, encode_share_state_json};

use cp_ast_core::operation::AstEngine;

/// Serialize an `AstEngine` to a JSON string.
///
/// Wraps in a versioned envelope with `schema_version`.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_ast(engine: &AstEngine) -> Result<String, ConversionError> {
    let envelope = to_dto::engine_to_envelope(engine);
    serde_json::to_string_pretty(&envelope).map_err(ConversionError::from)
}

/// Serialize an `AstEngine` to a compact JSON string.
///
/// This is intended for transport-oriented use cases such as share links.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_ast_compact(engine: &AstEngine) -> Result<String, ConversionError> {
    let envelope = to_dto::engine_to_envelope(engine);
    serde_json::to_string(&envelope).map_err(ConversionError::from)
}

/// Deserialize an `AstEngine` from a JSON string.
///
/// Validates schema version and arena consistency.
///
/// # Errors
/// Returns `ConversionError` if JSON is invalid, version unsupported,
/// or arena data is inconsistent.
pub fn deserialize_ast(json: &str) -> Result<AstEngine, ConversionError> {
    let envelope: AstDocumentEnvelope = serde_json::from_str(json)?;
    from_dto::envelope_to_engine(envelope)
}
