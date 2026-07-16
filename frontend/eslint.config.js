import eslint from "@eslint/js";
import sveltePlugin from "eslint-plugin-svelte";
import globals from "globals";

export default [
  eslint.configs.recommended,
  ...sveltePlugin.configs["flat/recommended"],
  {
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: {
        ...globals.browser,
        ...globals.es2021,
      },
    },
    rules: {
      "no-unused-vars": ["warn", { "argsIgnorePattern": "^_" }],
      "svelte/no-at-html-tags": "off"
    },
  },
  {
    ignores: [
      "dist/",
      "node_modules/",
      ".svelte-kit/",
      "build/"
    ]
  }
];
