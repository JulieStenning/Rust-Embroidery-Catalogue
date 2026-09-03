#!/usr/bin/env node
// ---------------------------------------------------------------------------
// relativise-npm-licences.mjs
//
// `license-checker-rseidelsohn --json` embeds the ABSOLUTE install directory of
// each package (e.g. `D:\Someone\repo\frontend\node_modules\@scope\pkg`). That
// leaks the generating developer's machine layout into a committed asset. The
// frontend only renders `licenses`/`repository` (path/licenseFile are inert),
// so we rewrite those absolute roots to a repo-relative form.
//
// We parse the JSON, normalise any `path`/`licenseFile` string containing a
// `/node_modules/` segment down to `frontend/node_modules/<rest>` (forward
// slashes), and re-serialise with the same 4-space indent license-checker uses.
// The output is deterministic and idempotent, so it is identical on any machine
// regardless of where the repo was checked out. Run right after licence-checker
// writes `src/assets/npm-licences.json`, and on the already-synced frontend copy
// so existing checkouts are cleaned too.
// ---------------------------------------------------------------------------
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const relRoot = "frontend/";

/**
 * Collapse an absolute install path to a repo-relative one. License-checker runs
 * with `--start ./frontend`, so every emitted `path`/`licenseFile` begins with
 * the checkout root followed by the repo's `frontend` directory — either for a
 * dependency (`.../frontend/node_modules/<pkg>`) or the app's own private entry
 * (`.../frontend`). We cut at that stable `frontend` segment, discarding the
 * developer-specific prefix before it.
 * @param {unknown} value
 * @returns {unknown}
 */
function relativisePath(value) {
  if (typeof value !== "string") return value;
  // Normalise Windows backslashes so both the match and the output are stable.
  const normalised = value.replace(/\\/g, "/");

  // Already repo-relative or not under the frontend tree: leave untouched.
  const lower = normalised.toLowerCase();
  if (
    lower.startsWith(relRoot) ||
    lower.startsWith(".") ||
    !lower.includes("/frontend")
  ) {
    return value;
  }

  // Locate the repo's `frontend` segment and keep everything from it onward.
  const segments = normalised.split("/");
  const frontendIndex = segments.findIndex((s) => s.toLowerCase() === "frontend");
  if (frontendIndex === -1) return value;
  return segments.slice(frontendIndex).join("/");
}

/**
 * Clean a full npm-licences.json document.
 * @param {string} raw JSON text to clean.
 * @returns {string} cleaned JSON text.
 */
export function relativise(raw) {
  const data = JSON.parse(raw);
  for (const record of Object.values(data)) {
    if (!record || typeof record !== "object") continue;
    for (const field of ["path", "licenseFile"]) {
      if (field in record) {
        record[field] = relativisePath(record[field]);
      }
    }
  }
  return `${JSON.stringify(data, null, 4)}\n`;
}

// CLI entry point: relativise one or more files given as argv[2..].
const targets = process.argv.slice(2);
if (targets.length > 0) {
  for (const target of targets) {
    const resolved = join(process.cwd(), target);
    const original = readFileSync(resolved, "utf8");
    const cleaned = relativise(original);
    if (cleaned !== original) {
      writeFileSync(resolved, cleaned, "utf8");
      console.log(`[relativise-npm-licences] cleaned ${resolved}`);
    } else {
      console.log(`[relativise-npm-licences] no change: ${resolved}`);
    }
  }
}

