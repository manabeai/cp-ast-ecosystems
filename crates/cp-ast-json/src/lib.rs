//! Lossless JSON serialization for `cp-ast-core` AST documents.
//!
//! Provides full-snapshot JSON roundtrip that preserves arena structure,
//! tombstones, IDs, counters, and ordering through Rust → JS → Rust cycles.
//!
//! Use [`serialize_ast`] and [`deserialize_ast`] for complete documents, or
//! [`serialize_ast_compact`] when byte size matters. Editor integrations can
//! serialize actions with [`serialize_action`] / [`deserialize_action`] and
//! serialize UI projections with [`serialize_projection`].
//!
//! # Example
//!
//! ```
//! use cp_ast_core::operation::AstEngine;
//! use cp_ast_json::{deserialize_ast, serialize_ast};
//!
//! let engine = AstEngine::new();
//! let json = serialize_ast(&engine).expect("serialize");
//! let restored = deserialize_ast(&json).expect("deserialize");
//!
//! assert_eq!(restored.structure.root(), engine.structure.root());
//! ```
//!
//! # Schema Notes
//!
//! IDs are encoded as decimal strings so JavaScript callers do not lose integer
//! precision. The top-level JSON document is versioned with
//! [`dto::CURRENT_SCHEMA_VERSION`].

/// DTO types that define the stable JSON schema.
pub mod dto;
/// Error type returned by JSON conversion functions.
pub mod error;
mod from_dto;
mod to_dto;

mod action_dto;
mod projection_dto;
/// Compact share-link encoding helpers.
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
