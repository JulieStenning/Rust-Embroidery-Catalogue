import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  // Enable Svelte preprocessing (needed for TypeScript, SCSS, etc. later)
  preprocess: vitePreprocess(),
};
