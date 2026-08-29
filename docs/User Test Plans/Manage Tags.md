## 📋 User Test Suite: Manage Tags

This issue tracks the user-facing functionality for the **Manage Tags** view.

### 🧪 Test Setup

Before starting the test suite, ensure the application is running with a standard database containing:
- At least 5 embroidery designs that are already tagged with existing tags (e.g., "Alphabets", "Applique").
- Ensure a sample tag is associated with a design to verify database dissociation behavior.

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load**
  - [ ] Screen opens without lag or visual stutter
  - [ ] Top navigation bar renders correctly with active state on "Tags"
  - [ ] "Manage Tags" heading and description subtitle render clearly
  - [ ] "Add new tag" section loads with an empty description input field, a "Group" dropdown defaulted to "Image", and an "Add" button
  - [ ] "IMAGE TAGS" and "STITCHING TAGS" expandable drawers render closed by default (as seen in image_dcb72c.jpg)
  - [ ] Success message banner ("Designer added.") renders correctly if navigating from a prior successful save action

- [ ] **Drawer Interaction & Data Rendering**
  - [ ] Clicking on the "IMAGE TAGS" header toggles the drawer open smoothly
  - [ ] Opened "IMAGE TAGS" drawer correctly displays descriptions ("Alphabets", "Angels", "Animals", etc.), group dropdowns, and a red "Delete" option for each record (as seen in image_dcbad1.jpg)
  - [ ] Clicking on the "STITCHING TAGS" header toggles the drawer open smoothly
  - [ ] Opened "STITCHING TAGS" drawer correctly displays descriptions ("Applique", "Blackwork", "Cross Stitch", etc.), group dropdowns, and a red "Delete" option for each record (as seen in image_dcbb83.jpg)
  - [ ] Opening one drawer does not forcefully close or break the layout of the other drawer

- [ ] **Primary Action: Add New Tag**
  - [ ] Typing into the "Description" input field updates the UI text correctly without lag
  - [ ] Leaving "Description" empty and clicking "Add" triggers a field validation error or blocks submission
  - [ ] Selecting "Image" from the Group dropdown and clicking "Add" successfully appends the tag to the "IMAGE TAGS" drawer
  - [ ] Selecting "Stitching" from the Group dropdown and clicking "Add" successfully appends the tag to the "STITCHING TAGS" drawer
  - [ ] Adding a tag updates the respective list instantly without requiring a manual page refresh

- [ ] **Primary Action: Delete Tag & Dissociation**
  - [ ] Clicking "Delete" next to any tag triggers a browser confirmation prompt asking the user to confirm deletion
  - [ ] Canceling the browser confirmation prompt leaves the tag intact in the UI and database
  - [ ] Confirming the prompt removes the tag from the UI list immediately
  - [ ] Deleting a tag successfully removes its association from any tagged embroidery designs in the database
  - [ ] Verify that the associated embroidery design record itself **is not** deleted and remains perfectly intact in the catalog

- [ ] **Data Modification: Inline Group Change**
  - [ ] Changing a tag's group dropdown inline (e.g., changing "Alphabets" from Image to Stitching) re-sorts or updates its location accordingly upon save or auto-save
  - [ ] Verified state updates instantly in the underlying database schema

- [ ] **Navigation**
  - [ ] User can safely exit or go back by clicking other top navigation links ("Browse", "Import", "Projects") without freezing the app
  - [ ] Changing a tag's inline properties and leaving the page midway prompts a "Save changes?" warning if data is unsaved (or auto-saves robustly via Tauri IPC)

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*