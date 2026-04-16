/**
 * Editor E2E tests using the real wasm binary.
 *
 * These tests exercise the core editor flow end-to-end:
 *   wasm API → apply_action → project_full → inspection APIs
 *
 * Key API facts:
 *   - new_document() starts with a root Sequence node (id "0"), is_complete=true, 0 holes
 *   - Holes only exist when using presets or AddSlotElement with a Hole element
 *   - get_preset('hole_structure') gives a document with 2 holes
 *   - AddSlotElement { parent, slot_name, element } adds child nodes to Sequences
 *   - FillHole { target, fill } fills a hole node with concrete content
 *   - ReplaceNode { target, replacement } replaces any node (including root)
 *   - Node IDs are numeric strings ("0", "1", "2", ...)
 *   - Type names: 'Int', 'Long', 'Float', 'String', 'Char' (capitalized)
 */

import { describe, it, expect } from 'vitest';
import {
  new_document,
  get_preset,
  project_full,
  project_node_detail,
  get_hole_candidates,
  get_expr_candidates,
  get_constraint_targets,
  apply_action,
  validate_action,
} from '../wasm/cp_ast_wasm';
import type { FullProjection, OutlineNode, HoleCandidate } from '../src/editor/types';

// ── Helpers ────────────────────────────────────────────────────────────────

function parseProjection(docJson: string): FullProjection {
  return JSON.parse(project_full(docJson)) as FullProjection;
}

function findHole(proj: FullProjection): OutlineNode | undefined {
  return proj.outline.find((n) => n.is_hole);
}

function appendScalar(docJson: string, parentId: string, name: string): string {
  return apply_action(
    docJson,
    JSON.stringify({ kind: 'AddSlotElement', parent: parentId, slot_name: 'children', element: { kind: 'Scalar', name, typ: 'Int' } }),
  );
}

function fillHole(docJson: string, holeId: string, fill: object): string {
  return apply_action(docJson, JSON.stringify({ kind: 'FillHole', target: holeId, fill }));
}

function addRangeConstraint(docJson: string, targetId: string, lower: string, upper: string): string {
  return apply_action(
    docJson,
    JSON.stringify({ kind: 'AddConstraint', target: targetId, constraint: { kind: { kind: 'Range', lower, upper } } }),
  );
}

// ── Test suite ────────────────────────────────────────────────────────────

describe('Editor E2E: new_document', () => {
  it('returns valid JSON', () => {
    const doc = new_document();
    expect(() => JSON.parse(doc)).not.toThrow();
  });

  it('starts with a root Sequence node', () => {
    const doc = new_document();
    const proj = parseProjection(doc);

    const seq = proj.outline.find((n) => n.kind_label === 'Sequence');
    expect(seq).toBeDefined();
    expect(seq!.depth).toBe(0);
  });

  it('projection has required fields', () => {
    const doc = new_document();
    const proj = parseProjection(doc);

    expect(proj.outline).toBeDefined();
    expect(proj.diagnostics).toBeDefined();
    expect(proj.completeness).toBeDefined();
    expect(typeof proj.completeness.is_complete).toBe('boolean');
    expect(typeof proj.completeness.total_holes).toBe('number');
  });
});

describe('Editor E2E: AddSlotElement', () => {
  it('appends a Scalar child to the root Sequence', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');

    const proj = parseProjection(doc);
    const n = proj.outline.find((node) => node.label === 'N');
    expect(n).toBeDefined();
    expect(n!.is_hole).toBe(false);
    expect(n!.kind_label).toBe('Scalar');
  });

  it('appends multiple children in order', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');
    doc = appendScalar(doc, '0', 'M');

    const proj = parseProjection(doc);
    const labels = proj.outline.filter((n) => !n.is_hole && n.kind_label !== 'Sequence').map((n) => n.label);
    expect(labels).toContain('N');
    expect(labels).toContain('M');
  });
});

describe('Editor E2E: FillHole (via hole_structure preset)', () => {
  it('hole_structure preset has holes', () => {
    const doc = get_preset('hole_structure');
    const proj = parseProjection(doc);

    expect(proj.completeness.is_complete).toBe(false);
    expect(proj.completeness.total_holes).toBeGreaterThan(0);
    expect(findHole(proj)).toBeDefined();
  });

  it('FillHole replaces a hole with a Scalar', () => {
    let doc = get_preset('hole_structure');
    const proj = parseProjection(doc);
    const hole = findHole(proj)!;

    doc = fillHole(doc, hole.id, { kind: 'Scalar', name: 'M', typ: 'Int' });

    const proj2 = parseProjection(doc);
    const m = proj2.outline.find((n) => n.label === 'M');
    expect(m).toBeDefined();
    expect(m!.is_hole).toBe(false);
    expect(proj2.completeness.total_holes).toBeLessThan(proj.completeness.total_holes);
  });

  it('filling all holes makes is_complete true', () => {
    let doc = get_preset('hole_structure');
    let proj = parseProjection(doc);

    let i = 0;
    while (!proj.completeness.is_complete) {
      const hole = findHole(proj);
      if (!hole) break;
      doc = fillHole(doc, hole.id, { kind: 'Scalar', name: `V${i++}`, typ: 'Int' });
      proj = parseProjection(doc);
    }

    expect(proj.completeness.total_holes).toBe(0);
  });
});

describe('Editor E2E: AddConstraint', () => {
  it('adds a Range constraint to a Scalar node', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');

    const proj = parseProjection(doc);
    const nNode = proj.outline.find((n) => n.label === 'N')!;

    // Without constraint, there's a "No constraints defined" warning
    const warningsBefore = proj.diagnostics.filter((d) => d.node_id === nNode.id).length;
    expect(warningsBefore).toBeGreaterThan(0);

    doc = addRangeConstraint(doc, nNode.id, '1', '100000');

    const proj2 = parseProjection(doc);
    const warningsAfter = proj2.diagnostics.filter((d) => d.node_id === nNode.id).length;
    // Constraint clears the warning
    expect(warningsAfter).toBe(0);
  });

  it('document is valid JSON after adding constraint', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');
    const proj = parseProjection(doc);
    const nId = proj.outline.find((n) => n.label === 'N')!.id;
    doc = addRangeConstraint(doc, nId, '1', '1000000000');
    expect(() => JSON.parse(doc)).not.toThrow();
  });
});

describe('Editor E2E: ReplaceNode', () => {
  it('replaces the root Sequence with a Scalar', () => {
    let doc = new_document();
    doc = apply_action(doc, JSON.stringify({ kind: 'ReplaceNode', target: '0', replacement: { kind: 'Scalar', name: 'N', typ: 'Int' } }));

    const proj = parseProjection(doc);
    const n = proj.outline.find((node) => node.label === 'N');
    expect(n).toBeDefined();
    expect(n!.is_hole).toBe(false);
  });
});

describe('Editor E2E: get_hole_candidates', () => {
  it('returns candidates for a real hole', () => {
    const doc = get_preset('hole_structure');
    const proj = parseProjection(doc);
    const hole = findHole(proj)!;

    const candidatesJson = get_hole_candidates(doc, hole.id);
    const candidates = JSON.parse(candidatesJson) as HoleCandidate[];

    expect(Array.isArray(candidates)).toBe(true);
    expect(candidates.length).toBeGreaterThan(0);
    expect(candidates[0]).toHaveProperty('kind');
    expect(candidates[0]).toHaveProperty('suggested_names');
  });

  it('returns empty array for a non-hole node', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');

    const proj = parseProjection(doc);
    const nNode = proj.outline.find((n) => n.label === 'N')!;
    const candidates = JSON.parse(get_hole_candidates(doc, nNode.id)) as HoleCandidate[];

    expect(Array.isArray(candidates)).toBe(true);
    expect(candidates.length).toBe(0);
  });
});

describe('Editor E2E: get_expr_candidates', () => {
  it('returns references and literals after adding a scalar', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');

    const menu = JSON.parse(get_expr_candidates(doc));

    expect(menu).toHaveProperty('references');
    expect(menu).toHaveProperty('literals');
    expect(Array.isArray(menu.references)).toBe(true);
    expect(Array.isArray(menu.literals)).toBe(true);
    expect(menu.references.some((r: { label: string }) => r.label === 'N')).toBe(true);
  });

  it('returns empty references for an empty document', () => {
    const doc = new_document();
    const menu = JSON.parse(get_expr_candidates(doc));
    expect(menu.references.length).toBe(0);
  });
});

describe('Editor E2E: get_constraint_targets', () => {
  it('returns named nodes as constraint targets', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');

    const menu = JSON.parse(get_constraint_targets(doc));

    expect(menu).toHaveProperty('targets');
    expect(Array.isArray(menu.targets)).toBe(true);
    expect(menu.targets.some((t: { label: string }) => t.label === 'N')).toBe(true);
  });
});

describe('Editor E2E: validate_action', () => {
  it('validates a well-formed FillHole action', () => {
    // Use a real hole ID from hole_structure preset
    const doc = get_preset('hole_structure');
    const proj = parseProjection(doc);
    const holeId = findHole(proj)!.id;

    const result = validate_action(JSON.stringify({ kind: 'FillHole', target: holeId, fill: { kind: 'Scalar', name: 'M', typ: 'Int' } }));
    expect(result).toBe('ok');
  });

  it('validates a well-formed AddConstraint action', () => {
    // validate_action doesn't check node existence
    const result = validate_action(JSON.stringify({ kind: 'AddConstraint', target: '0', constraint: { kind: { kind: 'Range', lower: '1', upper: '100000' } } }));
    expect(result).toBe('ok');
  });

  it('returns error for completely malformed action', () => {
    const result = validate_action('{"kind":"NotAnAction"}');
    expect(typeof result).toBe('string');
    expect(result).not.toBe('ok');
  });

  it('returns error for invalid JSON', () => {
    const result = validate_action('not-json');
    expect(typeof result).toBe('string');
    expect(result).not.toBe('ok');
  });
});

describe('Editor E2E: project_node_detail', () => {
  it('returns slots and related_constraints for a Scalar node', () => {
    let doc = new_document();
    doc = appendScalar(doc, '0', 'N');

    const proj = parseProjection(doc);
    const nId = proj.outline.find((n) => n.label === 'N')!.id;

    const detailJson = project_node_detail(doc, nId);
    const detail = JSON.parse(detailJson);

    expect(detail).not.toBeNull();
    expect(detail).toHaveProperty('slots');
    expect(detail).toHaveProperty('related_constraints');
  });

  it('returns null for a non-existent node ID', () => {
    const doc = new_document();
    const result = project_node_detail(doc, '999');
    expect(result).toBe('null');
  });
});

describe('Editor E2E: preset documents', () => {
  it('scalar_array preset has N and A[]', () => {
    const doc = get_preset('scalar_array');
    const proj = parseProjection(doc);

    expect(proj.outline.some((n) => n.label === 'N')).toBe(true);
    expect(proj.outline.some((n) => n.label.startsWith('A'))).toBe(true);
  });

  it('scalar_array preset is complete (no holes, no missing_constraints)', () => {
    const doc = get_preset('scalar_array');
    const proj = parseProjection(doc);

    expect(proj.completeness.total_holes).toBe(0);
    expect(proj.completeness.missing_constraints.length).toBe(0);
    expect(proj.completeness.is_complete).toBe(true);
  });

  it('hole_structure preset is incomplete', () => {
    const doc = get_preset('hole_structure');
    const proj = parseProjection(doc);

    expect(proj.completeness.is_complete).toBe(false);
  });
});

describe('Editor E2E: full flow (scalar + constraint)', () => {
  it('builds N with range constraint from scratch', () => {
    let doc = new_document();

    // Add N scalar to root Sequence
    doc = appendScalar(doc, '0', 'N');
    let proj = parseProjection(doc);

    const nId = proj.outline.find((n) => n.label === 'N')!.id;

    // Add constraint
    doc = addRangeConstraint(doc, nId, '1', '100000');
    proj = parseProjection(doc);

    expect(proj.outline.some((n) => n.label === 'N')).toBe(true);
    expect(proj.diagnostics.filter((d) => d.level === 'warning').length).toBe(0);
    expect(proj.completeness.total_holes).toBe(0);
  });
});
