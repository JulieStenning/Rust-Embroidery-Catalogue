import { defineConfig, type PluginOption } from 'vite'; // <-- Added PluginOption type
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [
    tailwindcss(),
    svelte()
  ] as PluginOption[], // <-- Explicitly cast the whole array here


  // Vite dev server settings — Tauri listens on this port
  server: {
    port: 5173,
    strictPort: true,
    // Allow connections from the Tauri webview
    host: "localhost",
  },

  // Build output goes into dist/, which tauri.conf.json points to via frontendDist
  build: {
    outDir: "dist",
    emptyOutDir: true,
    // Tauri supports ES modules on all target platforms
    target: ["es2021", "chrome105", "safari14"],
    // Minify for production; no source maps needed in the bundled binary
    minify: "esbuild",
    sourcemap: false,
  },

  // Make Vite environment variables available to Svelte components
  envPrefix: ["VITE_", "TAURI_"],
});
