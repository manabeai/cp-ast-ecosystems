import { StructurePane } from './StructurePane';
import { ConstraintPane } from './ConstraintPane';

export function ViewerPage() {
  return (
    <div class="viewer-page">
      <div class="viewer-panes">
        <StructurePane />
        <ConstraintPane />
        <div class="pane">
          <div class="pane-header">
            <span class="pane-title">Preview</span>
          </div>
          <pre class="pane-content">Preview pane — coming in Task 8</pre>
        </div>
      </div>
      <div class="toolbar">Toolbar — coming in Task 8</div>
    </div>
  );
}
