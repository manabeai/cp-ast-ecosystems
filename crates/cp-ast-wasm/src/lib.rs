//! WebAssembly bindings for the cp-ast-core AST viewer.
//!
//! All functions accept and return strings (JSON String API).
//! The frontend never interprets AST structure directly.

use wasm_bindgen::prelude::*;

/// Returns the crate version.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Renders input format as human-readable text.
///
/// Input: cp-ast-json document JSON string.
/// Output: formatted input specification (e.g., `"N\nA_1 A_2 ... A_N"`).
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_input_format(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_core::render::render_input(&engine))
}

fn deserialize(json: &str) -> Result<cp_ast_core::operation::AstEngine, JsError> {
    cp_ast_json::deserialize_ast(json).map_err(|e| JsError::new(&e.to_string()))
}
