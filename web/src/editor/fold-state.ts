/**
 * Fold state for SP (mobile) responsive layout.
 * Each pane has a signal controlling its collapsed state.
 * On mobile, panes can be toggled between open/collapsed.
 */
import { signal } from '@preact/signals';

// Structure pane: open by default on SP
export const structureFolded = signal(false);

// Constraint pane: collapsed by default on SP
export const constraintFolded = signal(true);

// Preview pane: collapsed by default on SP
export const previewFolded = signal(true);

export function toggleStructureFold() {
  structureFolded.value = !structureFolded.value;
}

export function toggleConstraintFold() {
  constraintFolded.value = !constraintFolded.value;
}

export function togglePreviewFold() {
  previewFolded.value = !previewFolded.value;
}
