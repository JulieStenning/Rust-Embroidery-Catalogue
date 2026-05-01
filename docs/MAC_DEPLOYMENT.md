# Portable Deployment — macOS

This guide explains how to copy the Embroidery Catalogue to any macOS deployment
target (mounted volume, USB drive, SD card, or folder) so it runs on any Mac
without needing a dedicated Python environment.

> **v1 scope:**  This guide covers the **portable-first, unsigned internal build**.
> Notarization, signed distribution, and DMG/PKG installers are deferred to v2.
> See [CODE_SIGNING.md](CODE_SIGNING.md) for the signing roadmap.

---

## Supported macOS configurations

| Architecture | Status |
|---|---|
| Apple Silicon (arm64) | Supported |
| Intel (x86_64) | Supported |
| Universal (arm64 + x86_64) | Build-time option (see Phase 6) |

> macOS 13 Ventura or later is recommended.  macOS 12 Monterey should work.

---

## Overview

The portable app lives in an `EmbroideryApp/` folder on the target volume.  It
includes:

- Application source code, templates, and static files
- Pre-downloaded Python package wheels (installed offline on first launch)
- A `data/` folder that holds the SQLite database and all imported design files

On first launch, the cross-platform setup script (`scripts/portable_setup.py`)
creates a Python virtual environment and installs packages from the bundled
`wheels/` folder — **no internet connection is needed**.

---

## Prerequisites (developer / build machine)

| Item | Notes |
|---|---|
| Python 3.10 or later | System Python or pyenv; must include `tkinter` for the GUI launcher |
| `wheels/` folder | Download with the `pip download` step below (macOS-compatible wheels) |
| `scripts/` folder | Included in the repository |

### Download macOS wheels

```bash
# From the project root:
pip download -d wheels --platform macosx_12_0_arm64 --python-version 3.12 \
    --only-binary=:all: -r requirements.txt

# For Intel Macs (add both sets for a dual-arch deployment):
pip download -d wheels --platform macosx_12_0_x86_64 --python-version 3.12 \
    --only-binary=:all: -r requirements.txt
```

> Re-run with `--platform macosx_13_0_arm64` etc. if the above does not find
> all packages.  Some pure-Python packages publish a single `py3-none-any`
> wheel that works on both architectures.

---

## Method 1 — GUI launcher (recommended)

`EmbroideryPortableDeploy` is the easiest way to prepare a deployment target.

1. Build or obtain the macOS launcher binary (see *Building the launcher* below).
2. Open a Terminal and run:
   ```bash
   # One-time Gatekeeper bypass for an unsigned build:
   xattr -d com.apple.quarantine EmbroideryPortableDeploy
   ```
3. Double-click **`EmbroideryPortableDeploy`** (or run `./EmbroideryPortableDeploy`).
4. In *Deployment target location*, click **Browse…** and select the target folder or
   mounted volume (e.g. `/Volumes/MyUSBDrive`).
5. Click **OK — Run**.
6. The output panel shows progress.  On success, the chosen path is remembered.

> **"Not a removable drive" prompt:**  If the target is a mounted USB volume, the
> launcher automatically recognises `/Volumes/*` paths and skips the confirmation
> prompt.  For other absolute paths (e.g. a folder in your home directory), you will
> be asked to confirm before proceeding.

---

## Method 2 — Command line (power users)

Run the cross-platform deploy script directly from the project root:

```bash
# Deploy to a mounted USB volume (default designs path skipped with --no-designs):
python3 scripts/portable_deploy.py /Volumes/MyUSBDrive --no-designs

# Deploy with designs folder:
python3 scripts/portable_deploy.py /Volumes/MyUSBDrive /path/to/MachineEmbroideryDesigns

# Deploy to a local folder:
python3 scripts/portable_deploy.py ~/EmbroideryPortable --no-designs
```

Re-running is safe — `rsync` skips files that are already up to date.

---

## What gets copied

```
EmbroideryApp/
  app/
    src/                         ← Application source code
    templates/                   ← Jinja2 templates
    static/                      ← CSS, JavaScript, images
    alembic/                     ← Database migration files
    scripts/                     ← Cross-platform start/stop/setup scripts
    wheels/                      ← Offline Python package wheels
    data/                        ← Portable data folder (created on first run)
      database/                  ←   catalogue.db (created on first run)
      MachineEmbroideryDesigns/  ←   all imported design files
    alembic.ini
    requirements.txt
    .env                         ← Created from .env.example (no secrets)
```

> The bundled Windows `python/` folder and `.bat` files are not copied on macOS.

---

## Launching on another Mac

1. Connect the USB drive or navigate to the target folder.
2. Open a Terminal and navigate to the app folder:
   ```bash
   cd /Volumes/MyUSBDrive/EmbroideryApp/app
   ```
3. Run the start script:
   ```bash
   python3 scripts/portable_start.py
   ```

**First launch only:**  The setup script runs automatically, creating `venv/` and
installing packages from `wheels/`.  This takes about 30–60 seconds and requires
**no internet connection**.

4. Your browser opens at <http://localhost:8002>.

> **Port note:**  The portable app uses port **8002** by default.  Set `APP_PORT`
> in `.env` if you need a different port.

---

## Stopping the app

```bash
python3 scripts/portable_stop.py
```

Or press `Ctrl+C` in the Terminal where `portable_start.py` is running.

---

## First-run Gatekeeper bypass (unsigned build)

Because the launcher binary is unsigned, macOS Gatekeeper will block it by default.
Remove the quarantine flag before running:

```bash
xattr -d com.apple.quarantine EmbroideryPortableDeploy
```

You will still see a dialog saying "macOS cannot verify the developer".  Click
**Open** to proceed.  You only need to do this once per machine.

For the deployed `EmbroideryPortableDeploy.app` bundle, use:

```bash
xattr -rd com.apple.quarantine EmbroideryPortableDeploy.app
```

---

## Updating the portable copy

After making changes on your developer machine, re-run the deploy script or the
GUI launcher.  `rsync --delete` copies only changed files, so updates are fast.

> **Your portable data is preserved.**  The deploy script does not copy or replace
> the `data/` folder automatically.

---

## Building the launcher binary

Prerequisites: Python 3.10+, `tkinter` support, PyInstaller.

```bash
pip install ".[build]"         # or: pip install pyinstaller

# Native architecture build:
./build_portable_deployment_mac.sh

# Default and recommended in CI: build for the current runner architecture:
./build_portable_deployment_mac.sh

# Best-effort universal build (may fall back to native if dependencies are not fat binaries):
./build_portable_deployment_mac.sh --arch universal2

# Force a specific architecture:
./build_portable_deployment_mac.sh --arch arm64
./build_portable_deployment_mac.sh --arch x86_64
```

The output appears at `dist/EmbroideryPortableDeploy` (binary) or
`dist/EmbroideryPortableDeploy.app` (bundle).

> For the easiest repeatable workflow, use the native build produced by the current macOS GitHub runner. If you later need separate Intel and Apple Silicon artifacts, publish one build from each runner architecture rather than relying on a universal2 binary.

> **Note:** Do not commit the built binary to source control.  Distribute it as
> a release artifact (e.g. `release/EmbroideryPortableDeploy-macos-v1.0.0`).

---

## Troubleshooting

### "Python 3 not found on PATH"
Install Python from <https://www.python.org/downloads/macos/> or via Homebrew:
```bash
brew install python@3.12
```

### "wheels/ folder not found"
Run the `pip download` step from this guide to populate `wheels/` with
macOS-compatible wheel files.

### "rsync: command not found"
`rsync` is pre-installed on macOS.  If it is missing (unlikely), install Xcode
Command Line Tools:
```bash
xcode-select --install
```

### Permission denied on /Volumes/...
Check that the volume is not write-protected.  For NTFS volumes on macOS,
write support requires a third-party driver (e.g. Tuxera NTFS or Paragon NTFS).
Use an exFAT-formatted drive for cross-platform compatibility.

### "Operation not permitted" on first run
On macOS 12+, full-disk access restrictions may prevent reading from some
mounted volumes.  Go to **System Settings → Privacy & Security → Full Disk Access**
and add Terminal (or the launcher app) if needed.

---

## Related guides

- [USB_DEPLOYMENT.md](USB_DEPLOYMENT.md) — Windows portable deployment
- [GETTING_STARTED.md](GETTING_STARTED.md) — run the app locally from source
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) — back up the data/ folder
- [CODE_SIGNING.md](CODE_SIGNING.md) — signing and notarization roadmap (v2)
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — fix common problems
