import { selectedNodeId } from '../../editor/state';
import { HoleIndicator } from './HoleIndicator';
import type { OutlineNode } from '../../editor/types';

interface StructureNodeViewProps {
  node: OutlineNode;
}

export function StructureNodeView({ node }: StructureNodeViewProps) {
  const isSelected = selectedNodeId.value === node.id;

  const handleClick = () => {
    selectedNodeId.value = isSelected ? null : node.id;
  };

  return (
    <div class="structure-node">
      <div 
        class={`node-content ${isSelected ? 'selected' : ''}`}
        style={{ paddingLeft: `${node.depth * 1.5}rem` }}
        onClick={handleClick}
      >
        {node.is_hole ? (
          <HoleIndicator holeId={node.id} />
        ) : (
          <div class="node-info">
            <span class="node-label">{node.label}</span>
            <span class="node-kind">{node.kind_label}</span>
          </div>
        )}
      </div>
    </div>
  );
}