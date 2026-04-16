import { signal } from '@preact/signals';
import { editorDocumentJson } from '../../editor/state';
import { render_input_format, render_input_tex } from '../../wasm';

type PreviewMode = 'text' | 'tex';

const previewMode = signal<PreviewMode>('text');
const textFormat = signal<string>('');
const texFormat = signal<string>('');
const formatError = signal<string | null>(null);

export function CanonicalPreview() {
  const updatePreview = () => {
    const docJson = editorDocumentJson.value;
    if (!docJson) {
      textFormat.value = '';
      texFormat.value = '';
      formatError.value = 'No document loaded';
      return;
    }

    try {
      // Try to render both formats
      try {
        textFormat.value = render_input_format(docJson);
      } catch (error) {
        textFormat.value = `Error: ${error instanceof Error ? error.message : 'Format rendering failed'}`;
      }

      try {
        texFormat.value = render_input_tex(docJson);
      } catch (error) {
        texFormat.value = `Error: ${error instanceof Error ? error.message : 'TeX rendering failed'}`;
      }

      formatError.value = null;
    } catch (error) {
      textFormat.value = '';
      texFormat.value = '';
      formatError.value = error instanceof Error ? error.message : 'Preview generation failed';
    }
  };

  // Update preview when document changes
  if (editorDocumentJson.value && !textFormat.value && !texFormat.value && !formatError.value) {
    updatePreview();
  }

  const renderTexContent = () => {
    if (texFormat.value.startsWith('Error:')) {
      return <div class="tex-error">{texFormat.value}</div>;
    }

    // Try to render with KaTeX if available, otherwise show raw TeX
    try {
      // Check if KaTeX is available globally
      if (typeof window !== 'undefined' && (window as any).katex) {
        return (
          <div 
            class="tex-content"
            dangerouslySetInnerHTML={{
              __html: (window as any).katex.renderToString(texFormat.value, {
                displayMode: true,
                throwOnError: false
              })
            }}
          />
        );
      }
    } catch (error) {
      // Fall through to raw display
    }

    // Fallback to raw TeX
    return (
      <div class="tex-fallback">
        <div class="tex-fallback-label">TeX (KaTeX not available):</div>
        <pre>{texFormat.value}</pre>
      </div>
    );
  };

  return (
    <div class="canonical-preview">
      <div class="preview-controls">
        <div class="tab-buttons">
          <button 
            class={`tab-btn ${previewMode.value === 'text' ? 'active' : ''}`}
            onClick={() => previewMode.value = 'text'}
          >
            Text
          </button>
          <button 
            class={`tab-btn ${previewMode.value === 'tex' ? 'active' : ''}`}
            onClick={() => previewMode.value = 'tex'}
          >
            TeX
          </button>
        </div>
        <button class="preview-refresh-btn" onClick={updatePreview}>
          🔄 Refresh
        </button>
      </div>
      <div class="preview-content-container">
        {formatError.value ? (
          <div class="preview-error">
            <div class="error-icon">❌</div>
            <div class="error-message">{formatError.value}</div>
          </div>
        ) : previewMode.value === 'text' ? (
          <pre class="text-format-output">{textFormat.value || 'No text format generated'}</pre>
        ) : (
          <div class="tex-format-output">
            {texFormat.value ? renderTexContent() : 'No TeX format generated'}
          </div>
        )}
      </div>
    </div>
  );
}