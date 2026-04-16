// Types matching the JSON shape from wasm DTOs
// Note: JSON uses snake_case, so we use snake_case here too

export interface FullProjection {
  outline: OutlineNode[];
  diagnostics: Diagnostic[];
  completeness: CompletenessInfo;
}

export interface OutlineNode {
  id: string;
  label: string;
  kind_label: string;
  depth: number;
  is_hole: boolean;
  child_ids: string[];
}

export interface Diagnostic {
  level: 'error' | 'warning' | 'info';
  message: string;
  node_id?: string;
  constraint_id?: string;
}

export interface CompletenessInfo {
  total_holes: number;
  is_complete: boolean;
  missing_constraints: string[];
}

export interface NodeDetailProjection {
  slots: SlotInfo[];
  related_constraints: ConstraintSummary[];
}

export interface SlotInfo {
  kind: SlotKind;
  current_expr?: string;
  is_editable: boolean;
}

export type SlotKind =
  | 'ArrayLength'
  | 'RepeatCount'
  | 'RangeLower'
  | 'RangeUpper'
  | 'RelationLhs'
  | 'RelationRhs'
  | 'LengthLength';

export interface SlotId {
  owner: string;
  kind: SlotKind;
}

export interface ConstraintSummary {
  id: string;
  label: string;
  kind_label: string;
}

export interface HoleCandidate {
  kind: string;
  suggested_names: string[];
}

export interface ExprCandidateMenu {
  references: ReferenceCandidate[];
  literals: number[];
}

export interface ReferenceCandidate {
  node_id: string;
  label: string;
}

export interface ConstraintTargetMenu {
  targets: ReferenceCandidate[];
}

// Action types for dispatch
export type EditorAction =
  | { kind: 'FillHole'; target: string; fill: FillContent }
  | { kind: 'ReplaceNode'; target: string; replacement: FillContent }
  | { kind: 'AddConstraint'; target: string; constraint: ConstraintDef }
  | { kind: 'RemoveConstraint'; constraint_id: string }
  | { kind: 'AddSlotElement'; parent: string; slot_name: string; element: FillContent }
  | { kind: 'RemoveSlotElement'; parent: string; slot_name: string; child: string }
  | { kind: 'SetExpr'; slot: SlotId; expr: ExpressionInput };

export type FillContent =
  | { kind: 'Scalar'; name: string; typ: string }
  | { kind: 'Array'; name: string; element_type: string; length: LengthSpec }
  | { kind: 'Section'; label: string };

export type LengthSpec =
  | { kind: 'Fixed'; value: number }
  | { kind: 'RefVar'; node_id: string }
  | { kind: 'Expr'; value: string };

export interface ConstraintDef {
  kind: ConstraintDefKind;
}

export type ConstraintDefKind =
  | { kind: 'Range'; lower: string; upper: string }
  | { kind: 'TypeDecl'; typ: string }
  | { kind: 'Relation'; op: string; rhs: string }
  | { kind: 'Distinct' }
  | { kind: 'Sorted'; order: string };

export type ExpressionInput =
  | { kind: 'Lit'; value: number }
  | { kind: 'Var'; reference: { kind: 'VariableRef'; node_id: string } };

// Draft constraint for the constraint builder
export type DraftConstraint =
  | { kind: 'Range'; target: string | null; lower: string; upper: string }
  | { kind: 'TypeDecl'; target: string | null; expected_type: string }
  | { kind: 'LengthRelation'; target: string | null; length: string }
  | { kind: 'Relation'; lhs: string; op: string; rhs: string };