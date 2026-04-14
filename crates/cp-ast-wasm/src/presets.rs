//! Preset AST definitions — populated in Task 3.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
}

#[must_use]
pub fn list() -> Vec<PresetInfo> {
    vec![]
}

#[must_use]
pub fn build(_name: &str) -> Option<cp_ast_core::operation::AstEngine> {
    None
}
