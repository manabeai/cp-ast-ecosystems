import init from '../wasm/cp_ast_wasm';

export {
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
} from '../wasm/cp_ast_wasm';

let initialized = false;

export async function initWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}
