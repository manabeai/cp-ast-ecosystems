/**
 * Build Action JSON strings for WASM dispatch.
 *
 * Maps UI popup state → Action JSON string for `apply_action()`.
 */

import type { Hotspot, ExprCandidate } from './editor-state';

// ── Type mapping ───────────────────────────────────────────────────

const TYPE_MAP: Record<string, string> = {
  number: 'Int',
  string: 'Str',
  char: 'Char',
};

function mapType(uiType: string): string {
  return TYPE_MAP[uiType] ?? 'Int';
}

// ── LengthSpec builders ────────────────────────────────────────────

interface LengthSpecRefVar {
  kind: 'RefVar';
  node_id: string;
}

interface LengthSpecExpr {
  kind: 'Expr';
  expr: string;
}

type LengthSpec = LengthSpecRefVar | LengthSpecExpr;

function buildLengthSpec(varName: string, availableVars: ExprCandidate[]): LengthSpec {
  const found = availableVars.find(v => v.name === varName);
  if (found) {
    return { kind: 'RefVar', node_id: found.node_id };
  }
  return { kind: 'Expr', expr: varName };
}

// ── FillContent builders ───────────────────────────────────────────

export interface FillContent {
  kind: string;
  [key: string]: unknown;
}

export function buildScalarFill(name: string, uiType: string): FillContent {
  return { kind: 'Scalar', name, typ: mapType(uiType) };
}

export function buildArrayFill(
  name: string,
  uiType: string,
  lengthVar: string,
  availableVars: ExprCandidate[],
): FillContent {
  return {
    kind: 'Array',
    name,
    element_type: mapType(uiType),
    length: buildLengthSpec(lengthVar, availableVars),
  };
}

export function buildGridTemplateFill(
  rowsVar: string,
  colsVar: string,
  availableVars: ExprCandidate[],
): FillContent {
  return {
    kind: 'GridTemplate',
    name: 'S',
    rows: buildLengthSpec(rowsVar, availableVars),
    cols: buildLengthSpec(colsVar, availableVars),
    cell_type: 'Char',
  };
}

export function buildEdgeListFill(countExpr: string, availableVars: ExprCandidate[] = []): FillContent {
  return {
    kind: 'EdgeList',
    edge_count: buildLengthSpec(countExpr, availableVars),
  };
}

export function buildWeightedEdgeListFill(
  countVar: string,
  weightName: string,
  uiType: string,
  availableVars: ExprCandidate[],
): FillContent {
  return {
    kind: 'WeightedEdgeList',
    edge_count: buildLengthSpec(countVar, availableVars),
    weight_name: weightName,
    weight_type: mapType(uiType),
  };
}

export function buildQueryListFill(
  countVar: string,
  availableVars: ExprCandidate[],
): FillContent {
  return {
    kind: 'QueryList',
    query_count: buildLengthSpec(countVar, availableVars),
  };
}

export function buildMultiTestCaseFill(
  countVar: string,
  availableVars: ExprCandidate[],
): FillContent {
  return {
    kind: 'MultiTestCaseTemplate',
    count: buildLengthSpec(countVar, availableVars),
  };
}

// ── Action JSON builders ───────────────────────────────────────────

export function buildAddSlotElement(
  parentId: string,
  slotName: string,
  element: FillContent,
): string {
  return JSON.stringify({
    action: 'AddSlotElement',
    parent: parentId,
    slot_name: slotName,
    element,
  });
}

export function buildAddSibling(targetId: string, element: FillContent): string {
  return JSON.stringify({
    action: 'AddSibling',
    target: targetId,
    element,
  });
}

export function buildFillHole(targetId: string, fill: FillContent): string {
  return JSON.stringify({
    action: 'FillHole',
    target: targetId,
    fill,
  });
}

export function buildAddChoiceVariant(
  choiceId: string,
  tagValue: number,
  firstName: string,
): string {
  return JSON.stringify({
    action: 'AddChoiceVariant',
    choice: choiceId,
    tag_value: { kind: 'IntLit', value: tagValue },
    first_element: { kind: 'Scalar', name: firstName, typ: 'Int' },
  });
}

export function buildAddConstraintRange(
  targetId: string,
  lower: string,
  upper: string,
): string {
  return JSON.stringify({
    action: 'AddConstraint',
    target: targetId,
    constraint: { kind: 'Range', lower, upper },
  });
}

export function buildAddConstraintProperty(targetId: string, tag: string): string {
  return JSON.stringify({
    action: 'AddConstraint',
    target: targetId,
    constraint: { kind: 'Property', tag },
  });
}

export function buildAddConstraintSumBound(
  targetId: string,
  overVar: string,
  upper: string,
): string {
  return JSON.stringify({
    action: 'AddConstraint',
    target: targetId,
    constraint: { kind: 'SumBound', over_var: overVar, upper },
  });
}

export function buildAddConstraintCharSet(targetId: string, spec: string): string {
  return JSON.stringify({
    action: 'AddConstraint',
    target: targetId,
    constraint: { kind: 'CharSet', spec },
  });
}

// ── Hotspot → Action routing ───────────────────────────────────────

/**
 * Determine the slot name for a below/inside hotspot based on the node type.
 * The projection gives us `parent_id` which may be a Sequence, Repeat, etc.
 * We infer the slot from the hotspot direction:
 * - below on Sequence → "children"
 * - below on Repeat → "body"
 * - inside on Repeat → FillHole (handled separately)
 */
export function buildHotspotAction(
  hotspot: Hotspot,
  element: FillContent,
  nodes: { id: string; label: string; is_hole: boolean; depth?: number }[],
): string {
  switch (hotspot.direction) {
    case 'below': {
      // The parent could be a Sequence (slot="children") or Repeat (slot="body").
      // We check if the label contains "Repeat" to decide.
      const parentNode = nodes.find(n => n.id === hotspot.parent_id);
      const slotName = parentNode?.label.includes('Repeat') ? 'body' : 'children';
      return buildAddSlotElement(hotspot.parent_id, slotName, element);
    }
    case 'right':
      return buildAddSibling(hotspot.parent_id, element);
    case 'inside': {
      // Find the hole node that is a child of this hotspot's parent.
      // In the projected node list, the hole will appear after the parent
      // and have depth == parent.depth + 1.
      const parentIdx = nodes.findIndex(n => n.id === hotspot.parent_id);
      const parentDepth = parentIdx >= 0 ? (nodes[parentIdx].depth ?? 0) : -1;
      let targetId = hotspot.parent_id;
      if (parentIdx >= 0) {
        for (let i = parentIdx + 1; i < nodes.length; i++) {
          const n = nodes[i];
          const d = n.depth ?? 0;
          if (d <= parentDepth) break; // left the parent's subtree
          if (n.is_hole && d === parentDepth + 1) {
            targetId = n.id;
            break;
          }
        }
      }
      return buildFillHole(targetId, element);
    }
    case 'variant':
      // Handled separately by buildAddChoiceVariant
      return '';
  }
}

/**
 * Build the FillContent for a popup confirmation based on the selected candidate.
 */
export function buildFillFromPopup(
  candidate: string,
  name: string,
  uiType: string,
  lengthVar: string,
  lengthVar2: string,
  weightName: string,
  countExpr: string,
  availableVars: ExprCandidate[],
): FillContent {
  switch (candidate) {
    case 'scalar':
      return buildScalarFill(name, uiType);
    case 'array':
      return buildArrayFill(name, uiType, lengthVar, availableVars);
    case 'grid-template':
      return buildGridTemplateFill(lengthVar, lengthVar2, availableVars);
    case 'edge-list':
      return buildEdgeListFill(countExpr, availableVars);
    case 'weighted-edge-list':
      return buildWeightedEdgeListFill(lengthVar, weightName, uiType, availableVars);
    case 'query-list':
      return buildQueryListFill(lengthVar, availableVars);
    case 'multi-testcase':
      return buildMultiTestCaseFill(lengthVar, availableVars);
    default:
      return buildScalarFill(name, uiType);
  }
}
