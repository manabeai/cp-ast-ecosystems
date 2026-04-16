import { signal } from '@preact/signals';
import { editorDocumentJson } from '../../editor/state';
import { generate_sample } from '../../wasm';

const sampleSeed = signal<number>(Math.floor(Math.random() * 0xffffffff));
const sampleOutput = signal<string>('');
const sampleError = signal<string | null>(null);

export function SamplePreview() {
  const generateSample = () => {
    const docJson = editorDocumentJson.value;
    if (!docJson) {
      sampleOutput.value = '';
      sampleError.value = 'No document loaded';
      return;
    }

    try {
      const output = generate_sample(docJson, sampleSeed.value);
      sampleOutput.value = output;
      sampleError.value = null;
    } catch (error) {
      sampleOutput.value = '';
      sampleError.value = error instanceof Error ? error.message : 'Sample generation failed';
    }
  };

  const newSeed = () => {
    sampleSeed.value = Math.floor(Math.random() * 0xffffffff);
    generateSample();
  };

  // Generate sample when component mounts or document changes
  if (editorDocumentJson.value && !sampleOutput.value && !sampleError.value) {
    generateSample();
  }

  return (
    <div class="sample-preview">
      <div class="sample-controls">
        <label class="sample-seed-label">
          Seed:
          <input 
            type="number" 
            class="sample-seed-input"
            value={sampleSeed.value}
            onInput={(e) => {
              const value = parseInt((e.target as HTMLInputElement).value, 10);
              if (!isNaN(value)) {
                sampleSeed.value = value;
                generateSample();
              }
            }}
          />
        </label>
        <button class="sample-regen-btn" onClick={newSeed}>
          🎲 New Seed
        </button>
        <button class="sample-regen-btn" onClick={generateSample}>
          🔄 Regenerate
        </button>
      </div>
      <div class="sample-output-container">
        {sampleError.value ? (
          <div class="sample-error">
            <div class="error-icon">❌</div>
            <div class="error-message">{sampleError.value}</div>
          </div>
        ) : (
          <pre class="sample-output">{sampleOutput.value || 'No sample generated'}</pre>
        )}
      </div>
    </div>
  );
}