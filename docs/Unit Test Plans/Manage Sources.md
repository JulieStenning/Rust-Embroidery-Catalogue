## 📋 User Test Suite: Manage Sources

This issue tracks the user-facing functionality for the **Manage Sources** view.

### 🧪 Test Setup

Before executing this test suite, ensure the application is running with a test catalogue initialized with:

At least 2 separate design sources already created.
Multiple existing embroidery design files linked to at least one of these sources to test how the app handles unlinking them.

---

### Test Cases

### 🔄 Core Workflows to Test

- [x]  **Initial Load**

  - [x]  Screen opens without lag, flashing, or layout shifting.
  - [x]  Top navigation bar renders correctly with an active focus underline or text weight on "Sources".
  - [x]  "Manage Sources" title header and its descriptive instruction line render properly.
  - [x]  "Add new source" container renders with a single blank input textbox, adjacent to the "Add" button.
  - [x]  The add button is disabled on initial load, using the standard colours
  - [x]  The clear button is disabled on initial load, using the standard colours
  - [x]  Existing records render in a structured matrix containing table headers: `Name`and `Used by`.
  - [x]  Read-only rows display plain text for metrics and action links ("Edit", "Delete") right-aligned.
  - [x]  About and Licence links are present. No need to click them as the same code is used for all pages.
- [x]  **Primary Action: Edit Lifecycle**

  - [x]  **Editing source details**
    - [x]  Clicking "Edit" on a source row transforms that specific row into active input text box for Name.
    - [x]  Action items on the row dynamically change from text strings "Edit Delete" to "Save Cancel".
    - [x]  While one row is actively in Edit mode, clicking "Edit" on a different row either safely handles focus or locks out secondary activations to prevent concurrent conflicting operations.
  - [x]  **Cancel Modification**
    - [x]  Modifying data inside the active row and then clicking "Cancel" reverts the text inputs back to original read-only values.
  - [x]  **Save Modifications**
    - [x]  Modifying the name and clicking "Save" commits changes smoothly.
    - [x]  Message Source updated. is shown above the page title.
    - [x]  Row elements switch cleanly back to the read only layout.
- [x]  **Primary Action: Add New Source**

  - [x]  **Input Field Constraints**
    - [x]  The Add button is not enabled until the name field has a value.
  - [x]  **Saving The New Source**
    - [x]  Completing the name field correctly and pressing "Add" inserts the record.
    - [x]  A message Source added. is shown above the page title.
    - [x]  The newly added source registers in the table layout immediately without needing a refresh of the page, an app restart or manual navigation sequence.
    - [x] An error is shown if a source is added which already exists. The match is not case sensitive, i.e. Janome and janome are the same.
  - [x]  **Clear Action**
    - [x]  Entering a single character into any input field immediately enables the "Clear" button
    - [x]  Deleting the character manually (leaving the field empty again) immediately disables the "Clear" button
    - [x]  Pressing the "Clear" button wipes all the text from the field instantly
    - [x]  After pressing "Clear", the "Clear" button immediately returns to a disabled state
    - [x]  Clearing the row does not accidentally trigger the "Add" action or submit a blank record
- [x]  **Primary Action: Delete Source**

  - [x]  **Zero Usage Items**
    - [x]  Clicking "Delete" on an item with a `Used By` counter of `0` triggers an immediate confirmation window.
    - [x]  Confirming the removal immediately removes the from the list of Sources on screen.
    - [x]  A message Source deleted. is shown above the title.
  - [x]  **Source used by 1 or more designs**
    - [x]  Clicking "Delete" on an item possessing linked designs shows a message telling the user that the source is in use and asking them to confirm the deletion.
    - [x]  Confirming the deletion clears the target source from the list of Sources.
    - [x]  **Critical Integrity Check**: Ensure all linked embroidery design files remain entirely untouched in the catalog; only the empty source relationship reference is dissolved.
    - [x]  No change is made if the deletion is cancelled.
- [x]  **Navigation**

  - [x]  Moving to a different page using the navbar whilst a source is being edited preserves the changed text upon return.
  - [x]  Moving to a different page using the navbar whilst a source is being added preserves the changed text upon return.

### ❌ Failed Tests / Discovered Friction

Track failed tests in the comments below.
