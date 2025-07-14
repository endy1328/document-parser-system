import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    svelte()
  ],
  server: {
    host: '0.0.0.0',
    port: 3000
  },
  optimizeDeps: {
    include: ['pdfjs-dist']
  },
  assetsInclude: ['**/*.worker.js']
});
