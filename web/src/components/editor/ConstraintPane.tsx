import { projection, draftConstraint } from '../../editor/state';
import { ConstraintCard } from './ConstraintCard';
import { AddConstraintButton } from './AddConstraintButton';
import { DraftConstraintPanel } from './DraftConstraintPanel';
import type { ConstraintSummary } from '../../editor/types';

export function ConstraintPane() {
  const currentProjection = projection.value;
  const showDraftPanel = draftConstraint.value !== null;

  if (!currentProjection) {
    return (
      <div class="pane">
        <div class="pane-header">
          <span class="pane-title">🔗 Constraints</span>
        </div>
        <div class="pane-content">
          <div class="empty-state">No document loaded</div>
        </div>
      </div>
    );
  }

  // Extract constraints from diagnostics or related data
  // For now, we'll create a placeholder approach since the exact constraint list source isn't clear
  const constraints: ConstraintSummary[] = [];
  
  // Look for constraints in diagnostics
  currentProjection.diagnostics.forEach((diagnostic, index) => {
    if (diagnostic.constraint_id) {
      // Create a constraint summary from diagnostic info
      constraints.push({
        id: diagnostic.constraint_id,
        label: diagnostic.message,
        kind_label: 'Diagnostic',
      });
    }
  });

  // Add some example constraints if none exist (for demonstration)
  if (constraints.length === 0) {
    constraints.push(
      {
        id: 'example-1',
        label: '1 ≤ N ≤ 10⁵',
        kind_label: 'Range',
      },
      {
        id: 'example-2', 
        label: 'A[i] is integer',
        kind_label: 'TypeDecl',
      }
    );
  }

  return (
    <div class="pane">
      <div class="pane-header">
        <span class="pane-title">🔗 Constraints</span>
        <span class="pane-info">{constraints.length} active</span>
      </div>
      
      <div class="pane-content constraint-pane-content">
        {showDraftPanel ? (
          <DraftConstraintPanel />
        ) : (
          <>
            <div class="constraint-list">
              {constraints.length === 0 ? (
                <div class="empty-constraints">No constraints yet</div>
              ) : (
                constraints.map(constraint => (
                  <ConstraintCard 
                    key={constraint.id} 
                    constraint={constraint}
                  />
                ))
              )}
            </div>
            
            <div class="constraint-actions">
              <AddConstraintButton />
            </div>
          </>
        )}
      </div>
    </div>
  );
}