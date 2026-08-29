## 📋 User Test Suite: Manage Designers

This issue tracks the user-facing functionality for the **Manage Designers** view.

### 🧪 Test Setup

Before executing this test suite, ensure the application is running with a test catalogue initialized with:

At least 2 separate designers already created.
Multiple existing embroidery design files linked to at least one of these designers to test how the app handles unlinking them.

---

### Test Cases

### 🔄 Core Workflows to Test

- [x]  **Initial Load**

  - [x]  Screen opens without lag, flashing, or layout shifting.
  - [x]  Top navigation bar renders correctly with an active focus underline or text weight on "Designers".
  - [x]  "Manage Designers" title header and its descriptive instruction line render properly.
  - [x]  "Add new designer" container renders with a single blank input textbox, adjacent to the "Add" button.
  - [x]  The add button is disabled on initial load, using the standard colours
  - [x]  The clear button is disabled on initial load, using the standard colours
  - [x]  Existing records render in a structured matrix containing table headers: `Name`and `Used by`.
  - [x]  Read-only rows display plain text for metrics and action links ("Edit", "Delete") right-aligned.
  - [x]  About and Licence links are present. No need to click them as the same code is used for all pages.
- [x]  **Primary Action: Edit Lifecycle**

  - [x]  **Editing designer details**
    - [x]  Clicking "Edit" on a designer row transforms that specific row into active input text box for Name.
    - [x]  Action items on the row dynamically change from text strings "Edit Delete" to "Save Cancel".
    - [x]  While one row is actively in Edit mode, clicking "Edit" on a different row either safely handles focus or locks out secondary activations to prevent concurrent conflicting operations.
  - [x]  **Cancel Modification**
    - [x]  Modifying data inside the active row and then clicking "Cancel" reverts the text inputs back to original read-only values.
  - [x]  **Save Modifications**
    - [x]  Modifying the name and clicking "Save" commits changes smoothly.
    - [x]  Message Designer updated. is shown above the page title.
    - [x]  Row elements switch cleanly back to the read only layout.
- [x]  **Primary Action: Add New Designer**

  - [x]  **Input Field Constraints**
    - [x]  The Add button is not enabled until the Add new designer field has a value.
  - [x]  **Saving The New Designer**
    - [x]  Completing the name field correctly and pressing "Add" inserts the record.
    - [x]  A message Designer added. is shown above the page title.
    - [x]  The newly added designer registers in the table layout immediately without needing a refresh of the page, an app restart or manual navigation sequence.
    - [x]  An error is shown if a designer is added which already exists. The match is not case sensitive, i.e. Janome and janome are the same.
  - [x]  **Clear Action**
    - [x]  Entering a single character into the input field immediately enables the "Clear" button
    - [x]  Deleting the character manually (leaving the field empty again) immediately disables the "Clear" button
    - [x]  Pressing the "Clear" button wipes all the text from the field instantly
    - [x]  After pressing "Clear", the "Clear" button immediately returns to a disabled state
    - [x]  Clearing the row does not accidentally trigger the "Add" action or submit a blank record
- [ ]  **Primary Action: Delete Designer**

  - [x]  **Zero Usage Items**
    - [x]  Clicking "Delete" on an item with a `Used By` counter of `0` triggers an immediate confirmation window.
    - [x]  Confirming the removal immediately removes the from the list of Designers on screen.
    - [x]  A message Designer deleted. is shown above the title.
  - [x]  **Designer used by 1 or more designs**
    - [x]  Clicking "Delete" on an item possessing linked designs shows a message telling the user that the designer is in use and asking them to confirm the deletion.
    - [x]  Confirming the deletion clears the target designer from the list of Designers.
    - [x]  **Critical Integrity Check**: Ensure all linked embroidery design files remain entirely untouched in the catalog; only the empty designer relationship reference is dissolved.
    - [x]  No change is made if the deletion is cancelled.
- [x]  **Navigation**

  - [x]  Moving to a different page using the navbar whilst a designer is being edited preserves the changed text upon return.
  - [x]  Moving to a different page using the navbar whilst a designer is being added preserves the changed text upon return.

### ❌ Failed Tests / Discovered Friction

Track failed tests in the comments below.
