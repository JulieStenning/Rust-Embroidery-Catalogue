# Getting Started — Embroidery Catalogue (Local Use)

This guide explains how to run the Embroidery Catalogue directly from the repository
on a Windows PC.  It is intended for first-time users with no prior knowledge of the
project.

---

## Requirements

| Requirement | Details |
|---|---|
| Operating system | **Windows 10 or Windows 11** |
| Python version | **Python 3.12** |
| Internet access | Only for the first-time dependency install |

---

## Step 1 — Install Python 3.12

1. Open a browser and go to <https://www.python.org/downloads/>.
2. Download and run the **Python 3.12** installer for Windows.
3. On the first installer screen, tick **"Add Python to PATH"** before clicking *Install Now*.
4. After installation, open a **Command Prompt** and check the version:

   ```bat
   py --version
   ```

   The output should start with `Python 3.12`.

---

## Step 2 — Get the repository

Clone (or download and unzip) the repository to a folder on your PC, for example
`C:\Projects\Embroidery-Catalogue`.

If you have Git installed:

```bat
git clone https://github.com/JulieStenning/Embroidery-Catalogue.git
cd Embroidery-Catalogue
```

Otherwise, download the ZIP from GitHub (*Code → Download ZIP*), extract it, and
open a Command Prompt in the extracted folder.

---

## Step 3 — Create a virtual environment

From the project root folder, run:

```bat
py -3.12 -m venv .venv
```

This creates an isolated Python environment in a folder called `.venv`.

---

## Step 4 — Activate the virtual environment

```bat
.venv\Scripts\activate
```

Your Command Prompt prompt should now start with `(.venv)`.

---

## Step 5 — Install the application and its dependencies

```bat
pip install -r requirements.txt
```

Or, if you also want the developer test tools:

```bat
pip install -e ".[dev]"
```

This step requires an internet connection.  It may take a minute or two.

---

## Step 6 — Configure settings (optional)

The application works without any configuration file.  You only need a Google API key if
you want to use optional AI-assisted auto-tagging.

The easiest way to add the key is via **Admin → Settings** in the app.  You can also
place it in a `.env` file in the project root alongside `start.bat`:

```
GOOGLE_API_KEY=AIzaSy_your_actual_key_here
```

Once the key is saved, open **Admin → Settings** and tick
**Run Tier 2 automatically during import** and/or
**Run Tier 3 automatically during import** to enable Gemini-based tagging during import.
See [AI_TAGGING.md](AI_TAGGING.md) for full details, including cost/quota information.

---

## Step 7 — Start the application

Double-click **`start.bat`** in the project root, or run it from the Command Prompt:

```bat
start.bat
```

`start.bat` will:

1. Detect your `.venv` virtual environment.
2. Create the managed `data\database\` and `data\MachineEmbroideryDesigns\` folders if they do not already exist.
3. Apply any pending database migrations automatically.
4. Open your browser at <http://localhost:8003>.
5. Start the web server in the background.

> **First run only:** The first time you start the app, a new SQLite database file is
> created at `data\database\catalogue_dev.db`. All your catalogue data is stored there.

Once you open a design's detail page, you can use **Open in Editor** to launch the file
with the normal Windows default app, or **Show in Explorer** to reveal it in File Explorer.

---

## Step 8 — Stop the application

Press **Ctrl+C** in the Command Prompt window where `start.bat` is running, or close
that window or run stop.bat.

---

## Step 9 — Import your first designs

1. Open the app and go to **Import**.
2. Add **one or more source folders** containing your embroidery files. You can type the paths,
   use **Browse…**, or add extra folder rows manually.
3. Review the scan results. For each folder, you can leave **Designer** and **Source** as inferred,
   choose existing values, create new ones during the import, or leave them blank.
4. Click **Continue**. If this is your first-ever import into an empty catalogue, the app will ask
   you to review your tags before importing. On later imports, tag review is optional.
5. Confirm the import. The selected files are copied into `data\MachineEmbroideryDesigns\` and
   added to the catalogue database.

---

## Where your data lives

| File or folder | Purpose |
|---|---|
| `data\database\catalogue_dev.db` | Your development catalogue database (all designs, tags, projects) |
| `data\database\catalogue.db` | Production/portable database (used on portable device) |
| `data\MachineEmbroideryDesigns\` | Managed storage for imported embroidery files |

> **Back up the whole `data\` folder regularly.** See [BACKUP_RESTORE.md](BACKUP_RESTORE.md)
> for instructions.

---

## Keeping the application up to date

After pulling new changes from the repository:

1. Re-run `start.bat`.  Database migrations are applied automatically on startup.
2. If new Python packages have been added to `requirements.txt`, reinstall them:

   ```bat
   .venv\Scripts\activate
   pip install -r requirements.txt
   ```

---

## Optional environment variables

You can change default settings by creating a `.env` file in the project root.
Any variables you set override the built-in defaults.

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:///…/data/database/catalogue_dev.db` | Database location |
| `APP_PORT` | `8003` | Port the web server listens on |
| `GOOGLE_API_KEY` | *(not set)* | Enables AI auto-tagging (see [AI_TAGGING.md](AI_TAGGING.md)) |

Imported design files are stored automatically under `data\MachineEmbroideryDesigns`.

---

## Next steps

- [USB_DEPLOYMENT.md](USB_DEPLOYMENT.md) — copy the app to a USB stick or SD card so it
  runs on any Windows PC without Python installed.
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) — back up and restore your catalogue database.
- [AI_TAGGING.md](AI_TAGGING.md) — enable optional AI-powered design tagging.
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — fix common problems.
- [CODE_SIGNING.md](CODE_SIGNING.md) — optional Windows signing notes for future releases.
- [COMMERCIAL.md](COMMERCIAL.md) — paid Windows installer build for non-technical users.
