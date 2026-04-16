import { isComplete, holeCount, lastError, projection } from '../../editor/state';

export function HeaderBar() {
  const errorCount = projection.value?.diagnostics?.length ?? 0;
  
  return (
    <div class="header-bar">
      <div class="header-bar-left">
        <h2 class="header-bar-title">🔧 AST Editor</h2>
      </div>
      
      <div class="header-bar-center">
        <div class="completeness-info">
          <div class={`completeness-indicator ${isComplete.value ? 'complete' : 'incomplete'}`}>
            {isComplete.value ? '✓' : '⚠'}
          </div>
          <span class="completeness-text">
            {isComplete.value ? 'Complete' : `${holeCount.value} holes`}
          </span>
          {errorCount > 0 && (
            <span class="error-count">
              {errorCount} {errorCount === 1 ? 'error' : 'errors'}
            </span>
          )}
        </div>
        
        <div class="completeness-bar">
          <div 
            class="completeness-bar-fill"
            style={{
              width: `${isComplete.value ? 100 : Math.max(0, 100 - (holeCount.value / 10) * 100)}%`
            }}
          />
        </div>
      </div>
      
      <div class="header-bar-right">
        {lastError.value && (
          <div class="error-message" title={lastError.value}>
            ⚡ Error
          </div>
        )}
      </div>
    </div>
  );
}