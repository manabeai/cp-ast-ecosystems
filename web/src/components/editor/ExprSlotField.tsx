import { signal } from '@preact/signals';
import { useState, useEffect } from 'preact/hooks';
import { editorDocumentJson } from '../../editor/state';
import { dispatchAction } from '../../editor/actions';
import { get_expr_candidates } from '../../wasm';
import type { ExprCandidateMenu, SlotKind, SlotId, ExpressionInput } from '../../editor/types';

interface Props {
  nodeId: string;
  slotKind: SlotKind;
  currentExpr?: string;
}

export function ExprSlotField({ nodeId, slotKind, currentExpr }: Props) {
  const [isOpen, setIsOpen] = useState(false);
  const [candidates, setCandidates] = useState<ExprCandidateMenu | null>(null);
  const [loading, setLoading] = useState(false);

  const openCandidates = async () => {
    const doc = editorDocumentJson.value;
    if (!doc) return;

    setLoading(true);
    try {
      const result = get_expr_candidates(doc);
      const menu = JSON.parse(result) as ExprCandidateMenu;
      setCandidates(menu);
      setIsOpen(true);
    } catch (e) {
      console.error('Failed to get expression candidates:', e);
      setCandidates(null);
    } finally {
      setLoading(false);
    }
  };

  const closeCandidates = () => {
    setIsOpen(false);
    setCandidates(null);
  };

  const selectLiteral = (value: number) => {
    const slotId: SlotId = { owner: nodeId, kind: slotKind };
    const expr: ExpressionInput = { kind: 'Lit', value };
    
    const success = dispatchAction({
      kind: 'SetExpr',
      slot: slotId,
      expr
    });

    if (success) {
      closeCandidates();
    }
  };

  const selectReference = (refNodeId: string) => {
    const slotId: SlotId = { owner: nodeId, kind: slotKind };
    const expr: ExpressionInput = { 
      kind: 'Var', 
      reference: { kind: 'VariableRef', node_id: refNodeId } 
    };
    
    const success = dispatchAction({
      kind: 'SetExpr',
      slot: slotId,
      expr
    });

    if (success) {
      closeCandidates();
    }
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest('.expr-slot-field')) {
        closeCandidates();
      }
    };

    if (isOpen) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [isOpen]);

  return (
    <div class="expr-slot-field">
      <button 
        class="expr-slot-button" 
        onClick={openCandidates}
        disabled={loading}
      >
        {loading ? '…' : (currentExpr || 'Set expression')}
      </button>

      {isOpen && candidates && (
        <div class="expr-candidates-popup">
          <div class="candidates-header">
            Select Expression
            <button class="candidates-close-btn" onClick={closeCandidates}>
              ×
            </button>
          </div>

          {candidates.references.length > 0 && (
            <div class="candidate-group">
              <div class="candidate-kind">Variables</div>
              <div class="candidate-list">
                {candidates.references.map((ref) => (
                  <button
                    key={ref.node_id}
                    class="candidate-item"
                    onClick={() => selectReference(ref.node_id)}
                  >
                    {ref.label}
                  </button>
                ))}
              </div>
            </div>
          )}

          {candidates.literals.length > 0 && (
            <div class="candidate-group">
              <div class="candidate-kind">Literals</div>
              <div class="candidate-list">
                {candidates.literals.map((value) => (
                  <button
                    key={value}
                    class="candidate-item"
                    onClick={() => selectLiteral(value)}
                  >
                    {value}
                  </button>
                ))}
              </div>
            </div>
          )}

          {candidates.references.length === 0 && candidates.literals.length === 0 && (
            <div class="empty-candidates">
              No expression candidates available
            </div>
          )}
        </div>
      )}
    </div>
  );
}