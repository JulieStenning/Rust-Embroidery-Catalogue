# Manual User Test Plan: Restore Workflows

Use this checklist to manually test and verify the Backup and Restore feature set in the application, including handling edge cases such as missing paths and corrupt database files.

---

## Test Environment Setup

* [ ] **Working Backup Location:** Ensure a valid secondary directory exists (e.g., external drive or dedicated backup folder `H:\Catalogue Backups`).
* [ ] **Test Database Backup:** Ensure at least one valid database snapshot (`EmbroideryCatalogue.db`) is available.
* [ ] **Test Corrupt Database File:** Create a text file containing random text (e.g., `"Not a database"`), save it as `corrupt_test.db`, and place it in a reachable directory.

---

## 1. UI Navigation & Default Folder Resolution

* [ ] Open @BackupView.svelte and verify the **Restore** card section renders correctly alongside the **Backup** cards.
* [ ] Click **Choose database backup...** under Database Restore.
* [ ] Verify the file picker opens defaulted to the configured database backup path (e.g., `H:\Catalogue Backups\Database`).
* [ ] Verify the file extension filter is set to `.db` or `EmbroideryCatalogue.db`.


* [ ] Cancel the file picker and verify no errors or visual breaks occur on @BackupView.svelte.
* [ ] Pick a valid `EmbroideryCatalogue.db` file and verify the selected file path is correctly displayed in the UI.

---

## 2. Happy Path Database Restore

* [ ] Select a valid database backup file (`EmbroideryCatalogue.db`).
* [ ] Click **Restore database**.
* [ ] Verify that real-time progress is displayed via @RestoreProgressPanel.svelte during execution.
* [ ] Verify a success toast notification appears upon completion.
* [ ] Check the local database folder on disk (`dev_data/Database/` or app data folder) and confirm a safety fallback copy (`catalogue.pre-restore-<timestamp>.db`) was created and retained.
* [ ] Navigate to @MainView.svelte and confirm the library contents reflect the restored database snapshot.

---

## 3. Incremental Designs Restore (Skipping Existing Disk Files)

* [ ] Ensure your live design folder (`MachineEmbroideryDesigns`) contains existing files that match the backup folder.
* [ ] Click **Sync designs from backup** on @BackupView.svelte.
* [ ] Observe the metrics streamed in @RestoreProgressPanel.svelte.
* [ ] Verify that files already present on disk are counted under **Skipped** (saving disk I/O).
* [ ] Verify that any missing or modified files are counted under **Copied/Updated**.


* [ ] Verify a success toast notification appears detailing total scanned, copied, and skipped files.

---

## 4. Restore Both (Combined Execution) - Manual Test Required

* [ ] Select a valid database backup file.
* [ ] Click **Restore Both**.
* [ ] Verify the system executes the Database Restore phase first.
* [ ] Verify the system seamlessly transitions into the Incremental Designs Restore phase without user intervention.
* [ ] Confirm both progress indicators update accurately throughout the operation.

---

## 5. Unmatched Files Reconciliation (Files on Disk absent from Restored DB)

* [ ] Add a new dummy design file (e.g., `test_unmatched.pes`) directly into your live designs directory on disk.
* [ ] Restore an older database backup that does **not** contain a record for `test_unmatched.pes`.
* [ ] Upon completion of the restore, verify an inline @Notice.svelte banner appears on @BackupView.svelte stating that unmatched files were detected on disk.
* [ ] Verify the banner presents two action buttons: **Import Unmatched Files** and **Dismiss**.
* [ ] Click **Dismiss** on one run to verify the notice clears gracefully.
* [ ] Trigger the restore again and click **Import Unmatched Files**:
* [ ] Verify the background batch import process parses the unmatched file without launching a full wizard.
* [ ] Verify the imported design now appears when browsing @MainView.svelte.



---

## 6. Edge Case: Non-Existent or Invalid Folder Paths - Needs Manual Test

* [ ] Manually edit the **Designs backup folder** path in the UI field to a non-existent drive or folder (e.g., `Z:\NonExistentFolder`).
* [ ] Click **Sync designs from backup**.
* [ ] Verify the application catches the invalid path gracefully.
* [ ] Verify an error notification (via @ToastContainer.svelte or progress panel) displays a clear message without crashing the application.


* [ ] Attempt to trigger **Restore Both** with an invalid designs path.
* [ ] Verify the database restore step completes safely (or halts cleanly) and reports the directory error for the design phase.



---

## 7. Edge Case: Corrupt Database File & Automatic Rollback

* [ ] Click **Choose database backup...** and select the prepared `corrupt_test.db` file.
* [ ] Click **Restore database**.
* [ ] Observe the execution process:
* [ ] Verify the backend creates a safety snapshot of the live database prior to testing the candidate file.
* [ ] Verify `PRAGMA integrity_check` / validation fails on the corrupt file.
* [ ] Verify the backend performs an **automatic rollback**, restoring the live database from the safety snapshot.


* [ ] Verify an error message appears in @BackupView.svelte informing you that the backup file was corrupt and the database was safely rolled back.
* [ ] Navigate to @MainView.svelte and confirm the library remains fully functional and uncorrupted.

---

## 8. Edge Case: Out-of-Sync / Differing Schema Version

* [ ] Select a database backup created from a different or older application schema version (where `PRAGMA user_version` differs).
* [ ] Perform a database restore.
* [ ] Verify the restore completes without throwing a hard database error.
* [ ] Verify a mild informational warning banner appears in @BackupView.svelte indicating a schema version mismatch without running forced auto-migrations.