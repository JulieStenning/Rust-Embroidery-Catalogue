---
name: feature-planning
description: >-
  Use this skill when preparing, structuring, or reviewing a pre-execution implementation plan for new features,
  settings, or cross-boundary changes in Embroidery Catalogue.
---

# Feature & Settings Planning Skill

When tasked with implementing features, refactoring, or introducing new settings, produce a concise structural implementation plan adhering strictly to the pre-execution rules.

## Core Planning Invariants

### 1. Tag Synonym Invariants
- **No Singular/Plural Duplication:** The engine inflects words automatically (`to_singular()` and `to_plural()`). Seed base singular forms only (`quote`, `bookmark`).
- **No Self-Synonyms:** Do not suggest synonyms matching significant words already in the tag name (e.g. no `word` for `Words`, no `alphabet` for `Alphabets`).

### 2. Import-Relevant System Settings Invariants
Any system setting or configuration influencing file discovery, scanning, master/stitch pairing, archive extraction, cataloguing granularity, or design filtering during import must be audited for first-run onboarding impact:
- **First-Run Impact Audit:** Determine whether importing files immediately after a fresh install could cause unwanted, noisy, or irreversible database entries (e.g. hundreds of individual alphabet glyphs instead of companion zip archives, or missed master formats).
- **Setup Wizard Placement:** If impactful, specify its exposure and placement in `@InitialSetupView.svelte` (e.g. in the combined Import Settings step).
- **Safe Defaults:** Choose a sensible, conservative default (e.g., `zip_alphabets: true`) to safeguard user databases against accidental clutter.
- **Dual Persistence:** Ensure the setting is handled and persisted via `saveSettings` in both `@InitialSetupView.svelte` and `@SettingsView.svelte`.
- **Pipeline Consistency:** Verify that both the pre-import scan (`precheck.rs`) and the bulk import executor (`executor.rs`) query and apply the setting consistently.

### 3. British English Spelling Invariant
All user-facing copy, documentation (`docs/User-Facing-Guidance/`), in-app help (`@HelpView.svelte`), UI labels, tooltips, modals, notifications, and error messages must strictly use **British English** spelling:
- **-our Endings:** `colour`, `behaviour`, `neighbour`, `flavour`.
- **-ise / -isation Suffixes:** `optimise`, `recognise`, `normalise`, `customise`, `initialise`, `categorise`, `optimisation`, `synchronisation`.
- **Noun/Verb Distinctions & Double Consonants:** `licence` (noun), `license` (verb), `catalogue`, `dialogue`, `cancelling`, `cancelled`, `traveller`.
- **Technical Identifier Exception:** Code symbols, Rust crate names, SQLite PRAGMA commands (e.g. `PRAGMA optimize`), and external API parameters remain verbatim in code and markdown backticks, but all surrounding prose, documentation, headings, and UI text must strictly use British English.

### 4. Clickable In-App Navigation Links Invariant
Desktop application users have no browser address/URL bar to view or type routes into.
- **Never display raw URL hash strings or instruct users to type paths:** Avoid unlinked prose such as `Admin → About or #/about` or `type #/import`.
- **Always use clickable markdown links:** Any route reference in user-facing guidance (`docs/User-Facing-Guidance/`) or in-app help (@HelpView.svelte) must be formatted as a clickable markdown link using the Svelte hash route (e.g. `[Admin → About](#/about)`, `[Admin → Settings](#/admin/system/settings)`, `[Admin → Batch Operations](#/admin/batch-operations)`).
- **In-App Link Resolution:** In-app markdown renderers (e.g. `markdown.js`) preserve hash routes (`target.startsWith("#")`) as working internal links within the WebView2 desktop app.

## Structural Implementation Plan Sections

A pre-execution plan must strictly contain:
1. **Affected Files & Scope Boundary:** Svelte views using `@View.svelte` syntax, backend routes/services, and components intentionally left untouched.
2. **Boundary & IPC Contracts:** Tauri v2 commands, camelCase keys for JS invocations, and matching Rust/TypeScript types.
3. **Specification & Safety Constraints:** Read-only original designs invariant, SQLite schema checks, tag invariants, and import-relevant setting invariants.
4. **Step-by-Step Execution Sequence & Test Plan:** Sequenced execution steps, unit tests (`cargo test`, Vitest), and mandatory Playwright E2E test file (`tests/e2e/*.spec.ts`).
5. **Documentation Updates Plan:** Updates to `@HelpView.svelte`, `HelpView.test.ts`, and `docs/User-Facing-Guidance/`.

