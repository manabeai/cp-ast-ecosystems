import { currentPage } from './state';
import { ViewerPage } from './components/viewer/ViewerPage';
import { PreviewPage } from './components/preview/PreviewPage';

export function App() {
  return (
    <div class="app">
      <header class="header">
        <h1 class="header-title">🌳 AST Viewer</h1>
        <nav class="header-nav">
          <a
            href="#/viewer"
            class={`nav-link ${currentPage.value === 'viewer' ? 'active' : ''}`}
          >
            Viewer
          </a>
          <a
            href="#/preview"
            class={`nav-link ${currentPage.value === 'preview' ? 'active' : ''}`}
          >
            Preview
          </a>
          <a
            href="#/editor"
            class={`nav-link ${currentPage.value === 'editor' ? 'active' : ''}`}
          >
            Editor
          </a>
        </nav>
      </header>
      <main class="main">
        {currentPage.value === 'viewer' ? <ViewerPage /> :
         currentPage.value === 'preview' ? <PreviewPage /> :
         <div class="editor-placeholder">Editor (Coming Soon)</div>}
      </main>
    </div>
  );
}
