# Release Manual Acceptance Test Plan

This manual acceptance suite validates critical system interactions and release gates defined in [Release Types and Migration Scope Policy](../policies/releases/release-types-and-migration-scope.md).

> **Execution Cadence:**
> - **Hotfix Releases:** Execute Section 2 (Existing DB Upgrade Smoke) and Section 4.1 (Native Dialogs).
> - **Minor Releases:** Execute Sections 1, 2, 3, 4, and 5.
> - **Major Releases:** Execute all sections, including full uninstallation and rollback rehearsal.

---

## 1. Clean Machine / Fresh Installation Scenario

* [ ] **1.1 Fresh Install Execution**
  * Run the release installer (`.msi` / `.exe` / `Install.lnk`) on a test machine or clean environment with no existing `%APPDATA%\embroidery-catalogue` or configured data directory.
  * Verify the installer completes without errors and creates desktop/start-menu shortcuts.
* [ ] **1.2 First-Run Database Initialisation**
  * Launch the newly installed application.
  * Verify the application starts smoothly, runs all initial schema migrations, and displays an empty catalogue ready for import.
  * Confirm that default system tables and initial settings are created cleanly without warning dialogues.

---

## 2. Existing Database Schema Upgrade Scenario

* [ ] **2.1 Backup Before Upgrade**
  * Prepare a copy of a real, legacy user database (`EmbroideryCatalogue.db`) from a prior release.
* [ ] **2.2 Migration Execution on Existing Data**
  * Place the legacy database into the active data directory and launch the new release build.
  * Observe startup: verify that schema migrations execute automatically in the background without locking or crashing.
* [ ] **2.3 Data Integrity & Feature Verification**
  * Confirm existing design counts, thumbnails, tags, ratings, hoops, designers, and projects are intact and display correctly on the **Browse** screen.
  * Test any new features introduced in this release against the migrated records.

---

## 3. Custom Data-Root Persistence Scenario

* [ ] **3.1 Custom Data Location in `config.json`**
  * Configure a non-default storage directory in `config.json` (e.g. `D:\CustomEmbroideryData` or a secondary partition).
  * Launch the application and perform an import of several designs.
* [ ] **3.2 Path Verification**
  * Verify that newly imported files, thumbnails, and database records reside inside the specified custom directory.
  * Close and re-open the app; confirm all designs remain accessible from the custom location.

---

## 4. Native OS & Shell Integrations

* [ ] **4.1 Native Windows File / Directory Dialogs**
  * Open **Import** → Click **Select Folder** (or **Backup & Restore** → **Choose database backup...**).
  * Verify the native Windows file/directory chooser opens.
  * Verify the default path correctly points to the configured directory.
  * Cancel the dialog → Verify the app handles cancellation gracefully without UI freeze or error state.
  * Pick a valid directory/file → Verify path correctly populates in the UI.
* [ ] **4.2 File Drag-and-Drop from Windows Explorer**
  * Drag an embroidery file (`.pes`, `.jef`, `.vp3`) from a Windows Explorer window and drop it onto the application window.
  * Verify the application recognizes the dropped file appropriately.
* [ ] **4.3 Native OS Recycle Bin Verification**
  * In **Browse Designs**, select a design and perform **Delete** with the **Database & Trash** option checked.
  * Open the Windows **Recycle Bin** on the desktop.
  * Confirm the physical embroidery file is present in the Recycle Bin and can be restored.

---

## 5. Real-World Database Backup & Restore Verification

* [ ] **5.1 Database & Design Backup**
  * Go to **Backup & Restore** → Trigger a full database and designs backup to a secondary folder (e.g. external drive).
  * Verify backup files (`EmbroideryCatalogue.db` and design files) are generated on disk.
* [ ] **5.2 Safety Snapshot Retention**
  * Perform a database restore using a valid backup file.
  * Inspect the database folder on disk and verify a safety fallback snapshot (`catalogue.pre-restore-<timestamp>.db`) was created prior to applying the restore.
* [ ] **5.3 Corrupt Database File Handshake**
  * Select a corrupted or invalid `.db` file in the restore picker and attempt a restore.
  * Verify the app catches the integrity check failure, displays a clear error toast, performs an automatic rollback from the safety snapshot, and leaves the catalogue fully intact.

---

## 6. Uninstallation & Cleanup Scenario

* [ ] **6.1 Uninstaller Execution**
  * Run the uninstaller (`uninstall.bat` / Windows Add or Remove Programs).
  * Verify application binaries, shortcut icons, and registry keys are cleanly removed.
* [ ] **6.2 User Data Protection**
  * Verify the uninstallation does not delete user embroidery designs or external backup directories.
