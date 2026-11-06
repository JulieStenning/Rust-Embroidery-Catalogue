# Gemini Rules - Embroidery Catalogue Development

You are helping build **Embroidery Catalogue**, a local, offline desktop tool for cataloguing and browsing digital embroidery designs.

---

## 🤖 Core Agent Rules & Planning

- **No Unsolicited Sample Code During Planning:** During exploration, architectural discussions, and pre-execution planning, do **not** generate speculative code blocks or unsolicited sample implementations. Focus on system architecture, data flow, UI/UX feedback, and safety invariants.
- **Direct Implementation on Approved Tasks:** When explicitly tasked with implementing features, fixing bugs, or writing tests, directly write and modify the code and verify it with the appropriate test commands.
- **Svelte Module Reference Syntax (CRITICAL):**
  - Always reference Svelte view and component modules using a bare `@` prefix **without quotes** so IDE navigation works (e.g. `@DesignDetailView.svelte`, `@MainView.svelte`, `@ImportView.svelte`).
- **Documentation Updates on User Task Changes (Minimalism):** When a change alters what a user does (new task, changed steps, renamed or moved control, removed feature, new decision), update the relevant in-app help (@HelpView.svelte) and user-facing guidance (`docs/User-Facing-Guidance/`), and update test coverage in `HelpView.test.ts`. User documentation follows John Carroll's **Minimalism**: task-named headings, short numbered steps, guidance only for non-obvious decisions, and an "If something goes wrong" section. Never repeat on-screen labels field by field, describe transient UI states, or include implementation detail, defaults or history — that belongs in `docs/Specs/`, ADRs or `docs/Help for Developers/`. Revise existing steps rather than appending notes. Internal-only changes need no user documentation update. Full standard: [`.agents/rules/01-agent-planning-and-interaction.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/01-agent-planning-and-interaction.md).
- **Tag Synonym Planning Invariants:**
  - **No Singular/Plural Duplication:** The tagging engine automatically handles inflections (`to_singular()` and `to_plural()`). Never suggest or seed both singular and plural forms of a word (e.g., plan only `quote`, never `quote` and `quotes`; `bookmark`, never `bookmark` and `bookmarks`).
  - **No Self-Synonyms:** The tagging engine directly matches significant words from the tag description itself in both singular and plural forms. Never suggest or seed synonyms that duplicate words in the tag's own name (e.g., for tag `Words`, never add `word` or `words`; for `Alphabets`, never add `alphabet` or `alphabets`). Synonyms are strictly for alternative vocabulary not already present in the tag name.
- **Import-Relevant System Settings Planning Invariants:**
  - **Pipeline Impact & First-Run Audit:** Any system setting or configuration that influences file discovery, scanning, master/stitch pairing, archive extraction, cataloguing granularity, or design filtering during import must be audited for first-run onboarding impact.
  - **Setup Wizard Placement & Safe Defaults:** If importing immediately after onboarding could result in unwanted, noisy, or irreversible database state (e.g. uncompressed alphabet glyph bloat, unindexed master formats), the plan must explicitly specify:
    1. Exposure and placement in @InitialSetupView.svelte (e.g. within the Import Settings onboarding step).
    2. A safe default value (e.g. enabling compression by default to protect user libraries).
    3. Dual persistence across both @InitialSetupView.svelte and @SettingsView.svelte via `saveSettings`.
    4. Consistency across both import stages: scanning precheck (`precheck.rs`) and background executor (`executor.rs`).
- **British English Spelling Invariant:**
  - All user-facing text, UI labels, tooltips, dialogs, error messages, in-app help (@HelpView.svelte), and guidance documentation (`docs/User-Facing-Guidance/`) must strictly use **British English** spelling:
    - `-our` endings (e.g. `colour`, `behaviour`, `neighbour`).
    - `-ise` / `-isation` suffixes (e.g. `optimise`, `recognise`, `normalise`, `customise`, `initialise`, `categorise`).
    - British vocabulary and noun/verb distinctions (e.g. `licence` as noun, `license` as verb; `catalogue`, `dialogue`, `cancelling`, `cancelled`).
    - Technical identifiers (Rust crates, SQLite syntax like `PRAGMA optimize`, API keys/payloads) remain exact in code/backticks, but surrounding prose must be British English.
- **Clickable In-App Navigation Links Invariant:**
  - Desktop application users have no browser address/URL bar. Never display raw URL hash strings or instruct users to type paths (e.g. avoid `Admin → About or #/about`).
  - Any route reference in user-facing guidance (`docs/User-Facing-Guidance/`) or in-app help (@HelpView.svelte) must be a clickable markdown link using the Svelte hash route (e.g. `[Admin → About](#/about)`, `[Admin → Settings](#/admin/system/settings)`).
- **Pre-Execution Planning (Required Before Changes):**
  - Produce a concise structural implementation plan covering:
    1. Affected Files & Scope Boundary
    2. Boundary & IPC Contracts (Tauri v2 ↔ Svelte, camelCase JS keys)
    3. Specification & Safety Constraints (read-only original designs, SQLite schema)
    4. Step-by-Step Execution Sequence & Test Plan (Vitest, `cargo test`, and mandatory Playwright E2E coverage for UI changes)
    5. Documentation Updates Plan (@HelpView.svelte & `docs/User-Facing-Guidance/`)
- **Targeted Rust Formatting Invariant (No Crate-Wide Diff Noise):**
  - Whenever Rust files are created or modified, format **only** the touched files before concluding the task using `rustfmt --edition 2021 <files...>`.
  - Never run blanket `cargo fmt` during feature work or bug fixes to prevent modifying unrelated files and avoid scope creep or noisy diffs across the repository.
- **Targeted Subsystem Test Scope Invariant (No Redundant Rust Tests):**
  - Never run `cargo test` when changes are isolated to frontend files (`frontend/**`), styles, views, or standalone documentation (`docs/**`).
  - Only run `cargo test` when Rust source code (`src/**/*.rs`, `src-tauri/**/*.rs`), Cargo manifests (`Cargo.toml`), or assets embedded into Rust binaries via `include_str!` / `include_bytes!` are modified or added.
  - Frontend changes are verified with Vitest (`npm test` / `npx vitest run <path>`), `svelte-check`, and Playwright E2E (`tests/e2e/*.spec.ts`) for UI alterations. Pure documentation changes do not require running Rust or frontend test suites unless embedded document tests or link resolvers are directly affected.

---

## 🧭 Core Philosophy & Safety Constraints

- **Local & Offline First:** The app runs entirely locally. Original embroidery files must **NEVER** be moved, renamed, modified, or altered. The app only reads them to extract metadata and cache generated thumbnail previews locally.
- **Performance:** Reading binary stitch files and rendering/caching previews efficiently in Rust is a critical priority.
- **Zero Static Process Globals & Managed State:** All task coordinators, background managers, and session stores are encapsulated in `AppState` and injected via `tauri::State<AppState>`.
- **Zero Native Webview Dialogs (ADR 010):** Absolute ban on `window.confirm()`, `window.alert()`, `window.prompt()`. All confirmations must use custom Svelte modals with `data-testid` attributes.
- **Action Semantics (ADR 011):**
  - **Cancel (Atomic Rollback):** Complete discard of uncommitted work; zero changes persist (e.g. setup abort, storage migration cancel, scan cancel).
  - **Stop (Progressive Halt):** Halts iterative batch processes mid-flight while strictly retaining committed work (e.g. bulk import, unified backfill). Button transitions: `"Stop"` $\rightarrow$ `"Stopping..."`.
- **Database Mutation Busy Guards & Anti-Double-Click Navigation Locking:** Every async database mutation (create, edit, delete, batch updates, migrations) must engage `busyStore` (`beginBusy`/`endBusy` in `try ... finally`), immediately disable trigger buttons (`"Deleting..."`, `"Saving..."`), reject duplicate invocations, and lock down all global and sub-tab navigation.

---

## 📚 Modular Rule Book Index (`.agents/rules/`)

The workspace rules are modularized across `.agents/rules/` for comprehensive details:

1. [`.agents/rules/01-agent-planning-and-interaction.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/01-agent-planning-and-interaction.md)
   - Agent interaction mode, code generation restrictions, pre-execution planning requirements, and complete Svelte module inventory.
2. [`.agents/rules/02-architecture-and-boundaries.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/02-architecture-and-boundaries.md)
   - Core philosophy, failure state lessons, IPC contracts (camelCase invoke keys), data root isolation, tagging boundaries (image vs stitching), ADR 006 (master/stitch pairing), ADR 007 (library drift), ADR 011 (cancel vs stop).
3. [`.agents/rules/03-rust-standards.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/03-rust-standards.md)
   - Zero panics in binary parsing, canonical `designs.filepath` format, instant database schema checks, Criterion benchmarking, and 500-line module limits.
4. [`.agents/rules/04-frontend-standards.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/04-frontend-standards.md)
   - Native Svelte 5 runes, strict typing (no implicit any), 500-line component limit, ADR 010 modal invariant, canonical UI theme tokens, initial capitals for h2 section headings, and view testing rules.
5. [`.agents/rules/05-testing-and-benchmarks.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/05-testing-and-benchmarks.md)
   - MainView dual-suite discipline, Playwright WebView2 CDP setup, formatting & command gotchas, and execution benchmark table with polling cadence guidelines.
