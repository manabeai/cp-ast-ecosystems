import { signal } from '@preact/signals';
import { dispatchAction } from '../../editor/actions';
import { editorDocumentJson } from '../../editor/state';
import type { FillContent } from '../../editor/types';

export function AddNodeMenu() {
  const showMenu = signal(false);

  const addNode = (nodeType: 'Scalar' | 'Array' | 'Section') => {
    let fill: FillContent;
    
    switch (nodeType) {
      case 'Scalar':
        fill = {
          kind: 'Scalar',
          name: 'N',
          typ: 'int',
        };
        break;
      case 'Array':
        fill = {
          kind: 'Array',
          name: 'A',
          element_type: 'int',
          length: { kind: 'Fixed', value: 1 },
        };
        break;
      case 'Section':
        fill = {
          kind: 'Section',
          label: 'New Section',
        };
        break;
    }

    // Check if we have an empty document to create initial structure
    const docJson = editorDocumentJson.value;
    if (!docJson || docJson === '{"root":null}') {
      // For a completely empty document, we'd need to handle the root creation
      // For now, we'll just try the AddSlotElement action assuming there's a root with slots
      const success = dispatchAction({
        kind: 'AddSlotElement',
        parent: 'root',
        slot_name: 'elements', // Common slot name for collections
        element: fill,
      });
      
      if (success) {
        showMenu.value = false;
      }
    } else {
      // For non-empty documents, we could try to find an appropriate parent
      // For now, let's just use a generic approach
      const success = dispatchAction({
        kind: 'AddSlotElement',
        parent: 'root',
        slot_name: 'elements',
        element: fill,
      });
      
      if (success) {
        showMenu.value = false;
      }
    }
  };

  return (
    <div class="add-node-menu">
      {!showMenu.value ? (
        <button 
          class="add-node-trigger"
          onClick={() => showMenu.value = true}
        >
          + Add Element
        </button>
      ) : (
        <div class="add-node-options">
          <button 
            class="add-node-option"
            onClick={() => addNode('Scalar')}
          >
            + Scalar
          </button>
          <button 
            class="add-node-option"
            onClick={() => addNode('Array')}
          >
            + Array
          </button>
          <button 
            class="add-node-option"
            onClick={() => addNode('Section')}
          >
            + Section
          </button>
          <button 
            class="add-node-cancel"
            onClick={() => showMenu.value = false}
          >
            ✕
          </button>
        </div>
      )}
    </div>
  );
}