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

fn parse_node_id(s: &str) -> Result<cp_ast_core::structure::NodeId, JsError> {
    let raw: u64 = s
        .parse()
        .map_err(|_| JsError::new(&format!("invalid node ID: {s}")))?;
    Ok(cp_ast_core::structure::NodeId::from_raw(raw))
}

#[derive(serde::Serialize)]
struct PreviewResultDto {
    new_holes_created: Vec<String>,
    constraints_affected: Vec<String>,
}

// ── Editor Functions ────────────────────────────────────────────────

/// Projects full AST outline for the editor UI.
/// Returns JSON: { "outline": [...], "diagnostics": [...], "completeness": {...} }
///
/// # Errors
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn project_full(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let projection = cp_ast_core::projection::project_full(&engine);
    cp_ast_json::serialize_full_projection(&projection).map_err(|e| JsError::new(&e.to_string()))
}

/// Projects detail for a specific node (slots + constraints).
/// `node_id` is a decimal string.
/// Returns JSON or null if node not found.
///
/// # Errors
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document or node ID is invalid.
#[wasm_bindgen]
pub fn project_node_detail(document_json: &str, node_id: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let id = parse_node_id(node_id)?;
    match cp_ast_core::projection::project_node_detail(&engine, id) {
        Some(detail) => cp_ast_json::serialize_node_detail_projection(&detail)
            .map_err(|e| JsError::new(&e.to_string())),
        None => Ok("null".to_owned()),
    }
}

/// Gets candidates for filling a hole node.
/// Returns JSON array of candidates.
///
/// # Errors
/// Returns `JsError` if the JSON cannot be deserialized or hole ID is invalid.
#[wasm_bindgen]
pub fn get_hole_candidates(document_json: &str, hole_id: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let id = parse_node_id(hole_id)?;
    let candidates = cp_ast_core::projection::get_hole_candidates(&engine, id);
    cp_ast_json::serialize_hole_candidates(&candidates).map_err(|e| JsError::new(&e.to_string()))
}

/// Gets expression candidates (references + literals).
/// Returns JSON with references and literal arrays.
///
/// # Errors
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn get_expr_candidates(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let menu = cp_ast_core::projection::get_expr_candidates(&engine);
    cp_ast_json::serialize_expr_candidate_menu(&menu).map_err(|e| JsError::new(&e.to_string()))
}

/// Gets nodes that can be targets for new constraints.
/// Returns JSON with target nodes.
///
/// # Errors
/// Returns `JsError` if the JSON cannot be deserialized into a valid AST document.
#[wasm_bindgen]
pub fn get_constraint_targets(document_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let menu = cp_ast_core::projection::get_constraint_targets(&engine);
    cp_ast_json::serialize_constraint_target_menu(&menu).map_err(|e| JsError::new(&e.to_string()))
}

/// Applies an action to the AST and returns the updated document JSON.
/// `action_json`: serialized `ActionDto` (e.g., {"kind": "`FillHole`", ...})
/// Returns: updated document JSON on success, throws `JsError` on failure.
///
/// # Errors
/// Returns `JsError` if the JSON cannot be deserialized, action is invalid, or application fails.
#[wasm_bindgen]
pub fn apply_action(document_json: &str, action_json: &str) -> Result<String, JsError> {
    let mut engine = deserialize(document_json)?;
    let action =
        cp_ast_json::deserialize_action(action_json).map_err(|e| JsError::new(&e.to_string()))?;
    engine
        .apply(&action)
        .map_err(|e| JsError::new(&e.to_string()))?;
    cp_ast_json::serialize_ast(&engine).map_err(|e| JsError::new(&e.to_string()))
}

/// Preview an action without applying it (dry run).
/// Returns JSON: { "`new_holes_created`": [...], "`constraints_affected`": [...] }
///
/// # Errors
/// Returns `JsError` if the JSON cannot be deserialized, action is invalid, or preview fails.
#[wasm_bindgen]
pub fn preview_action(document_json: &str, action_json: &str) -> Result<String, JsError> {
    let engine = deserialize(document_json)?;
    let action =
        cp_ast_json::deserialize_action(action_json).map_err(|e| JsError::new(&e.to_string()))?;
    let preview = engine
        .preview(&action)
        .map_err(|e| JsError::new(&e.to_string()))?;

    let dto = PreviewResultDto {
        new_holes_created: preview
            .new_holes_created
            .iter()
            .map(|id| id.value().to_string())
            .collect(),
        constraints_affected: preview
            .constraints_affected
            .iter()
            .map(ToString::to_string)
            .collect(),
    };

    serde_json::to_string(&dto).map_err(|e| JsError::new(&e.to_string()))
}

/// Creates a new empty AST document.
/// Returns JSON string of the new document.
///
/// # Panics
/// Panics if new document serialization fails (should never happen).
#[wasm_bindgen]
#[must_use]
pub fn new_document() -> String {
    let engine = cp_ast_core::operation::AstEngine::new();
    cp_ast_json::serialize_ast(&engine).expect("new document serialization should not fail")
}

/// Validates whether an action JSON is well-formed and can be deserialized.
/// Returns "ok" if valid, or an error message string.
#[wasm_bindgen]
#[must_use]
pub fn validate_action(action_json: &str) -> String {
    match cp_ast_json::deserialize_action(action_json) {
        Ok(_) => "ok".to_owned(),
        Err(e) => e.to_string(),
    }
}

#[cfg(test)]
mod editor_tests {
    use super::*;

    #[test]
    fn test_new_document() {
        let doc_json = new_document();
        assert!(!doc_json.is_empty());
        assert!(doc_json.contains("schema_version"));
    }

    #[test]
    fn test_project_full_with_new_document() {
        let doc_json = new_document();
        let result = project_full(&doc_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_action_valid() {
        let action_json = r#"{"kind": "IntroduceMultiTestCase", "count_var_name": "i"}"#;
        let result = validate_action(action_json);
        assert_eq!(result, "ok");
    }

    #[test]
    fn test_validate_action_invalid() {
        let action_json = r#"{"invalid": "json"}"#;
        let result = validate_action(action_json);
        assert_ne!(result, "ok");
    }

    #[test]
    fn test_get_expr_candidates_with_new_document() {
        let doc_json = new_document();
        let result = get_expr_candidates(&doc_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_constraint_targets_with_new_document() {
        let doc_json = new_document();
        let result = get_constraint_targets(&doc_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_node_detail_nonexistent_node() {
        let doc_json = new_document();
        let result = project_node_detail(&doc_json, "999999").unwrap();
        assert_eq!(result, "null");
    }

    // Note: test_project_node_detail_invalid_id cannot run in unit tests 
    // due to wasm_bindgen limitations. The function works correctly in wasm context.
}
