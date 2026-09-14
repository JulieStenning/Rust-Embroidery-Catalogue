## 📋 User Test Suite: Projects

This issue tracks the user-facing functionality for the **Projects** management workflows, covering the Project List view, New Project form, Project Details page, and the Print Sheet layout.

> **Automated coverage:** `tests/e2e/projects.spec.ts` (Playwright driving the real Tauri app over WebView2) automates this checklist — run it from the repo root with `npx playwright test tests/e2e/projects.spec.ts`. The shared test catalogue starts with **no projects**, so the spec creates its projects through the New Project form and links designs from Design Detail. View-level unit coverage lives in `frontend/src/lib/views/__tests__/ProjectsViewList.test.ts`, `ProjectsViewNew.test.ts`, `ProjectsViewDetail.test.ts` and `ProjectsViewPrint.test.ts`.
>
> **Corrections applied to this revision** (the original plan no longer matched the shipped UI):
> - The **Remove** control is a `<button>`, not an anchor link.
> - **Create Project** is *disabled* while **Name** is empty (there is no validation-error highlight); the Name input is also marked `required`.
> - The Project Detail title is an editable `<input>` rendered above an editable Description `<textarea>`.
> - The Print Sheet action row is two individually styled buttons (secondary **Back to Project** + primary purple **Print**), not one coloured bar.
> - The design-count badge is singular/plural aware (`1 design` / `2 designs`).

---

### ⚙️ Test Setup & Prerequisites
Before executing the manual tests, ensure the following database and directory state is configured:
* **Existing Projects:**
  * At least 1 project with 0 designs.
  * At least 1 project with 1 or more designs.
* **Design metadata for the print-sheet checks:** one design with full metadata (e.g. `Cake 3.jef`) and one with none at all — no dimensions, hoop, designer or preview (e.g. `ZZ-broken.pes`) — so the "skip empty fields" behaviour can be proven in both directions.

---

### 🔄 Core Workflows to Test

#### 🔍 Initial Load & UI Layout
- [x] **Project List View Layout**
  - [x] Navigation bar highlights the active **Projects** tab (`menu-link-active`).
  - [x] Sub-header renders the project summary text plus a **Learn more** link to `#/help?section=projects`.
  - [x] Existing projects populate correctly inside the card grid wrapper (`.projects-grid`).
  - [x] Individual project cards display the accurate Name, Description, Created Date (`Created YYYY-MM-DD`) and a design-count badge.
  - [x] The design-count badge is singular/plural aware (`0 designs`, `1 design`, `2 designs`).
  - [x] With no projects in the catalogue, the list shows "No projects yet." and a **Create one** shortcut.
- [x] **Project Details View Layout**
  - [x] Header displays a breadcrumb control styled as `← Projects` to safely return to the dashboard.
  - [x] Project title is prominently rendered as an editable inline `<input>` above an editable Description `<textarea>`.
  - [x] Action buttons **Print Sheet** and **Delete Project** are visible on the top right.
  - [x] Section header **Designs (X)** accurately reflects the number of linked items.
  - [x] Grid instantiates a card per design showing a thumbnail preview image, filename, designer name (when present) and a **Remove** button.
  - [x] With no designs linked, the section shows **Designs (0)** and "No designs in this project yet."
- [x] **Print Sheet Layout**
  - [x] Action row renders **Back to Project** (secondary) and **Print** (primary purple) triggers.
  - [x] Layout switches to a print-optimised vertical list containing the Project Name and Description.
  - [x] Each item provides a preview image (or a **No image** fallback) alongside labelled specifications (Size, Hoop, Stitches, Colours, Colour changes, Designer, Rating, Stitched, Notes).
- [x] **New Project Form Layout**
  - [x] Form fields display the documented placeholders (`e.g. Christmas Stockings 2024`, `Optional notes, goals, or deadline`).
  - [x] The **Name** field is marked mandatory with an asterisk (`Name *`) and carries the `required` attribute.


#### ⚡ Primary & Secondary Actions
- [x] **Project Lifecycle Management**
  - [x] **Creation guard**: with an empty **Name**, the **Create Project** button is disabled — no submission occurs and no error highlight is shown.
  - [x] **Creation walkthrough**: entering valid data and clicking **Create Project** provisions a new record, returns to `/projects/`, appends a fresh card, and raises a "Project created." toast.
  - [x] **Name updates**: editing the Project Detail name enables **Save**/**Undo**; **Undo** restores the stored value; **Save** persists through the backend (survives a reload) and raises a "Project updated." toast.
  - [x] **Description updates**: the same Save/Undo behaviour applies to the Description field, persisted independently of the name.
  - [x] **Project destruction**: clicking **Delete Project** opens a confirmation prompt. Dismissing it leaves the project untouched; confirming removes the project and returns to `/projects/`.
  - [x] Deleting a project removes the project↔design links but leaves the design record (and its file) completely intact — the design simply becomes unassigned.
- [x] **Design Associations Management**
  - [x] **Linking**: a design can be added to a project from Design Detail's **Projects** card (pick the project, then **Add**), after which it appears in the project's grid.
  - [x] **Grid interactivity**: clicking a design card routes the active window to the matching Design Detail page (`#/designs/<id>`).
  - [x] **Disassociation**: clicking the **Remove** button unlinks that chosen design from the project workspace.
  - [x] **Dynamic interface updates**: removing a design decrements the parent **Designs (X)** heading and removes the targeted card from the visible grid.
- [x] **Report Printing Workflow**
  - [x] **Print dispatcher**: clicking **Print** invokes `window.print()` (the native OS print preview dialog).
  - [x] **Conditional fields render**: a blank Size / Hoop / Designer reference skips that label cleanly instead of outputting "null"/"undefined".
  - [x] A project with no designs prints the project Name and Description plus "No designs in this project yet."

#### 🗺️ Navigation & State Persistence
- [x] **View route escapes**
  - [x] Clicking `← Projects` on the details or new-project screens reliably returns to the project list view without hangs or stuttering.
  - [x] Clicking **Back to Project** on the printable sheet returns to that project's detail view.

---

❌ Failed Tests / Discovered Friction
Track failed tests in the comments below.

