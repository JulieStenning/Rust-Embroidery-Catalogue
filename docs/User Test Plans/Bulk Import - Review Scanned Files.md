## 📋 User Test Suite: Bulk Import — Step 2: Review Scanned Files

This issue tracks the user-facing functionality for the **Review scanned files** step
(`#/import/step2`) based on `image_d14eff.jpg`, which follows a successful scan from
`Bulk Import — Step 1` (`Import.md`). It hands off to
`Bulk Import - Pre-Import Actions.md` (`#/import/step3`).

> **Automated coverage:** Step 2 is automated end-to-end by `tests/e2e/import.spec.ts`
> (Playwright driving the real Tauri app), which asserts the scan summary, the global
> override panel, per-folder shells and the select-all / deselect-all counters, and by
> unit tests in `frontend/src/lib/views/__tests__/ImportView.test.ts` (describes *step 2
> file review and selection*, *step 2 global and per-folder assignment*, *step 2 very
> large folders*, *ImportView loading and disabled states*, and the *precheck flow*).

> **Import is File &amp; Folder Rules only.** No AI tagging is configured, previewed or
> run from this step; see `docs/User-Facing-Guidance/IMPORT_WORKFLOW.md`.

### ⚙️ Test Setup & Prerequisites

Before executing these tests, prepare the following local environment states:
- **Scan Directory State**: run a scan containing at least two nested directories with a
  diverse distribution of supported file formats (e.g. 112 files across `.pes`, `.hus`,
  `.jef`). Only `jef`, `pes`, `hus`, `dst`, `exp` and `vp3` are scanned.
- **Large-folder State**: include one folder holding more files than the auto-expand
  threshold so the collapsed "summary row" form is exercised.
- **Metadata State**: ensure the catalogue already has Designers and Sources so the
  override dropdowns have records to offer.
- **Sub-folder naming State**: include a folder whose name matches an existing Designer
  and another whose name does not, to compare inferred labels.

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load & Layout Verification**
  - [ ] Screen opens smoothly immediately following the "Scan folder(s)" execution handshake
  - [ ] Default layout elements and dynamic summary texts render correctly exactly matching `image_d14eff.jpg`:
    - [ ] Title header text: "Bulk Import" (page-level `<h1>`)
    - [ ] Step label: "Review scanned files"
    - [ ] Summary text displays the correct dynamic counts: "2 folder(s) scanned - 112 file(s) found. Selected files will be **copied into the catalogue**."
      - The file count is the number of **new** files after already-catalogued rows are
        filtered out, not the raw number of files on disk — **Automated (e2e)**
    - [ ] Inline text link "Import help" is present and opens `#/help?section=importing`
  - [ ] **Global Override Control UI**:
    - [ ] Header label: "Apply to all folders (optional override)"
    - [ ] "Designer" dropdown defaults to "Keep inferred (per folder)"
    - [ ] "Source" dropdown defaults to "Keep inferred (per folder)"
    - [ ] Both dropdowns list the existing Designers / Sources from the catalogue
      - **Automated (e2e)**
  - [ ] **Primary Action Buttons**:
    - [ ] Primary validation button reads: "Continue with 112 designs" (dynamically reflects the checked count, singular "design" for exactly one)
    - [ ] With nothing selected it reads "Continue" and is **disabled** (the old "Continue with 0 designs" wording is gone) — **Automated (e2e)**
    - [ ] Secondary action button reads: "Cancel"
  - [ ] **Mass Selection Utilities**:
    - [ ] "Select all" and "Deselect all" are aligned right and interactive — **Automated (e2e)**
    - [ ] "Select all" is disabled once everything is selected; "Deselect all" is disabled once nothing is selected
    - [ ] Both are disabled while a precheck is in flight — **Automated (unit):** "disables Select all / Deselect all while a precheck is in flight"

- [ ] **Folder Group Separation & Per-Folder Configurations**
  - [ ] **Folder Header Rendering**: each scanned directory renders as a discrete card
    (`data-testid="import-folder-shell"`) separating:
    - [ ] Display name and absolute system path (e.g. `The Rose Studio - Borders - D:/...`)
    - [ ] A selection-count badge: "All N selected", "N of M selected" or "None selected" — **Automated (e2e)**
  - [ ] **Per-Folder Metadata Dropdowns**:
    - [ ] "Designer for this folder" displays the inferred entity name in brackets when one can be derived (e.g. `Keep inferred (The Rose Studio)`), or a plain `Keep inferred` when none matches — **Automated (unit):** "shows the inferred designer label from path-based suggestion", "shows the inferred designer label from resolved assignments", "shows plain 'Keep inferred' when no match exists"
    - [ ] "Source for this folder" behaves identically
    - [ ] Each dropdown offers the inferred option plus the existing catalogue records —
      there is **no** "Create new" / "Leave blank" choice on this step (new reference data
      is created in Admin or via the Initial Setup wizard, not from the import wizard)
    - [ ] Selecting a record stores an explicit per-folder override and travels in the
      confirm payload — **Automated (unit):** "sets a per-folder designer override and
      includes it in the confirm wire"
  - [ ] **Large folders** (more files than the auto-expand threshold):
    - [ ] Rendered collapsed with a "Show files (N)" toggle and the note "Files are hidden
      for this large folder (N files). Use the controls above to select or deselect the
      whole folder." — **Automated (unit):** "renders a large folder collapsed with a
      summary, not per-file rows"
    - [ ] Expanding reveals a paged, filtered file list — **Automated (unit):** "pages the
      file rows of an expanded large folder"
    - [ ] "Hide files" collapses it again

- [ ] **File Grid Interactivity & Batch Selection**
  - [ ] **Checkbox States**: all successfully scanned files default to a checked/selected state on initial paint — **Automated (e2e)**
  - [ ] **Individual File Toggling**:
    - [ ] Unchecking an individual file checkbox (e.g. `97603.pes`) updates its state without affecting adjacent files
    - [ ] Unchecking a file instantly decrements the counter on the primary action button (e.g. "Continue with 112 designs" → "Continue with 111 designs") — **Automated (e2e)**
    - [ ] The folder's own badge changes from "All N selected" to "N-1 of N selected"
  - [ ] **Per-Folder Mass Actions**: each folder header carries its own "Select all" /
    "Deselect all" (distinguished by accessible name, e.g. "Select all files in <folder>")
    that apply to that folder only — **Automated (unit):** "selects/deselects a whole large
    folder without rendering its files"
  - [ ] **Global Mass Action - Deselect all**: clears the checkbox checks across all folder
    groups simultaneously; the primary button drops to "Continue" and becomes disabled — **Automated (e2e)**
  - [ ] **Global Mass Action - Select all**: re-checks every available file across all
    groups, restoring the baseline maximum counter — **Automated (e2e)**
  - [ ] **Per-Folder Filter**: the "Filter files in this folder…" box narrows the visible
    rows and resets the page to 1 when the filter changes

- [ ] **Global and Local Metadata Assignment Logic**
  - [ ] **Global Overrides**: selecting a Designer or Source in the top "Apply to all
    folders" section applies it to every folder and travels in the confirm payload as
    `global_designer_id` / `global_source_id` — **Automated (unit):** "sets a global
    designer override and includes its id in the confirm wire"
  - [ ] **Precedence**: an explicit per-folder choice wins over the global override, which
    wins over an inferred match from the folder name, which wins over nothing
    (per-folder → global → inferred → blank)
  - [ ] **Inferred ids**: when no override is chosen, the inferred Designer/Source ids are
    resolved from matching folder names and sent in the wire — **Automated (unit):**
    "includes inferred designer/source ids from matching folder names in the wire"

- [ ] **Workflow Progression & Navigation Transitions**
  - [ ] **Cancel Execution**: clicking **Cancel** returns the user to step 1
    (`#/import/step1`) — **Automated (e2e)**
  - [ ] **Continue Validation Gate**: clicking **Continue with [X] designs** runs the
    precheck and moves the user forward to the **Before You Import** step
    (`#/import/step3`) once a context token is issued — **Automated (e2e)**
    - [ ] The precheck request carries only the scan token plus the compact selection
      (not the full file list) — **Automated (unit):** "sends a compact precheck request
      (scan token + selection) without the full file list"
    - [ ] A precheck failure shows an error toast and leaves the user on step 2 —
      **Automated (unit):** "shows an error toast and stays on step 2 when precheck throws"
    - [ ] An expired scan token triggers an automatic re-scan and retries the precheck —
      **Automated (unit):** stale-scan-token recovery in the precheck flow
    - [ ] While the precheck runs the button reads "Running…", file checkboxes are
      disabled, and the global busy lock is raised — **Automated (unit):** "shows
      Running... on the continue button while precheck is pending", "disables file
      checkboxes while a precheck is running"
  - [ ] **Top Menu Redirection Safeguards**: clicking "Browse", "Projects" or "Help" in the
    global navigation strip **does not** prompt a dirty-flag warning. The wizard state is
    preserved, so returning to `#/import/step2` restores the scan, the selections and the
    metadata overrides — **Automated (e2e)** for the no-prompt behaviour, **Automated
    (unit):** "restores step 2 review with selections after unmount and remount"

---

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*

