import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  base: './',
  plugins: [wasm()],
  build: {
    target: 'esnext',
  },
  resolve: {
    alias: {
      // The package ships only a "module" field (no "main"/"exports"), which
      // Vite's Node/SSR resolver (used to run tests) can't find on its own.
      '@dimforge/rapier2d-deterministic': fileURLToPath(
        new URL('./node_modules/@dimforge/rapier2d-deterministic/rapier.js', import.meta.url),
      ),
    },
  },
  test: {
    environment: 'node',
  },
});
