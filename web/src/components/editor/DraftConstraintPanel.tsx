import { draftConstraint, editorDocumentJson } from '../../editor/state';
import { dispatchAction } from '../../editor/actions';
import { get_constraint_targets } from '../../wasm';
import type { DraftConstraint, ConstraintTargetMenu, ConstraintDef } from '../../editor/types';

export function DraftConstraintPanel() {
  const draft = draftConstraint.value;
  if (!draft) return null;

  const handleCancel = () => {
    draftConstraint.value = null;
  };

  const handleComplete = () => {
    if (!draft) return;

    let constraintDef: ConstraintDef;
    let targetId: string | null = null;

    switch (draft.kind) {
      case 'Range':
        if (!draft.target || !draft.lower || !draft.upper) return;
        constraintDef = {
          kind: {
            kind: 'Range',
            lower: draft.lower,
            upper: draft.upper,
          }
        };
        targetId = draft.target;
        break;
      case 'TypeDecl':
        if (!draft.target || !draft.expected_type) return;
        constraintDef = {
          kind: {
            kind: 'TypeDecl',
            typ: draft.expected_type,
          }
        };
        targetId = draft.target;
        break;
      case 'LengthRelation':
        if (!draft.target || !draft.length) return;
        // LengthRelation doesn't map directly to current ConstraintDefKind
        // For now, we'll treat it as a relation
        constraintDef = {
          kind: {
            kind: 'Relation',
            op: '=',
            rhs: draft.length,
          }
        };
        targetId = draft.target;
        break;
      case 'Relation':
        if (!draft.lhs || !draft.op || !draft.rhs) return;
        constraintDef = {
          kind: {
            kind: 'Relation',
            op: draft.op,
            rhs: `${draft.lhs} ${draft.op} ${draft.rhs}`,
          }
        };
        // For relations, we might need a different target strategy
        targetId = 'root'; // fallback
        break;
      default:
        return;
    }

    const success = dispatchAction({
      kind: 'AddConstraint',
      target: targetId,
      constraint: constraintDef,
    });

    if (success) {
      draftConstraint.value = null;
    }
  };

  const getTargetOptions = (): { label: string; value: string }[] => {
    try {
      const targetMenuJson = get_constraint_targets(editorDocumentJson.value);
      const targetMenu: ConstraintTargetMenu = JSON.parse(targetMenuJson);
      return targetMenu.targets.map(target => ({
        label: target.label,
        value: target.node_id,
      }));
    } catch (e) {
      console.error('Failed to get constraint targets:', e);
      return [];
    }
  };

  const updateDraft = (field: string, value: any) => {
    if (!draft) return;
    draftConstraint.value = { ...draft, [field]: value };
  };

  const targetOptions = getTargetOptions();

  return (
    <div class="draft-constraint-panel">
      <div class="draft-panel-header">
        <span class="draft-panel-title">Add {draft.kind} Constraint</span>
        <button class="draft-panel-cancel" onClick={handleCancel}>
          ✕
        </button>
      </div>
      
      <div class="draft-panel-content">
        {/* Target field for Range, TypeDecl, LengthRelation */}
        {(draft.kind === 'Range' || draft.kind === 'TypeDecl' || draft.kind === 'LengthRelation') && (
          <div class="draft-field">
            <label class="draft-field-label">Target</label>
            <select
              class="draft-field-select"
              value={draft.target || ''}
              onChange={(e) => updateDraft('target', (e.target as HTMLSelectElement).value || null)}
            >
              <option value="">Select target...</option>
              {targetOptions.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        )}

        {/* Range fields */}
        {draft.kind === 'Range' && (
          <>
            <div class="draft-field">
              <label class="draft-field-label">Lower bound</label>
              <input
                class="draft-field-input"
                type="text"
                value={draft.lower}
                onChange={(e) => updateDraft('lower', (e.target as HTMLInputElement).value)}
                placeholder="e.g., 1"
              />
            </div>
            <div class="draft-field">
              <label class="draft-field-label">Upper bound</label>
              <input
                class="draft-field-input"
                type="text"
                value={draft.upper}
                onChange={(e) => updateDraft('upper', (e.target as HTMLInputElement).value)}
                placeholder="e.g., 10^5"
              />
            </div>
          </>
        )}

        {/* TypeDecl fields */}
        {draft.kind === 'TypeDecl' && (
          <div class="draft-field">
            <label class="draft-field-label">Expected Type</label>
            <select
              class="draft-field-select"
              value={draft.expected_type}
              onChange={(e) => updateDraft('expected_type', (e.target as HTMLSelectElement).value)}
            >
              <option value="int">Integer</option>
              <option value="float">Float</option>
              <option value="string">String</option>
            </select>
          </div>
        )}

        {/* LengthRelation fields */}
        {draft.kind === 'LengthRelation' && (
          <div class="draft-field">
            <label class="draft-field-label">Length expression</label>
            <input
              class="draft-field-input"
              type="text"
              value={draft.length}
              onChange={(e) => updateDraft('length', (e.target as HTMLInputElement).value)}
              placeholder="e.g., N"
            />
          </div>
        )}

        {/* Relation fields */}
        {draft.kind === 'Relation' && (
          <>
            <div class="draft-field">
              <label class="draft-field-label">Left side</label>
              <input
                class="draft-field-input"
                type="text"
                value={draft.lhs}
                onChange={(e) => updateDraft('lhs', (e.target as HTMLInputElement).value)}
                placeholder="e.g., N"
              />
            </div>
            <div class="draft-field">
              <label class="draft-field-label">Operator</label>
              <select
                class="draft-field-select"
                value={draft.op}
                onChange={(e) => updateDraft('op', (e.target as HTMLSelectElement).value)}
              >
                <option value="=">=</option>
                <option value="<">&lt;</option>
                <option value="≤">≤</option>
                <option value=">">&gt;</option>
                <option value="≥">≥</option>
              </select>
            </div>
            <div class="draft-field">
              <label class="draft-field-label">Right side</label>
              <input
                class="draft-field-input"
                type="text"
                value={draft.rhs}
                onChange={(e) => updateDraft('rhs', (e.target as HTMLInputElement).value)}
                placeholder="e.g., 10^5"
              />
            </div>
          </>
        )}
      </div>

      <div class="draft-panel-actions">
        <button class="draft-action-complete" onClick={handleComplete}>
          Add Constraint
        </button>
        <button class="draft-action-cancel" onClick={handleCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}