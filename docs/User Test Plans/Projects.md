## 📋 User Test Suite: Projects

This issue tracks the user-facing functionality for the **Projects** management workflows, covering the Project List view, New Project form, Project Details page, and the Print Sheet layout.

---

### ⚙️ Test Setup & Prerequisites
Before executing the tests, ensure the following database and directory state is configured:
* **Existing Projects:**
  * At least 1 project with 0 designs.
  * At least 1 project with 1 or more designs.

---

### 🔄 Core Workflows to Test

#### 🔍 Initial Load & UI Layout
- [x] **Project List View Layout**
  - [x] Navigation bar correctly highlights the active **Projects** tab.
  - [x] Sub-header renders the total project summary text and a clickable **Learn more** link.
  - [x] Existing projects populate correctly inside the card grid wrapper.
  - [x] Individual project cards display the accurate Name, Description, Created Date, and a badge indicating the design count.
- [x] **Project Details View Layout**
  - [x] Header displays a breadcrumb link styled as `← Projects` to safely return to the dashboard.
  - [x] Project title is prominently rendered above an editable inline Description textarea.
  - [x] Action buttons **Print Sheet** and **Delete Project** are visible on the top right.
  - [x] Section header **Designs** accurately reflects the total items currently linked.
  - [x] Grid correctly instantiates cards for each design showing a clear thumbnail preview image, filename, designer name, and a **Remove** anchor link.
- [x] **Print Sheet Layout**
  - [x] Top header renders a purple action bar with **Back to Project** and **Print** triggers.
  - [x] Layout shifts to a print-optimized vertical list format containing the Project Name and Description.
  - [x] Individual item sections provide clear, high-resolution preview images alongside tabular layout specifications (Size, Hoop, Stitches, Colours, Colour changes, Designer).
- [x] **New Project Form Layout**
  - [x] Form fields display appropriate text placeholder values (e.g. `Christmas Stockings 2024`, `Optional notes, goals, or deadline`).
  - [x] Asterisk symbol (`*`) explicitly identifies the **Nae** input field as a mandatory setup requirement.

#### ⚡ Primary & Secondary Actions
- [x] **Project Lifecycle Management**
  - [x] **Creation Walkthrough **: Submitting an empty **Name** prevents form processing and throws a validation error highlight.
  - [x] **Creation Walkthrough **: Entering valid data fields and clicking **Create Project** successfully provisions a new record, shifts the view back to `/projects/`, and appends a fresh card to the layout.
  - [x] **Name Updates **: Modifying the text within the Project Detail Name textarea enables the **Save** button state. Clicking **Save** persists the string changes directly to the underlying database file system safely.
  - [x] **Name Updates **: Modifying the text within the Project Detail Name textarea enables the **Undo** button state. Clicking **Undo** undoes the string changes.
  - [x] **Description Updates **: Modifying the text within the Project Detail textarea enables the **Save** button state. Clicking **Save** persists the string changes directly to the underlying database file system safely.
  - [x] **Description Updates **: Modifying the text within the Project Detail textarea enables the **Undo** button state. Clicking **Undo** undoes the string changes.
  - [x] **Project Destruction **: Clicking **Delete Project** correctly calls a browser context confirmation prompt modal. Confirming the action purges the project link record but safely preserves the associated design files completely intact.
- [x] **Design Associations Management **
  - [x] **Grid Interactivity**: Clicking on an individual design card thumbnail properly routes the active window context directly forward into the matching Design Details page view.
  - [x] **Disassociation Trigger**: Clicking the **Remove** text hyperlink immediately unlinks that chosen file from the active project workspace layout.
  - [x] **Dynamic Interface Updates**: Triggering a design removal downscales the parent **Designs (X)** count value incrementally and removes the targeted element container block smoothly from the visible UI grid view.
- [x] **Report Printing Workflow **
  - [x] **Print Dispatcher**: Clicking the **Print** option button seamlessly instantiates the native system OS print preview dialog context wrapper.
  - [x] **Conditional Fields Render**: Verifies that formatting missing structural metadata values (like a blank Hoop or Designer reference field) skips displaying that label cleanly on the page output rather than outputting generic null text errors.

#### 🗺️ Navigation & State Persistence
- [x] **View Route Escapes**
  - [x] Clicking `← Projects` on the details or new project screens reliably backs out to the main menu view route without performance hangs or stuttering.
  - [x] Clicking **Back to Project** on the printable sheet view drops the system layout view squarely back down to the target Project 

---

❌ Failed Tests / Discovered Friction
Track failed tests in the comments below.
