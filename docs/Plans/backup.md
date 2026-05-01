# Backup Feature Plan

## Goal

Add a user-facing backup feature that allows the user to back up:

1. the SQLite database, and/or
2. the managed designs folder

from inside the app, with **separate destination locations** for each.

> This file is a **handoff plan only**. No code changes are included here.

---

## Current behavior and constraints

- The live database is stored in `data/database/catalogue.db`.
- Imported design files are stored in `data/MachineEmbroideryDesigns/`.
- `docs/BACKUP_RESTORE.md` currently recommends copying the whole `data/` folder.
- For large libraries, a full copy of the designs folder on every backup is not practical.
- Deleting a design in the app currently removes the **database record only**.
- If a user wants to delete the actual file, they do that separately in File Explorer.

### Important implication

The **designs backup must follow the filesystem**, not just the database.

If a record is removed from the catalogue but the file still exists in `data/MachineEmbroideryDesigns`, that file should still be included in the backup.

---

## Recommended backup model

### Database backup
- Uses SQLite’s built-in backup API for a consistent in-app backup of `catalogue.db`
- Written to a user-selected **Database Backup Location**
- Uses timestamped filenames, for example:
  - `catalogue_2026-04-06_1430.db`

### Designs backup
- Backup from the actual `data/MachineEmbroideryDesigns/` folder
- Written to a user-selected **Designs Backup Location**
- Use **incremental mirror backup**:
  - copy new files
  - update changed files
  - skip unchanged files
- Do **not** re-copy the whole library every time

### Deleted files policy
If a file previously backed up no longer exists in the source designs folder, do **not** permanently delete it from the backup immediately.

Instead, move it into a dated archive such as:

`<DesignsBackupLocation>\_deleted\2026-04-06\...`

This provides a safety net if a file was deleted by mistake.

---

## Step-by-step implementation plan

## 1. Confirm scope and wording

Define the first release clearly:

- Include **manual backups only**
- Support:
  - `Backup database now`
  - `Backup designs now`
  - optional `Backup both`
- Exclude scheduled/automatic backups for now
- Exclude changing the app’s live storage model

Also standardise the wording in the UI:

- **Remove from catalogue** = deletes DB row only
- **Delete file from disk** = different action, not part of this feature unless explicitly added later

---

## 2. Add backup configuration settings

Persist separate backup destinations for:

- `backup.database_destination`
- `backup.designs_destination`

Optional future settings:

- `backup.designs_deleted_policy` (`archive`, `keep`, `strict-mirror`)
- `backup.retention_count`
- `backup.last_run_*` metadata

Recommended first release:
- store only the two destination paths
- calculate last-run details from the most recent backup files or a lightweight manifest

---

## 3. Add a new admin backup page

Create a dedicated page under admin/maintenance, for example:

- `/admin/maintenance/backup`

### Page layout

#### A. Database Backup
- destination folder field
- browse/select folder action if available in the app pattern
- `Backup now` button
- last backup time
- last backup size

#### B. Designs Backup
- destination folder field
- `Run incremental backup` button
- result summary:
  - files copied
  - files updated
  - files skipped
  - files archived because they were deleted from source

#### C. Optional combined action
- `Run both backups`

Use the existing admin page styling and success/error banners already used elsewhere in the app.

---

## 4. Implement database backup service

Add a service module for backup logic, for example:

- `src/services/backup_service.py`

### Database backup behavior

1. Resolve the live database path from configuration.
2. Validate that the destination folder exists or can be created.
3. Create a timestamped backup filename.
4. Use SQLite’s backup API to create a consistent backup while the app is still running.
5. Return summary metadata such as:
   - backup file path
   - size in MB
   - completed timestamp

### Notes

- The in-app implementation should prefer SQLite’s backup API over a raw file copy.
- This avoids WAL-mode consistency problems when the app is running during the backup.

---

## 5. Implement designs incremental backup service

Use filesystem-based comparison between:

- source: `data/MachineEmbroideryDesigns/`
- destination: chosen designs backup folder

### Compare by
- relative path
- file size
- modified timestamp

### Rules

- **new file** → copy it to backup
- **changed file** → overwrite/update backup copy
- **unchanged file** → skip it
- **missing from source but present in backup** → move it to `_deleted\YYYY-MM-DD\...`
- **empty folders left behind after archiving** → remove them from the live backup tree

### Important rule

Do **not** use the database as the authoritative source for the designs backup.

The filesystem is authoritative because the app currently allows DB-only deletion.

---

## 6. Add safety and error handling

Handle these cases explicitly:

- destination folder not writable
- missing database file
- missing designs source folder
- insufficient disk space
- interrupted backup process
- invalid destination path

Use the app’s existing redirect-with-message pattern to show:

- success notices
- error notices
- a plain-language backup summary after each run

---

## 7. Add backup result reporting

After each run, show a concise summary such as:

### Database backup summary
- file created
- backup path
- size
- completed time

### Designs backup summary
- scanned files count
- copied count
- updated count
- unchanged count
- archived deleted count
- total bytes copied
- completed time

This can be shown directly on the backup page and optionally logged to the app log.

---

## 8. Optional restore phase

Restore can be a separate follow-up phase.

If included later, keep it separate for:

- `Restore database`
- `Restore designs`

### Restore safety requirements

Before restoring:

1. create a rollback copy of the current database and/or designs state
2. warn clearly before overwriting live data
3. keep database restore separate from designs restore

This should be a later phase unless a restore workflow is needed immediately.

---

## 9. Documentation updates

Update documentation so the user guidance matches the new in-app feature.

Likely documents:

- `docs/BACKUP_RESTORE.md`
- `docs/GETTING_STARTED.md`
- `docs/TROUBLESHOOTING.md`

The docs should explain:

- how the database backup works
- how the designs incremental backup works
- why unchanged files are skipped
- how deleted files are archived instead of removed permanently
- that app deletion currently only removes the catalogue entry, not the file from disk

---

## 10. Testing and verification plan

### Functional checks

1. Back up the database to Location A and confirm a timestamped `.db` file is created.
2. Back up the designs to Location B and confirm only new/changed files are copied.
3. Run the designs backup again without changes and confirm most files are skipped.
4. Modify a small set of design files and confirm only those files are updated in backup.
5. Delete a design file from the source folder in File Explorer and confirm the next backup archives the previous backup copy into `_deleted\YYYY-MM-DD\`.
6. Remove a design from the app only and confirm the file remains in the designs backup if it still exists on disk.

### UX checks

- success and error banners display correctly
- backup destinations are remembered correctly
- `Save destinations` is enabled only when there is an outstanding change
- the backup action buttons depend on the last saved destination values
- a visible “keep this page open” overlay appears while backup is running
- large libraries do not trigger a full recopy on every run

---

## Suggested implementation order

1. Add the backup plan document and confirm scope.
2. Add persisted settings for separate database and designs backup destinations.
3. Create `backup_service.py` with database backup logic.
4. Add designs incremental backup logic with deleted-file archiving.
5. Add `/admin/maintenance/backup` page and actions.
6. Show backup summaries and error messages in the UI.
7. Update `docs/BACKUP_RESTORE.md` to explain the new flow.
8. Consider restore support as a separate phase.

---

## Relevant files for the implementation agent

- `src/config.py` — live database and managed designs paths
- `src/routes/maintenance.py` — likely route area for backup UI/actions
- `src/routes/settings.py` — possible settings persistence pattern
- `src/services/settings_service.py` — settings helpers
- `src/services/bulk_import.py` — file-copying/path-handling patterns
- `templates/admin/settings.html` — existing admin form style
- `templates/admin/orphans.html` — existing admin maintenance UX style
- `docs/BACKUP_RESTORE.md` — current backup documentation

---

## Decisions

### Included
- separate backup destinations for database and designs
- SQLite API-based in-app database backup
- incremental designs backup
- deleted-file archiving for designs backups
- empty-folder cleanup after deleted files are archived
- manual user-triggered backups from inside the app

### Excluded for now
- scheduled/automatic backups
- changing the app’s live storage locations
- treating DB deletion as file deletion
- strict mirror deletion from the backup destination

### Recommendation

For this codebase and current workflow, the best first implementation is:

> **Database backup using SQLite’s backup API into a timestamped `.db` file, and designs backup as an incremental filesystem-based mirror with deleted-file archiving and empty-folder cleanup.**
