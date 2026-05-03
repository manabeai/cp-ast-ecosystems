//! Share-link state codec used by the web UI and companion CLIs.

use std::io::{Read, Write};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::{deserialize_ast, serialize_ast_compact, ConversionError};
use cp_ast_core::operation::AstEngine;

/// Encode AST document JSON into the compact share-link `state` payload.
///
/// The input is first deserialized and reserialized through the canonical JSON
/// boundary, then gzip-compressed and base64url-encoded without padding.
///
/// # Errors
/// Returns `ConversionError` if the input JSON is invalid or gzip encoding
/// fails.
#[must_use = "share-state encoding can fail and the result should be handled"]
pub fn encode_share_state_json(document_json: &str) -> Result<String, ConversionError> {
    let engine = deserialize_ast(document_json)?;
    let canonical_json = serialize_ast_compact(&engine)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(canonical_json.as_bytes())
        .map_err(|err| ConversionError::Gzip(err.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|err| ConversionError::Gzip(err.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(compressed))
}

/// Decode a compact share-link `state` payload into canonical AST document JSON.
///
/// # Errors
/// Returns `ConversionError` if the state is not valid base64url, not valid
/// gzip data, or does not contain a valid AST document.
#[must_use = "share-state decoding can fail and the result should be handled"]
pub fn decode_share_state_json(state: &str) -> Result<String, ConversionError> {
    let compressed = URL_SAFE_NO_PAD
        .decode(state)
        .map_err(|err| ConversionError::Base64(err.to_string()))?;
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut json = String::new();
    decoder
        .read_to_string(&mut json)
        .map_err(|err| ConversionError::Gzip(err.to_string()))?;
    let engine = deserialize_ast(&json)?;
    serialize_ast_compact(&engine)
}

/// Deserialize an `AstEngine` directly from a compact share-link `state`.
///
/// # Errors
/// Returns `ConversionError` if state decoding or AST deserialization fails.
#[must_use = "share-state deserialization can fail and the result should be handled"]
pub fn deserialize_share_state(state: &str) -> Result<AstEngine, ConversionError> {
    let json = decode_share_state_json(state)?;
    deserialize_ast(&json)
}
