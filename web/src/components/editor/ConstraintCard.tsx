import { selectedConstraintId } from '../../editor/state';
import { dispatchAction } from '../../editor/actions';
import type { ConstraintSummary } from '../../editor/types';

interface ConstraintCardProps {
  constraint: ConstraintSummary;
}

export function ConstraintCard({ constraint }: ConstraintCardProps) {
  const isSelected = selectedConstraintId.value === constraint.id;

  const handleClick = () => {
    selectedConstraintId.value = constraint.id;
  };

  const handleDelete = (e: Event) => {
    e.stopPropagation(); // Prevent selection when deleting
    dispatchAction({
      kind: 'RemoveConstraint',
      constraint_id: constraint.id,
    });
  };

  return (
    <div
      class={`constraint-card ${isSelected ? 'selected' : ''}`}
      onClick={handleClick}
    >
      <div class="constraint-info">
        <div class="constraint-label">{constraint.label}</div>
        <div class="constraint-kind">{constraint.kind_label}</div>
      </div>
      <button 
        class="constraint-delete"
        onClick={handleDelete}
        title="Remove constraint"
      >
        ✕
      </button>
    </div>
  );
}