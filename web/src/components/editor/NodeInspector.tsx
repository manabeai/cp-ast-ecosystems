import { signal, computed } from '@preact/signals';
import { editorDocumentJson } from '../../editor/state';
import { project_node_detail } from '../../wasm';
import type { NodeDetailProjection } from '../../editor/types';
import { NameField } from './NameField';
import { ExprSlotField } from './ExprSlotField';

interface Props {
  nodeId: string;
}

export function NodeInspector({ nodeId }: Props) {
  const nodeDetail = computed<NodeDetailProjection | null>(() => {
    const doc = editorDocumentJson.value;
    if (!doc) return null;
    
    try {
      const result = project_node_detail(doc, nodeId);
      if (result === '"null"' || result === 'null') return null;
      return JSON.parse(result) as NodeDetailProjection;
    } catch (e) {
      console.error('Node detail projection error:', e);
      return null;
    }
  });

  const detail = nodeDetail.value;
  if (!detail) {
    return (
      <div class="node-detail-error">
        Unable to load node details
      </div>
    );
  }

  const formatSlotKind = (kind: string): string => {
    switch (kind) {
      case 'ArrayLength': return 'Length';
      case 'RepeatCount': return 'Count';
      case 'RangeLower': return 'Lower bound';
      case 'RangeUpper': return 'Upper bound';
      case 'RelationLhs': return 'Left side';
      case 'RelationRhs': return 'Right side';
      case 'LengthLength': return 'Length';
      default: return kind;
    }
  };

  return (
    <div class="node-inspector">
      <div class="inspector-section">
        <h3 class="inspector-section-title">Node</h3>
        <div class="inspector-field">
          <NameField nodeId={nodeId} />
        </div>
      </div>

      {detail.slots.length > 0 && (
        <div class="inspector-section">
          <h3 class="inspector-section-title">Slots</h3>
          <div class="slot-list">
            {detail.slots.map((slot, index) => (
              <div key={index} class="slot-item">
                <label class="slot-label">
                  {formatSlotKind(slot.kind)}
                </label>
                <div class="slot-field">
                  {slot.is_editable ? (
                    <ExprSlotField
                      nodeId={nodeId}
                      slotKind={slot.kind}
                      currentExpr={slot.current_expr}
                    />
                  ) : (
                    <div class="slot-readonly">
                      {slot.current_expr || '—'}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {detail.related_constraints.length > 0 && (
        <div class="inspector-section">
          <h3 class="inspector-section-title">Related Constraints</h3>
          <div class="constraint-list">
            {detail.related_constraints.map((constraint) => (
              <div key={constraint.id} class="constraint-item">
                <div class="constraint-label">{constraint.label}</div>
                <div class="constraint-kind">{constraint.kind_label}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}