# Backup and Restore

Use [Admin → System Maintenance → Backup](#/admin/system/backup) to protect your catalogue database and design files, or to restore them on a new computer.

A complete catalogue backup covers two parts:
1. **The database**: your tags, search index, and design information.
2. **The design files**: your actual embroidery stitch files.

---

## Set your backup folders

Before backing up for the first time:

1. Open [Admin → System Maintenance → Backup](#/admin/system/backup).
2. Under **Database backup folder**, click **Browse…** and select where database snapshots should be saved (e.g. an external drive or cloud sync folder).
3. Under **Designs backup folder**, click **Browse…** and select where copies of your design files should be saved.
4. Click **Save destinations**.

---

## Back up your catalogue

1. Open [Admin → System Maintenance → Backup](#/admin/system/backup).
2. Choose what to back up:
   - **Backup Everything Now** (recommended): creates a fresh database snapshot and copies any new or changed design files.
   - **Backup Database Now**: creates only a timestamped copy of your database.
   - **Run incremental backup**: mirrors your design files to your backup folder, skipping unchanged files.
3. Keep the app open until the progress finishes.

---

## Restore from a backup

1. Open [Admin → System Maintenance → Backup](#/admin/system/backup) and click the **Restore** tab at the top.
2. Under **Restore Database**, click **Choose file…** and select a `.db` database backup file.
3. Click **Restore Database Now** (or **Restore Everything Now** to restore both database and designs).
4. The app checks the backup file, replaces the database, and verifies the catalogue.

A safety copy of your current database is automatically created before any restore. If the backup file is corrupted, your previous database is restored automatically.

---

## Find unmatched files after a restore

If you copied design files into your library outside the app, or after restoring design files:

1. Go to [Admin → System Maintenance → Library Discrepancies](#/admin/system/orphans).
2. Click **Scan for unmatched files**.
3. If unmatched files are found, click **Import** to add them to your catalogue database.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| A warning says a backup file is 4 GB or larger | The backup drive is formatted as FAT32. Reformat the destination drive to exFAT or NTFS, which support large files. |
| Restore fails verification | The app automatically rolls back to your previous database. Try selecting an earlier backup snapshot. |
| You want to cancel a running backup | Click **Cancel Backup**. Database snapshots already saved remain valid. |
| Designs are restored but not showing in Browse | Go to [System → Library Discrepancies](#/admin/system/orphans) and click **Scan for unmatched files**, then **Import**. |

---

## Related guides

- [Data Storage Guide](DATA_STORAGE_GUIDE.md)
- [Settings](SETTINGS.md)
- [Import Workflow](IMPORT_WORKFLOW.md)
