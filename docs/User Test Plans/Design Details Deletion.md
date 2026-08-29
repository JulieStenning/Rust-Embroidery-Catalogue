## Deletion Feature Test Cases

### 1. Browse Page Multi-Item Deletion

* [ ] **Batch Database-Only Deletion (Up to 50 items):** Select multiple designs (e.g., 5 to 50 items) on the Browse page, click Delete, choose *"Catalogue only"*, and confirm. Verify that:
* All selected cards are immediately removed from the UI.
* The entries are removed from the local SQLite database.
* The actual embroidery files (e.g., `.pes`, `.jef`) still exist in their original local directories.


* [ ] **Batch Database + Disk File Deletion:** Select multiple designs, click Delete, choose *"Catalogue and move files to Trash"*, and confirm. Verify that:
* Cards are removed from the Browse page UI.
* Database records are deleted.
* The original design files are moved to the OS Recycle Bin / Trash (and **not** permanently unlinked directly).


* [ ] **Batch Selection Summary & Drawer Toggle:** Select 10+ items on the Browse page and open the delete modal. Verify that:
* The modal correctly displays the exact count (e.g., "10 designs selected").
* Toggling the preview drawer expands a scrollable list of selected file names/thumbnails.
* Closing or canceling the modal leaves all selected items untouched in both the UI and database.



---

### 2. Design Details Page Single-Item Deletion

* [ ] **Single Item Database-Only Deletion:** Open a single design's details page, click Delete, select *"Catalogue only"*, and confirm. Verify that:
* The single-item modal appears **without** the collapsible summary/preview list.
* Upon confirmation, the application navigates back to the Browse view cleanly without rendering errors or missing metadata warnings.
* The design entry is removed from the SQLite database while keeping the local binary file intact.


* [ ] **Single Item Database + Disk File Deletion:** Open a single design's details page, click Delete, select *"Catalogue and move files to Trash"*, and confirm. Verify that:
* The source embroidery file is moved to the OS Trash.
* The database entry is removed.
* The app redirects to the Browse page, and the deleted item is no longer visible in the grid.


* [ ] **Detail Page Cancellation:** Open the delete modal from the Design Details view and click *Cancel*. Verify that the modal closes and the user remains on the active details view with all metadata intact.

---

### 3. Edge Cases & Error Handling

* [ ] **Missing Source File on Disk:** Attempt to perform a *"Catalogue and move files to Trash"* operation on a design whose physical file was already moved or deleted via OS File Explorer prior to app usage. Verify that:
* The backend handles the missing path gracefully (e.g., purges the SQLite record or alerts the user without crashing the Tauri app).
* No broken state or freeze occurs on the UI.


* [ ] **Read-Only or Permission Error:** Attempt to delete a file located in a restricted or read-only directory. Verify that:
* The Tauri IPC backend catches the file system permission error.
* An appropriate error toast/dialog is surfaced to the user.
* The SQLite transaction rolls back (the item remains in the catalogue if file deletion fails).


* [ ] **Empty Selection Prevention:** Ensure the bulk deletion button/action on the Browse page is disabled or hidden when no items are actively selected.