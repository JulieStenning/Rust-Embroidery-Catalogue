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
- @BrowseCardGrid.svelte
- @BrowseFilterPanel.svelte
- @BrowseSelectionBar.svelte
- @CancelBackupModal.svelte
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
4. **Step-by-Step Execution Sequence:**
   - A short, numbered list of the order of execution and targeted test commands.
