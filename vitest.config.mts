
import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [
    svelte({
      // The Svelte config (preprocessor setup) lives inside frontend/
      configFile: "frontend/svelte.config.js",
    }),
  ],
  resolve: {
    // Svelte 5's package exports map exposes "browser" → index-client.js
    // and "default" → index-server.js. Vitest resolves using "default" by
    // default, which would load the server build where `mount` is unavailable.
    // Requesting the "browser" condition ensures the client build is used.
    conditions: ["browser"],
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["frontend/src/**/*.test.ts"],
    setupFiles: ["./vitest.setup.ts"],
    coverage: {
      provider: "istanbul",
      reporter: ["text"],
      // Needed on Windows: vitest's isIncluded() uses a case-sensitive
      // startsWith() against the project root, but Vite transform IDs keep
      // the real drive letter ("D:/...") while the root is normalized to
      // lowercase ("d:/..."). That makes every file look "external" and
      // nothing gets instrumented. allowExternal: true skips that check.
      allowExternal: true,
      exclude: ["frontend/src/**/*.test.ts"],
    },
  },
});
