# Backup and Restore Specification

Reference for backup and restore subsystem behaviour (`src/services/backup.rs`, `frontend/src/lib/views/BackupView.svelte`).

## Storage Locations

| Item | Location | Purpose |
|---|---|---|
| Live Database | `<data_root>/Database/EmbroideryCatalogue.db` | SQLite catalogue containing design metadata, tags, and settings |
| Live Designs | `<data_root>/MachineEmbroideryDesigns/` | Managed embroidery design files |
| Pre-Restore Snapshot | `<data_root>/Database/EmbroideryCatalogue.pre-restore-<timestamp>.db` | Safety copy created immediately before restoring a database |
| Archived Deleted Designs | `<backup_dest>/_deleted/<YYYY-MM-DD>/` | Files removed from live collection, preserved during incremental mirror |

## Backup Operations

### Database Backup
- Invokes SQLite online backup API to stream `<data_root>/Database/EmbroideryCatalogue.db` to `<backup_db_dest>/EmbroideryCatalogue_<YYYY-MM-DD_HHMM>.db`.
- Safe to run concurrently while the app is active.
- Emits advisory warning if live database exceeds 4 GB and destination filesystem is FAT32.

### Designs Backup
- Executes an incremental mirror from `<data_root>/MachineEmbroideryDesigns/` to `<backup_designs_dest>/`.
- Compares relative file path, size in bytes, and last modification timestamp.
- Unchanged files are skipped.
- New files are copied to destination.
- Changed files overwrite destination copy.
- Files deleted from source are moved into `<backup_designs_dest>/_deleted/<YYYY-MM-DD>/` instead of permanent deletion.

## Restore Operations

### Database Restore
1. Validates selected `.db` snapshot existence and header.
2. Closes active SQLite connection pool.
3. Copies live database to `<database>.pre-restore-<timestamp>.db`.
4. Overwrites live database with selected snapshot.
5. Reopens SQLite pool and verifies schema integrity (`PRAGMA integrity_check`).
6. If integrity check fails, automatically restores pre-restore safety copy and alerts user.
7. Checks schema version against current application version. If versions differ, informs user that no automated schema migrations were executed.

### Designs Restore (Sync Designs from Backup)
- Incrementally copies files from `<backup_designs_dest>/` into `<data_root>/MachineEmbroideryDesigns/`.
- Skips files with identical size and timestamp.
- Does not modify database records.

### Complete Restore & Reconciliation
- Executes Database Restore followed by Designs Sync.
- Scans `MachineEmbroideryDesigns` for unmatched design files (files present on disk without matching records in the restored database).
- Displays reconciliation prompt offering to import unmatched files into the catalogue.

