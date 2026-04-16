import { editorDocumentJson, lastError } from './state';
import { apply_action } from '../wasm';
import type { EditorAction } from './types';

/**
 * Dispatch an action to the AST engine.
 * Updates editorDocumentJson on success.
 * Sets lastError on failure.
 * Returns true on success, false on failure.
 */
export function dispatchAction(action: EditorAction): boolean {
  try {
    const newDoc = apply_action(
      editorDocumentJson.value,
      JSON.stringify(action),
    );
    editorDocumentJson.value = newDoc;
    lastError.value = null;
    return true;
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    lastError.value = message;
    console.error('Action dispatch failed:', message);
    return false;
  }
}