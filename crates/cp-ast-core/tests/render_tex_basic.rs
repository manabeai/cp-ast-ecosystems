use cp_ast_core::operation::AstEngine;
use cp_ast_core::render_tex::{render_constraints_tex, render_input_tex, SectionMode, TexOptions};

#[test]
fn empty_engine_produces_empty_tex() {
    let engine = AstEngine::new();
    let options = TexOptions::default();

    let input_result = render_input_tex(&engine, &options);
    assert!(input_result.tex.is_empty() || input_result.tex.trim().is_empty());
    assert!(input_result.warnings.is_empty());

    let constraint_result = render_constraints_tex(&engine, &options);
    assert!(constraint_result.tex.is_empty());
    assert!(constraint_result.warnings.is_empty());
}

#[test]
fn default_options() {
    let options = TexOptions::default();
    assert_eq!(options.section_mode, SectionMode::Fragment);
    assert!(options.include_holes);
}
