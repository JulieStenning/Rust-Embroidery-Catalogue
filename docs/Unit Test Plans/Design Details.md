## 📋 User Test Suite: Design Details Page

This issue tracks the user-facing functionality for the **Design Details** view (`@DesignDetailView.svelte`), including recent UI/UX refinements, rating consolidation, tag management, designer/source auto-save, project associations, and transient notification behavior via `@Notice.svelte`.

---

### ⚙️ Setup & Test Environment
* SQLite database pre-populated with at least 74 embroidery designs.
* Test files covering standard formats: `01dstPeacock - Copy.dst` (DST), `02butterfly.pes` (PES), `03flower.jef` (JEF), `04motif.vp3` (VP3).
* At least two pre-configured Projects (e.g., `Test 1`, `Test 2`).
* Pre-existing Tag hierarchy split into **Image tags** and **Stitching tags**.
* Pre-configured Designers (e.g., `Husqvarna`) and Sources (e.g., `www.husqvarna.com`).

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load & Layout Rendering**
  - [ ] Screen opens without lag, layout shifts, or visual stutter when navigating from Browse view
  - [ ] Header pagination accurately reflects record count (e.g., `< Prev`, `1 / 74`, `Next >`, `Print`)
  - [ ] **Left Panel Rendering**:
    - [ ] Navigation button `← Back to Browse` renders cleanly
    - [ ] `FILENAME` card correctly displays active file name (e.g., `01dstPeacock - Copy.dst`)
    - [ ] Expander `► Show file path` toggles full absolute file path display
    - [ ] Image preview area renders 2D design image centered with clean borders
    - [ ] Quick action toolbar renders action buttons: `✏️ Open in Editor`, `📁 Show in Explorer`, `Render 3D Preview`
  - [ ] **Right Panel Rendering**:
    - [ ] `DESIGNER & SOURCE` card displays dropdown controls initialized to active database values
    - [ ] `TECHNICAL DATA` card renders correct metadata fields (`HOOP`, `DIMENSIONS`, `COLOURS`, `DATE ADDED`, `STITCHES`, `COLOUR CHANGES`)
    - [ ] `RATING & STATUS` card displays consolidated single-row control bar without inline alert banners
    - [ ] `TAGS` card lists active tag pills with hover `×` detach badges alongside `Choose tags...` button
    - [ ] `NOTES` card displays editable textarea and `Save Notes` button
    - [ ] `PROJECTS` card renders interactive chip/pill badges and single dropdown row (`-- Select project to add --`) with adjacent `Add` button
    - [ ] Footer area renders high-contrast `Delete design` button

- [ ] **Navigation & Pagination**
  - [ ] Pressing `← Back to Browse` returns user safely to Browse view maintaining state
  - [ ] Pressing `Next >` updates UI to next design record (`2 / 74`) without page blink or layout jump
  - [ ] Pressing `< Prev` updates UI to previous design record (`1 / 74`)
  - [ ] Pagination controls disable appropriately on boundaries (`< Prev` disabled on record `1`, `Next >` disabled on last record)
  - [ ] Pressing `Print` triggers system print modal/preview for design sheet

- [ ] **Designer & Source Auto-Save Persistence**
  - [ ] Changing selection in `Designer` dropdown automatically triggers IPC command to SQLite
  - [ ] Selection change triggers transient floating toast notification via `@Notice.svelte` without causing page layout shift
  - [ ] Changing selection in `Source` dropdown automatically saves value to SQLite silently with `@Notice.svelte` toast confirmation
  - [ ] Navigating to another design and returning confirms `Designer` and `Source` dropdown state persists

- [ ] **Rating & Workflow Controls Consolidation**
  - [ ] Pressing individual interactive stars (★ 1 through ★ 5) instantly updates rating display (e.g., `Rating: ★ 3 / 5`)
  - [ ] Pressing `Clear` resets rating state and removes star selection
  - [ ] Pressing `Mark as Stitched` toggles stitched status and updates button active visual state
  - [ ] Pressing `✓ Verified` toggles single high-contrast verification badge state without duplicate status badges appearing elsewhere on screen
  - [ ] All rating and status updates silently persist to SQLite and display confirmation via floating `@Notice.svelte` toast

- [ ] **Tag Management Workflow Enhancements (`@TagSelectionModal.svelte`)**
  - [ ] Active tag pills in `TAGS` card display green (`Image tags`) and blue (`Stitching tags`) pill styling
  - [ ] Hovering over a tag pill reveals `×` detach badge; pressing `×` instantly detaches tag from SQLite and reactively updates `@DesignDetailView.svelte`
  - [ ] Pressing `Choose tags...` opens `@TagSelectionModal.svelte` multi-column grid modal overlay
  - [ ] Modal displays design identifier (e.g., `Design #2817`) and active multi-column tag checkboxes
  - [ ] **Tag Search & Creation**:
    - [ ] Typing into `🔍 Search or create tag...` filters visible tag list in real time across columns
    - [ ] Entering a non-existent tag name displays prompt/option to create tag on-the-fly
    - [ ] Newly created tags automatically categorize into `Image tags` or `Stitching tags` groups and auto-sort alphabetically
  - [ ] **Tag Selection & Instant Persistence**:
    - [ ] Checking or unchecking tag checkboxes instantly binds changes to active design in SQLite
    - [ ] Scrolling modal list maintains distinct section headers (`Image tags`, `Stitching tags`)
    - [ ] Pressing `Done` closes modal overlay and reactively reflects updated tag list on `@DesignDetailView.svelte`

- [ ] **Notes Management**
  - [ ] Typing into `NOTES` textarea accepts arbitrary multiline text input
  - [ ] Unsaved notes changes keep `Save Notes` button enabled
  - [ ] Pressing `Save Notes` persists notes string to SQLite and displays transient success toast via `@Notice.svelte`
  - [ ] Leaving page with unsaved notes triggers "Save changes?" warning dialog before allowing exit

- [ ] **Projects Section Workflow**
  - [ ] Associated projects display as interactive chip/pill badges (e.g., `📁 Test 2 ×`) matching Tags UX styling
  - [ ] Hovering/pressing `×` on project pill badge detaches design from project in SQLite and reactively removes chip
  - [ ] Selecting a project from `-- Select project to add --` dropdown and pressing `Add` binds design to selected project in SQLite
  - [ ] Upon successful project association, dropdown automatically resets to default placeholder `-- Select project to add --`
  - [ ] Transient toast notification via `@Notice.svelte` confirms project addition/removal

- [ ] **External Actions & Destructive Controls**
  - [ ] Pressing `✏️ Open in Editor` launches configured external design editing software with active file path
  - [ ] Pressing `📁 Show in Explorer` opens system file manager highlighted on the design file
  - [ ] Pressing `Render 3D Preview` executes 3D thread rendering pipeline
  - [ ] Pressing `Delete design` displays `@DeleteDesignsModal.svelte` confirmation dialog
  - [ ] Confirming deletion removes record from SQLite, deletes file assets if configured, displays `@Notice.svelte` toast, and navigates back to Browse view safely

---

### ❌ Failed Tests / Discovered Friction

*Hover over a failed subtest above and click "Convert to issue", or track them below:*