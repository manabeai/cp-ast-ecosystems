//! WebAssembly bindings for the cp-ast-core AST viewer.
//!
//! All functions accept and return strings (JSON String API).
//! The frontend never interprets AST structure directly.

use wasm_bindgen::prelude::*;

use cp_ast_core::render_tex::{SectionMode, TexOptions};
use cp_ast_tree::TreeOptions;

mod presets;

/// Returns the crate version.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Renders input format as human-readable text.
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_input_format(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_core::render::render_input(&engine))
}

/// Renders structure AST as an ASCII tree.
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_structure_tree(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_tree::render_structure_tree(
        &engine,
        &TreeOptions::default(),
    ))
}

/// Renders constraints as human-readable text.
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_constraints_text(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_core::render::render_constraints(&engine))
}

/// Renders constraint AST as an ASCII tree.
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_constraint_tree(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    Ok(cp_ast_tree::render_constraint_tree(
        &engine,
        &TreeOptions::default(),
    ))
}

/// Renders input format as TeX (KaTeX-compatible, fragment mode, no holes).
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_input_tex(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let opts = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    Ok(cp_ast_core::render_tex::render_input_tex(&engine, &opts).tex)
}

/// Renders constraints as TeX (fragment mode).
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_constraints_tex(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let opts = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    Ok(cp_ast_core::render_tex::render_constraints_tex(&engine, &opts).tex)
}

/// Renders full TeX (input + constraints combined, fragment mode).
///
/// # Errors
///
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn render_full_tex(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let opts = TexOptions {
        section_mode: SectionMode::Fragment,
        include_holes: false,
    };
    Ok(cp_ast_core::render_tex::render_full_tex(&engine, &opts).tex)
}

/// Generates a sample test case from the AST.
///
/// `seed` is `u32` for JS `Number` compatibility (cast to `u64` internally).
///
/// # Errors
///
/// Returns `JsError` if deserialization or sample generation fails.
#[wasm_bindgen]
pub fn generate_sample(document_json: &str, seed: u32) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let sample = cp_ast_core::sample::generate(&engine, u64::from(seed))
        .map_err(|e| JsError::new(&format!("{e:?}")))?;
    Ok(cp_ast_core::sample::sample_to_text(&engine, &sample))
}

/// Returns preset list as JSON: `[{"name": "...", "description": "..."}]`.
///
/// # Panics
///
/// Panics if preset list serialization fails (should never happen with derived `Serialize`).
#[wasm_bindgen]
#[must_use]
pub fn list_presets() -> String {
    serde_json::to_string(&presets::list()).expect("preset list serialization should not fail")
}

/// Returns preset document JSON for the given name.
///
/// # Errors
///
/// Returns `JsError` if the preset name is unknown or serialization fails.
#[wasm_bindgen]
pub fn get_preset(name: &str) -> Result<String, JsError> {
    let engine =
        presets::build(name).ok_or_else(|| JsError::new(&format!("unknown preset: {name}")))?;
    cp_ast_json::serialize_ast(&engine).map_err(|e| JsError::new(&e.to_string()))
}

fn deserialize(json: &str) -> Result<cp_ast_core::operation::AstEngine, JsError> {
    cp_ast_json::deserialize_ast(json).map_err(|e| JsError::new(&e.to_string()))
}
