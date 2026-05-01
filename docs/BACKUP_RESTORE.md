# Backup and Restore

The Embroidery Catalogue stores all its data — designs, tags, projects, settings, and
preview images — in a single **SQLite database file**.  Backing it up is as simple as
copying that file.

Because imported design files are also copied into the catalogue's managed folder, a
full backup means covering both the **`data/database/catalogue.db`** file and the
**`data/MachineEmbroideryDesigns/`** folder.

---

## In-app backup

The easiest way to back up is to use the **Backup** page inside the app:

1. Open the app in your browser.
2. Click **Backup** in the navigation bar (under Admin).
3. Set your **Database backup folder** and/or your **Designs backup folder**, then click
   **Save destinations**.
4. Click **Backup database now**, **Run incremental backup**, or **Run both backups**.
5. **Keep the page open while the backup runs** so the app can finish and return the result summary.

> **Important:** The database backup and the designs backup are separate. To make a
> **complete backup** of your catalogue, make sure you run **both** backups.

### Database backup

- Creates a timestamped backup of `catalogue.db`, for example:
  `catalogue_2026-04-06_1430.db`
- The source is the live database file at `data/database/catalogue.db`.
- The in-app backup uses **SQLite’s backup API**, so the backup can be created safely while
  the app is running.
- A new timestamped file is created on every run, so you build up a history of snapshots.

### Designs backup

- Runs an **incremental mirror** from `data/MachineEmbroideryDesigns/` to your chosen
  destination folder.
- Files are compared by relative path, size, and modification time:
  - **New file** → copied to the backup folder.
  - **Changed file** → the backup copy is overwritten.
  - **Unchanged file** → skipped (no re-copy).
  - **File removed from source** → moved into a dated archive folder inside the backup
    destination, for example `_deleted\2026-04-06\...`, rather than being permanently
    deleted.  This provides a safety net if a file was removed by mistake.
- The designs backup is based on the **filesystem**, not just the database.  If you remove
  a design from the catalogue (which removes only the database record) but the file still
  exists on disk, it will still be included in the next designs backup.

### Why unchanged files are skipped

Large embroidery libraries can contain thousands of files.  Re-copying everything every
time would be slow and would use unnecessary disk space.  The incremental approach only
copies what has changed, so repeat backups complete quickly.

### Why deleted files are archived, not removed

Deleting from the catalogue only removes the database record — the file remains on disk
until you delete it manually.  Even if you do delete the file from disk, the backup keeps
a dated copy under `_deleted\YYYY-MM-DD\` so you have a window in which to recover it if
the deletion was accidental.

---

## Where is the database?

The database lives inside the app's `data/` directory alongside the managed design files:

| Environment | Database file |
|---|---|
| Developer / local | `data\database\catalogue.db` inside the project root |
| Portable (USB / SD card) | `data\database\catalogue.db` inside `EmbroideryApp\app\` |

---

## Where are the design files?

Embroidery files imported through the catalogue are copied into:

```
data\MachineEmbroideryDesigns\
```

Both the database and the design files live inside `data\`, so **copying the `data\`
folder is a complete backup of the entire catalogue**.

---

## Manual backup — File Explorer

1. Close the application (press **Ctrl+C** in the server window, or run `stop.bat`).
2. Open **File Explorer** and navigate to the project (or `EmbroideryApp\app\`) folder.
3. Copy the entire **`data\`** folder to a safe location such as a USB drive, network
   share, or cloud folder.

> **Name the backup with today's date** so you can tell copies apart, for example:
> `data_backup_2025-06-15\`

### Automated / scripted backup

```bat
REM Run this from the project root (adjust paths as needed)
set TODAY=%DATE:~-4%-%DATE:~3,2%-%DATE:~0,2%
xcopy /E /I /Y "data" "D:\Backups\Embroidery\data_%TODAY%"
```

---

## Restore the catalogue

1. **Stop the application** if it is running.
2. Copy your backup `data\` folder back into the project (or `EmbroideryApp\app\`) folder,
   replacing the existing one:

   ```bat
   xcopy /E /I /Y "D:\Backups\Embroidery\data_2025-06-15" "data"
   ```

3. Start the application again with `start.bat`.

> **Database migrations run automatically on startup.**  If the backup was made from an
> older version of the app, `alembic upgrade head` will update the schema safely without
> losing data.

---

## Copy your catalogue to the USB stick

`prepare_portable_target.bat` (and `EmbroiderySdLauncher.exe`) **do not copy the `data\`
folder automatically**. If you want your current catalogue data on the deployment target,
copy it manually after running the deployment:

```bat
xcopy /E /I /Y "data" "F:\EmbroideryApp\app\data"
```

If you skip this step, the portable app will create a **new empty** `catalogue.db` on
first launch (inside `data\database\`).

---

## Optional: copy the portable catalogue back to your developer machine

```bat
xcopy /E /I /Y "F:\EmbroideryApp\app\data" "data"
```

Use this only when the portable copy contains data you want to import into your developer
environment.  For normal recovery, restore from your **developer backup** instead.

---

## Tips

- Use the **in-app Backup page** for quick, guided backups without closing the app.
- For the **database**, the in-app backup uses SQLite’s built-in backup support for a
  consistent snapshot while the app is open.
- For the **designs folder**, use incremental backups to avoid re-copying large libraries.
- If you rely on the in-app backup page, **keep the page open until the backup completes**.
- **Back up before upgrading** to a new version of the application.
- **Back up before manually replacing the `data\` folder** on the USB stick or SD card.
- If you need to inspect or repair the database, you can open it with
  [DB Browser for SQLite](https://sqlitebrowser.org/) (free, Windows/Mac/Linux).

---

## Related guides

- [GETTING_STARTED.md](GETTING_STARTED.md) — run the app locally
- [USB_DEPLOYMENT.md](USB_DEPLOYMENT.md) — copy the app to removable media
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — fix common problems
