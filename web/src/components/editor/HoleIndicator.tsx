import { signal } from '@preact/signals';
import { get_hole_candidates } from '../../wasm';
import { dispatchAction } from '../../editor/actions';
import { editorDocumentJson } from '../../editor/state';
import type { HoleCandidate, FillContent } from '../../editor/types';

interface HoleIndicatorProps {
  holeId: string;
}

export function HoleIndicator({ holeId }: HoleIndicatorProps) {
  const showCandidates = signal(false);
  const candidates = signal<HoleCandidate[]>([]);
  const loading = signal(false);

  const handleClick = async () => {
    if (!showCandidates.value) {
      loading.value = true;
      try {
        const candidatesJson = get_hole_candidates(editorDocumentJson.value, holeId);
        candidates.value = JSON.parse(candidatesJson) as HoleCandidate[];
        showCandidates.value = true;
      } catch (e) {
        console.error('Failed to get hole candidates:', e);
      } finally {
        loading.value = false;
      }
    } else {
      showCandidates.value = false;
    }
  };

  const selectCandidate = (candidate: HoleCandidate, suggectedName?: string) => {
    let fill: FillContent;
    
    switch (candidate.kind) {
      case 'Scalar':
        fill = {
          kind: 'Scalar',
          name: suggectedName || candidate.suggested_names[0] || 'var',
          typ: 'int', // Default type, could be made configurable
        };
        break;
      case 'Array':
        fill = {
          kind: 'Array',
          name: suggectedName || candidate.suggested_names[0] || 'arr',
          element_type: 'int', // Default type
          length: { kind: 'Fixed', value: 1 }, // Default length
        };
        break;
      case 'Section':
        fill = {
          kind: 'Section',
          label: suggectedName || candidate.suggested_names[0] || 'Section',
        };
        break;
      default:
        console.warn('Unknown candidate kind:', candidate.kind);
        return;
    }

    const success = dispatchAction({
      kind: 'FillHole',
      target: holeId,
      fill,
    });

    if (success) {
      showCandidates.value = false;
    }
  };

  return (
    <div class="hole-indicator">
      <button 
        class="hole-button" 
        onClick={handleClick}
        disabled={loading.value}
      >
        {loading.value ? '...' : '[?]'}
      </button>
      
      {showCandidates.value && (
        <div class="hole-candidates-popup">
          <div class="candidates-header">Fill hole:</div>
          {candidates.value.map((candidate, idx) => (
            <div key={idx} class="candidate-group">
              <div class="candidate-kind">{candidate.kind}</div>
              <div class="candidate-names">
                {candidate.suggested_names.map((name, nameIdx) => (
                  <button
                    key={nameIdx}
                    class="candidate-name-btn"
                    onClick={() => selectCandidate(candidate, name)}
                  >
                    {name}
                  </button>
                ))}
              </div>
            </div>
          ))}
          <button 
            class="candidates-close-btn"
            onClick={() => showCandidates.value = false}
          >
            ✕
          </button>
        </div>
      )}
    </div>
  );
}