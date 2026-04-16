import { projection } from '../../editor/state';
import { StructureNodeView } from './StructureNodeView';
import { AddNodeMenu } from './AddNodeMenu';
import type { OutlineNode } from '../../editor/types';

export function StructurePane() {
  const currentProjection = projection.value;

  if (!currentProjection) {
    return (
      <div class="pane">
        <div class="pane-header">
          <span class="pane-title">📝 Structure</span>
        </div>
        <div class="pane-content">
          <div class="empty-state">No document loaded</div>
        </div>
      </div>
    );
  }

  const outline = currentProjection.outline;

  return (
    <div class="pane">
      <div class="pane-header">
        <span class="pane-title">📝 Structure</span>
        <span class="pane-info">
          {currentProjection.completeness.total_holes} holes
        </span>
      </div>
      <div class="pane-content structure-pane-content">
        <div class="structure-tree">
          {outline.length === 0 ? (
            <div class="empty-tree">No elements yet</div>
          ) : (
            outline.map((node, idx) => (
              <StructureNodeView key={`${node.id}-${idx}`} node={node} />
            ))
          )}
        </div>
        <div class="structure-actions">
          <AddNodeMenu />
        </div>
      </div>
    </div>
  );
}