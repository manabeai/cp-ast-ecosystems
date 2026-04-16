//! Lossless JSON serialization for `cp-ast-core` AST.
//!
//! Provides full-snapshot JSON roundtrip that preserves arena structure,
//! tombstones, IDs, counters, and ordering through Rust → JS → Rust cycles.

pub mod dto;
pub mod error;
mod from_dto;
mod to_dto;

pub use dto::{
    ActionDto,
    ApplyResultDto,
    AstDocumentEnvelope,
    ConstraintTargetMenuDto,
    ExprCandidateMenuDto,
    // Editor projection DTOs
    FullProjectionDto,
    HoleCandidateDto,
    NodeDetailProjectionDto,
    OperationErrorDto,
    SlotIdDto,
};
pub use error::ConversionError;

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

// ── Editor Projection Serialization ────────────────────────────────

/// Serialize a `FullProjection` to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_full_projection(
    fp: &cp_ast_core::projection::FullProjection,
) -> Result<String, ConversionError> {
    let dto = to_dto::full_projection_to_dto(fp);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Serialize a `NodeDetailProjection` to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_node_detail_projection(
    nd: &cp_ast_core::projection::NodeDetailProjection,
) -> Result<String, ConversionError> {
    let dto = to_dto::node_detail_to_dto(nd);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Serialize hole candidates to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_hole_candidates(
    candidates: &[cp_ast_core::projection::HoleCandidate],
) -> Result<String, ConversionError> {
    let dto = to_dto::hole_candidates_to_dto(candidates);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Serialize expression candidate menu to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_expr_candidate_menu(
    menu: &cp_ast_core::projection::ExprCandidateMenu,
) -> Result<String, ConversionError> {
    let dto = to_dto::expr_candidates_to_dto(menu);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Serialize constraint target menu to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_constraint_target_menu(
    menu: &cp_ast_core::projection::ConstraintTargetMenu,
) -> Result<String, ConversionError> {
    let dto = to_dto::constraint_targets_to_dto(menu);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

// ── Action Serialization/Deserialization ───────────────────────────

/// Serialize an `Action` to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_action(
    action: &cp_ast_core::operation::Action,
) -> Result<String, ConversionError> {
    let dto = to_dto::action_to_dto(action);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Deserialize an `Action` from JSON string.
///
/// # Errors
/// Returns `ConversionError` if JSON is invalid or contains unknown variants.
pub fn deserialize_action(json: &str) -> Result<cp_ast_core::operation::Action, ConversionError> {
    let dto: ActionDto = serde_json::from_str(json)?;
    from_dto::action_from_dto(dto)
}

// ── Operation Results and Errors ────────────────────────────────────

/// Serialize an `OperationError` to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_operation_error(
    err: &cp_ast_core::operation::OperationError,
) -> Result<String, ConversionError> {
    let dto = to_dto::operation_error_to_dto(err);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Serialize an `ApplyResult` to JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_apply_result(
    result: &cp_ast_core::operation::ApplyResult,
) -> Result<String, ConversionError> {
    let dto = to_dto::apply_result_to_dto(result);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}
