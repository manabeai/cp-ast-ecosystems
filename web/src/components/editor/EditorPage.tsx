import { useEffect } from 'preact/hooks';
import { editorDocumentJson, initEditor } from '../../editor/state';
import { StructurePane } from './StructurePane';
import { DetailPane } from './DetailPane';
import { ConstraintPane } from './ConstraintPane';
import { BottomPanel } from './BottomPanel';
import { HeaderBar } from './HeaderBar';

export function EditorPage() {
  // Initialize editor on mount
  useEffect(() => {
    if (!editorDocumentJson.value) {
      initEditor();
    }
  }, []);

  return (
    <div class="editor-page">
      <HeaderBar />
      <div class="editor-main">
        <StructurePane />
        <DetailPane />
        <ConstraintPane />
      </div>
      <BottomPanel />
    </div>
  );
}