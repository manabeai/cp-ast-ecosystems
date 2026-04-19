import { currentPage } from './state';
import { ViewerPage } from './components/viewer/ViewerPage';
import { PreviewPage } from './components/preview/PreviewPage';
import { EditorPage } from './editor/EditorPage';

export function App() {
  const page = currentPage.value;

  return (
    <div class="app">
      <header class="header">
        <h1 class="header-title">🌳 AST Editor</h1>
        <nav class="header-nav">
          <a
            href="#/"
            class={`nav-link ${page === 'editor' ? 'active' : ''}`}
          >
            Editor
          </a>
          <a
            href="#/viewer"
            class={`nav-link ${page === 'viewer' ? 'active' : ''}`}
          >
            Viewer
          </a>
          <a
            href="#/preview"
            class={`nav-link ${page === 'preview' ? 'active' : ''}`}
          >
            Preview
          </a>
        </nav>
      </header>
      <main class="main">
        {page === 'editor' && <EditorPage />}
        {page === 'viewer' && <ViewerPage />}
        {page === 'preview' && <PreviewPage />}
      </main>
    </div>
  );
}
