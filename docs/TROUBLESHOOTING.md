# Troubleshooting

Solutions to the most common problems when running or deploying the Embroidery Catalogue.

---

## Table of contents

- [The app won't start](#the-app-wont-start)
- [Port already in use](#port-already-in-use)
- [Browser does not open automatically](#browser-does-not-open-automatically)
- [Database errors on startup](#database-errors-on-startup)
- [Designs show no preview image](#designs-show-no-preview-image)
- [setup.bat fails on first launch (portable/USB)](#setupbat-fails-on-first-launch-portableusb)
- [App is slow or unresponsive](#app-is-slow-or-unresponsive)
- [AI tagging is not working](#ai-tagging-is-not-working)
- [EmbroiderySdLauncher.exe won't open](#embroideryslauncherexe-wont-open)
- [prepare_portable_target.bat exits with an error](#prepare_portable_targetbat-exits-with-an-error)

---

## The app won't start

**Symptom:** Double-clicking `start.bat` shows a brief black window that closes
immediately, or a Command Prompt window stays open with an error message.

**Common causes and fixes:**

> **Portable logs:** if the SD-card copy fails during startup or first-run setup, check
> `EmbroideryApp\app\logs\startup-error.log`.
>
> Once the web app is already running, the regular runtime log folder is still
> `EmbroideryApp\app\logs\`.

1. **Python not found.**  `start.bat` requires Python 3.12 and a `.venv` virtual
   environment (or `venv` on the portable version).

   - Check Python is installed: `py --version`
   - If the command is not found, install Python 3.12 from <https://www.python.org/>.
   - Recreate the virtual environment:
     ```bat
     py -3.12 -m venv .venv
     .venv\Scripts\activate
     pip install -r requirements.txt
     ```

2. **Dependencies not installed.**  If you skipped Step 5 of
   [GETTING_STARTED.md](GETTING_STARTED.md), run:

   ```bat
   .venv\Scripts\activate
   pip install -r requirements.txt
   ```

3. **Wrong working directory.**  Always run `start.bat` from the project root (the
   folder that contains `start.bat` itself). You can create a shortcut to start.bat in the 
   root folder of the portable version.

---

## Port already in use

**Symptom:** The server starts but the browser shows a connection error, or `start.bat`
reports `OSError: [Errno 98] Address already in use`.

**Fix:**

- Another instance of the app (or a different programme) is using port 8003 (dev) or
  8002 (portable).  Stop the other process, or choose a different port by setting
  `APP_PORT` in `.env`:

  ```
  APP_PORT=8010
  ```

- To find what is using the port on Windows:

  ```bat
  netstat -ano | findstr :8003
  ```

  Note the PID in the last column, then end the process in Task Manager.

---

## Browser does not open automatically

**Symptom:** The server starts but the browser does not open.

**Fix:**  Navigate manually to <http://localhost:8003> (or <http://localhost:8002> for
the portable version).

If the page still does not load, check the Command Prompt for error messages.

---

## Database errors on startup

**Symptom:** `start.bat` shows an error such as:

```
alembic.util.exc.CommandError: Can't locate revision identified by '…'
```

or

```
sqlalchemy.exc.OperationalError: no such table: designs
```

or

```
FileNotFoundError: Could not locate delivered tags.csv
```

**Fixes:**

1. **Run migrations manually:**

   ```bat
   .venv\Scripts\activate
   alembic upgrade head
   ```

2. **Database is from a newer version of the app.**  If you copied a database from a
   newer version of the app back to an older codebase, the schema may not match.  Pull
   the latest code and rerun `start.bat`.

3. **Portable copy is missing `data\tags.csv`.**
   The starter tag list is needed during first-run database setup.

   - Re-run `prepare_portable_target.bat` or `EmbroiderySdLauncher.exe` from the updated project.
   - Or copy the file manually from the developer machine:

     ```bat
     copy /Y "data\tags.csv" "F:\EmbroideryApp\app\data\tags.csv"
     ```

4. **Corrupted database.**  Restore from a backup (see
   [BACKUP_RESTORE.md](BACKUP_RESTORE.md)).

---

## Designs show no preview image

**Symptom:** The design list or detail page shows a placeholder instead of a stitch
preview image.

**Common causes:**

- **Image was never generated.**  Preview images are created automatically when a design
  is imported.  If the image generation failed (for example, because of an unsupported
  file format), the preview will be missing.  Re-import the design file using *Bulk
  Import* to regenerate it.

- **Design file has moved.**  If files in `data\MachineEmbroideryDesigns\` have been
  moved or removed outside the app, the catalogue may not be able to re-read them.
  Restore or re-import the missing files into the managed storage folder.

---

## setup.bat fails on first launch (portable/USB)

**Symptom:** The first launch of `start.bat` on a new machine shows a setup error and
then exits.

**Common causes and fixes:**

1. **`python\` folder is missing or incomplete.**
   The portable app needs a bundled Python runtime.  Make sure the `python\` folder
   (containing `python.exe`, `python312.dll`, and the `.pth` file) was copied to the
   `EmbroideryApp\` folder alongside `app\`.

   - Re-run `prepare_portable_target.bat` or `EmbroiderySdLauncher.exe` on the developer
     machine to ensure the `python\` folder is included.

2. **`import site` is not uncommented in `python312._pth`.**
   Open `EmbroideryApp\python\python312._pth` in Notepad and make sure the line
   `import site` is present and **not** commented out (no leading `#`).

3. **`wheels\` folder is empty or missing required packages.**
   The offline installer needs all wheels in `EmbroideryApp\app\wheels\`.
   Re-run on the developer machine:

   ```bat
   pip download -d wheels\ --platform win_amd64 --python-version 3.12 --only-binary=:all: -r requirements.txt
   ```

   Then re-deploy to the USB stick.

4. **You see `No matching distribution found for pywebview>=5.0`.**
   `pywebview` is only needed for the installer/desktop window build, not for the
   portable copy. This error means the portable files were populated from an
   older dependency list. Re-run `prepare_portable_target.bat` (or `EmbroiderySdLauncher.exe`)
   from the updated project so the copied `requirements.txt` matches the bundled
   `wheels\` folder.

> **Portable logs:** once the app itself has started, the log folder is
> `EmbroideryApp\app\logs\`. But if `setup.bat` fails before Python starts, there is
> no `app.log` yet — the Command Prompt output is the error to use.

---

## App is slow or unresponsive

**Symptom:** Pages take several seconds to load.

**Common causes:**

- **Large design collection.**  Queries over tens of thousands of designs can be slow.
  Filtering or searching narrows the result set and improves performance.

- **Running from a slow USB stick.**  USB 2.0 flash drives are significantly slower
  than internal storage for database reads.  Use a USB 3.0 drive or an SD card with
  UHS-I speed class or faster.

- **Preview images loading slowly.**  The first time each image is requested, it may
  need to be generated.  Subsequent requests are faster.

---

## AI tagging is not working

**Symptom:** Tiers 2 or 3 do not run during import, or running `auto_tag.py` produces
a message such as `GOOGLE_API_KEY not set` or requests to the Gemini API fail with an
authentication error.

**Fix:**

- Ensure the API key is saved in **Admin → Settings** (the preferred method), or add it
  manually to a `.env` file in the project root:

  ```
  GOOGLE_API_KEY=your_actual_key_here
  ```

- In **Admin → Settings**, confirm that **Run Tier 2 automatically during import** and/or
  **Run Tier 3 automatically during import** are ticked.  Without these settings enabled,
  no Gemini calls are made during import even when a key is present.

- Check the key is valid by testing it in [Google AI Studio](https://aistudio.google.com/).
- Free-tier keys are rate-limited to 15 requests per minute.  If you hit the limit,
   use the **AI tagging batch size** setting to spread calls across multiple import runs,
   use the **Import database commit batch size** setting for better large-import DB stability,
  or increase the CLI delay:

  ```bat
  python auto_tag.py --delay 6.0
  ```

See [AI_TAGGING.md](AI_TAGGING.md) for full setup instructions.

---

## EmbroiderySdLauncher.exe won't open

**Symptom:** Double-clicking `EmbroiderySdLauncher.exe` does nothing, or shows a
"missing DLL" error.

**Common causes:**

- **Visual C++ redistributable missing.**  Install the
  [Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).

- **Windows Defender / antivirus has blocked the file.**  Because the exe is compiled
  with PyInstaller and not code-signed, some security software may quarantine it.
  Check your quarantine list and add an exception if needed.

- **Run the script directly instead:**

  ```bat
  .venv\Scripts\activate
  python portable_launcher.py
  ```

---

## prepare_portable_target.bat exits with an error

**Symptom:** `prepare_portable_target.bat` prints an error and exits before completing the copy.

**Common causes:**

| Error message | Fix |
|---|---|
| `Target root does not look like a valid drive letter or UNC path` | Provide a valid drive root such as `F:\` or a UNC path such as `\\server\share` |
| `Designs source not found` | Check that the designs source folder exists and is readable |
| `robocopy exited with code 8 or higher` | Insufficient disk space on target, or a file lock.  Free space and retry. |
| `Missing prerequisites` | One of `src\`, `templates\`, `wheels\`, or `python\` is missing from the project root |

To see more detail, run the script directly from a Command Prompt rather than by
double-clicking it, so the window stays open after an error.

---

## Still stuck?

Check the output in the Command Prompt window for the exact error message, then search
for that message in the planning documents in `docs/Plans/`.

If you believe you have found a bug, open an issue on GitHub with:

- The exact error message
- What you were doing when the error occurred
- Your Windows version and Python version (`py --version`)

---

## Windows installer build (desktop app)

This section covers the paid Windows installer build.  For the free repository build,
see the sections above.

### The desktop app does not start

**Symptom:** Double-clicking the Start Menu shortcut or `EmbroideryCatalogue.exe` does
nothing, or shows a brief error dialog.

**Fix:**

1. Check the log file at `%LOCALAPPDATA%\EmbroideryCatalogue\logs\app.log` for the
   exact error message.
2. **WebView2 runtime missing.**  The desktop window requires the
   [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
   On Windows 10 (November 2020 Update or later) and Windows 11 this is pre-installed.
   On older machines, download and install it manually from the link above.
3. **Visual C++ redistributable missing.**  Install the
   [Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).
4. **Antivirus / SmartScreen block.**  Because the executable is compiled with
   PyInstaller and may not be code-signed, some security software may block it.
   Check your quarantine list and add an exception if needed.

### The app starts but shows a blank window

**Symptom:** The desktop window opens but the UI is blank or shows a network error.

**Fix:**

- The embedded browser may be slow to connect to the local server on first launch.
  Wait a few seconds and then refresh (`F5`).
- Check `%LOCALAPPDATA%\EmbroideryCatalogue\logs\app.log` for server startup errors.

### Port conflict in the desktop build

The installed build selects a free port dynamically so port conflicts should not occur.
If the log shows a port error:

1. Restart the application — a different free port will be selected automatically.
2. If the problem persists, check for security software that blocks local loopback
   connections and add an exception for `EmbroideryCatalogue.exe`.

### Uninstalling the desktop app

1. Open **Settings → Apps** (Windows 10/11) or **Control Panel → Programs and Features**.
2. Find **Embroidery Catalogue** and click **Uninstall**.
3. The uninstaller will ask whether to remove your catalogue data from
   `%LOCALAPPDATA%\EmbroideryCatalogue\`.  Click **No** to keep your designs for a
   future reinstall, or **Yes** to remove everything.

### Upgrading the desktop app

Run the newer installer over the existing installation.  Your catalogue data in
`%LOCALAPPDATA%\EmbroideryCatalogue\` is preserved automatically.
