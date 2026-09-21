// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  // Enable Svelte preprocessing (needed for TypeScript, SCSS, etc. later)
  preprocess: vitePreprocess(),
};
