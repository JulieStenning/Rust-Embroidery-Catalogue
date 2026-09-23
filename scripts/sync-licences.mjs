#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

// ---------------------------------------------------------------------------
// sync-licences.mjs
//
// Copies the licence artifacts produced by `npm run generate:licences` from the
// Rust source tree into the frontend source tree, so AboutView.svelte can
// import them as static Vite assets (no IPC / filesystem reads at runtime):
//
//   src/assets/licences.html     -> frontend/src/lib/assets/licences.html
//   src/assets/npm-licences.json -> frontend/src/lib/assets/npm-licences.json
//   LICENSE (repo root)          -> frontend/src/LICENSE
//   NOTICE (repo root)           -> frontend/src/NOTICE
//
// The two GENERATED artifacts are normalised to LF line endings before they are
// copied. cargo-about splices in bundled licence texts whose own endings are
// CRLF, so its output mixes CRLF and LF lines, while the committed blobs are
// pure LF (".gitattributes" sets "* text=auto"). On a Windows worktree that made
// both licences.html files show as modified after every `cargo tauri build`,
// even though their content was byte-identical once normalised. Normalising here
// makes the generated output match the committed blobs exactly, so a build no
// longer dirties the working tree.
//
// LICENSE and NOTICE are deliberately NOT normalised: they are hand-maintained
// verbatim GPL text, and their mirrors must stay byte-identical to the root
// files.
//
// Run automatically via the root package.json "postgenerate:licences" hook.
// ---------------------------------------------------------------------------
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)));

/** @type {Array<{ from: string, to: string, normaliseEol?: boolean }>} */
const copies = [
  {
    from: join(repoRoot, "src", "assets", "licences.html"),
    to: join(repoRoot, "frontend", "src", "lib", "assets", "licences.html"),
    normaliseEol: true,
  },
  {
    from: join(repoRoot, "src", "assets", "npm-licences.json"),
    to: join(repoRoot, "frontend", "src", "lib", "assets", "npm-licences.json"),
    normaliseEol: true,
  },
  {
    from: join(repoRoot, "LICENSE"),
    to: join(repoRoot, "frontend", "src", "LICENSE"),
  },
  {
    from: join(repoRoot, "NOTICE"),
    to: join(repoRoot, "frontend", "src", "NOTICE"),
  },
];

/**
 * Rewrites a generated file with LF endings so that it matches its committed
 * blob byte-for-byte. Returns true when the file actually changed.
 * @param {string} path
 * @returns {boolean}
 */
function normaliseToLf(path) {
  const original = readFileSync(path, "utf8");
  const normalised = original.replace(/\r\n/g, "\n");
  if (normalised === original) {
    return false;
  }
  writeFileSync(path, normalised, "utf8");
  return true;
}

let copied = 0;
let warned = 0;
let normalised = 0;

for (const { from, to, normaliseEol } of copies) {
  if (!existsSync(from)) {
    console.warn(`[sync-licences] SKIP: source missing: ${from}`);
    warned += 1;
    continue;
  }
  if (normaliseEol && normaliseToLf(from)) {
    console.log(`[sync-licences] normalised CRLF -> LF: ${from}`);
    normalised += 1;
  }
  mkdirSync(dirname(to), { recursive: true });
  copyFileSync(from, to);
  console.log(`[sync-licences] ${from} -> ${to}`);
  copied += 1;
}

const notes = [];
if (normalised > 0) {
  notes.push(`${normalised} normalised to LF`);
}
if (warned > 0) {
  notes.push(`${warned} skipped`);
}
const suffix = notes.length > 0 ? ` (${notes.join(", ")})` : "";
console.log(`[sync-licences] done: ${copied} file(s) copied${suffix}.`);
