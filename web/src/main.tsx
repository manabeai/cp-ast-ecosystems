import { render } from 'preact';
import { initWasm } from './wasm';
import { loadPreset } from './state';
import { initEditor } from './editor/editor-state';
import { App } from './app';
import './index.css';

async function main() {
  await initWasm();
  initEditor();
  loadPreset('scalar_array');
  render(<App />, document.getElementById('app')!);
}

main().catch(console.error);
