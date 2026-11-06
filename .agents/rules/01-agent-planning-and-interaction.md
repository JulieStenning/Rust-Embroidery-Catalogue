# Agent Interaction Mode & Planning Rules

## 🤖 Agent Interaction Mode & Code Generation Rules

- **No Unsolicited Sample Code During Planning:** During exploration, architectural discussions, and pre-execution planning, do **not** generate speculative code blocks or unsolicited sample implementations. Focus on:
  - High-level system architecture and data flow.
  - UI/UX feedback, layout specifications, and component boundaries.
  - Verification of safety invariants, failure modes, and edge cases.
- **Direct Implementation on Approved Tasks:** When explicitly tasked with implementing features, fixing bugs, or writing tests, directly write and modify the code and verify it with the appropriate test commands.
- **Svelte Module Reference Syntax (CRITICAL):**
  - Always reference Svelte view and component modules using a bare `@` prefix **without quotes** so IDE navigation works:
    - ✅ `@DesignDetailView.svelte`, `@MainView.svelte`
    - ❌ `'@DesignDetailView.svelte'`, `"@MainView.svelte"`, `DesignDetailView.svelte`

- **Documentation Updates on User Task Changes:** When a change alters what a user does — a new task, changed steps, a renamed or moved control, a removed feature, or a new decision the user must make — update the relevant in-app help (@HelpView.svelte) and user-facing guidance (in `docs/User-Facing-Guidance/`), and maintain test coverage in `HelpView.test.ts`. Changes to internal behaviour that do not alter the user's task do **not** require a user documentation update; record them in developer documentation instead. All user documentation must follow the **User Documentation Standard (Minimalism)** below.

## 📖 User Documentation Standard (Minimalism)

User documentation (`docs/User-Facing-Guidance/` and @HelpView.svelte) follows John Carroll's **Minimalism** approach. Users read help in order to *do* something; the on-screen labels already explain what each control is. Documentation exists to help users reach a goal, make decisions the screen cannot make for them, and recover when things go wrong.

### The four principles

1. **Action first:** Organise every page around real user goals. Headings are tasks phrased as verbs ("Move your catalogue to another drive", "Reclaim disk space"), not screen regions or field names. Open with at most one sentence of context, then the steps.
2. **Anchor in the user's world:** Use embroidery and everyday vocabulary. Explain *why* a user would do something in user terms (e.g. "frees up disk space"), never *how* the app does it internally.
3. **Support error recognition and recovery:** Each page ends with a short "If something goes wrong" section covering the problems users actually hit and what to do about them.
4. **Support reading to do, and finding:** Steps are short and scannable so users can follow them with the app open; pages are easy to scan for the task they need.

### What to write

- Numbered steps, one action per step, ideally no more than 7 steps per task.
- Exact on-screen labels in **bold** (as they appear in the UI), and clickable in-app links for routes.
- Guidance only where the user faces a **decision** that is not obvious from the screen (e.g. whether to tick the free tier box, which master formats to enable). Recommend a choice where one exists.
- Rough size guide: a task section fits on one screen (around 15 lines); a whole page stays under about 100 lines. Split or trim rather than grow.

### What not to write (belongs in developer documentation)

- Field-by-field descriptions that repeat on-screen labels or helper text.
- Transient UI states and visuals (button text while busy, indicator colours, toast wording, modal component names).
- Internal implementation: SQL/PRAGMA statements, caching, thresholds, thread counts, internal paths, algorithms, matching rules, file-level pairing logic.
- Defaults, minimums, maximums and step sizes, unless the user needs them to make a decision.
- History and rationale: evaluation results, dates, "previously…", or change-log style notes.
- One-off caveat notes appended for each change. When a change affects a page, **revise the existing steps** rather than adding another note.

Put that material in `docs/Specs/`, `docs/architecture_decisions/` (ADRs) or `docs/Help for Developers/` instead, so it stays available to contributors.

### When updating existing pages

- Prefer editing or deleting over appending. If a page grows, check whether detail has crept in that belongs in developer documentation.
- Keep the British English and clickable in-app link invariants.
- **British English Spelling Invariant:** All user-facing text, UI labels, tooltips, dialogs, error messages, in-app help (@HelpView.svelte), and guidance documentation (`docs/User-Facing-Guidance/`) must strictly use **British English** spelling (`colour`, `behaviour`, `optimise`, `recognise`, `normalise`, `customise`, `licence` [noun], `cancelling`). Technical identifiers in code or backticks remain exact.

### Svelte Module Inventory (Current as of October 2026)

**Primary Views (`frontend/src/lib/views/` & `frontend/src/lib/`):**

- @AboutDocumentView.svelte
- @AboutView.svelte
- @BackupView.svelte
- @BatchOperationsView.svelte
- @BrowseView.svelte
- @DatabaseRecoveryView.svelte
- @DesignDetailView.svelte
- @DesignPrintView.svelte
- @HelpView.svelte
- @ImportView.svelte
- @InitialSetupView.svelte
- @LicenceActivationView.svelte
- @MainView.svelte
- @OrphansView.svelte
- @ProjectsView.svelte
- @ReferenceDataView.svelte
- @SettingsView.svelte
- @SystemMaintenanceView.svelte
- @TagsView.svelte
- @TagWordMatchesView.svelte

**Shared Components & Modals (`frontend/src/lib/components/`):**

- @App.svelte
- @AlphabetCompressionSetting.svelte
- @BrowseActiveFilterChips.svelte
- @BrowseCardGrid.svelte
- @BrowseFilterCheckboxList.svelte
- @BrowseFilterPanel.svelte
- @BrowseSelectionBar.svelte
- @CancelBackupModal.svelte
- @ConfirmCompactDatabaseModal.svelte
- @ConfirmDeleteProjectModal.svelte
- @ConfirmRestoreModal.svelte
- @DeleteDesignsModal.svelte
- @FirstImportSuccessBanner.svelte
- @MasterFormatsSelector.svelte
- @Notice.svelte
- @Pagination.svelte
- @QuickAddEntityModal.svelte
- @RestoreProgressPanel.svelte
- @SelectionHeader.svelte
- @TagCombobox.svelte
- @TagSelectionModal.svelte
- @TagTable.svelte
- @TagWordMatchModal.svelte
- @TechnicalDataGrid.svelte
- @ToastContainer.svelte
- @UnmatchedFilesReconciler.svelte

---

## 📋 Pre-Execution Planning Rules (Required Before Code Changes)

When tasked with feature implementation, refactoring, or cross-boundary changes, produce a **concise structural implementation plan** and await approval before touching files.

The plan must be structural and high-level—**no code snippets or pseudocode**—and must strictly cover:

1. **Affected Files & Scope Boundary:**
   - Specific files to create or modify (using `@Component.svelte` format for Svelte files).
   - Reusable/shared components that are intentionally **not** to be touched (e.g., @Pagination.svelte, @TechnicalDataGrid.svelte).
2. **Boundary & IPC Contracts (Tauri v2 ↔ Svelte):**
   - Exact `#[tauri::command]` names and argument signatures.
   - Enforce camelCase keys for JS `invoke()` payloads.
   - Rust return types and their matching TypeScript interfaces.
3. **Specification & Safety Constraints Check:**
   - Affirm that source embroidery files remain strictly read-only and unmutated.
   - SQLite queries / schema adjustments involved.
   - File cache / thumbnail storage locations.
   - **Tag Synonym Planning Invariants:**
     - *No Singular/Plural Duplication:* The tagging engine automatically computes inflections (`to_singular()` and `to_plural()`). Plans must never include both singular and plural forms (use only singular base words: `quote`, `bookmark`).
     - *No Self-Synonyms:* Tag names are directly token-matched in singular and plural forms during Pass 2. Plans must never include synonyms matching words already in the tag name (e.g. do not suggest `word`/`words` for `Words`, or `alphabet`/`alphabets` for `Alphabets`).
   - **Import-Relevant System Settings Planning Invariants:**
     - *Pipeline Impact & First-Run Audit:* Any system setting or configuration that influences file discovery, scanning, master/stitch pairing, archive extraction, cataloguing granularity, or design filtering during import must be audited for first-run onboarding impact.
     - *Setup Wizard Placement & Safe Defaults:* If importing immediately after onboarding could result in unwanted, noisy, or irreversible database state (e.g. uncompressed alphabet glyph bloat, unindexed master formats), the plan must explicitly specify:
       1. Exposure and placement in @InitialSetupView.svelte (e.g. within the Import Settings onboarding step).
       2. A safe default value (e.g. enabling compression by default to protect user libraries).
       3. Dual persistence across both @InitialSetupView.svelte and @SettingsView.svelte via `saveSettings`.
       4. Consistency across both import stages: scanning precheck (`precheck.rs`) and background executor (`executor.rs`).
    - **British English Spelling Invariant:**
      - All user-facing copy, documentation (`docs/User-Facing-Guidance/`), in-app help (`@HelpView.svelte`), UI labels, tooltips, modals, notifications, and error messages must strictly use British English spelling (`colour`, `behaviour`, `optimise`, `recognise`, `normalise`, `licence` as noun).
    - **Clickable In-App Navigation Links Invariant:**
      - Desktop application users have no browser address/URL bar. Never display raw URL hash strings or instruct users to type paths.
      - Any in-app route reference in documentation (`docs/User-Facing-Guidance/`) or in-app help (@HelpView.svelte) must be a clickable markdown link using the Svelte hash route (e.g. `[Admin → About](#/about)`, `[Admin → Settings](#/admin/system/settings)`).
4. **Step-by-Step Execution Sequence & Test Plan:**
   - A short, numbered list of the order of execution.
   - Targeted unit and integration test commands (Vitest, `cargo test`).
   - **Subsystem-Targeted Test Scope Invariant:** Plans must right-size testing to the affected subsystem:
     - *Frontend-only changes:* Specify Vitest (`npx vitest run <path>`) and Playwright E2E; explicitly omit `cargo test`.
     - *Backend or full-stack changes:* Specify targeted `cargo test <module>` or `cargo test`, plus Vitest if frontend files are modified.
     - *Documentation-only changes:* Specify markdown link/routing verification; omit `cargo test`. Never run `cargo test` if zero Rust source files, Cargo configurations, or embedded assets are touched.
   - **Mandatory Playwright E2E Coverage:** For any UI changes, interactive workflows, modal behavior, button states (e.g. enabled, disabled, busy), or navigation changes, explicitly specify the Playwright E2E test file (`tests/e2e/*.spec.ts`) and test scenario to add or update.
5. **Documentation Updates Plan:**
   - Explicitly identify the user-facing guidance documents (`docs/User-Facing-Guidance/*.md`) and the section(s) in @HelpView.svelte to update, along with corresponding assertions in `HelpView.test.ts`. Updates must follow the User Documentation Standard (Minimalism): state which user task changes and how its steps change.
   - If no user task changes, state "No user task change — no user documentation update" and identify the developer documentation (`docs/Specs/`, ADRs, `docs/Help for Developers/`) to update instead, if any.
