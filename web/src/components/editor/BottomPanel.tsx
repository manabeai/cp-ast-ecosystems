import { signal } from '@preact/signals';
import { SamplePreview } from './SamplePreview';
import { CanonicalPreview } from './CanonicalPreview';
import { DiagnosticsPanel } from './DiagnosticsPanel';

type TabType = 'sample' | 'preview' | 'diagnostics';

const activeTab = signal<TabType>('sample');
const collapsed = signal<boolean>(false);

export function BottomPanel() {
  const handleTabClick = (tab: TabType) => {
    if (activeTab.value === tab && !collapsed.value) {
      collapsed.value = true;
    } else {
      activeTab.value = tab;
      collapsed.value = false;
    }
  };

  const renderTabContent = () => {
    if (collapsed.value) return null;
    
    switch (activeTab.value) {
      case 'sample':
        return <SamplePreview />;
      case 'preview':
        return <CanonicalPreview />;
      case 'diagnostics':
        return <DiagnosticsPanel />;
      default:
        return null;
    }
  };

  return (
    <div class="bottom-panel">
      <div class="bottom-panel-tabs">
        <button 
          class={`tab-btn ${activeTab.value === 'sample' ? 'active' : ''}`}
          onClick={() => handleTabClick('sample')}
        >
          Sample
        </button>
        <button 
          class={`tab-btn ${activeTab.value === 'preview' ? 'active' : ''}`}
          onClick={() => handleTabClick('preview')}
        >
          Preview
        </button>
        <button 
          class={`tab-btn ${activeTab.value === 'diagnostics' ? 'active' : ''}`}
          onClick={() => handleTabClick('diagnostics')}
        >
          Diagnostics
        </button>
        <button 
          class="collapse-btn"
          onClick={() => collapsed.value = !collapsed.value}
          title={collapsed.value ? "Expand panel" : "Collapse panel"}
        >
          {collapsed.value ? '▲' : '▼'}
        </button>
      </div>
      {!collapsed.value && (
        <div class="bottom-panel-content">
          {renderTabContent()}
        </div>
      )}
    </div>
  );
}