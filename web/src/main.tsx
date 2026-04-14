import { render } from 'preact';
import { initWasm } from './wasm';
import { loadPreset } from './state';
import { App } from './app';
import './index.css';

async function main() {
  await initWasm();
  loadPreset('scalar_array');
  render(<App />, document.getElementById('app')!);
}

main().catch(console.error);
