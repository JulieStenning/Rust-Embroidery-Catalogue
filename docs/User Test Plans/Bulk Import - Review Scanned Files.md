## 📋 User Test Suite: Bulk Import - Review Scanned Files

This issue tracks the user-facing functionality for the **Bulk Import - Review Scanned Files** view based on `image_d14eff.jpg`.

### ⚙️ Test Setup & Prerequisites
Before executing these tests, prepare the following local environment states:
- **Scan Directory State**: Run a scan containing at least two nested directories with a diverse distribution of file formats (e.g., 112 files total distributed across `.pes`, `.hus`, and other embroidery extensions).
- **Metadata State**: Ensure the application has pre-populated reference data for existing Designers and Sources to validate selection overrides.

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load & Layout Verification**
  - [ ] Screen opens smoothly immediately following the "Scan folder(s)" execution handshake.
  - [ ] Default layout elements and dynamic summary texts render correctly exactly matching `image_d14eff.jpg`:
    - [ ] Title header text: "Bulk Import"
    - [ ] Scanned summary text displays the correct dynamic counts: "2 folder(s) scanned - 112 file(s) found. Selected files will be copied into the catalogue."
    - [ ] Inline text link "Import help" is present and functional.
  - [ ] **Global Override Control UI**:
    - [ ] Header label: "Apply to all folders (optional override)"
    - [ ] "Designer" dropdown defaults to "Keep inferred (per folder)"
    - [ ] "Source" dropdown defaults to "Keep inferred (per folder)"
  - [ ] **Primary Action Buttons**:
    - [ ] Primary validation button reads: "Continue with 112 designs" (dynamically reflects the checked count)
    - [ ] Secondary action button reads: "Cancel"
  - [ ] **Mass Selection Utilities**:
    - [ ] "Select all" and "Deselect all" inline text buttons are aligned right and interactive.

- [ ] **Folder Group Separation & Per-Folder Configurations**
  - [ ] **Folder Header Rendering**: Each scanned directory renders as a discrete UI card block separating:
    - [ ] Display name and absolute system path (e.g., `Amazing Designs - Borders - D:/...`)
  - [ ] **Per-Folder Metadata Dropdowns**:
    - [ ] "Designer for this folder" dropdown displays and correctly appends the inferred entity name in brackets (e.g., `Keep inferred (Amazing Designs)`).
    - [ ] "Source for this folder" dropdown defaults cleanly to `Keep inferred`.
    - [ ] Clicking any per-folder dropdown displays a menu containing: *Keep inferred*, *Choose existing* (listing current DB records), *Create new*, and *Leave blank*.

- [ ] **File Grid Interactivity & Batch Selection**
  - [ ] **Checkbox States**: All successfully scanned files default to a checked/selected state (`true`) upon initial page layout paint.
  - [ ] **Individual File Toggling**: 
    - [ ] Unchecking an individual file checkbox (e.g., `97603.pes`) updates its state without affecting adjacent files.
    - [ ] Unchecking a file instantly decrements the counter badge on the primary action button (e.g., drops from "Continue with 112 designs" to "Continue with 111 designs").
  - [ ] **Mass Action - Deselect All**: Clicking **Deselect all** clears the checkbox checks across all folder groups simultaneously; primary button drops to "Continue with 0 designs" and becomes visually disabled or restricted.
  - [ ] **Mass Action - Select All**: Clicking **Select all** re-checks all available file boxes across all groups, restoring the baseline maximum counter state.

- [ ] **Global and Local Metadata Assignment Logic**
  - [ ] **Global Overrides**: Selecting a specific Designer or Source in the top "Apply to all folders" section cascades down, locking or visually transforming the per-folder fields to reflect the broad hierarchy decision.
  - [ ] **Create New Flow**: Choosing *Create new* inside any Designer/Source dropdown prompts an inline input text box.
    - [ ] Entering an existing value case-insensitively resolves to the original ID without duplication.
    - [ ] Entering a net-new name flags it for generation upon workflow submission.
  - [ ] **Leave Blank Mode**: Selecting *Leave blank* successfully overwrites inferred string values, preparing the database payload to pass a null or blank field for that category metadata block.

- [ ] **Workflow Progression & Navigation Transitions**
  - [ ] **Cancel Execution**: Clicking **Cancel** clears the parsed scanner cache and smoothly rolls the user interface back out to the base folder selection configuration step (`image_d152bc.jpg`).
  - [ ] **Continue Validation Gate**: Clicking **Continue with [X] designs** securely packages the chosen selection configurations:
    - [ ] Passes chosen metadata rules safely into the database staging framework.
    - [ ] Seamlessly steps the user forward into the **Step 3 - Pre-import actions** decision screen layout.
  - [ ] **Top Menu Redirection Safeguards**: Clicking "Browse", "Projects", or "Help" in the absolute top purple global navigation strip prompts a dirty-flag configuration alert if any custom folder metadata adjustments or individual file deselections have been performed.

---

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*