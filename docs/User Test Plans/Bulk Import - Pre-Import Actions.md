## 📋 User Test Suite: Bulk Import — Step 3: Before You Import

This issue tracks the user-facing functionality for the final Bulk Import decision step
(`#/import/step3`) which follows the review-selection screen
(`Bulk Import - Review Scanned Files.md`).

> **Automated coverage:** Step 3 is automated end-to-end by `tests/e2e/import.spec.ts`
> (Playwright driving the real Tauri app) and `tests/e2e/import-hoop-setup.spec.ts`
> (the first-import hoop gate, on throwaway empty data roots), plus unit tests in
> `frontend/src/lib/views/__tests__/ImportView.test.ts` (describes *step 3 actions*,
> *ImportView bulk import progress events*, *stale context token recovery*).

> **⚠️ AI tagging is NOT part of the import.** Import runs **File &amp; Folder Rules only**:
> it is fast, local and needs no Google API key, and it applies free keyword/stitching
> tags. Visual AI tagging is a separate operation run afterwards from
> **Admin → Batch Operations** (`docs/User-Facing-Guidance/BATCH_OPERATIONS_BACKFILL.md`).
> An earlier revision of this document described an API-key banner, live Tier 2/Tier 3
> counters, a "Change in Settings" link and a per-session 2D/3D preview picker on this
> screen. **None of those controls exist any more** — see the negative checklist below.

### ⚙️ Test Setup & Prerequisites

Before executing these tests, prepare the following local environment states:
- **Catalogue State A (normal import)**: a catalogue that already has at least one hoop
  configured, so the first-import gate does not appear.
- **Catalogue State B (first import)**: an empty catalogue with **no designs and no
  hoops**, to exercise the hoop-setup confirmation gate.
- **API Key**: not required for any test in this document. The screen must behave
  identically whether or not a Google API key is configured — that is itself a test case.

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load & Layout Verification**
  - [ ] Screen opens smoothly immediately following the "Continue with [X] designs" review handshake
  - [ ] Default layout elements and informational blocks render correctly:
    - [ ] Title header text: "Bulk Import"
    - [ ] Sub-section label header: "Before You Import"
    - [ ] Info panel header: "Note on Visual AI Tagging"
    - [ ] Info panel body: "Initial import uses fast, offline File & Folder Rules to index your designs instantly. Once finished, you can run automated Visual AI tagging anytime from Batch Operations to enrich your collection." — **Automated (e2e)**
  - [ ] Action buttons render clearly in a horizontal control layout:
    - [ ] Execution actions: "Import Designs" and "Cancel" — **Automated (e2e)** (the former "Review Hoops / Review Tags / Review Sources / Review Designers" buttons and the "Continue with import" return flow have been removed)
  - [ ] **Removed-UI negative checks** (these must all be absent — **Automated (e2e)** as
    explicit negative assertions, so the obsolete controls cannot silently return):
    - [ ] No API-key / AI-tagging banner ("Google AI tagging is enabled for this installation")
    - [ ] No rate-limit copy ("15 requests per minute and 1,500 requests per day")
    - [ ] No live tier counters ("Tier 2 auto:", "Tier 3 auto:", "AI batch limit:",
      "DB commit batch:")
    - [ ] No "Change in Settings" inline link
    - [ ] No preview-preference radio group ("2D - Fast flat preview", "3D - Detailed
      stitch simulation", "(Saved setting: …)")
    - [ ] No "Review Hoops" / "Review Tags" / "Review Sources" / "Review Designers" buttons

- [ ] **AI-Tagging Information Note**
  - [ ] With a Google API key configured **and** with none configured, the panel text is
    byte-for-byte identical (no conditional branch) — **Automated (e2e)**
  - [ ] Changing AI settings in **Admin → Settings** does not change anything on this
    screen, because nothing here reads them (contrast with the removed "Dynamic Sync
    Reflectivity" behaviour)
  - [ ] The note correctly directs the user to **Batch Operations** for automated Visual AI
    tagging; that page is tested separately

- [ ] **First-Import Hoop Gate** (only when `design_count == 0 && hoop_count == 0`)
  - [ ] **Gate Appears**: clicking **Import Designs** on a first import into a hoops-less
    catalogue does **not** start the import; it reveals the amber panel "Hoops are not
    configured for a first import. Confirm to continue anyway." with a
    "Confirm import without hoop setup" button — **Automated (e2e):**
    `import-hoop-setup.spec.ts`
  - [ ] **Gate Absent**: with at least one hoop configured the same button starts the
    import immediately and the gate text never appears — **Automated (e2e)** (negative
    assertion in `import-hoop-setup.spec.ts`)
  - [ ] **Confirmation Proceeds**: confirming runs the import to completion
  - [ ] **No Side Effects**: skipping hoop setup does **not** create any hoops — **Automated (e2e)**

- [ ] **Import Execution & Progress**
  - [ ] **Commit**: clicking **Import Designs** starts the background batch operation:
    copies the selected files into managed storage, writes the database rows, generates
    previews, and applies File &amp; Folder Rules tags — **Automated (e2e)**
  - [ ] **Progress Reporting**: the primary button changes to "Running Import..." and
    appends the live stage text (started / per-file / image generation / batch commit /
    completed) — **Automated (unit):** the *ImportView bulk import progress events* describe
    covers `started`, `processing_file`, `generating_images`, `batch_committed`,
    `completed`, `stopped` and the unknown-stage fallback
  - [ ] **Busy Locking**: while the import runs, the wizard's other controls are disabled
    and the global busy lock only allows the Stop request
  - [ ] **Stop**: the **Cancel** button becomes **Stop** during a run; clicking it changes
    to "Stopping..." until the backend acknowledges, then the run halts cleanly — **Automated (unit):** "requests a stop for the running import"
  - [ ] **Completion Routing**: on success the user is routed to **Browse Designs** and the
    new designs are present — **Automated (e2e)**
  - [ ] **Newest First**: with sort order set to date added / descending, the newly imported
    designs are at the front of Browse
  - [ ] **Source Files Preserved**: the original source files still exist (they are copied,
    never moved) — **Automated (e2e)**
  - [ ] **Nothing Persisted**: an import that persists zero designs does not raise the
    import-completed handoff (no false "success" state) — **Automated (unit):** "does not
    call onImportCompleted when nothing was persisted"

- [ ] **Cancel & Boundary Transitions**
  - [ ] **Cancel Before Import**: clicking **Cancel** consumes the import context and
    returns the wizard to step 1 with all rows and selections cleared — **Automated (unit):**
    "cancels and resets the wizard back to step 1"
  - [ ] **Cancel is safe**: no files are copied and no database rows are written when
    cancel is pressed before the import runs

- [ ] **Expired Import Context (long session)**
  - [ ] The backend keeps the import context under a ~15-minute token. If the user lingers
    on another page for longer than that and then presses **Import Designs**, the app shows
    "Import context expired. Re-checking your selections before retrying..." and
    transparently re-runs the precheck before retrying the action — **Automated (unit):**
    "re-runs the precheck and retries the import when the backend token expires"
  - [ ] If the context cannot be refreshed, the user sees an error and remains on a usable
    screen rather than a dead end

- [ ] **Direct Navigation Guard**
  - [ ] Opening `#/import/step3` without a completed precheck shows "Step 3 needs precheck
    to be completed first." with a **Go to previous step** button — **Automated (unit):**
    "shows the step 3 fallback when no precheck has been run"
  - [ ] Step-3 actions are disabled when the precheck returned no context token —
    **Automated (unit):** "disables step 3 actions when the precheck returns an empty
    context token"

---

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*

