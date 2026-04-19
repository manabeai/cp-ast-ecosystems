import { defineConfig } from 'vite';
import preact from '@preact/preset-vite';

export default defineConfig({
  plugins: [preact()],
  root: '.',
  base: '/cp-ast-ecosystems/',
  build: {
    outDir: 'dist',
  },
});
