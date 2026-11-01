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

- **Mandatory Documentation Updates on Functional Changes:** Whenever user-facing functionality, UI workflows, or features are introduced, modified, or extended, you must update the relevant in-app help (@HelpView.svelte) and user-facing guidance (in `docs/User-Facing-Guidance/`), and maintain test coverage in `HelpView.test.ts`.
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
   - **Mandatory Playwright E2E Coverage:** For any UI changes, interactive workflows, modal behavior, button states (e.g. enabled, disabled, busy), or navigation changes, explicitly specify the Playwright E2E test file (`tests/e2e/*.spec.ts`) and test scenario to add or update.
5. **Documentation Updates Plan:**
   - Explicitly identify the user-facing guidance documents (`docs/User-Facing-Guidance/*.md`) and the section(s) in @HelpView.svelte to update, along with corresponding assertions in `HelpView.test.ts`.
