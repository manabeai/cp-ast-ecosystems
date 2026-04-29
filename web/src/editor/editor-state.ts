/**
 * Editor state management using Preact signals.
 *
 * TEA Pattern:
 *   documentJson (Signal)  = Model
 *   apply_action() (WASM)  = Update
 *   project_full() (WASM)  = View
 */
import { signal, computed } from '@preact/signals';
import {
  new_document,
  project_full,
  apply_action,
  render_input_tex,
  render_constraints_tex,
  generate_sample,
} from '../wasm';

// ── Types (mirrors FullProjectionDto from Rust) ────────────────────

export interface ProjectedNode {
  id: string;
  label: string;
  depth: number;
  is_hole: boolean;
}

export interface StructureLine {
  depth: number;
  nodes: ProjectedNode[];
}

export interface Hotspot {
  parent_id: string;
  direction: 'below' | 'right' | 'inside' | 'variant';
  candidates: string[];
}

export interface DraftConstraint {
  index: number;
  target_id: string;
  target_name: string;
  display: string;
  template: string;
}

export interface CompletedConstraint {
  index: number;
  constraint_id: string;
  display: string;
}

export interface ProjectedConstraints {
  items: ConstraintItem[];
  drafts: DraftConstraint[];
  completed: CompletedConstraint[];
}

export interface ConstraintItem {
  index: number;
  status: 'draft' | 'completed';
  target_id: string;
  target_name: string;
  display: string;
  template?: string;
  constraint_id?: string;
  draft_index?: number;
  completed_index?: number;
}

export interface ExprCandidate {
  name: string;
  node_id: string;
}

export interface CompletenessSummary {
  total_holes: number;
  filled_slots: number;
  unsatisfied_constraints: number;
  is_complete: boolean;
}

export interface FullProjection {
  nodes: ProjectedNode[];
  structure_lines: StructureLine[];
  hotspots: Hotspot[];
  constraints: ProjectedConstraints;
  available_vars: ExprCandidate[];
  completeness: CompletenessSummary;
}

// ── Signals ────────────────────────────────────────────────────────

export const documentJson = signal<string>('');
export const sampleSeed = signal<number>(42);

// ── Derived state ──────────────────────────────────────────────────

function safeCall<T>(fn: () => T, fallback: T): T {
  try {
    return fn();
  } catch (e) {
    console.error(e);
    return fallback;
  }
}

const emptyProjection: FullProjection = {
  nodes: [],
  structure_lines: [],
  hotspots: [],
  constraints: { items: [], drafts: [], completed: [] },
  available_vars: [],
  completeness: { total_holes: 0, filled_slots: 0, unsatisfied_constraints: 0, is_complete: false },
};

export const projection = computed<FullProjection>(() => {
  if (!documentJson.value) return emptyProjection;
  return safeCall(() => JSON.parse(project_full(documentJson.value)) as FullProjection, emptyProjection);
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
  return safeCall(() => generate_sample(documentJson.value, sampleSeed.value), '');
});

// ── Actions ────────────────────────────────────────────────────────

export function initEditor(): void {
  try {
    documentJson.value = new_document();
  } catch (e) {
    console.error('Failed to create new document:', e);
  }
}

export function setDocumentJson(json: string): void {
  documentJson.value = json;
}

export function dispatchAction(actionJson: string): void {
  try {
    documentJson.value = apply_action(documentJson.value, actionJson);
  } catch (e) {
    console.error('Action failed:', e, actionJson);
  }
}

export function shuffleSeed(): void {
  sampleSeed.value = Math.floor(Math.random() * 0xffffffff);
}
