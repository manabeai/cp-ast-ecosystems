import { signal, computed } from '@preact/signals';
import {
  render_input_format,
  render_structure_tree,
  render_constraints_text,
  render_constraint_tree,
  render_input_tex,
  render_constraints_tex,
  generate_sample,
  get_preset,
} from './wasm';

// ── Page routing ────────────────────────────────────────────────────

export const currentPage = signal<'viewer' | 'preview' | 'editor'>(
  window.location.hash === '#/preview' ? 'preview' :
  window.location.hash === '#/editor' ? 'editor' : 'viewer',
);

window.addEventListener('hashchange', () => {
  const hash = window.location.hash;
  currentPage.value = hash === '#/preview' ? 'preview' :
    hash === '#/editor' ? 'editor' : 'viewer';
});

// ── Viewer state ────────────────────────────────────────────────────

export const documentJson = signal<string>('');
export const activePreset = signal<string>('scalar_array');
export const sampleSeed = signal<number>(0);
export const activePreviewTab = signal<'tex' | 'sample'>('tex');
export const structureAstMode = signal<boolean>(false);
export const constraintAstMode = signal<boolean>(false);

// ── Derived state ───────────────────────────────────────────────────

function safeCall<T>(fn: () => T, fallback: T): T {
  try {
    return fn();
  } catch (e) {
    console.error(e);
    return fallback;
  }
}

export const structureText = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(
    () =>
      structureAstMode.value
        ? render_structure_tree(documentJson.value)
        : render_input_format(documentJson.value),
    'Error rendering structure',
  );
});

export const constraintText = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(
    () =>
      constraintAstMode.value
        ? render_constraint_tree(documentJson.value)
        : render_constraints_text(documentJson.value),
    'Error rendering constraints',
  );
});

export const inputTexString = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(() => render_input_tex(documentJson.value), '');
});

export const constraintsTexString = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(() => render_constraints_tex(documentJson.value), '');
});

export const sampleText = computed(() => {
  if (!documentJson.value) return '';
  return safeCall(
    () => generate_sample(documentJson.value, sampleSeed.value),
    'Error generating sample',
  );
});

// ── Actions ─────────────────────────────────────────────────────────

export function loadPreset(name: string): void {
  try {
    documentJson.value = get_preset(name);
    activePreset.value = name;
  } catch (e) {
    console.error('Failed to load preset:', e);
  }
}

export function shuffleSeed(): void {
  sampleSeed.value = Math.floor(Math.random() * 0xffffffff);
}
