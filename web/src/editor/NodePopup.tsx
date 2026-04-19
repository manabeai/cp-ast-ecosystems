/**
 * Node creation wizard popup.
 *
 * Appears when a hotspot is clicked, shows candidate options and input fields.
 */
import { projection } from './editor-state';
import { dispatchAction } from './editor-state';
import {
  popupState,
  selectCandidate,
  closePopup,
  popupName,
  popupType,
  popupLengthVar,
  popupLengthVar2,
  popupWeightName,
  popupVariantTag,
  countExprState,
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
  'grid-template': 'Grid',
  'edge-list': 'Edge List',
  'weighted-edge-list': 'Weighted Edge List',
  'query-list': 'Query List',
  'multi-testcase': 'Multi-Testcase',
};

export function NodePopup() {
  const state = popupState.value;
  if (state.step === 'closed') return null;

  return (
    <div class="node-popup" data-testid="node-popup">
      {state.step === 'candidates' && <CandidateList candidates={state.hotspot.candidates} />}
      {state.step === 'fields' && <FieldsPanel />}
    </div>
  );
}

function CandidateList({ candidates }: { candidates: string[] }) {
  return (
    <div class="popup-candidates">
      {candidates.map(c => (
        <button
          key={c}
          class="popup-option"
          data-testid={`popup-option-${c}`}
          onClick={() => selectCandidate(c)}
        >
          {CANDIDATE_LABELS[c] ?? c}
        </button>
      ))}
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
  const needsCountExpr = candidate === 'edge-list';
  const needsWeightName = candidate === 'weighted-edge-list';
  const isVariant = candidate === 'variant';

  // For grid-template, we need TWO length selectors. The first unset one gets the testid.
  const gridRowsSet = popupLengthVar.value !== '';
  const gridColsSet = popupLengthVar2.value !== '';

  const handleConfirm = () => {
    if (isVariant) {
      const tagNum = parseInt(popupVariantTag.value, 10) || 0;
      const actionJson = buildAddChoiceVariant(state.hotspot.parent_id, tagNum, popupName.value);
      dispatchAction(actionJson);
      closePopup();
      return;
    }

    const countExpr = needsCountExpr ? getCountExprValue() : '';
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

  return (
    <div class="popup-fields">
      {isVariant && (
        <div class="popup-field">
          <label>Tag Value</label>
          <input
            type="text"
            data-testid="variant-tag-input"
            value={popupVariantTag.value}
            onInput={(e) => { popupVariantTag.value = (e.target as HTMLInputElement).value; }}
          />
        </div>
      )}

      {needsType && (
        <div class="popup-field">
          <label>Type</label>
          <select
            data-testid="type-select"
            value={popupType.value}
            onChange={(e) => { popupType.value = (e.target as HTMLSelectElement).value; }}
          >
            <option value="number">number</option>
            <option value="string">string</option>
            <option value="char">char</option>
          </select>
        </div>
      )}

      {(needsName || isVariant) && (
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
          <select
            data-testid="length-select"
            value={popupLengthVar.value}
            onChange={(e) => { popupLengthVar.value = (e.target as HTMLSelectElement).value; }}
          >
            <option value="">-- select --</option>
            {availableVars.map(v => (
              <option key={v.name} value={v.name}>{v.name}</option>
            ))}
          </select>
        </div>
      )}

      {needsGridLength && (
        <>
          <div class="popup-field">
            <label>Rows</label>
            <select
              data-testid={!gridRowsSet ? 'length-select' : undefined}
              value={popupLengthVar.value}
              onChange={(e) => { popupLengthVar.value = (e.target as HTMLSelectElement).value; }}
            >
              <option value="">-- select --</option>
              {availableVars.map(v => (
                <option key={v.name} value={v.name}>{v.name}</option>
              ))}
            </select>
          </div>
          <div class="popup-field">
            <label>Cols</label>
            <select
              data-testid={gridRowsSet && !gridColsSet ? 'length-select' : undefined}
              value={popupLengthVar2.value}
              onChange={(e) => { popupLengthVar2.value = (e.target as HTMLSelectElement).value; }}
            >
              <option value="">-- select --</option>
              {availableVars.map(v => (
                <option key={v.name} value={v.name}>{v.name}</option>
              ))}
            </select>
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

      <button class="popup-confirm" data-testid="confirm-button" onClick={handleConfirm}>
        Confirm
      </button>
    </div>
  );
}
