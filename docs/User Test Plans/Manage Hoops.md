## 📋 User Test Suite: Manage Hoops

This issue tracks the user-facing functionality for the **Manage Hoops** view.

### 🧪 Test Setup

Before executing this test suite, ensure the application state matches the following:

- The application contains at least three existing hoops. Suggestions are for Janome hoops:
  - **Hoop A**: Max Width 126 mm, Max Height 110 mm.
  - **Hoop B**: Max Width 200 mm, Max Height 140 mm.
  - **Giga Hoop**: Max Width 230 mm, Max Height 200 mm.
- At least one embroidery design uses **Hoop A** and at least one other uses **Hoop B**.

---

### Test Cases

### 🔄 Core Workflows to Test

- [X]  **Initial Load**

  - [X]  Screen opens without lag, flashing, or layout shifting.
  - [X]  Top navigation bar renders correctly with an active focus underline or text weight on "Hoops".
  - [X]  "Manage Hoops" title header and its descriptive instruction line render properly.
  - [X]  "Add new hoop" container renders with three distinct blank input controls ("Name" with placeholder `e.g. 5x7 hoop`, "Max Width (mm)", and "Max Height (mm)") adjacent to the "Add" button.
  - [X]  The add button is disabled on initial load, using the standard colours
  - [X]  The clear button is disabled on initial load, using the standard colours
  - [X]  Existing records render in a structured matrix containing table headers: `Name`, `Max width (mm)`, `Max height (mm)`, and `Used by`.
  - [X]  Read-only rows display plain text for metrics and action links ("Edit", "Delete") right-aligned.
  - [X]  About and Licence links are present. No need to click them as the same code is used for all pages.
- [X]  **Primary Action: Edit Lifecycle**

  - [X]  **Editing hoop details**
    - [X]  Clicking "Edit" on a hoop row transforms that specific row into active input text boxes for Name, Width, and Height parameters .
    - [X]  Action items on the row dynamically change from text strings "Edit Delete" to "Save Cancel".
    - [X]  Text fields are pre-populated with the exact pre-existing values from the database row.
    - [X]  While one row is actively in Edit mode, clicking "Edit" on a different row either safely handles focus or locks out secondary activations to prevent concurrent conflicting operations.
    - [X]  It is not possible to type an alphabetic strings inside "Max Width (mm)" or "Max Height (mm)".
  - [X]  **Cancel Modification**
    - [X]  Modifying data inside the active row boxes and then clicking "Cancel" reverts the text inputs back to original read-only values.
  - [X]  **Save Modifications**
    - [X]  Modifying fields (e.g., editing the dimension from `126` to `130`) and clicking "Save" commits changes smoothly.
    - [X]  Message Hoop updated. is shown above the page title.
    - [X]  Row elements switch cleanly back to text status with updated coordinates.
- [X]  **Primary Action: Add New Hoop**

  - [X]  **Input Field Constraints**
    - [X]  It is not possible to type an alphabetic strings inside "Max Width (mm)" or "Max Height (mm)".
    - [X]  The Add button is not enabled until all three fields have values.
  - [X]  **Saving The New Hoop**
    - [X]  Completing all three parameter fields correctly and pressing "Add" inserts the record.
    - [X]  A message Hoop added. is shown above the page title.
    - [X]  The newly added hoop registers in the table layout immediately without needing a refresh of the page, an app restart or manual navigation sequence.
    - [X]  Input fields return back to their original layout on success.
    - [x] An error is shown if a hoop name is added which already exists. The match is not case sensitive, i.e. Hoop a and Hoop A are the same.
  - [X]  **Clear Action**
    - [X]  Entering a single character into any input field immediately enables the "Clear" button
    - [X]  Deleting the character manually (leaving all fields empty again) immediately disables the "Clear" button
    - [X]  Entering data across multiple fields keeps the "Clear" button enabled
    - [X]  Pressing the "Clear" button wipes all text, numbers, and selections from the top row fields instantly
    - [X]  After pressing "Clear", the "Clear" button immediately returns to a disabled state
    - [X]  Clearing the row does not accidentally trigger the "Add" action or submit a blank record
- [X]  **Primary Action: Delete Hoop**

  - [X]  **Zero Usage Items**
    - [X]  Clicking "Delete" on an item with a `Used By` counter of `0` triggers an immediate confirmation window.
    - [X]  Confirming the removal immediately removes the from the list of hoops on screen.
    - [X]  A message Hoop deleted. is shown above the title.
  - [X]  **Hoop used by 1 or more designs**
    - [X]  Clicking "Delete" on an item possessing linked designs shows a message telling the user that the hoop is in use and asking them to confirm the deletion.
    - [X]  Confirming the deletion clears the target hoop from the list of hoops.
    - [X]  **Critical Integrity Check**: Ensure all linked embroidery design files remain entirely untouched in the catalog; only the empty hoop relationship reference is dissolved.
    - [X]  No change is made if the deletion is cancelled.
- [X]  **Navigation**

  - [X]  Moving to a different page using the navbar whilst a hoop is being edited preserves the changed text upon return.
  - [X]  Moving to a different page using the navbar whilst a hoop is being added preserves the changed text upon return.

### ❌ Failed Tests / Discovered Friction

Track failed tests in the comments below.
