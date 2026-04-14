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
        </nav>
      </header>
      <main class="main">
        {currentPage.value === 'viewer' ? <ViewerPage /> : <PreviewPage />}
      </main>
    </div>
  );
}
