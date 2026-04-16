import { signal, computed } from '@preact/signals';
import { project_full, new_document } from '../wasm';
import type { FullProjection, DraftConstraint } from './types';

// ── Core state ──────────────────────────────────────────────────────

/** The current AST document JSON (source of truth) */
export const editorDocumentJson = signal<string>('');

/** Currently selected node ID */
export const selectedNodeId = signal<string | null>(null);

/** Currently selected constraint ID */
export const selectedConstraintId = signal<string | null>(null);

/** Draft constraint being built */
export const draftConstraint = signal<DraftConstraint | null>(null);

/** Error message from last failed operation */
export const lastError = signal<string | null>(null);

// ── Derived state ───────────────────────────────────────────────────

/** Computed full projection — auto-updates when documentJson changes */
export const projection = computed<FullProjection | null>(() => {
  const doc = editorDocumentJson.value;
  if (!doc) return null;
  try {
    return JSON.parse(project_full(doc)) as FullProjection;
  } catch (e) {
    console.error('Projection error:', e);
    return null;
  }
});

/** Whether the AST is complete (no holes, no violations) */
export const isComplete = computed<boolean>(() => {
  return projection.value?.completeness.is_complete ?? false;
});

/** Number of remaining holes */
export const holeCount = computed<number>(() => {
  return projection.value?.completeness.total_holes ?? 0;
});

// ── Initialization ──────────────────────────────────────────────────

/** Initialize editor with an empty AST */
export function initEditor(): void {
  editorDocumentJson.value = new_document();
  selectedNodeId.value = null;
  selectedConstraintId.value = null;
  draftConstraint.value = null;
  lastError.value = null;
}

/** Initialize editor from an existing document JSON */
export function initEditorFromJson(json: string): void {
  editorDocumentJson.value = json;
  selectedNodeId.value = null;
  selectedConstraintId.value = null;
  draftConstraint.value = null;
  lastError.value = null;
}