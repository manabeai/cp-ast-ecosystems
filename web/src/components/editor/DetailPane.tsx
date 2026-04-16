import { selectedNodeId } from '../../editor/state';
import { NodeInspector } from './NodeInspector';

export function DetailPane() {
  const nodeId = selectedNodeId.value;

  return (
    <div class="pane">
      <div class="pane-header">
        <span class="pane-title">🔍 Details</span>
      </div>
      <div class="pane-content">
        {nodeId ? (
          <NodeInspector nodeId={nodeId} />
        ) : (
          <div class="empty-state">Select a node to inspect</div>
        )}
      </div>
    </div>
  );
}