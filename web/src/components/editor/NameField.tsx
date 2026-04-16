import { signal } from '@preact/signals';
import { useState } from 'preact/hooks';
import { projection } from '../../editor/state';
import { dispatchAction } from '../../editor/actions';
import type { FillContent } from '../../editor/types';

interface Props {
  nodeId: string;
}

export function NameField({ nodeId }: Props) {
  const [isEditing, setIsEditing] = useState(false);
  const editingValue = signal('');

  // Get current node name
  const currentProjection = projection.value;
  const currentNode = currentProjection?.outline.find(n => n.id === nodeId);
  const currentName = currentNode?.label || '';

  const startEdit = () => {
    editingValue.value = currentName;
    setIsEditing(true);
  };

  const cancelEdit = () => {
    setIsEditing(false);
    editingValue.value = '';
  };

  const saveEdit = () => {
    const newName = editingValue.value.trim();
    if (!newName || newName === currentName) {
      cancelEdit();
      return;
    }

    // Find the node's type to construct replacement
    if (!currentNode) {
      cancelEdit();
      return;
    }

    // Create replacement content based on node type
    let replacement: FillContent;
    
    if (currentNode.kind_label.includes('Scalar')) {
      // Extract type from kind_label if possible, otherwise default to int
      const typeMatch = currentNode.kind_label.match(/Scalar\((\w+)\)/);
      const nodeType = typeMatch ? typeMatch[1] : 'int';
      replacement = { kind: 'Scalar', name: newName, typ: nodeType };
    } else if (currentNode.kind_label.includes('Array')) {
      // Extract type if possible, otherwise default to int
      const typeMatch = currentNode.kind_label.match(/Array\((\w+)\)/);
      const elementType = typeMatch ? typeMatch[1] : 'int';
      replacement = { 
        kind: 'Array', 
        name: newName, 
        element_type: elementType, 
        length: { kind: 'Fixed', value: 1 } // Default length
      };
    } else if (currentNode.kind_label.includes('Section')) {
      replacement = { kind: 'Section', label: newName };
    } else {
      // Fallback to scalar
      replacement = { kind: 'Scalar', name: newName, typ: 'int' };
    }

    const success = dispatchAction({
      kind: 'ReplaceNode',
      target: nodeId,
      replacement
    });

    if (success) {
      setIsEditing(false);
      editingValue.value = '';
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      saveEdit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelEdit();
    }
  };

  if (isEditing) {
    return (
      <div class="name-field-edit">
        <input
          class="name-input"
          type="text"
          value={editingValue.value}
          onInput={(e) => editingValue.value = (e.target as HTMLInputElement).value}
          onBlur={saveEdit}
          onKeyDown={handleKeyDown}
          autoFocus
          placeholder="Node name"
        />
      </div>
    );
  }

  return (
    <div class="name-field-display">
      <span class="name-value" onClick={startEdit}>
        {currentName}
      </span>
      <button class="name-edit-btn" onClick={startEdit} title="Edit name">
        ✏️
      </button>
    </div>
  );
}