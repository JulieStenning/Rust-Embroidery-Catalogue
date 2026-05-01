# Feature Inventory: Embroidery Catalogue App

> **Status:** Current — reflects the application as of April 2026.
> This document describes every major user-facing and admin feature available in the
> running app.  Keep it updated whenever a feature is added, changed, or removed.

---

## Overview

The Embroidery Catalogue is a local FastAPI + SQLite web application for cataloguing
machine embroidery design files.  It supports a broad range of pyembroidery-readable
embroidery formats (see [SUPPORTED_FORMATS.md](SUPPORTED_FORMATS.md)), with limited `.art` support.
It runs entirely on a Windows PC or from an SD card with no external services, internet
connection, or database server required.

---

## 0. Architecture & Data Model

The current application stores its catalogue data in a local SQLite database (`data/database/catalogue.db`)
and keeps imported embroidery files in the managed folder `data/MachineEmbroideryDesigns/`.
Schema changes are handled with SQLAlchemy + Alembic, and the live schema uses the
canonical **tags** naming throughout.

### Core tables

| Table | Purpose |
|-------|---------|
| `designers` | Designer reference data |
| `sources` | Source reference data (e.g. Purchased, Downloaded) |
| `hoops` | Hoop size rules used for auto-selection |
| `tags` | Tags attached to designs |
| `design_tags` | Many-to-many junction between designs and tags |
| `designs` | Main catalogue records, including file path, preview image, notes, rating, stitched flag, and tagging state |
| `projects` | Named project groups |
| `project_designs` | Many-to-many junction between projects and designs |
| `settings` | Small key/value store for app configuration |

This summary is the current source of truth for the app's live data model.

---

## 1. Design Browsing & Search

### 1.1 Basic Browse

- Paginated thumbnail grid of all designs (50 per page).
- Each card shows: preview image, filename, hoop size, designer, rating (star icon),
  and a stitched badge when applicable.
- Each card can be selected for bulk actions; the page also supports **Select all** and
  row-level selection checkboxes.
- When one or more designs are selected, a sticky action bar appears with options to
  choose tags, **Verify selected**, **Add to project**, or clear the selection.
- Default sort is by design name (A→Z). Users can switch to folder or date-added
  sorting and reverse the order.

### 1.2 Quick Filters (sidebar / filter bar)

| Filter | Values |
|--------|--------|
| Tag | Any saved tag (multi-select), plus a special `Untagged` option |
| Designer | Any designer in the database |
| Hoop size | Any saved hoop record (default seeded values include Hoop A, Hoop B, and Gigahoop) |
| Source | Any saved source in the database |
| Rating | 1–5 stars (current filter matches the selected rating) |
| Stitched status | All / Stitched / Not stitched |
| Review status | Optional **Unverified only** toggle |

Filters are applied via HTML GET parameters; selections persist across pagination within the
same filtered view.

### 1.3 Quick Search

The main search box on the browse page searches the currently enabled scopes —
**File name**, **Tags**, and **Folder name** — which all default to on.
Results are paginated identically to the browse view.

### 1.4 Advanced Search

Advanced search capabilities are built into the main browse page at `/designs/`.  There
is no separate advanced-search page.

**Google-like syntax supported:**

| Syntax | Behaviour |
|--------|-----------|
| `word` | All results containing the word |
| `"exact phrase"` | Exact phrase match (straight or curly quotes) |
| `word1 OR word2` | Either term present |
| `-word` | Exclude results containing the word |
| `*.hus` or `rose?` | Wildcard match using `*` and `?` |

**Field scope checkboxes:**

- **File names** — searches the `filename` column
- **Tags** — searches tag descriptions
- **Folder names** — searches the folder portion of `filepath`

All checkboxes default to checked (search everywhere).

Results are paginated consistently with the browse view.

---

## 2. Design Detail & Metadata

The design detail page (`/designs/{id}`) shows:

- Full-size preview image
- Filename and filepath
- Quick file actions:
  - **Open in Editor** — launches the file with the normal Windows default-app behaviour
  - **Show in Explorer** — opens File Explorer with the design file selected
- Dimensions (width × height in mm)
- Hoop size (auto-selected at import; can be changed)
- Designer and source metadata
- Tags (zero or more; editable)
- Rating (1–5 stars; clickable to update)
- Stitched toggle (click to mark/unmark as stitched)
- Notes (free-text; editable inline)
- Date added
- Project membership list

### 2.1 Inline Editing

- Rating: click a star to set or clear the personal rating.
- Stitched: click the toggle button to mark/unmark as stitched.
- Notes / Designer / Source / Hoop: edit these together in the metadata form and save.
- Tags: add or remove tags via the detail page form; saving marks the design as verified.
- Verification state: mark the design's tags/metadata as verified or switch it back to unverified for later review.

---

## 3. Tags (formerly Design Types)

Tags categorise designs.  The canonical endpoint is `/admin/tags/`.

### Tag Groups

Tags belong to one of two groups:

| Group | Purpose | Examples |
|-------|---------|---------|
| `stitching` | Stitch technique or thread usage | "Appliqué", "Satin stitch", "3D foam" |
| `image` | Subject matter or visual theme | "Children", "Seaside", "Floral" |

- New tags must be saved with a group: `image` or `stitching`.
- The admin tags page separates the library into **Image Tags** and **Stitching Tags** sections.
- To change a tag's group, choose the new value from the dropdown and click the `✓` button to save it.
- Deleting a tag removes it from all designs (cascade via junction table).

### Admin Access

Tags are managed directly via the canonical `/admin/tags/` routes and the shared
`src.services.tags` service.

---

## 4. Bulk Import

A guided import workflow at `/import/` lets the user scan **one or more source folders**,
review the files found, and adjust metadata before anything is copied into the catalogue.

### Step 1 — Folder Selection

The user can enter or paste one or more source folder paths, click **Browse…**, or add
extra folder rows manually. On Windows, the folder picker can select several folders in
one go, with manual add/remove still available as a fallback.

Sub-folders are scanned automatically. Each chosen source folder is treated as an import
root, and its name is preserved as the top-level folder inside the managed catalogue
storage.

### Step 2 — Scan & Review

The server scans the selected folders for pyembroidery-readable embroidery files, and can
also catalogue `.art` files with limited metadata and preview support.

The review screen groups results by source folder and shows:

- Filename
- Auto-detected dimensions (mm × mm via `pyembroidery`) where available
- Auto-selected hoop size where available
- Per-file OK/error status
- Per-folder designer and source controls before import

During review, the user can:

- select or deselect files to import
- keep the app's inferred **Designer** / **Source** values
- choose an existing Designer or Source
- create a new Designer or Source during the import
- leave Designer or Source blank deliberately
- optionally apply the same Designer / Source choice to all folders in a multi-folder import

> For `.art` files, dimensions and automatic hoop selection may be unavailable because
> the proprietary Wilcom format is not fully decodable via `pyembroidery`.

### Step 3 — Tag Review Decision and AI Tagging Banner

Before the import runs, the app pauses for a tag-check step:

- **First import (empty catalogue):** tag review is required before continuing.
- **Later imports:** the user can choose to review tags first, import immediately, or cancel.

The precheck screen also shows an AI tagging status banner:

- **No API key:** a blue notice explains that only Tier 1 keyword tagging will run, with
  links to Admin Settings and the AI Tagging Guide.
- **API key present:** an amber cost/quota notice is shown with the current Tier 2 / Tier 3
  settings and a link to change them before proceeding.

If review is chosen, the admin reference-data pages open in **import mode** and keep the
pending import context alive until the user continues or cancels. From **Manage Tags**,
the user can jump to **Hoops**, **Sources**, or **Designers** before returning to the
import flow.

### Step 4 — Confirm & Save

After the final confirmation, the selected files are copied into managed storage, design
records and preview images are created, and the user is redirected to the browse page.

Files already in the database are skipped to prevent duplicates.

---

## 5. Projects

Projects group designs for a sewing session.

### Project List (`/projects/`)

- Card/grid view of all projects with name, description, and created date.
- Create new project (name + optional description).

### Project Detail (`/projects/{id}`)

- Full list of member designs with thumbnails.
- Add designs from the design detail page or in bulk from the main browse page.
- Remove a design from the project.
- Edit project name and description.
- Delete project (designs are not deleted).

### Print Sheet (`/projects/{id}/print`)

Printer-friendly HTML sheet showing each design's preview image and key metadata
(size, hoop, designer, rating, stitched status, and notes where present).
Uses a CSS print stylesheet; no extra software needed.

---

## 6. Auto-Tagging Pipeline

Three-tier pipeline, applied automatically during bulk import and available standalone
via `auto_tag.py`.

### Tier 1 — Keyword Matching (always active)

Filename tokens are matched against a built-in keyword map in
`src/services/auto_tagging.py`.  No network or API key required.  Always runs during
import.

### Tier 2 — Gemini AI Text Suggestions (optional, settings-controlled)

Uses the Google Gemini API to suggest tags from a text description of the design.
Requires `GOOGLE_API_KEY` in `.env` (or saved via Admin → Settings).  Only runs during
import if **Run Tier 2 automatically during import** is ticked in Admin → Settings.

### Tier 3 — Vision-Based Suggestions (optional, settings-controlled)

Sends the generated preview image to the Gemini Vision API for tag suggestions.
Requires `GOOGLE_API_KEY` in `.env`.  Only runs during import if **Run Tier 3
automatically during import** is ticked in Admin → Settings.  Can also be run
standalone via `auto_tag.py --tier3-only`.

### Batch sizes and delay

An optional **AI tagging batch size** setting limits how many newly imported designs are
sent to Gemini per import run.  An optional **delay** setting controls the seconds
between Gemini API calls (default 5.0 s).  Both are configurable in Admin → Settings
and can be overridden per-action on the Tagging Actions page.

A separate optional **Import database commit batch size** setting controls how many
design records are written or tag-updated before each database commit during import.
Leave it blank to use the default (1000).

### In-app Tagging Actions

The **Admin → Tagging Actions** page exposes CLI-style tagging operations directly in the
browser UI, without requiring command-line access:

| Action | Scope |
|--------|-------|
| Tag only untagged designs | Designs with no tags; verified designs untouched |
| Tag untagged and unverified designs | Untagged + unverified; manually verified untouched |
| Re-tag ALL designs | All designs, overwriting verified tags (requires explicit confirmation) |

Each action allows choosing which tiers to run, an optional batch size override, and a
delay override.  Actions run synchronously and show a result summary after completion.

### Standalone Scripts

| Script | Purpose |
|--------|---------|
| `auto_tag.py` | Run Tier 1 / Tier 2 tagging and, optionally, Tier 3; supports flags such as `--skip-verified`, `--redo`, `--tier1-only`, `--tier3`, `--tier3-only`, `--limit`, and delay overrides |

---

## 7. Reference Data / Admin Pages

All admin pages require no authentication (single-user local app).

| URL | Purpose |
|-----|---------|
| `/admin/designers/` | List, create, rename, delete designers |
| `/admin/sources/` | List, create, rename, delete sources |
| `/admin/hoops/` | List, create, rename, delete hoops; seeded defaults shown |
| `/admin/tags/` | Browse tags by section, create new tags, change a tag's group with the dropdown + `✓`, and delete tags |
| `/admin/settings/` | Manage API key, AI tagging tier preferences, AI batch size, Import database commit batch size, delay, and storage information |
| `/admin/tagging-actions/` | Run in-app batch tagging actions (tag untagged, tag unverified, re-tag all) with tier/batch/delay controls |
| `/admin/maintenance/orphans` | Orphaned design finder and optional bulk cleanup |
| `/admin/maintenance/backup` | Configure backup destinations and run database/designs backups |

### Seeded Hoops

The hoop table is pre-populated with three sizes used for auto-selection at import:

| Name | Max Width | Max Height |
|------|-----------|------------|
| Hoop A | 126 mm | 126 mm |
| Hoop B | 200 mm | 140 mm |
| Gigahoop | 230 mm | 200 mm |

Designs are matched to the smallest hoop that fits their dimensions.

---

## 8. Information, Backup & Maintenance Tools

### First-use Disclaimer (`/disclaimer`)

Before normal navigation begins, the app shows the project disclaimer once and requires
an explicit acceptance. After accepting, the user is redirected back to the page they
were trying to open.

### Help & About (`/help`, `/about`, `/about/document/{slug}`)

- **Help** provides built-in guidance for searching, importing, projects, tagging, and
  common troubleshooting.
- **About** links to the shipped project documents such as Disclaimer, Privacy,
  Security, AI Tagging Guide, Third-Party Notices, and Licence, all viewable in-app.

### Backup (`/admin/maintenance/backup`)

The backup page lets the user set separate destination folders for database and designs backups.
It supports:

- **Backup database now** — creates a timestamped copy of the live SQLite database.
- **Run incremental backup** — mirrors the managed designs folder, copying only new or
  changed files and archiving removed files into `_deleted\YYYY-MM-DD\` inside the backup.
- **Run both backups** — performs both actions in one step.
- Native folder-picking shortcuts plus success/error summaries after each run.

### Orphaned Designs (`/admin/maintenance/orphans`)

Scans the database for design records whose files no longer exist on disk. Presents a
list for review and provides **Delete selected** and **Delete all** actions to remove
stale records. A quick scan is also available from the top navigation bar.

### Settings (`/admin/settings/`)

The settings page provides controls for:

- **Google Gemini API key** — paste the key here to enable AI-assisted tagging.
- **AI tagging during import** — tick to run Tier 2 (text AI) and/or Tier 3 (vision AI)
  automatically during each import. A cost/quota notice is shown when a key is present.
- **AI tagging batch size** — optionally limit how many designs are sent to Gemini per run.
- **Import database commit batch size** — controls how many designs are written or
  tag-updated before each database commit during import. Leave blank to use the default (1000).
- **AI delay** — adjust the seconds between Gemini API calls.
- **Managed storage location** — shows the path where imported files are stored (not editable).

---

## 9. Portable Deployment / Launcher Flow

The app is designed to run portably from any writable target location (SD card, USB drive,
external drive, or folder path) with no preinstalled Python or Docker on the target machine.

### 9.1 Source-machine prerequisites

Before preparing the portable target, the source machine should have the following in the
project root:

| Item | Why it is needed |
|------|------------------|
| `prepare_portable_target.bat` | Performs the actual copy to the deployment target location |
| `EmbroiderySdLauncher.exe` or `portable_launcher.py` | Lets the operator choose the target location and designs source visually |
| `python\` | Extracted Windows Embeddable Python 3.12 used by the portable build |
| `wheels\` | Offline Python packages used by `setup.bat` on first launch |
| `src\`, `templates\`, `static\`, `alembic\` | Core application code and UI assets |
| `alembic.ini`, `requirements.txt`, `setup.bat`, `start.bat`, `stop.bat`, `get-pip.py` | Runtime and setup files copied to the portable app |
| Optional `.env` with `GOOGLE_API_KEY` | Allows the copied portable app to keep AI-assisted tagging enabled |
| Accessible deployment target location (for example `F:\`) | The destination drive or path |
| Accessible designs source folder | Usually `J:\MachineEmbroideryDesigns`, but any valid folder or subfolder can be chosen |

> **Important:** the bundled `python\python312._pth` must have `import site`
> enabled so `setup.bat` can bootstrap `pip` and create the portable virtualenv.

### 9.2 Operator workflow with `EmbroiderySdLauncher`

`EmbroiderySdLauncher.exe` is the normal operator entry point. It is used to set the
parameters that will be passed to `prepare_portable_target.bat`.

1. Place `EmbroiderySdLauncher.exe` in the same folder as `prepare_portable_target.bat`.
2. Double-click the launcher (or run `python portable_launcher.py` if using the script directly).
3. In the **Deployment target location** field, choose the destination drive root such as `F:\`.
4. In the **Designs source folder** field, choose the folder to copy from — usually `J:\MachineEmbroideryDesigns`, but a subfolder also works.
5. Optionally tick **Skip MachineEmbroideryDesigns copy** if only the app needs updating.
6. Click **OK — Run**.
7. The launcher validates that the target exists and is writable, warns if it does not look like a removable drive, and then runs:

```bat
prepare_portable_target.bat <target_root> <designs_source> [--no-designs]
```

8. Batch output is streamed live into the launcher window.
9. On success, the last-used values are stored in the Windows Registry for next time:

```text
HKCU\Software\EmbroideryCatalogue\LastDeploymentRoot
HKCU\Software\EmbroideryCatalogue\LastDesignsSource
```

### 9.3 What `prepare_portable_target.bat` does

When the batch script runs, it:

1. Creates `\EmbroideryApp\app` and `\EmbroideryApp\python` under the chosen target root.
2. Copies the embeddable Python runtime into `EmbroideryApp\python`.
3. Copies `src`, `templates`, `static`, `alembic`, `wheels`, and the required root files into `EmbroideryApp\app`.
4. Copies `.env.example` to `.env` on the target app (so no live API key is ever transferred to the portable copy).
5. Optionally mirrors the selected designs source folder into `<target>\MachineEmbroideryDesigns` unless `--no-designs` is specified.

### 9.4 Direct batch usage (without the launcher)

Power users can still run the batch file directly:

```bat
REM Default — F:\ root, J:\MachineEmbroideryDesigns source
prepare_portable_target.bat

REM Custom target root
prepare_portable_target.bat G:\

REM Custom target + designs source
prepare_portable_target.bat G:\ J:\MachineEmbroideryDesigns

REM Skip the large designs copy
prepare_portable_target.bat G:\ --no-designs
```

### 9.5 First launch on the target machine

After the files are copied, the target machine uses the app like this:

1. Open `EmbroideryApp\app\start.bat` on the target machine.
2. On first run, `start.bat` calls `setup.bat` automatically.
3. `setup.bat` bootstraps `pip` using the bundled Python, installs `virtualenv`, and installs all packages from `wheels\` with no internet connection required.
4. `start.bat` then runs `alembic upgrade head` to create or update `data\database\catalogue.db`.
5. The FastAPI app starts on `http://localhost:8002` in portable mode.
6. On later launches, the existing `venv\` is reused, so setup is skipped.
7. On first use in the browser, the app shows the disclaimer once and requires acceptance before normal navigation continues.

### 9.6 Portable deployment directory layout

```text
EmbroideryApp\
  python\                        ← Windows Embeddable Python 3.12
  app\
    src\
    templates\
    static\
    alembic\
    data\
      database\                  ← catalogue.db created or updated on first launch
      MachineEmbroideryDesigns\  ← all imported design files managed by the app
    wheels\                      ← pre-downloaded .whl files (win_amd64 / py3.12)
    venv\                        ← created by setup.bat on first launch
    .env                         ← optional GOOGLE_API_KEY and overrides
    alembic.ini
    requirements.txt
    setup.bat
    start.bat
    stop.bat
    auto_tag.py
```

---

## 10. API & Health Check

The FastAPI app also exposes:

| Endpoint | Purpose |
|----------|---------|
| `GET /health` | Returns `{"status": "ok"}` — used by start scripts to poll readiness |
| `GET /docs` | Automatic Swagger UI (FastAPI) |
| `GET /redoc` | ReDoc API documentation |

---

## 11. Configuration

Configuration is loaded from a `.env` file in the project root (or environment
variables).  All settings have self-relative defaults so the app works without any
`.env` file.

| Variable | Default | Purpose |
|----------|---------|---------|
| `DATABASE_URL` | `sqlite:///<app_root>/data/database/catalogue.db` | Database path |
| `DATABASE_URL_TEST` | `sqlite:///<app_root>/data/test_catalogue.db` | Test database path |
| `GOOGLE_API_KEY` | *(none)* | Enables Tier 2 and Tier 3 auto-tagging |

Managed design files are always stored at `<app_root>/data/MachineEmbroideryDesigns`.

---

## 12. Contributor Note

**Whenever you add, change, or remove a feature, please update this file** so it
continues to reflect what the app actually does.  If a planning document in
`docs/Plans/` drove the work, mark the relevant section or file as implemented.

For new features, consider adding a planning document to `docs/Plans/` before
implementing, so the intent is clear to reviewers.  After the feature ships, either
update the plan to reflect the final implementation or add an "Implementation Update"
section.

The goal is that a new contributor can open `docs/feature-inventory.md` and quickly
understand what the app currently includes, then look at the relevant planning
documents for fuller context on *why* each feature was designed the way it was.
