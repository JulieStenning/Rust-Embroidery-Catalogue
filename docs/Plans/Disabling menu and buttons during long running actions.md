All three decisions are confirmed. Here is the updated, final plan with the confirmations locked in.

---

## Final Implementation Plan — Global UI State Lock

### Confirmed decisions
1. ✅ **Block all programmatic routing while busy** (`navigateTo()` early-returns, nav/footer/Back links guard via `event.preventDefault()`, and `syncRouteFromHash()` snaps back on stray hash changes).
2. ✅ **`@TagView.svelte` → `@TagsView.svelte`** (correct real filename). **`@Inspector.svelte` dropped** (does not exist in the codebase).
3. ✅ **`@BrowseView.svelte` included** in scope (owns bulk tag / verify / delete / add-to-project long-running actions).

---

## 1. Architecture & Shared State Strategy

### New store: `frontend/src/lib/stores/busyStore.ts`
Reference-counted `writable` (mirrors `toastStore.ts` / `restoreProgressStore.ts` conventions):

```ts
import { writable } from "svelte/store";

export interface BusyState {
  active: boolean;
  label: string;   // human-readable operation name (tooltips / aria)
  count: number;   // ref count — safe against nested/overlapping guards
}

const idle: BusyState = { active: false, label: "", count: 0 };
export const busyState = writable<BusyState>(idle);

export function beginBusy(label: string): void {
  busyState.update((s) => ({ active: true, label, count: s.count + 1 }));
}
export function endBusy(): void {
  busyState.update((s) => {
    const count = Math.max(0, s.count - 1);
    return count === 0 ? { ...idle } : { ...s, count };
  });
}
```

**Wiring rule:** every long-running async operation wraps its body in `beginBusy(label)` / `endBusy()` inside `try { … } finally { … }`. `busyState.active` is the **single source of truth** for "an operation is running".

**Consumption:**
- `MainView.svelte` derives `busyActive` and locks all chrome.
- Each view adds `$busyState.active` to the `disabled` of **secondary/non-essential** controls.
- **Stop/Cancel controls are never fed the global flag** — they remain gated only by their own local flag, guaranteeing they stay active.

### CSS / cursor rules
Button cursor behaviour is **already implemented** in `app.css` (`.menu-button-primary[disabled]`, etc. → `cursor: not-allowed`), which is exactly what "Backup Database Now" uses. **Add a nav-link disabled variant:**

```css
.menu-link[aria-disabled="true"],
.menu-link-disabled {
  cursor: not-allowed;
  opacity: 0.5;
  /* deliberately NOT pointer-events:none, so the not-allowed cursor is observable on hover */
}
.menu-link[aria-disabled="true"]:hover {
  text-decoration: none;
}
```

---

## 2. Page-by-Page Implementation Steps (ordered)

### Step 0 — New store
- Create `@busyStore` (`frontend/src/lib/stores/busyStore.ts`) as above.
- Add `busyStore.test.ts`.

### Step 1 — `@MainView.svelte` (global chrome — highest priority)
- Import `busyState`; derive `busyActive`.
- **Top nav:** every primary + admin `<a>` gets `aria-disabled={busyActive}`, `menu-link-disabled` class when busy, and `onclick={(e) => busyActive && e.preventDefault()}`.
- **`navigateTo()`:** early-return when `busyActive` (confirmed — block all programmatic routing).
- **`syncRouteFromHash()`:** if `busyActive`, restore hash to the prior/current route and return (no loop: compare before assigning).
- **Back button:** disabled while busy.
- **Footer links** (`About`, `Licence`): same `aria-disabled` + `onclick` guard.
- **Tests:** update **both** suites — `frontend/src/__tests__/MainView.test.ts` and `frontend/src/lib/__tests__/MainView.test.ts`.

### Step 2 — `@App.svelte`
- No action buttons; optionally render a global busy indicator. Otherwise no code change (verify `ToastContainer` stays passive).

### Step 3 — `@BackupView.svelte`
- Wrap `runDatabaseBackup`, `runDesignsBackup`, `runBothBackups`, `restoreDatabase`, `restoreDesignsIncremental`, `restoreBoth`, `handleImportUnmatched` with `beginBusy`/`endBusy`.
- **Exception:** Cancel-backup button + `CancelBackupModal` confirm + restore cancel stay active (local flags only).
- Add `$busyState.active` to secondary controls (Browse, Save settings, tab switching, "Sync designs idle", "Dismiss").
- **Tests:** `BackupView.test.ts`.

### Step 4 — `@ImportView.svelte`
- Wrap `executeImportPrecheckAction("import_now")` (and `runPrecheckAction` if long-running) with `beginBusy`/`endBusy`.
- **Exception:** **Stop** button (and "Stopping…" state) stays active — never feed it `$busyState`.
- Add `$busyState.active` to step 1–3 secondary controls (Browse…, Remove, Select/Deselect all, Cancel, "Continue with N designs", per-folder selects).
- **Tests:** `ImportView.test.ts` (+ `ImportTestHarness.svelte` fixture if needed).

### Step 5 — `@TaggingActionsView.svelte`
- Wrap `runTaggingActions()` with `beginBusy`/`endBusy`.
- **Exception:** **Stop** stays enabled while `taggingRunInFlight` (already `disabled={!taggingRunInFlight}`).
- Add `$busyState.active` to Run button + all checkboxes/sub-options.
- **Tests:** `TaggingActionsView.*.test.ts` (esp. `stop`, `mount`, `checkboxes`).

### Step 6 — `@SettingsView.svelte`
- Wrap `startCatalogueStorageMigration` (`migrating`) and `runManualCompaction` (`isCompacting`) with `beginBusy`/`endBusy`.
- **Exception:** migration **Cancel** button + migration-error modal **Close** stay active.
- Add `$busyState.active` to secondary controls (Browse data-root, Save, Compact, API-key reveal, Restart now).
- **Tests:** `SettingsView.test.ts`.

### Step 7 — `@OrphansView.svelte`
- Wrap scan + `deleteAllOrphans` with `beginBusy`/`endBusy` (add local flags if absent).
- Disable secondary controls (scan/delete/select-all).
- **Tests:** `OrphansView.test.ts`.

### Step 8 — `@ProjectsView.svelte`
- Wrap create/update/delete/remove-design ops with `beginBusy`/`endBusy`.
- Disable secondary controls.
- **Tests:** `ProjectsView*.test.ts` (4 files).

### Step 9 — `@TagsView.svelte` (confirmed name)
- Wrap tag create/update/delete (`TagTable` mutations) with `beginBusy`/`endBusy`.
- Disable secondary controls; keep in-progress confirm active.
- **Tests:** `TagsView.test.ts` + `TagSelectionModal.test.ts`.

### Step 10 — `@DesignDetailView.svelte` & `@DesignPrintView.svelte`
- `@DesignDetailView.svelte`: wrap save (`detailSaving`), `reparseDesignFile`, delete with `beginBusy`/`endBusy`; disable secondary controls.
- `@DesignPrintView.svelte`: read-only print view — minimal/no change (verify no long-running trigger).
- **Tests:** `DesignDetailView.test.ts`, `DesignPrintView.test.ts`.

### Step 11 — `@BrowseView.svelte` (confirmed in scope)
- Wrap `bulkSetTagsForDesigns`, `bulkDeleteDesigns`, `bulkVerifyDesigns`, `bulkAddDesignsToProject` with `beginBusy`/`endBusy`.
- Disable filters, selection actions, Pagination, SelectionHeader.
- **Exception:** delete-confirm modal Confirm/Cancel stay active.
- **Tests:** `BrowseView.test.ts` + `SelectionHeader.test.ts` + `Pagination.test.ts`.

### Step 12 — `@TagSelectionModal.svelte` & `@DeleteDesignsModal.svelte`
- Add `$busyState.active` to non-essential controls (tag checkboxes; delete-files radio + preview toggles).
- **Exception:** Confirm / Cancel stay active.
- **Tests:** `TagSelectionModal.test.ts`, `DeleteDesignsModal.test.ts`.

### Step 13 — `@SelectionHeader.svelte`, `@Pagination.svelte`, `@TagTable.svelte`, `@TechnicalDataGrid.svelte`
- `@SelectionHeader.svelte`: disable select-all/clear + bulk triggers via `busyActive` prop (or `$busyState`).
- `@Pagination.svelte`: disable prev/next/page buttons.
- `@TagTable.svelte`: disable edit/delete/rename.
- `@TechnicalDataGrid.svelte`: read-only — verify; likely just accept a `busyActive` prop for consistency.
- **Tests:** `SelectionHeader.test.ts`; add `Pagination`/`TagTable`/`TechnicalDataGrid` tests as needed.

### Step 14 — `@AboutView.svelte`, `@AboutDocumentView.svelte`, `@HelpView.svelte`, `@DisclaimerView.svelte`, `@Notice.svelte`, `@ToastContainer.svelte`
- Read-only/passive. Guard any cross-route in-page `<a>` links with `aria-disabled` + `onclick` (AboutDocument/Help cross-links).
- `@Notice.svelte` and `@ToastContainer.svelte`: **no changes** (display-only).
- **Tests:** `AboutView.test.ts`, `AboutDocumentView.test.ts`, `HelpView.test.ts`, `Notice.test.ts`.

### Step 15 — `@ImportTestHarness.svelte`
- Test helper only; add a `beginBusy`/`endBusy` mock so `ImportView.test.ts` busy assertions are deterministic. No production change.

---

## 3. Automated Test Suite Plan

**New file:**
- `frontend/src/lib/stores/__tests__/busyStore.test.ts` — ref-count begin/end, reset-to-idle at zero, label propagation, safe double-`endBusy`.

**Shared helper:** a small test helper (or `vi.hoisted()` mocks) exposing `busyState.set({ active: true, label: "...", count: 1 })` + reset, so any view test can toggle the global signal without a real backend run.

**Matrix (disable-on-busy / stop-cancel-active / not-allowed):**

| Test file | Disable-on-busy | Stop/Cancel active | `not-allowed` assertion |
|---|---|---|---|
| `MainView.test.ts` (×2) | nav + Back + footer get `aria-disabled="true"` | n/a | `.menu-link` ⇒ `cursor: not-allowed` |
| `BackupView.test.ts` | secondary disabled | cancel-backup + restore cancel | `disabled` ⇒ `not-allowed` |
| `ImportView.test.ts` | step controls | **Stop** / "Stopping…" | `disabled` ⇒ `not-allowed` |
| `TaggingActionsView.*.test.ts` | Run + checkboxes | **Stop** | `disabled` ⇒ `not-allowed` |
| `SettingsView.test.ts` | Save/browse/compact | migration **Cancel** + error-modal **Close** | `disabled` ⇒ `not-allowed` |
| `OrphansView.test.ts` | scan/delete/select | scan cancel (if present) | `disabled` ⇒ `not-allowed` |
| `ProjectsView*.test.ts` | save/delete/remove | confirm-modal buttons | `disabled` ⇒ `not-allowed` |
| `TagsView.test.ts` + `TagSelectionModal.test.ts` | tag controls | Confirm/Cancel | `disabled` ⇒ `not-allowed` |
| `DesignDetailView.test.ts` | edit/delete/open | delete-confirm | `disabled` ⇒ `not-allowed` |
| `BrowseView.test.ts` + `SelectionHeader.test.ts` + `Pagination.test.ts` | filters/selection/pagination | delete/verify confirm | `disabled` ⇒ `not-allowed` |
| `AboutDocumentView.test.ts` / `HelpView.test.ts` | cross-links blocked | n/a | link `aria-disabled` + `not-allowed` |
| `busyStore.test.ts` | n/a | n/a | n/a |

**Standardised patterns (aligning with project rules):**
1. `tick()` to flush synchronous reactive updates after toggling the store; `waitFor` only for genuinely async flows.
2. Assert the wire attribute CSS keys off: `toHaveAttribute("aria-disabled", "true")` for links, `toBeDisabled()` for buttons; document that jsdom can't reliably compute `:hover` cursor, so assert class/`aria-disabled` (or a computed-style check if already used in the suite).
3. Write explicit **"must stay active"** negative assertions for Stop/Cancel (mirrors the project's must-NOT-happen test discipline).
4. Use `within()` scoping for duplicated copy and `vi.hoisted()` fixtures.

**Verification gates (Act mode):**
- `cmd /c "npx vitest run"` from repo root (never inside `frontend/`).
- `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"`.
- Run **both** `MainView` suites as part of the full run.

---

The plan is now finalised and ready for implementation. Toggle to **Act mode** and I'll begin with `busyStore.ts` + its unit test, then `MainView.svelte` chrome locking, proceeding page by page.