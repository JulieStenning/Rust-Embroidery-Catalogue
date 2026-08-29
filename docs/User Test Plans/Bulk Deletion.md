### 🔄 Core Workflows to Test: Bulk Deletion Workflow

- [ ] **Setup Requirements**
  - [x] Directory prepared with at least 60 embroidery design files (`.jef`, `.pes`, `.vp3`) imported into the application database.
  - [x] At least 5 design records marked as "Verified" and 5 marked as "Unverified" to test mixed-state multi-selection.

- [x] **Multi-Select Batch Operations (Browse Page Grid)**
  - [x] **Selection Limits & Controls**
    - [x] Multi-select controls allow selecting individual items up to the maximum limit of 50 designs.
  - [x] **Batch Action Trigger**
    - [x] Batch action bar appears dynamically when 1 or more designs are selected.
    - [x] Batch action bar correctly displays the exact count of selected items (e.g., "12 items selected").
    - [x] Pressing the batch "Delete" button triggers a single confirmation modal regardless of the selection count.
    - [x] Sequential individual popups do not appear when initiating batch deletion.

- [x] **Unified Confirmation & Disk File Handling Modal**
  - [x] **Modal Content & Messaging**
    - [x] Modal header and body text clearly state the exact number of designs queued for deletion.
    - [x] Modal retains focus and traps keyboard navigation while open.
  - [x] **Deletion Mode Toggles**
    - [x] Modal presents two clear options for file handling:
      - [x] **Database Only**: Removes records from SQLite database while leaving source files on local disk untouched.
      - [x] **Database & Trash**: Removes SQLite records and moves physical files to OS Trash / Recycle Bin.
    - [x] "Database Only" option is selected by default to prevent accidental file deletion.
  - [x] **Safety & Execution Controls**
    - [x] Modal includes a primary confirmation button ("Delete Items") and a distinct cancel button ("Cancel").
    - [x] Pressing "Cancel" closes the modal, clears no selections, and leaves database/disk completely unchanged.
    
- [x] **User Confidence & Safety Features**
  - [x] **Collapsible Preview Drawer**
    - [x] Modal includes a collapsible preview drawer or summary section for selected items.
    - [x] Expanding the drawer renders a scrollable list of selected items displaying thumbnail previews, filenames, and file paths.
     [ ] **Safe OS Trash Integration**
    - [x] Selecting "Database & Trash" and confirming moves source files (`.jef`, `.pes`, `.vp3`) to the native OS Trash/Recycle Bin via safe Tauri APIs.
    - [x] Files moved to OS Trash are recoverable from the system trash folder.
    - [x] No unrecoverable hard deletion (`rm` / permanent unlink) occurs on local files.
    - [x] Handles missing source files gracefully (e.g., if a file was manually moved externally), by removing the orphan record from SQLite.

- [x] **State Persistence & Post-Deletion Navigation**
  - [x] Completing the bulk deletion updates the Browse Page grid instantly without requiring a manual refresh.
  - [x] Selection state clears completely following a successful deletion operation.
  - [X] Navigating away from the Browse Page while items are selected prompts a confirmation dialog if a bulk operation is in progress. Deletion of 50 files is too fast to do this test.