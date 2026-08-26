# Test Plan: Restore Workflows

Use this checklist to test and verify the Backup and Restore feature set in the application, including handling edge cases such as missing paths and corrupt database files.

Some of these tests are carried out via the automated unit tests. These are identified in the document. Search for "manual"

---

## Test Environment Setup

* [X]  **Working Backup Location:** Ensure a valid secondary directory exists (e.g., external drive or dedicated backup folder `H:\Catalogue Backups`).
* [X]  **Test Database Backup:** Ensure at least one valid database snapshot (`EmbroideryCatalogue.db`) is available.
* [X]  **Test Corrupt Database File:** Check that a text file corrupt_test.db exists in the folder tests\\Test Assets.

---

## 1. UI Navigation & Default Folder Resolution

* [X]  Open @BackupView.svelte and verify the **Restore** card section renders correctly alongside the **Backup** cards. - Automated
* [X]  Click **Choose database backup...** under Database Restore. - Automated
* [X]  Verify the file picker opens defaulted to the configured database backup path (e.g., `H:\Catalogue Backups\Database`). - Needs manual test
* [X]  Verify the file extension filter is set to `.db` or `EmbroideryCatalogue.db`. - Needs user test
* [X]  Cancel the file picker and verify no errors or visual breaks occur on @BackupView.svelte. - Automated
* [X]  Pick a valid `EmbroideryCatalogue.db` file and verify the selected file path is correctly displayed in the UI. - Automated

---

## 2. Happy Path Database Restore - Automated

* [X]  Select a valid database backup file (`EmbroideryCatalogue.db`). - Automated
* [X]  Click **Restore database**. - Automated
* [X]  Verify that real-time progress is displayed via @RestoreProgressPanel.svelte during execution. - Automated`
* [X]  Verify a success toast notification appears upon completion. - Automated
* [X]  Check the local database folder on disk (`dev_data/Database/` or app data folder) and confirm a safety fallback copy (`catalogue.pre-restore-<timestamp>.db`) was created and retained. - Automated
* [X]  Navigate to @MainView.svelte and confirm the library contents reflect the restored database snapshot. - Needs manual test

---

## 3. Incremental Designs Restore (Skipping Existing Disk Files) - Automated

* [ ]  Ensure your live design folder (`MachineEmbroideryDesigns`) contains existing files that match the backup folder.
* [ ]  Click **Sync designs from backup** on @BackupView.svelte.
* [ ]  Observe the metrics streamed in @RestoreProgressPanel.svelte.
* [ ]  Verify that files already present on disk are counted under **Skipped** (saving disk I/O).
* [ ]  Verify that any missing or modified files are counted under **Copied/Updated**.
* [ ]  Verify a success toast notification appears detailing total scanned, copied, and skipped files.

---

## 4. Restore Both (Combined Execution) - Automated

* [ ]  Select a valid database backup file. - Automated
* [ ]  Click **Restore Both**. - Automated
* [ ]  Verify the system executes the Database Restore phase first. - Automated
* [ ]  Verify the system seamlessly transitions into the Incremental Designs Restore phase without user intervention. - Needs manual test
* [ ]  Confirm both progress indicators update accurately throughout the operation. - Automated

---

## 5. Unmatched Files Reconciliation (Files on Disk absent from Restored DB) - Automated

* [ ]  Add a new dummy design file (e.g., `test_unmatched.pes`) directly into your live designs directory on disk.
* [ ]  Restore an older database backup that does **not** contain a record for `test_unmatched.pes`.
* [ ]  Upon completion of the restore, verify an inline @Notice.svelte banner appears on @BackupView.svelte stating that unmatched files were detected on disk.
* [ ]  Verify the banner presents two action buttons: **Import Unmatched Files** and **Dismiss**.
* [ ]  Click **Dismiss** on one run to verify the notice clears gracefully.
* [ ]  Trigger the restore again and click **Import Unmatched Files**:
* [ ]  Verify the background batch import process parses the unmatched file without launching a full wizard.
* [ ]  Verify the imported design now appears when browsing @MainView.svelte.

---

## 6. Edge Case: Non-Existent or Invalid Folder Paths - Needs Manual Test

* [ ]  Manually edit the **Designs backup folder** path in the UI field to a non-existent drive or folder (e.g., `Z:\NonExistentFolder`). - Automated
* [ ]  Click **Sync designs from backup**. - Automated
* [ ]  Verify the application catches the invalid path gracefully. - Automated
* [ ]  Verify an error notification (via @ToastContainer.svelte or progress panel) displays a clear message without crashing the application. - Automated
* [ ]  Attempt to trigger **Restore Both** with an invalid designs path. - Needs manual test
* [ ]  Verify the database restore step completes safely (or halts cleanly) and reports the directory error for the design phase. - Needs manual test

---

## 7. Edge Case: Corrupt Database File & Automatic Rollback - Automated

* [ ]  Click **Choose database backup...** and select the prepared `corrupt_test.db` file.
* [ ]  Click **Restore database**.
* [ ]  Observe the execution process:
* [ ]  Verify the backend creates a safety snapshot of the live database prior to testing the candidate file.
* [ ]  Verify `PRAGMA integrity_check` / validation fails on the corrupt file.
* [ ]  Verify the backend performs an **automatic rollback**, restoring the live database from the safety snapshot.
* [ ]  Verify an error message appears in @BackupView.svelte informing you that the backup file was corrupt and the database was safely rolled back.
* [ ]  Navigate to @MainView.svelte and confirm the library remains fully functional and uncorrupted.

---

## 8. Edge Case: Out-of-Sync / Differing Schema Version - Automated

* [ ]  Select a database backup created from a different or older application schema version (where `PRAGMA user_version` differs).
* [ ]  Perform a database restore.
* [ ]  Verify the restore completes without throwing a hard database error.
* [ ]  Verify a mild informational warning banner appears in @BackupView.svelte indicating a schema version mismatch without running forced auto-migrations.
