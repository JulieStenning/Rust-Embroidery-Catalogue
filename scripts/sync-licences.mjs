#!/usr/bin/env node
// ---------------------------------------------------------------------------
// sync-licences.mjs
//
// Copies the licence artifacts produced by `npm run generate:licences` from the
// Rust source tree into the frontend source tree, so AboutView.svelte can import
// them as static Vite assets (no IPC / filesystem reads at runtime):
//
//   src/assets/licences.html        -> frontend/src/lib/assets/licences.html
//   src/assets/npm-licences.json    -> frontend/src/lib/assets/npm-licences.json
//   LICENCE (repo root)             -> frontend/src/LICENSE
//
// Run automatically via the root package.json "postgenerate:licences" hook.
// ---------------------------------------------------------------------------
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)));

/** @type {Array<{ from: string, to: string }>} */
const copies = [
  {
    from: join(repoRoot, "src", "assets", "licences.html"),
    to: join(repoRoot, "frontend", "src", "lib", "assets", "licences.html"),
  },
  {
    from: join(repoRoot, "src", "assets", "npm-licences.json"),
    to: join(repoRoot, "frontend", "src", "lib", "assets", "npm-licences.json"),
  },
  {
    from: join(repoRoot, "LICENCE"),
    to: join(repoRoot, "frontend", "src", "LICENSE"),
  },
];

let copied = 0;
let warned = 0;

for (const { from, to } of copies) {
  if (!existsSync(from)) {
    console.warn(`[sync-licences] SKIP: source missing: ${from}`);
    warned += 1;
    continue;
  }
  mkdirSync(dirname(to), { recursive: true });
  copyFileSync(from, to);
  console.log(`[sync-licences] ${from} -> ${to}`);
  copied += 1;
}

console.log(
  `[sync-licences] done: ${copied} file(s) copied${warned > 0 ? `, ${warned} skipped` : ""}.`
);