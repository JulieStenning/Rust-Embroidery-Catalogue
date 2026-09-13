## 📋 User Test Suite: Bulk Import — Step 1: Folder Selection & Scan

This issue tracks the user-facing functionality for **step 1** of the Bulk Import wizard
(`#/import`, which is the same view as `#/import/step1`) based on `image_d0cbb9.jpg`.
Step 2 is tracked by `Bulk Import - Review Scanned Files.md`; step 3 by
`Bulk Import - Pre-Import Actions.md`.

> **Automated coverage:** Step 1 is automated end-to-end by
> `tests/e2e/import-folder-selection.spec.ts` (Playwright driving the real Tauri app) and
> `tests/e2e/import.spec.ts`. Native folder-picker behaviour is covered by unit tests in
> `frontend/src/lib/views/__tests__/ImportView.test.ts` (describes *step 1 folder path
> management*, *browse flows* and *step 1 preview submission*): Playwright cannot drive an
> OS file dialog. The manual checks below cover the end-to-end user experience.

> **No AI tagging happens at import.** Import runs File &amp; Folder Rules only (offline,
> no API key). Visual AI tagging runs later from **Batch Operations** — see
> `docs/User-Facing-Guidance/IMPORT_WORKFLOW.md` and `BATCH_OPERATIONS_BACKFILL.md`.

### ⚙️ Test Setup & Prerequisites

Before executing these tests, prepare the following local environment states:
- **Empty State**: Ensure no previous import folder path is cached (the wizard remembers
  the last browsed folder in settings as `import.last_browse_folder`).
- **Supported formats**: `jef`, `pes`, `hus`, `dst`, `exp`, `vp3` — any other extension is
  ignored by the scanner.
- **Sample Directory Structures**:
  - `C:\EmbroideryTests\ValidFolder` (containing sub-folders with `.jef`, `.pes` and `.vp3` files)
  - `C:\EmbroideryTests\EmptyFolder` (completely empty)
  - `C:\EmbroideryTests\MixedFolder` (nested sub-folder with a design, plus `.txt` / `.pdf` decoys at the root)
  - `X:\EmbroideryTests\DoesNotExist` (a path that does not exist on disk)

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load**
  - [ ] Screen opens without lag or visual stutter from the "Import" top navigation item
  - [ ] Default layout elements and text render correctly exactly matching `image_d0cbb9.jpg`:
    - [ ] Title header text: "Bulk Import"
    - [ ] Instruction text: "Select one or more folders containing embroidery files.
      Sub-folders are included automatically." and "Your original files are never altered
      or moved. Files outside your main design directory are safely copied into the
      catalogue."
    - [ ] Inline "Import help" link, which opens `#/help?section=importing`
    - [ ] Form field label: "Source Folder(s) *" (a real `<label>` bound to the first input)
  - [ ] Default UI element interactive states:
    - [ ] A single text input row is visible with placeholder text: "Enter path to your embroidery designs folder…"
    - [ ] Input text field is empty by default
    - [ ] "Browse…" button (note the single-character ellipsis) is active and enabled
    - [ ] "Remove" button is **disabled while the input is empty** (it is not disabled
      merely because it is the only row) — **Automated (e2e)**
    - [ ] "Add another folder" button is **disabled while the primary input is empty** — **Automated (e2e)**
    - [ ] "Scan folder(s)" action button is **disabled** while no folder path is present (no
      validation toast is shown; the control is simply inert) — **Automated (e2e)**
    - [ ] "Reset" action button is **disabled** while no folder path is present — **Automated (e2e)**
  - [ ] Application window is responsive, scaling UI cards correctly across standard desktop resolutions

- [ ] **Primary Action: Entry Point Validation & Basic Interactivity**
  - [ ] **Text Input Field Focus & Manual Entry**
    - [ ] Clicking into the text box focuses the cursor correctly
    - [ ] Manually typing a valid path string preserves the string literal value — **Automated (e2e)**
    - [ ] Pasting a path string into the text box behaves correctly without truncation
    - [ ] Backslashes and duplicated separators are normalised to forward slashes, trailing
      slashes are stripped, and a bare drive root keeps a single trailing slash —
      **Automated (unit):** `ImportView.test.ts` → "normalises backslashes, trailing
      slashes, and drive-letter roots on submit"
  - [ ] **Action Component Initial States**
    - [ ] Hover states for "Browse…", "Add another folder", "Scan folder(s)", and "Reset" change cursor/background color smoothly
    - [ ] Focus ring or highlight transfers smoothly via keyboard Tab key navigation across all form elements

- [ ] **Primary Action: Folder Selection & Browse Window Behavior**
  - [ ] **Default/Fallback Open Directory**: Clicking **Browse…** on an empty row opens the
    native folder dialog seeded from the persisted last-browse folder, falling back to the
    system default when that folder no longer exists
    - **Automated (unit):** `ImportView.test.ts` → "seeds the picker from the persisted last
      browse folder when the row is empty", "uses the persisted parent for a single
      previously browsed folder", "prefers the current row's parent over the persisted
      folder when the row is populated"
  - [ ] **Session Memory Open Directory**: Clicking **Browse…** after a folder has already
    been chosen opens the dialog focused on that folder's parent — **Automated (unit):**
    same describe as above; the chosen path is persisted through
    `saveImportLastBrowseFolder`
  - [ ] **Multi-Folder OS Selection**: one or several folders may be selected at once; the
    first selection fills the row that launched the dialog and every extra selection is
    appended as its own row — **Automated (unit):** "fills additional rows when
    multi-selection returns extra paths", "keeps all four folders in order with no
    duplication when four are selected"
  - [ ] **Path Overwrites via Re-Selection**: clicking **Browse…** on a row that already
    contains a path replaces that specific path — **Automated (unit):** "updates an extra
    row when its browse button is used"
  - [ ] A cancelled picker leaves the form untouched, and a picker failure raises an error
    toast without breaking the view — **Automated (unit):** "does nothing when the folder
    picker is cancelled", "shows an error toast when the folder picker throws"

- [ ] **Dynamic Row Management (Add / Remove)**
  - [ ] **Add Another Folder Interaction**: with a path in the primary input, clicking
    **Add another folder** appends a new row — **Automated (e2e)**
    - [ ] The new row contains its own **read-only** text input (additional rows are filled
      by **Browse…** / multi-select, they are not typable), its own **Browse…** button, and
      its own active **Remove** button — **Automated (e2e)**
    - [ ] Clicking **Browse…** on the added row opens the same native dialog with the same
      single/multi-selection and session-memory behaviour — **Automated (unit)**
  - [ ] **Remove Folder Interaction**: clicking the **Remove** button next to any added row
    deletes only that row, leaving neighbouring inputs untouched — **Automated (e2e)**
  - [ ] **Row Constraint Safeguards**: when the primary row is the only row, its **Remove**
    button simply clears the input (the wizard does not enforce a minimum row count by
    disabling Remove) — **Automated (e2e)**
  - [ ] Duplicate paths are rejected case-insensitively, so the same folder can never be
    scanned twice — **Automated (unit):** "deduplicates root paths case-insensitively"

- [ ] **Form Clearing & State Resets**
  - [ ] **Reset Button Execution**: with at least one path entered, clicking **Reset**
    flushes the current operational state completely — **Automated (e2e)**
    - [ ] All added folder rows are instantly removed
    - [ ] The interface collapses back down to the exact single-row template layout
      pictured in `image_d0cbb9.jpg`
    - [ ] The primary text input is completely cleared, restoring the placeholder string
    - [ ] "Scan folder(s)" and "Reset" become disabled again
    - [ ] No confirmation prompt is shown (Reset is immediate and non-destructive to source files)


- [ ] **Primary Action: Scan Folder(s) Execution & Navigation Handshake**
  - [ ] **Input Validation Trigger**
    - [ ] **No Path Specified**: **Scan folder(s)** is disabled while the form is empty, so
      it cannot be submitted at all (there is no inline validation error on step 1) — **Automated (e2e)**
    - [ ] **Non-Existent Path**: typing a path that does not exist and clicking
      **Scan folder(s)** still runs the scan and moves the user to step 2, where an amber
      panel reads "No supported files discovered in this preview." together with
      "The selected folder(s) could not be found on disk. Check that the path is correct and
      the drive is available." and a **Back to Step 1** button — **Automated (e2e)**
    - [ ] **Empty Folder**: scanning an existing but empty folder shows the same amber panel
      with "No supported embroidery files (JEF, PES, HUS, DST, EXP, VP3) were found in the
      selected folder(s)." — **Automated (e2e)**
    - [ ] **Back to Step 1** returns to the picker with the previously typed path still in place
  - [ ] **Scanning Engine Processing States**
    - [ ] **Loading Indicator & UI Blocking**: while the scan runs, the action button reads
      "Running…", the path inputs and Browse…/Remove/Add buttons are disabled, and the
      global busy lock suppresses other long-running navigation — **Automated (unit):**
      "shows Running... on the submit button while preview is pending", "raises the global
      busy lock while scanning folders and clears it after"
    - [ ] **Background Processing Fluidity**: the application window remains responsive
      while the backend crawls the target directory structures
  - [ ] **File Format Identification & Filtering Logic**
    - [ ] **Valid Formats Extraction**: the scanner recursively traverses the targeted
      directories and picks up `jef`, `pes`, `hus`, `dst`, `exp` and `vp3` files
    - [ ] **Sub-folder Auto-Inclusion**: directories nested inside the chosen folder are
      parsed automatically without adding a second row — **Automated (e2e)**
    - [ ] **Non-Embroidery File Exclusion**: files that do not match a supported format
      (`.txt`, `.pdf`, `.jpg`, system metadata, …) are skipped and never appear in the
      import manifest — **Automated (e2e)**
    - [ ] **Already-Catalogued Exclusion**: files whose prospective stored path — or whose
      `(filename, size, hash)` fingerprint — already matches a catalogue row are filtered
      out of the preview, so "N file(s) found" counts only *new* files
  - [ ] **Import Rule Enforcement (Safety Check)**
    - [ ] **Source Integrity**: scanning does not alter, rename, move, or touch the
      timestamps of any source file — **Automated (e2e):**
      `import-folder-selection.spec.ts` snapshots name/mtime/size before and after a scan
    - [ ] **External Source Copy Management**: files outside the main design directory are
      **copied** into the catalogue repository at commit time, never moved — **Automated (e2e):**
      `import.spec.ts` asserts the source file still exists after a completed import
  - [ ] **Successful View Transition**
    - [ ] **Navigation to Next Page**: a scan that finds matches moves the user to the
      **Review scanned files** step (`#/import/step2`) — **Automated (e2e)**
      (step 2 itself is tracked by `Bulk Import - Review Scanned Files.md`)

- [ ] **Navigation**
  - [ ] User can safely exit or go back to "Browse", "Projects", or "Help" via the top navigation bar without freezing the app — **Automated (e2e)**
  - [ ] Leaving the page is unconditional: there is **no** "unsaved changes" prompt at any
    point in the wizard
  - [ ] Returning to **Import** after leaving restores whatever the user had entered (the
    wizard state is kept in `importSessionStore`) — **Automated (e2e)**
  - [ ] Returning to `#/import/step2` or `#/import/step3` after visiting another page
    restores the scan and every selection — **Automated (unit):** "restores step 3 with all
    selections after ImportView is unmounted and remounted"

---

### 🔗 Step Map

| Route | Step | Tracked by |
|---|---|---|
| `#/import` / `#/import/step1` | Folder selection & scan | this document |
| `#/import/step2` | Review scanned files | `Bulk Import - Review Scanned Files.md` |
| `#/import/step3` | Before You Import | `Bulk Import - Pre-Import Actions.md` |

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*

