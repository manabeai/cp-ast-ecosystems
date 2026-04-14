import { render } from 'preact';
import { initWasm, list_presets, version } from './wasm';

async function main() {
  await initWasm();
  const ver = version();
  const presets = JSON.parse(list_presets());
  console.log(`wasm v${ver} loaded, ${presets.length} presets`);

  render(
    <div>
      <h1>🌳 AST Viewer — wasm loaded (v{ver})</h1>
      <p>{presets.length} presets available</p>
      <ul>
        {presets.map((p: { name: string; description: string }) => (
          <li key={p.name}><strong>{p.name}</strong> — {p.description}</li>
        ))}
      </ul>
    </div>,
    document.getElementById('app')!,
  );
}

main().catch(console.error);
