/**
 * Preview pane: renders TeX input format, TeX constraints, and sample output.
 */
import { inputTexString, constraintsTexString, sampleText, shuffleSeed } from './editor-state';
import { renderInputTex, renderConstraintsTex } from '../tex-renderer';
import { previewFolded, togglePreviewFold } from './fold-state';

export function PreviewPane() {
  const inputTex = inputTexString.value;
  const constraintsTex = constraintsTexString.value;
  const sample = sampleText.value;
  const folded = previewFolded.value;

  return (
    <div class={`pane ${folded ? 'folded' : ''}`} data-testid="preview-pane">
      <div class="pane-header">
        <span class="pane-title">Preview</span>
        <div class="pane-header-controls">
          <button class="toggle-btn" onClick={() => shuffleSeed()}>
            🎲 Resample
          </button>
          <button class="fold-toggle" onClick={togglePreviewFold} aria-label={folded ? 'Expand' : 'Collapse'}>
            {folded ? '▶' : '▼'}
          </button>
        </div>
      </div>
      <div class="pane-content-scroll">
        <div class="tex-section">
          <div class="tex-section-label">Input Format</div>
          <div
            data-testid="tex-input-format"
            dangerouslySetInnerHTML={{ __html: renderInputTex(inputTex) }}
          />
        </div>
        <div class="tex-section">
          <div class="tex-section-label">Constraints</div>
          <div
            data-testid="tex-constraints"
            dangerouslySetInnerHTML={{ __html: renderConstraintsTex(constraintsTex) }}
          />
        </div>
        <div class="tex-section">
          <div class="tex-section-label">Sample</div>
          <pre class="sample-output" data-testid="sample-output">{sample}</pre>
        </div>
      </div>
    </div>
  );
}
