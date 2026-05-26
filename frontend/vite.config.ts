import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],

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
