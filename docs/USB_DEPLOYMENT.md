# Portable Deployment

This guide explains how to copy the Embroidery Catalogue to any deployment target
(SD card, USB drive, external drive, or folder path) so it runs on **any Windows PC
without needing Python installed**.  The portable copy is self-contained and carries
everything it needs — including all imported design files.

> **Who is this for?**  Anyone who wants to use the catalogue on a machine they do not
> own, or share it with someone who is not a developer.

---

## Overview

The portable app is stored in an `EmbroideryApp\` folder on the removable media.  It
bundles:

- A stripped-down Windows Embeddable Python 3.12 runtime
- All Python packages (installed offline from local wheel files on first launch)
- The application source code, templates, and static files
- A `data\` folder that holds **both** the SQLite database and all imported design files

Because the database and the design files are both stored inside `data\`, copying the
`data\` folder to removable media (or another machine) gives you a completely
self-contained, portable catalogue — **no drive letters, no external folders needed**.

Once copied, any Windows PC can launch the catalogue by double-clicking `start.bat`
inside the `EmbroideryApp\app\` folder — no installation required.

---

## Prerequisites (developer machine)

Before populating the USB stick you need the following on the **developer machine**
(the PC you copy *from*):

| Item | Where to get it |
|---|---|
| Project root with all source files | This repository |
| `python\` folder | Extract [Python 3.12 Windows Embeddable 64-bit](https://www.python.org/downloads/) into `python\` in the project root, then uncomment `import site` in `python\python312._pth` |
| `wheels\` folder | Run `pip download -d wheels\ --platform win_amd64 --python-version 3.12 --only-binary=:all: -r requirements.txt` once |
| Existing catalogue data (optional) | If you want the portable copy to include your current catalogue data and design files, copy the `data\` folder manually after running `prepare_portable_target.bat` |

> If the `python\` folder or `wheels\` folder is missing, `start.bat` on the USB stick
> will fail on first launch.  See [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

---

## Method 1 — GUI launcher (recommended)

`EmbroiderySdLauncher.exe` is the easiest way to prepare a deployment target.

1. Connect your deployment target (USB stick, SD card, external drive, or choose a folder).
2. Double-click **`EmbroiderySdLauncher.exe`** in the project root.
3. In the *Deployment target location* field, click **Browse…** and select the drive letter or
   path of your target (e.g. `F:\`).
4. Click **OK — Run**.
5. The output panel shows progress.
6. On success, the chosen path is saved for next time.

---

## Method 2 — Command line (power users)

Run `prepare_portable_target.bat` from the project root:

```bat
REM Copy to F:\ (the default)
prepare_portable_target.bat

REM Copy to G:\
prepare_portable_target.bat G:\

REM UNC path target
prepare_portable_target.bat \\server\share
```

Re-running is safe — `robocopy` only copies files that have changed.

---

## What gets copied

The script creates the following structure on the target drive:

```
EmbroideryApp\
  python\                        ← Windows Embeddable Python 3.12 runtime
  app\
    src\                         ← Application source code
    templates\                   ← Jinja2 templates
    static\                      ← CSS, JavaScript, images
    alembic\                     ← Database migration files
    wheels\                      ← Offline Python package wheels
    data\                        ← Portable data folder (created on first run)
      database\                  ←   catalogue.db (created on first run)
      MachineEmbroideryDesigns\  ←   all imported design files
    alembic.ini
    requirements.txt
    setup.bat                    ← First-run setup (called automatically by start.bat)
    start.bat                    ← Launch the application
    stop.bat
    get-pip.py
```

---

## Copying your existing catalogue data (optional)

`prepare_portable_target.bat` does **not** copy the `data\` folder automatically. This helps
protect an existing portable database from being overwritten by a developer or test copy.

If you want the portable copy to use your existing data and design files, copy the
entire `data\` folder yourself after the app files have been populated:

```bat
xcopy /E /I /Y "data" "F:\EmbroideryApp\app\data"
```

If you do nothing, the portable app creates a new empty `catalogue.db` on first start
and the `MachineEmbroideryDesigns\` folder will be empty — ready for you to import
designs directly on the portable device.

---

## Launching on another Windows PC

1. Connect the USB stick, SD card, or external drive, or navigate to your target folder.
2. Navigate to `EmbroideryApp\app\` on the device.
3. Double-click **`start.bat`**.

**First launch only:**  `start.bat` detects that the virtual environment is missing and
runs `setup.bat` automatically.  This installs packages from the bundled `wheels\`
folder — **no internet connection is needed**.  Setup takes about 30–60 seconds.

Subsequent launches skip setup and start immediately.

4. Your browser opens at <http://localhost:8002>.

> **Port note:**  The portable app uses port **8002** by default.  The developer version
> uses **8003**.  Set `APP_PORT` in `.env` if you need a different port.

---

## Importing designs on the portable device

When the app is running from the USB stick, use the built-in **Bulk Import** screen to
add designs. The import workflow lets you:

1. Select **one or more source folders** containing embroidery files.
2. Review the files found, grouped by folder.
3. Set **Designer** and **Source** values per folder by keeping the inferred value, choosing an existing one,
   creating a new one on the spot, or leaving it blank.
4. Review the AI tagging status banner, which shows whether Tier 1/2/3 tagging will run.
5. Review tags before import when prompted (required for the very first import, optional afterwards).
6. Copy the selected files into `data\MachineEmbroideryDesigns\` on the USB stick.

After import, the original source folder is **not required** — all design files live
inside the `data\` folder on the device.

### AI tagging on the portable device

If a Google API key was included in the `.env` file during population, the portable app
can use Gemini-based AI tagging.  Enable Tier 2 and/or Tier 3 via **Admin → Settings**
on the target machine.  The import screen will show a cost/quota notice when a key is
active.  For large imports, use the **AI tagging batch size** setting to spread API calls
across multiple runs, and use the **Import database commit batch size** setting to control
how many designs are committed per DB transaction.

---

## Updating the portable copy

After making changes on your developer machine and testing them, re-run
`prepare_portable_target.bat` (or the GUI launcher).  `robocopy /MIR` will copy only the
changed files, so the update is fast.

> **Your portable data is preserved.** The script does not copy or replace the `data\`
> folder automatically.

---

## Moving the USB stick to a different drive letter

The application uses paths relative to its own location, so it works correctly
regardless of which drive letter the USB stick is assigned.  All design files are
stored inside the app's `data\MachineEmbroideryDesigns\` folder, so no path
reconfiguration is needed when the drive letter changes.

---

## Related guides

- [GETTING_STARTED.md](GETTING_STARTED.md) — run the app locally from the repository
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) — back up the data/ folder before copying
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — fix common problems
