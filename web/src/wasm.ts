import init from '../wasm/cp_ast_wasm';

export {
  // Existing viewer exports
  render_input_format,
  render_structure_tree,
  render_constraints_text,
  render_constraint_tree,
  render_input_tex,
  render_constraints_tex,
  render_full_tex,
  generate_sample,
  list_presets,
  get_preset,
  version,
  // New editor exports
  project_full,
  project_node_detail,
  get_hole_candidates,
  get_expr_candidates,
  get_constraint_targets,
  apply_action,
  preview_action,
  new_document,
  validate_action,
} from '../wasm/cp_ast_wasm';

let initialized = false;

export async function initWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}
