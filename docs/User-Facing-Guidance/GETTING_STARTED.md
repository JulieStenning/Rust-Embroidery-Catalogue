# Getting Started — Embroidery Catalogue (Rust / Tauri build)

This guide explains how to run the Embroidery Catalogue directly from the repository
on a Windows PC.  It is intended for contributors and developers who want to build and
run the app from source.  If you are an end user, use the release installer or the
portable executable instead (see **Next steps** at the bottom of this guide).

---

## Requirements

| Requirement | Details |
|---|---|
| Operating system | **Windows 10 or Windows 11** |
| Rust toolchain | **Rust (stable)** — installed via <https://rustup.rs> |
| Tauri CLI | Installed via `cargo install tauri-cli --locked` (Step 3) |
| Node.js | **Node.js 18+** (with `npm`) for the Svelte frontend — installed via <https://nodejs.org/> |
| Internet access | Only for the first-time dependency install |

---

## Step 1 — Install Rust

1. Open a browser and go to <https://rustup.rs>.
2. Download and run **rustup-init.exe** for Windows (x86_64).
3. Choose the default installation profile when prompted.
4. Open a **new** Command Prompt and check the version:

   ```bat
   cargo --version
   ```

   The output should be `cargo 1.x.y` (any stable version).

---

## Step 2 — Install Node.js

1. Go to <https://nodejs.org/> and download the **LTS** installer for Windows.
2. Run the installer (keep the default options, including adding `npm` to PATH).
3. Open a **new** Command Prompt and check the version:

   ```bat
   node --version
   npm --version
   ```

   Both should print a version number.

---

## Step 3 — Install the Tauri CLI

From a Command Prompt, run:

```bat
cargo install tauri-cli --locked
```

Check it is available:

```bat
cargo tauri -V
```

> The `start-rust-app.bat` launcher also checks for the Tauri CLI and tells you to
> install it if it is missing.

---

## Step 4 — Get the repository

Clone (or download and unzip) the repository to a folder on your PC, for example
`C:\Projects\Rust-Embroidery-Catalogue`.

If you have Git installed:

```bat
git clone https://github.com/juliestenning/rust-embroidery-catalogue.git
cd rust-embroidery-catalogue
```

Otherwise, download the ZIP from GitHub (*Code → Download ZIP*), extract it, and
open a Command Prompt in the extracted folder.

---

## Step 5 — Install frontend dependencies

The Svelte frontend needs its npm packages installed before the app can build:

```bat
cd frontend
npm install
cd ..
```

This step requires an internet connection.  It may take a minute or two.
(`start-rust-app.bat` also runs `npm install` automatically on first launch if
`frontend\node_modules` is missing.)

---

## Step 6 — Configure settings (optional)

The application works without any configuration file.  You only need a Google API key if
you want to use optional AI-assisted auto-tagging.

The easiest way to add the key is via **Admin → Settings** in the app.  You can also
place it in a `.env` file in the repository root (where the launcher scripts live):

```
GOOGLE_API_KEY=AIzaSy_your_actual_key_here
```

Once the key is saved, open **Admin → Settings** and tick
**Run Tier 2 automatically during import** and/or
**Run Tier 3 automatically during import** to enable Gemini-based tagging during import.
See [AI_TAGGING.md](AI_TAGGING.md) for full details, including cost/quota information.

---

## Step 7 — Start the application

From the project root, double-click **`start-rust-app.bat`** or run it from the
Command Prompt:

```bat
start-rust-app.bat
```

`start-rust-app.bat` will:

1. Check that `cargo` and the Tauri CLI are available.
2. Install frontend dependencies (`npm install` in `frontend\`) if they are missing.
3. Start the Vite dev server on port **5173** in a separate window.
4. Check that `Data\Database\EmbroideryCatalogue.db` exists at the project root and
   copy it into `target\debug\Data\Database\`.
5. Launch the app via `cargo tauri dev --no-watch` — a **desktop window** opens.

> The Rust app is a desktop application (Tauri/WebView2).  It does **not** open a web
> browser or use a localhost web address.

> **First run only:**  The first time you start the app, the developer database
> `Data\Database\EmbroideryCatalogue.db` is copied into the debug output tree and used
> as the catalogue database.  All your catalogue data is stored there.

Once you open a design's detail page, you can use **Open in Editor** to launch the file
with the normal Windows default app, or **Show in Explorer** to reveal it in File Explorer.

### Alternative launch scripts

| Script | Purpose |
|---|---|
| `start-rust-app.bat` | Recommended — builds and launches the app in dev mode (`cargo tauri dev`) |
| `start-rust-debug-exe.bat` | Builds (`cargo tauri build --debug --no-bundle`) and launches the debug EXE |
| `start-rust-app-no-build.bat` | Launches a prebuilt `target\debug\embroidery-catalogue.exe` without building |

---

## Step 8 — Stop the application

Close the app window.  For dev mode, you can also press **Ctrl+C** in the Command Prompt
window where `start-rust-app.bat` is running, or close that window.

---

## Step 9 — Import your first designs

1. Open the app and go to **Import**.
2. Add **one or more source folders** containing your embroidery files. You can type the paths,
   use **Browse…**, or add extra folder rows manually.
3. Review the scan results. For each folder, you can leave **Designer** and **Source** as inferred,
   choose existing values, create new ones during the import, or leave them blank.
4. Click **Continue**. If this is your first-ever import into an empty catalogue, the app will ask
   you to complete **first import actions** before importing. This includes hoops, tags, sources,
   and designers. On later imports, these review actions remain optional.
5. Confirm the import. The selected files are copied into the managed storage folder
   (`MachineEmbroideryDesigns`) and added to the catalogue database.

See [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md) for the full first import actions workflow.

---

## Where your data lives

| File or folder | Purpose |
|---|---|
| `Data\Database\EmbroideryCatalogue.db` (project root) | Developer database source — copied into the debug tree by the launcher scripts |
| `target\debug\Data\` | Data root in dev mode (database, designs, thumbnails, logs) |
| `target\debug\Data\MachineEmbroideryDesigns\` | Managed storage for imported embroidery files in dev mode |
| `%APPDATA%\EmbroideryCatalogue\` | Data root in installed mode (release installer) |

> **Back up the whole `Data\` folder regularly.** See [BACKUP_RESTORE.md](BACKUP_RESTORE.md)
> for instructions.

---

## Keeping the application up to date

After pulling new changes from the repository:

1. Re-run `start-rust-app.bat`.  The launcher re-syncs the developer database and
   rebuilds the app as needed.
2. If frontend dependencies have changed, reinstall them:

   ```bat
   cd frontend
   npm install
   cd ..
   ```

3. If Rust crates have been added to `Cargo.toml`, they are downloaded automatically
   when the launcher builds the app (an internet connection is required).

---

## Optional environment variables

You can change default settings by creating a `.env` file in the project root.
Any variables you set override the built-in defaults.

| Variable | Default | Description |
|---|---|---|
| `GOOGLE_API_KEY` | *(not set)* | Enables AI auto-tagging (see [AI_TAGGING.md](AI_TAGGING.md)) |

Imported design files are stored automatically under the managed `MachineEmbroideryDesigns`
folder in the active data root.

---

## Next steps

- [App Installer.md](App Installer.md) — run the release installer or use portable mode on a USB stick / SD card.
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) — back up and restore your catalogue database.
- [AI_TAGGING.md](AI_TAGGING.md) — enable optional AI-powered design tagging.
- [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md) — first-import and later-import precheck review flow.
- [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md) — fix common problems.
- [../COMMERCIAL.md](../COMMERCIAL.md) — paid Windows installer build for non-technical users.