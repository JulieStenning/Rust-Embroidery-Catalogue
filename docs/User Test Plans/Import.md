## 📋 User Test Suite: Import

This issue tracks the user-facing functionality for the **Import** view based on `image_d0cbb9.jpg`.

### ⚙️ Test Setup & Prerequisites
Before executing these tests, prepare the following local environment states:
- **Empty State**: Ensure no previous paths are cached in the application's local state for the import directory configuration.
- **Sample Directory Structures**:
  - `C:\EmbroideryTests\ValidFolder` (Containing subfolders with `.jef`, `.pes`, and `.vp3` files)
  - `C:\EmbroideryTests\EmptyFolder` (Completely empty)

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load**
  - [ ] Screen opens without lag or visual stutter from the "Import" top navigation item
  - [ ] Default layout elements and text render correctly exactly matching `image_d0cbb9.jpg`:
    - [ ] Title header text: "Bulk Import"
    - [ ] Descriptive instruction text block including "Import help" inline text link
    - [ ] Form field label: "Source Folder(s) *"
  - [ ] Default UI element interactive states:
    - [ ] A single text input row is visible with placeholder text: "Enter path to your embroidery designs folder..."
    - [ ] Input text field is empty by default
    - [ ] "Browse..." button is active and enabled
    - [ ] "Remove" button is disabled (or visually neutralized) since only one required row exists
    - [ ] "Add another folder" button is active and enabled below the initial row
    - [ ] "Scan folder(s)" action button is disabled (or triggers validation) when the path field is empty
    - [ ] "Reset" action button is active and enabled
  - [ ] Application window is responsive, scaling UI cards correctly across standard desktop resolutions

- [ ] **Primary Action: Entry Point Validation & Basic Interactivity**
  - [ ] **Text Input Field Focus & Manual Entry**
    - [ ] Clicking into the text box focuses the cursor correctly
    - [ ] Manually typing a valid path string preserves the string literal value
    - [ ] Pasting a path string into the text box behaves correctly without truncation
  - [ ] **Action Component Initial States**
    - [ ] Hover states for "Browse...", "Add another folder", "Scan folder(s)", and "Reset" buttons change cursor/background color smoothly
    - [ ] Focus ring or highlight transfers smoothly via keyboard Tab key navigation across all form elements

- [ ] **Primary Action: Folder Selection & Browse Window Behavior**
  - [ ] **Default/Fallback Open Directory**: Clicking **Browse...** when no previous files have been imported (or if the previous path no longer exists) successfully opens the native system file dialog defaulted to the system **Documents** folder.
  - [ ] **Session Memory Open Directory**: Clicking **Browse...** after a previous folder has been imported successfully opens the file dialog with the **last selected folder** automatically focused/marked.
  - [ ] **Multi-Folder OS Selection**: When the native system file dialog is open, the user can select **one or multiple folders** simultaneously. Upon confirming, all selected paths populate into the interface as distinct rows.
  - [ ] **Path Overwrites via Re-Selection**: Clicking **Browse...** on a specific row that already contains an entered path successfully replaces/overwrites that specific path text with the newly selected folder from the native dialog.

- [ ] **Dynamic Row Management (Add / Remove)**
  - [ ] **Add Another Folder Interaction**: Clicking the **Add another folder** button appends a brand new row to the configuration panel.
    - [ ] The new row contains its own empty text input, its own unique functional **Browse...** button, and its own active **Remove** button.
    - [ ] Clicking **Browse...** on this dynamically added row opens the file dialog with the same selection capabilities (single or multi-folder selection) and session-memory inheritance as the initial row.
  - [ ] **Remove Folder Interaction**: Clicking the **Remove** button next to any dynamically added folder immediately deletes that specific row from the DOM list without altering or clearing neighboring inputs.
  - [ ] **Row Constraint Safeguards**: When only one folder row remains visible in the configuration block, the **Remove** button is automatically disabled or hidden to maintain the required baseline layout matching `image_d0cbb9.jpg`.

- [ ] **Form Clearing & State Resets**
  - [ ] **Reset Button Execution**: Clicking the **Reset** button flushes the current operational state completely:
    - [ ] All dynamically appended folder rows are instantly unmounted and purged from the Svelte reactive state.
    - [ ] The interface collapses back down to the exact single-row template layout pictured in `image_d0cbb9.jpg`.
    - [ ] The primary text input box is completely cleared of its string value, restoring the default placeholder string.

- [ ] **Primary Action: Scan Folder(s) Execution & Navigation Handshake**
  - [ ] **Empty / Invalid Input Validation Trigger**
    - [ ] **No Path Specified**: Clicking **Scan folder(s)** when the primary input field is completely empty prevents execution and triggers a visual validation error (e.g., input border highlight or validation text).
    - [ ] **Non-Existent Path**: Typing an invalid or non-existent file system path string and clicking **Scan folder(s)** halts execution and displays a clear error warning to the user.
  - [ ] **Scanning Engine Processing States**
    - [ ] **Loading Indicator & UI Blocking**: Clicking **Scan folder(s)** with at least one valid directory path successfully initiates the background scanner. The UI exhibits a clear processing state (e.g., a spinner or disabled action buttons) to prevent double-submission or state corruption during processing.
    - [ ] **Background Processing Fluidity**: The application window remains completely responsive and does not stutter or freeze while the backend crawls the target directory structures.
  - [ ] **File Format Identification & Filtering Logic**
    - [ ] **Valid Formats Extraction**: The scanner successfully recursive-traverses the targeted directories and implicitly extracts valid embroidery files containing relevant design metadata (specifically targeting `.jef`, `.pes`, and `.vp3` formats).
    - [ ] **Sub-folder Auto-Inclusion**: Directories nested multiple levels deep within the parent source folder are automatically parsed without requiring manual secondary row inclusion.
    - [ ] **Non-Embroidery File Exclusion**: Files within the targeted directories that do not match embroidery formats (such as `.txt`, `.pdf`, `.jpg`, or system metadata files) are completely skipped and omitted from the prospective import manifest.
  - [ ] **Import Rule Enforcement (Safety Check)**
    - [ ] **Source Integrity**: The scanner reads data streams safely without altering, renaming, moving, or modifying the timestamps of any source embroidery files.
    - [ ] **External Source Copy Management**: The system verifies that files located outside the main design directory are tagged to be safely copied into the core application catalog repository during finalization rather than cut/moved.
  - [ ] **Successful View Transition**
    - [ ] **Navigation to Next Page**: Upon completing a valid directory scan containing matches, the interface smoothly navigates the user forward to the secondary Bulk Import workflow view screen (which will be tracked in a separate test document).

- [ ] **Navigation**
  - [ ] User can safely exit or go back to "Browse", "Projects", or "Help" via the top navigation bar without freezing the app
  - [ ] Leaving the page with an empty path field transitions immediately without prompts
  - [ ] Leaving the page midway after a path string has been entered prompts a dirty-flag warning ("You have unsaved changes. Are you sure you want to leave?") before destroying the view state

---

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*