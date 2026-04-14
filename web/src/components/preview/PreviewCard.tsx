import { useMemo } from 'preact/hooks';
import {
  get_preset,
  render_input_format,
  render_constraints_text,
  generate_sample,
} from '../../wasm';
import { loadPreset } from '../../state';

interface PresetInfo {
  name: string;
  description: string;
}

export function PreviewCard({ preset }: { preset: PresetInfo }) {
  const data = useMemo(() => {
    try {
      const json = get_preset(preset.name);
      return {
        structure: render_input_format(json),
        constraints: render_constraints_text(json),
        sample: generate_sample(json, 0),
      };
    } catch (e) {
      return {
        structure: `Error: ${e}`,
        constraints: '',
        sample: '',
      };
    }
  }, [preset.name]);

  const handleClick = () => {
    loadPreset(preset.name);
    window.location.hash = '#/viewer';
  };

  return (
    <div class="preview-card" onClick={handleClick}>
      <div class="card-header">
        <span class="card-title">📝 {preset.description}</span>
        <span class="card-name">{preset.name}</span>
      </div>
      <div class="card-section">
        <div class="card-section-label">Structure</div>
        <pre class="card-content">{data.structure}</pre>
      </div>
      <div class="card-section">
        <div class="card-section-label">Constraints</div>
        <pre class="card-content">{data.constraints}</pre>
      </div>
      <div class="card-section">
        <div class="card-section-label">Sample (seed=0)</div>
        <pre class="card-content">{data.sample}</pre>
      </div>
    </div>
  );
}
