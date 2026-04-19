/**
 * Constraint pane: displays draft and completed constraints.
 *
 * Supports:
 * - Draft constraint editing (Range, CharSet)
 * - Property shortcut
 * - SumBound shortcut
 */
import { signal } from '@preact/signals';
import { projection, dispatchAction } from './editor-state';
import {
  constraintEditState,
  openConstraintEditor,
  closeConstraintEditor,
  openSumBound,
  sumBoundVar,
  sumBoundUpper,
  openValueInput,
  boundExprState,
  openBoundFnSelect,
  selectBoundFnOp,
  applyBoundFnOperand,
  charSetSelection,
} from './popup-state';
import {
  buildAddConstraintRange,
  buildAddConstraintProperty,
  buildAddConstraintSumBound,
  buildAddConstraintCharSet,
} from './action-builder';
import { ConstraintEditor } from './ConstraintEditor';
import { ValueInput, isValueInputOpen } from './ValueInput';
import { FunctionOpsPanel, FunctionOperandInput } from './ExpressionBuilder';

const showPropertyOptions = signal(false);

export function ConstraintPane() {
  const proj = projection.value;
  const editState = constraintEditState.value;

  const handleRangeConfirm = (lower: string, upper: string) => {
    if (editState.step === 'editing') {
      const actionJson = buildAddConstraintRange(editState.targetId, lower, upper);
      dispatchAction(actionJson);
      closeConstraintEditor();
    }
  };

  const handlePropertySelect = (tag: string) => {
    // Property applies to the root structure node
    const targetId = proj.nodes[0]?.id ?? '0';
    const actionJson = buildAddConstraintProperty(targetId, tag);
    dispatchAction(actionJson);
    showPropertyOptions.value = false;
  };

  const handleSumBoundConfirm = () => {
    if (sumBoundVar.value && sumBoundUpper.value) {
      const varCandidate = proj.available_vars.find(v => v.name === sumBoundVar.value);
      const targetId = varCandidate?.node_id ?? '0';
      const actionJson = buildAddConstraintSumBound(targetId, sumBoundVar.value, sumBoundUpper.value);
      dispatchAction(actionJson);
      closeConstraintEditor();
    }
  };

  const handleCharSetConfirm = () => {
    if (editState.step === 'charset' && charSetSelection.value) {
      const actionJson = buildAddConstraintCharSet(editState.targetId, charSetSelection.value);
      dispatchAction(actionJson);
      closeConstraintEditor();
    }
  };

  return (
    <div class="pane" data-testid="constraint-pane">
      <div class="pane-header">
        <span class="pane-title">Constraints</span>
        <div class="constraint-shortcuts">
          <button
            class="shortcut-btn"
            data-testid="property-shortcut"
            onClick={() => {
              closeConstraintEditor();
              showPropertyOptions.value = !showPropertyOptions.value;
            }}
          >
            Property
          </button>
          <button
            class="shortcut-btn"
            data-testid="sumbound-shortcut"
            onClick={() => {
              showPropertyOptions.value = false;
              openSumBound();
            }}
          >
            ΣBound
          </button>
        </div>
      </div>
      <div class="pane-content-scroll">
        {/* Property options (signal-driven visibility) */}
        {showPropertyOptions.value && (
          <PropertyOptions onSelect={handlePropertySelect} />
        )}

        {/* Draft constraints */}
        {proj.constraints.drafts.map(draft => (
          <div
            key={`draft-${draft.index}`}
            class={`constraint-item draft ${editState.step === 'editing' && editState.targetId === draft.target_id ? 'active' : ''}`}
            data-testid={`draft-constraint-${draft.index}`}
            onClick={() => {
              showPropertyOptions.value = false;
              openConstraintEditor(draft.target_id, draft.target_name, draft.template);
            }}
          >
            <span class="constraint-icon">○</span>
            <span class="constraint-display">{draft.display}</span>
          </div>
        ))}

        {/* Completed constraints */}
        {proj.constraints.completed.map(comp => (
          <div
            key={`completed-${comp.index}`}
            class="constraint-item completed"
            data-testid={`completed-constraint-${comp.index}`}
          >
            <span class="constraint-icon">●</span>
            <span class="constraint-display">{comp.display}</span>
          </div>
        ))}

        {/* Constraint Editor */}
        {editState.step === 'editing' && (
          <ConstraintEditor
            targetId={editState.targetId}
            targetName={editState.targetName}
            onConfirm={handleRangeConfirm}
          />
        )}

        {/* CharSet Editor: select charset then confirm */}
        {editState.step === 'charset' && (
          <CharSetEditor onConfirm={handleCharSetConfirm} />
        )}

        {/* SumBound Editor */}
        {editState.step === 'sumbound' && (
          <SumBoundEditor
            onConfirm={handleSumBoundConfirm}
          />
        )}
      </div>
    </div>
  );
}

function PropertyOptions({ onSelect }: { onSelect: (tag: string) => void }) {
  return (
    <div class="property-options visible">
      <button
        class="property-option"
        data-testid="property-option-tree"
        onClick={() => onSelect('Tree')}
      >
        Tree
      </button>
      <button
        class="property-option"
        data-testid="property-option-connected"
        onClick={() => onSelect('Connected')}
      >
        Connected
      </button>
      <button
        class="property-option"
        data-testid="property-option-simple"
        onClick={() => onSelect('Simple')}
      >
        Simple
      </button>
    </div>
  );
}

function CharSetEditor({ onConfirm }: { onConfirm: () => void }) {
  const selected = charSetSelection.value;

  return (
    <div class="charset-options">
      <div class="constraint-editor-label">Select Character Set</div>
      <button
        class={`charset-option ${selected === 'LowerAlpha' ? 'active' : ''}`}
        data-testid="charset-option-lowercase"
        onClick={() => { charSetSelection.value = 'LowerAlpha'; }}
      >
        a-z (lowercase)
      </button>
      <button
        class={`charset-option ${selected === 'UpperAlpha' ? 'active' : ''}`}
        data-testid="charset-option-uppercase"
        onClick={() => { charSetSelection.value = 'UpperAlpha'; }}
      >
        A-Z (uppercase)
      </button>
      <button
        class={`charset-option ${selected === 'Digit' ? 'active' : ''}`}
        data-testid="charset-option-digit"
        onClick={() => { charSetSelection.value = 'Digit'; }}
      >
        0-9 (digit)
      </button>
      <button
        class="constraint-confirm-btn"
        data-testid="constraint-confirm"
        onClick={onConfirm}
      >
        Confirm CharSet
      </button>
    </div>
  );
}

function SumBoundEditor({ onConfirm }: { onConfirm: () => void }) {
  const proj = projection.value;
  const upper = sumBoundUpper.value;
  const bExprState = boundExprState.value;

  return (
    <div class="sumbound-editor">
      <div class="constraint-editor-label">SumBound</div>
      <div class="sumbound-row">
        <label>Variable</label>
        <select
          data-testid="sumbound-var-select"
          value={sumBoundVar.value}
          onChange={(e) => { sumBoundVar.value = (e.target as HTMLSelectElement).value; }}
        >
          <option value="">-- select --</option>
          {proj.available_vars.map(v => (
            <option key={v.name} value={v.name}>{v.name}</option>
          ))}
        </select>
      </div>
      <div class="sumbound-row">
        <label>Upper Bound</label>
        <div
          class="bound-input"
          data-testid="sumbound-upper-input"
          onClick={() => {
            if (!upper) openValueInput('sumbound-upper');
          }}
        >
          {upper ? (
            <span
              class="bound-expression"
              data-testid="sumbound-upper-expression"
              onClick={(e) => {
                e.stopPropagation();
                openBoundFnSelect('sumbound-upper');
              }}
            >
              {upper}
            </span>
          ) : (
            <span class="bound-placeholder">upper...</span>
          )}
        </div>
        {isValueInputOpen('sumbound-upper') && <ValueInput target="sumbound-upper" />}
      </div>

      {bExprState.step === 'fn-select' && bExprState.target === 'sumbound-upper' && (
        <FunctionOpsPanel onSelectOp={selectBoundFnOp} />
      )}
      {bExprState.step === 'fn-operand' && bExprState.target === 'sumbound-upper' && (
        <FunctionOperandInput onConfirm={applyBoundFnOperand} />
      )}

      <button
        class="constraint-confirm-btn"
        data-testid="constraint-confirm"
        onClick={onConfirm}
      >
        Confirm SumBound
      </button>
    </div>
  );
}
