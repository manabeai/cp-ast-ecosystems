/**
 * Constraint editor: editing a draft Range constraint (lower/upper bounds).
 *
 * Supports bound editing with ValueInput popup and function application.
 */
import {
  constraintLower,
  constraintUpper,
  openValueInput,
  valueInputState,
  boundExprState,
  openBoundFnSelect,
  selectBoundFnOp,
  applyBoundFnOperand,
  type ValueInputTarget,
} from './popup-state';
import { ValueInput, isValueInputOpen } from './ValueInput';
import { FunctionOpsPanel, FunctionOperandInput } from './ExpressionBuilder';

interface ConstraintEditorProps {
  targetId: string;
  targetName: string;
  onConfirm: (lower: string, upper: string) => void;
}

export function ConstraintEditor({ targetName, onConfirm }: ConstraintEditorProps) {
  const lower = constraintLower.value;
  const upper = constraintUpper.value;

  return (
    <div class="constraint-editor">
      <div class="constraint-editor-label">
        Constraint for <strong>{targetName}</strong>
      </div>

      <div class="constraint-bound-row">
        <BoundArea label="Lower" target="lower" value={lower} />
        <span class="constraint-sep">≤ {targetName} ≤</span>
        <BoundArea label="Upper" target="upper" value={upper} />
      </div>

      <BoundExpressionUI />

      <button
        class="constraint-confirm-btn"
        data-testid="constraint-confirm"
        onClick={() => onConfirm(lower, upper)}
        disabled={!lower || !upper}
      >
        Confirm Constraint
      </button>
    </div>
  );
}

function BoundArea({ label, target, value }: { label: string; target: ValueInputTarget; value: string }) {
  const isOpen = isValueInputOpen(target);

  return (
    <div class="bound-area">
      <div
        class="bound-input"
        data-testid={`constraint-${target}-input`}
        onClick={() => {
          if (!value) openValueInput(target);
        }}
      >
        {value ? (
          <span
            class="bound-expression"
            data-testid={`constraint-${target}-expression`}
            onClick={(e) => {
              e.stopPropagation();
              openBoundFnSelect(target);
            }}
          >
            {value}
          </span>
        ) : (
          <span class="bound-placeholder">{label}...</span>
        )}
      </div>
      {isOpen && <ValueInput target={target} />}
    </div>
  );
}

function BoundExpressionUI() {
  const state = boundExprState.value;
  if (state.step === 'idle') return null;

  return (
    <div class="bound-expr-ui">
      {state.step === 'fn-select' && (
        <FunctionOpsPanel onSelectOp={selectBoundFnOp} />
      )}
      {state.step === 'fn-operand' && (
        <FunctionOperandInput onConfirm={applyBoundFnOperand} />
      )}
    </div>
  );
}
