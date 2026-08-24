# Troubleshooting

Solutions to the most common problems when running or deploying the Embroidery Catalogue (Rust/Tauri desktop app).

---

## Table of contents

- [The app won't start](#the-app-wont-start)
- [Vite dev server port conflict (developer mode)](#vite-dev-server-port-conflict-developer-mode)
- [Database errors on startup](#database-errors-on-startup)
- [Designs show no preview image](#designs-show-no-preview-image)
- [Desktop window is blank or crashes](#desktop-window-is-blank-or-crashes)
- [Portable mode not detected (data went to %APPDATA%)](#portable-mode-not-detected-data-went-to-appdata)
- [App is slow or unresponsive](#app-is-slow-or-unresponsive)
- [AI tagging is not working](#ai-tagging-is-not-working)
- [Still stuck?](#still-stuck)
- [Windows installer build (desktop app)](#windows-installer-build-desktop-app)

---

## The app won't start

**Symptom:** Double-clicking `start-rust-app.bat` shows a brief black window that
closes immediately, or a Command Prompt window stays open with an error message.

**Common causes and fixes:**

> **Where logs live:** once the app has started, the runtime log is written to
> `<data_root>\logs\app.yyyy-mm-dd.log`.  The data root depends on the execution mode:
>
> - **Developer / dev mode (`cargo tauri dev`):** `target\debug\Data\`
> - **Portable (SD card / USB):** the `Data\` folder next to the executable
> - **Installed:** `%APPDATA%\EmbroideryCatalogue\`

1. **Rust / Cargo not found.**  `start-rust-app.bat` requires the Rust toolchain.

   - Check Cargo is installed: `cargo --version`
   - If the command is not found, install Rust from <https://rustup.rs> and re-open
     your Command Prompt.

2. **Tauri CLI not found.**  The Tauri CLI is required for `cargo tauri dev`.

   - Check: `cargo tauri -V`
   - If missing, install it:
     ```bat
     cargo install tauri-cli --locked
     ```
   - Then open a new terminal and run `start-rust-app.bat` again.

3. **Frontend dependencies missing.**  On first launch `start-rust-app.bat` runs
   `npm install` inside `frontend\` automatically.  If that fails (no internet, or an
   npm error), run it manually:

   ```bat
   cd frontend
   npm install
   cd ..
   ```

4. **Development database missing.**  Both `start-rust-app.bat` and
   `start-rust-app-no-build.bat` require `Data\Database\EmbroideryCatalogue.db` at the
   repository root and copy it into `target\debug\Data\Database\` before launching.
   If the file is missing you will see `ERROR: Data\Database\EmbroideryCatalogue.db
   was not found at the project root.`

   - Restore it from a backup, or copy the seed database:
     ```bat
     copy /Y "src-tauri\resources\EmbroideryCatalogue.db" "Data\Database\EmbroideryCatalogue.db"
     ```

5. **Prebuilt EXE missing (when using `start-rust-app-no-build.bat`).**  This script
   does not build; it runs `target\debug\embroidery-catalogue.exe`.  If that file is
   missing, build it once:

   ```bat
   cargo tauri build --debug --no-bundle
   ```

   Alternatively use `start-rust-debug-exe.bat`, which builds automatically when the
   debug EXE is absent.

6. **Wrong working directory.**  Always run the launch scripts from the repository root
   (the folder that contains `start-rust-app.bat` itself).

---

## Vite dev server port conflict (developer mode)

**Symptom:** `start-rust-app.bat` hangs, or reports
`ERROR: Vite dev server did not start on port 5173 within 30 seconds.`

**Fix:**

- The developer mode serves the frontend from a Vite dev server on port **5173**.
  Another process may already be using that port.
- To find what is using the port on Windows:

  ```bat
  netstat -ano | findstr :5173
  ```

  Note the PID in the last column, then end the process in Task Manager.

- Check the **"Rust Frontend Dev Server"** window that `start-rust-app.bat` opens for
  the exact npm/Vite error message.

> This only applies to developer mode.  Release builds bundle the frontend inside the
> executable and do not use a dev server or any localhost port.

---

## Database errors on startup

**Symptom:** The app fails to start, or the log shows an error such as
`database migration failed`, `no such table`, or a failure to open
`Database\EmbroideryCatalogue.db`.

**Background:** The application ships a pre-migrated SQLite seed database
(`src-tauri/resources/EmbroideryCatalogue.db`).  SQLx migration files live in the
`migrations/` directory (timestamped `.up.sql` / `.down.sql` files tracked via the
`_sqlx_migrations` table).  The migration runner is currently **disabled at startup**
(`src/main.rs`) because the seed and developer databases are already pre-migrated.

**Fixes:**

1. **Database file is missing (release/portable).**  In release builds the app
   automatically copies the embedded seed database to
   `<data_root>\Database\EmbroideryCatalogue.db` on first run.  If that copy failed
   (e.g. read-only media), create the folder manually and copy the seed:

   ```bat
   mkdir "Data\Database"
   copy /Y "src-tauri\resources\EmbroideryCatalogue.db" "Data\Database\EmbroideryCatalogue.db"
   ```

   for portable mode, or restore the installed-mode file under
   `%APPDATA%\EmbroideryCatalogue\Database\`.

2. **Database is from a newer version of the app.**  If you copied a database from a
   newer version back to an older codebase, the schema may not match.  Pull the latest
   code and re-run the launch script, or restore the matching seed database.

3. **Corrupted database.**  Restore from a backup (see
   [BACKUP_RESTORE.md](User-Facing-Guidance/BACKUP_RESTORE.md)).  If no backup
   exists, you can start fresh by replacing the database with the seed file (see fix 1)
   — note this loses all catalogue data.

4. **Schema changes planned by a developer.**  When the schema needs to evolve, the
   SQLx migration files in `migrations/` are the source of truth, and the seed
   database in `src-tauri/resources/EmbroideryCatalogue.db` must be re-created with the
   new schema before release.  See `src/database/migrations.rs`.

---

## Designs show no preview image

**Symptom:** The design list or detail page shows a placeholder instead of a stitch
preview image.

**Common causes:**

- **Image was never generated.**  Preview images are rendered by the Rust PNG renderer
  from the embroidery file's stitch data when a design is imported.  If rendering
  failed (for example, because of an unsupported or malformed file format), the
  preview will be missing.  Re-import the design file using *Bulk Import* to
  regenerate it.
- **Preview cache was deleted.**  Previews are cached under
  `<data_root>\thumbnails\`.  If that folder is removed, previews are regenerated
  lazily the next time they are requested.
- **Design file has moved.**  If files in `<data_root>\MachineEmbroideryDesigns\` have
  been moved or removed outside the app, the catalogue may not be able to re-read them.
  Restore or re-import the missing files into the managed storage folder.

---

## Desktop window is blank or crashes

**Symptom:** The app icon appears, but the window is blank, white, or shows an error
dialog instead of the catalogue UI.

**Common causes and fixes:**

1. **WebView2 runtime missing.**  The Tauri desktop window requires the
   [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
   On Windows 10 (November 2020 Update or later) and Windows 11 this is pre-installed.
   On older machines, download and install it manually from the link above.

2. **Visual C++ redistributable missing.**  Install the
   [Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).

3. **Startup error in the log.**  Check the log file for the exact error:
   - Installed mode: `%APPDATA%\EmbroideryCatalogue\logs\app.log`
   - Portable mode: `<exe_dir>\Data\logs\app.log`
   - Developer mode: `target\debug\Data\logs\app.log`

4. **Antivirus / SmartScreen block.**  Because the executable is a newly built binary
   and may not be code-signed, some security software may block it.  Check your
   quarantine list and add an exception if needed.

---

## Portable mode not detected (data went to %APPDATA%)

**Symptom:** You copied the release executable and a `Data\` folder to a USB stick /
SD card, but the application behaves like an installed app (writes to
`%APPDATA%\EmbroideryCatalogue`, no data appears on the removable drive).

**Cause:** The app decides its execution mode at startup by looking for a `Data\`
folder **next to the executable** (`src/paths.rs`).  If no `Data\` folder is found,
it falls back to **Installed mode** and uses `%APPDATA%\EmbroideryCatalogue`.

**Fix:**

1. Make sure the folder structure on the removable drive looks like this:
   ```
   E:\EmbroideryCatalogue\
   ├── embroidery-catalogue.exe
   └── Data\
   ```
2. The `Data\` folder can be empty — the app creates the subfolders (`Database`,
   `MachineEmbroideryDesigns`, `thumbnails`, `logs`) on first run and seeds the
   database automatically.
3. If the app already ran in Installed mode, its data is in
   `%APPDATA%\EmbroideryCatalogue\` — move (or copy) that whole folder to the `Data\`
   folder on the stick.

> Drive letters don't matter (E:, F:, G:, etc.).  All paths are resolved relative to
> the executable at runtime.  See [App Installer.md](User-Facing-Guidance/App Installer.md)
> for the full portable setup.

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
  need to be generated.  Subsequent requests use the cached copy in
  `<data_root>\thumbnails\` and are faster.

---

## AI tagging is not working

**Symptom:** Tier 2 (text) or Tier 3 (vision) tagging does not run during import or a
backfill, or the app reports that no Gemini API key is configured.

**Fix:**

- Ensure the API key is saved in **Admin → Settings** (the preferred method — it writes
  the key to a `.env` file), or add it manually to a `.env` file in the project root:

  ```
  GOOGLE_API_KEY=your_actual_key_here
  ```

- In **Admin → Settings**, confirm that **Run Tier 2 automatically during import**
  and/or **Run Tier 3 automatically during import** are ticked.  Without these settings
  enabled, no Gemini calls are made during import even when a key is present.
  Tiers 2 and 3 are also only run when an API key is available.

- Check the key is valid by testing it in [Google AI Studio](https://aistudio.google.com/).

- **Rate limiting (`429 Too Many Requests`).**  Free-tier keys are rate-limited.  If
  you hit the limit, open the **Tagging Actions** maintenance page and either lower the
  **batch size** or increase the **delay** between requests, then run a smaller retry.

- If you are running a backfill from the **Tagging Actions** maintenance page
  (unified backfill, stitching backfill, or fingerprint backfill), confirm the run
  actually includes Tier 2/Tier 3 and that the API key was present when the run
  started.

See [AI_TAGGING.md](User-Facing-Guidance/AI_TAGGING.md) for full setup instructions.

---

## Still stuck?

Check the log file for the exact error message, then search for that message in the
planning documents in `docs/Plans/`.

The log files are:

| Mode | Log file |
|---|---|
| Developer (`cargo tauri dev`) | `target\debug\Data\logs\app.log` |
| Portable (SD card / USB) | `<exe_dir>\Data\logs\app.log` |
| Installed | `%APPDATA%\EmbroideryCatalogue\logs\app.log` |

If you believe you have found a bug, open an issue on GitHub with:

- The exact error message
- What you were doing when the error occurred
- Your Windows version and Rust version (`rustc --version`)

---

## Windows installer build (desktop app)

This section covers the release installer build.  For the free repository build, see
the sections above.

### The desktop app does not start

**Symptom:** Double-clicking the Start Menu shortcut or `EmbroideryCatalogue.exe` does
nothing, or shows a brief error dialog.

**Fix:**

1. Check the log file at `%APPDATA%\EmbroideryCatalogue\logs\app.log` for the exact
   error message.
2. **WebView2 runtime missing.**  The desktop window requires the
   [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
   On Windows 10 (November 2020 Update or later) and Windows 11 this is pre-installed.
   On older machines, download and install it manually from the link above.
3. **Visual C++ redistributable missing.**  Install the
   [Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).
4. **Antivirus / SmartScreen block.**  Because the executable is a newly built binary
   and may not be code-signed, some security software may block it.  Check your
   quarantine list and add an exception if needed.

### The app starts but shows a blank window

**Symptom:** The desktop window opens but the UI is blank or shows a network error.

**Fix:**

- Check `%APPDATA%\EmbroideryCatalogue\logs\app.log` for server startup errors.
- If the WebView2 runtime is outdated, update it via Windows Update or from the
  Microsoft WebView2 Runtime page (see above).

### Uninstalling the desktop app

1. Open **Settings → Apps** (Windows 10/11) or **Control Panel → Programs and Features**.
2. Find **Embroidery Catalogue** and click **Uninstall**.
3. The uninstaller will ask whether to remove your catalogue data from
   `%APPDATA%\EmbroideryCatalogue\`.  Click **No** to keep your designs for a
   future reinstall, or **Yes** to remove everything.

### Upgrading the desktop app

Run the newer installer over the existing installation.  Your catalogue data in
`%APPDATA%\EmbroideryCatalogue\` is preserved automatically.