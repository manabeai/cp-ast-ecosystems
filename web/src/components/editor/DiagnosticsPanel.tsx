import { projection, selectedNodeId } from '../../editor/state';
import type { Diagnostic } from '../../editor/types';

export function DiagnosticsPanel() {
  const currentProjection = projection.value;

  if (!currentProjection) {
    return (
      <div class="diagnostics-panel">
        <div class="diagnostics-empty">No document loaded</div>
      </div>
    );
  }

  const diagnostics = currentProjection.diagnostics;

  if (diagnostics.length === 0) {
    return (
      <div class="diagnostics-panel">
        <div class="diagnostics-empty">✓ No issues found</div>
      </div>
    );
  }

  const handleDiagnosticClick = (diagnostic: Diagnostic) => {
    if (diagnostic.node_id) {
      selectedNodeId.value = diagnostic.node_id;
    }
  };

  const getLevelIcon = (level: Diagnostic['level']) => {
    switch (level) {
      case 'error':
        return '🔴';
      case 'warning':
        return '⚠️';
      case 'info':
        return 'ℹ️';
      default:
        return '•';
    }
  };

  const getLevelClass = (level: Diagnostic['level']) => {
    return `diagnostic-${level}`;
  };

  return (
    <div class="diagnostics-panel">
      <div class="diagnostics-list">
        {diagnostics.map((diagnostic, index) => (
          <div
            key={index}
            class={`diagnostic-item ${getLevelClass(diagnostic.level)} ${
              diagnostic.node_id ? 'clickable' : ''
            }`}
            onClick={() => handleDiagnosticClick(diagnostic)}
          >
            <div class="diagnostic-icon">
              {getLevelIcon(diagnostic.level)}
            </div>
            <div class="diagnostic-content">
              <div class="diagnostic-message">{diagnostic.message}</div>
              {diagnostic.node_id && (
                <div class="diagnostic-location">Node: {diagnostic.node_id}</div>
              )}
              {diagnostic.constraint_id && (
                <div class="diagnostic-location">Constraint: {diagnostic.constraint_id}</div>
              )}
            </div>
          </div>
        ))}
      </div>
      <div class="diagnostics-summary">
        {diagnostics.filter(d => d.level === 'error').length} errors, {' '}
        {diagnostics.filter(d => d.level === 'warning').length} warnings, {' '}
        {diagnostics.filter(d => d.level === 'info').length} info
      </div>
    </div>
  );
}