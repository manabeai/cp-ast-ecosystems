import '@testing-library/jest-dom';
import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import init from '../wasm/cp_ast_wasm';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Initialize real wasm binary once before all tests (no fetch needed in Node.js)
beforeAll(async () => {
  const wasmPath = resolve(__dirname, '../wasm/cp_ast_wasm_bg.wasm');
  const wasmBuffer = readFileSync(wasmPath);
  const wasmModule = await WebAssembly.compile(wasmBuffer);
  await init({ module_or_path: wasmModule });
});
