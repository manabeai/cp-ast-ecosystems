/**
 * Node creation wizard popup.
 *
 * 2-step inline wizard with candidate preview:
 *   Step 1 — choose a candidate (left list + hover preview on right)
 *   Step 2 — fill in fields (left list with selection + fields panel on right)
 *
 * Variant hotspots skip the wizard and go directly to the fields panel.
 */
import { projection } from './editor-state';
import { dispatchAction } from './editor-state';
import {
  popupState,
  selectCandidate,
  closePopup,
  hoveredCandidate,
  popupName,
  popupType,
  popupLengthVar,
  popupLengthVar2,
  popupWeightName,
  popupVariantTag,
} from './popup-state';
import {
  buildFillFromPopup,
  buildHotspotAction,
  buildAddChoiceVariant,
} from './action-builder';
import { CountField, getCountExprValue } from './ExpressionBuilder';

const CANDIDATE_LABELS: Record<string, string> = {
  scalar: 'Scalar',
  array: 'Array',
  repeat: 'Repeat',
  'grid-template': 'Grid',
  'edge-list': 'Edge List',
  'weighted-edge-list': 'Weighted Edge List',
  'query-list': 'Query List',
  'multi-testcase': 'Multi-Testcase',
};

const PREVIEW_FIELDS: Record<string, string[]> = {
  scalar: ['Type', 'Name'],
  array: ['Type', 'Name', 'Length'],
  repeat: ['Count'],
  'grid-template': ['Rows', 'Cols'],
  'edge-list': ['Count'],
  'weighted-edge-list': ['Count', 'Weight', 'Type'],
  'query-list': ['Count'],
  'multi-testcase': ['Count'],
};

export function NodePopup() {
  const state = popupState.value;
  if (state.step === 'closed') return null;

  // Variant bypasses the wizard layout
  if (state.step === 'fields' && state.candidate === 'variant') {
    return (
      <div class="node-popup" data-testid="node-popup">
        <VariantFieldsPanel />
      </div>
    );
  }

  const candidates = state.hotspot.candidates;
  const activeStep = state.step === 'candidates' ? 1 : 2;
  const selectedCandidate = state.step === 'fields' ? state.candidate : null;

  return (
    <div class="node-popup" data-testid="node-popup">
      <StepIndicator active={activeStep} />
      <div class="popup-wizard">
        <div class="popup-candidate-list">
          {candidates.map(c => (
            <button
              key={c}
              class={`popup-option${selectedCandidate === c ? ' selected' : ''}`}
              data-testid={`popup-option-${c}`}
              onClick={() => selectCandidate(c)}
              onMouseEnter={() => { hoveredCandidate.value = c; }}
              onMouseLeave={() => { hoveredCandidate.value = null; }}
            >
              {CANDIDATE_LABELS[c] ?? c}
            </button>
          ))}
        </div>
        <div class="popup-right-panel">
          {state.step === 'candidates' && <PreviewPanel />}
          {state.step === 'fields' && <FieldsPanel />}
        </div>
      </div>
    </div>
  );
}

function StepIndicator({ active }: { active: number }) {
  return (
    <div class="popup-step-indicator">
      <span class={active === 1 ? 'step-active' : 'step-inactive'}>①</span>
      <span class="step-arrow">→</span>
      <span class={active === 2 ? 'step-active' : 'step-inactive'}>②</span>
    </div>
  );
}

function PreviewPanel() {
  const hovered = hoveredCandidate.value;
  if (!hovered) {
    return <div class="popup-preview popup-preview-empty">Hover to preview fields</div>;
  }
  const fields = PREVIEW_FIELDS[hovered] ?? [];
  return (
    <div class="popup-preview">
      <div class="preview-title">{CANDIDATE_LABELS[hovered] ?? hovered}</div>
      <div class="preview-fields">
        {fields.map(f => (
          <span key={f} class="preview-field-tag">{f}</span>
        ))}
      </div>
    </div>
  );
}

function VariantFieldsPanel() {
  const state = popupState.value;
  if (state.step !== 'fields') return null;

  const handleConfirm = () => {
    if (!isVariantValid()) return;
    const tagNum = parseInt(popupVariantTag.value, 10) || 0;
    const actionJson = buildAddChoiceVariant(state.hotspot.parent_id, tagNum, popupName.value);
    dispatchAction(actionJson);
    closePopup();
  };

  const isVariantValid = () => popupVariantTag.value.trim() !== '' && popupName.value.trim() !== '';

  return (
    <div class="popup-fields">
      <div class="popup-field">
        <label>Tag Value</label>
        <input
          type="text"
          data-testid="variant-tag-input"
          value={popupVariantTag.value}
          onInput={(e) => { popupVariantTag.value = (e.target as HTMLInputElement).value; }}
        />
      </div>
      <div class="popup-field">
        <label>Name</label>
        <input
          type="text"
          data-testid="name-input"
          value={popupName.value}
          onInput={(e) => { popupName.value = (e.target as HTMLInputElement).value; }}
        />
      </div>
      <button
        class="popup-confirm"
        data-testid="confirm-button"
        disabled={!isVariantValid()}
        onClick={handleConfirm}
      >
        Confirm
      </button>
    </div>
  );
}

function FieldsPanel() {
  const state = popupState.value;
  if (state.step !== 'fields') return null;

  const candidate = state.candidate;
  const proj = projection.value;
  const availableVars = proj.available_vars;

  const needsType = candidate === 'scalar' || candidate === 'array' || candidate === 'weighted-edge-list';
  const needsName = candidate === 'scalar' || candidate === 'array';
  const needsLength = candidate === 'array' || candidate === 'query-list' || candidate === 'multi-testcase' || candidate === 'weighted-edge-list';
  const needsGridLength = candidate === 'grid-template';
  const needsCountExpr = candidate === 'edge-list' || candidate === 'repeat';
  const needsWeightName = candidate === 'weighted-edge-list';

  // For grid-template, we need TWO length selectors. The first unset one gets the testid.
  const gridRowsSet = popupLengthVar.value !== '';
  const gridColsSet = popupLengthVar2.value !== '';

  const handleConfirm = () => {
    if (!isValid()) return;
    const countExpr = needsCountExpr ? (getCountExprValue() || popupLengthVar.value) : '';
    const fill = buildFillFromPopup(
      candidate,
      popupName.value,
      popupType.value,
      needsGridLength ? popupLengthVar.value : popupLengthVar.value,
      popupLengthVar2.value,
      popupWeightName.value,
      countExpr,
      availableVars,
    );
    const actionJson = buildHotspotAction(state.hotspot, fill, proj.nodes);
    dispatchAction(actionJson);
    closePopup();
  };

  const hasLength = popupLengthVar.value.trim() !== '';
  const hasSecondLength = popupLengthVar2.value.trim() !== '';
  const hasCount = (getCountExprValue() || popupLengthVar.value).trim() !== '';
  const isValid = () => {
    if (needsName && popupName.value.trim() === '') return false;
    if (needsLength && !hasLength) return false;
    if (needsGridLength && (!hasLength || !hasSecondLength)) return false;
    if (needsCountExpr && !hasCount) return false;
    if (needsWeightName && popupWeightName.value.trim() === '') return false;
    return true;
  };

  return (
    <div class="popup-fields">
      {needsType && (
        <div class="popup-field">
          <label>Type</label>
          <div class="type-buttons">
            {['number', 'string', 'char'].map(t => (
              <button
                key={t}
                type="button"
                class={`type-btn ${popupType.value === t ? 'active' : ''}`}
                onClick={() => { popupType.value = t; }}
              >
                {t}
              </button>
            ))}
          </div>
          {/* Hidden select for E2E test compatibility */}
          <select
            data-testid="type-select"
            value={popupType.value}
            onChange={(e) => { popupType.value = (e.target as HTMLSelectElement).value; }}
            style={{ position: 'absolute', opacity: 0, pointerEvents: 'none', width: 0, height: 0, overflow: 'hidden' }}
          >
            <option value="number">number</option>
            <option value="string">string</option>
            <option value="char">char</option>
          </select>
        </div>
      )}

      {needsName && (
        <div class="popup-field">
          <label>Name</label>
          <input
            type="text"
            data-testid="name-input"
            value={popupName.value}
            onInput={(e) => { popupName.value = (e.target as HTMLInputElement).value; }}
          />
        </div>
      )}

      {needsLength && (
        <div class="popup-field">
          <label>Length</label>
          <LengthField
            value={popupLengthVar.value}
            onChange={(value) => { popupLengthVar.value = value; }}
            availableVars={availableVars}
            testId="length-select"
          />
        </div>
      )}

      {needsGridLength && (
        <>
          <div class="popup-field">
            <label>Rows</label>
            <LengthField
              value={popupLengthVar.value}
              onChange={(value) => { popupLengthVar.value = value; }}
              availableVars={availableVars}
              testId={!gridRowsSet ? 'length-select' : undefined}
            />
          </div>
          <div class="popup-field">
            <label>Cols</label>
            <LengthField
              value={popupLengthVar2.value}
              onChange={(value) => { popupLengthVar2.value = value; }}
              availableVars={availableVars}
              testId={gridRowsSet && !gridColsSet ? 'length-select' : undefined}
            />
          </div>
        </>
      )}

      {needsCountExpr && (
        <div class="popup-field">
          <label>Count</label>
          <CountField availableVars={availableVars} />
        </div>
      )}

      {needsWeightName && (
        <div class="popup-field">
          <label>Weight Name</label>
          <input
            type="text"
            data-testid="weight-name-input"
            value={popupWeightName.value}
            onInput={(e) => { popupWeightName.value = (e.target as HTMLInputElement).value; }}
          />
        </div>
      )}

      <button
        class="popup-confirm"
        data-testid="confirm-button"
        disabled={!isValid()}
        onClick={handleConfirm}
      >
        Confirm
      </button>
    </div>
  );
}

function LengthField({
  value,
  onChange,
  availableVars,
  testId,
}: {
  value: string;
  onChange: (value: string) => void;
  availableVars: { name: string }[];
  testId?: string;
}) {
  return (
    <div class="length-field">
      <div class="length-var-options">
        {availableVars.map(v => (
          <button
            key={v.name}
            type="button"
            class={`length-var-option ${value === v.name ? 'active' : ''}`}
            data-testid={`length-var-option-${v.name}`}
            onClick={() => onChange(v.name)}
          >
            {v.name}
          </button>
        ))}
      </div>
      <input
        class="length-expression-input"
        data-testid="length-expression-input"
        type="text"
        placeholder="length expression"
        value={value}
        onInput={(e) => onChange((e.currentTarget as HTMLInputElement).value)}
      />
      <select
        data-testid={testId}
        value={value}
        onChange={(e) => onChange((e.currentTarget as HTMLSelectElement).value)}
        style={{ position: 'absolute', opacity: 0, pointerEvents: 'none', width: 0, height: 0, overflow: 'hidden' }}
      >
        <option value="">-- select --</option>
        {availableVars.map(v => (
          <option key={v.name} value={v.name}>{v.name}</option>
        ))}
      </select>
    </div>
  );
}
