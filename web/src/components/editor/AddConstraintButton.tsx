import { signal } from '@preact/signals';
import { draftConstraint } from '../../editor/state';
import type { DraftConstraint } from '../../editor/types';

export function AddConstraintButton() {
  const showMenu = signal(false);

  const initializeDraftConstraint = (kind: DraftConstraint['kind']) => {
    switch (kind) {
      case 'Range':
        draftConstraint.value = {
          kind: 'Range',
          target: null,
          lower: '1',
          upper: '10^5',
        };
        break;
      case 'TypeDecl':
        draftConstraint.value = {
          kind: 'TypeDecl',
          target: null,
          expected_type: 'int',
        };
        break;
      case 'LengthRelation':
        draftConstraint.value = {
          kind: 'LengthRelation',
          target: null,
          length: 'N',
        };
        break;
      case 'Relation':
        draftConstraint.value = {
          kind: 'Relation',
          lhs: '',
          op: '=',
          rhs: '',
        };
        break;
    }
    showMenu.value = false;
  };

  return (
    <div class="add-constraint-menu">
      {!showMenu.value ? (
        <button 
          class="add-constraint-trigger"
          onClick={() => showMenu.value = true}
        >
          + Constraint
        </button>
      ) : (
        <div class="add-constraint-options">
          <button 
            class="add-constraint-option"
            onClick={() => initializeDraftConstraint('Range')}
          >
            Range
          </button>
          <button 
            class="add-constraint-option"
            onClick={() => initializeDraftConstraint('TypeDecl')}
          >
            Type Declaration
          </button>
          <button 
            class="add-constraint-option"
            onClick={() => initializeDraftConstraint('LengthRelation')}
          >
            Length Relation
          </button>
          <button 
            class="add-constraint-option"
            onClick={() => initializeDraftConstraint('Relation')}
          >
            Relation
          </button>
          <button 
            class="add-constraint-cancel"
            onClick={() => showMenu.value = false}
          >
            ✕
          </button>
        </div>
      )}
    </div>
  );
}