# Commercial Distribution and Licensing

This project uses the **GNU Affero General Public License v3.0 or later** (`AGPL-3.0-or-later`) for the source code published in this repository.

## Plain-English Summary

- You may use, study, modify, and share the source code under the terms of the AGPL.
- If you distribute a modified version, or run a modified version for users over a network, you must also make the corresponding source code available under the AGPL.
- Third-party components remain under their own licences as listed in [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).

## Two delivery modes

| Mode | Audience | Delivery | Notes |
|---|---|---|---|
| **Free public version** | technical users / contributors | GitHub repository; portable USB/SD workflow | AGPL source code freely available |
| **Paid Windows installer** | general / non-technical users | polished Windows installer (`.exe`) | convenience build with installer, shortcuts, and uninstaller |

### Free public version (this repository)

The GitHub repository is and remains publicly available.  Anyone can clone it, run it
from source, or copy it to a USB stick for portable use — see
[USB_DEPLOYMENT.md](USB_DEPLOYMENT.md).

### Paid Windows installer

The paid build packages the same FastAPI/Jinja application as a normal Windows desktop
app using PyInstaller and an Inno Setup installer.  It adds:

- **No Python required** — Python is bundled inside the installer
- **Desktop window** — the UI opens in its own app window (via pywebview/WebView2) instead of the browser
- **No console window** — no visible command prompt during normal use
- **Dynamic port** — free localhost port selected at runtime; no conflicts with ports 8002/8003
- **Safe user data location** — catalogue database and imported files stored under `%LOCALAPPDATA%\EmbroideryCatalogue\` so data survives upgrades
- **Start Menu and Desktop shortcuts**
- **Uninstaller** — registered in Windows *Add or Remove Programs*

Charging for a packaged build or support does **not** replace or remove the AGPL rights
that apply to the source code in this repository.  The corresponding source code for
any distributed version is available at the canonical repository URL.

## Where user data is stored (installed build)

| Path | Contents |
|---|---|
| `%LOCALAPPDATA%\EmbroideryCatalogue\database\catalogue.db` | Catalogue database |
| `%LOCALAPPDATA%\EmbroideryCatalogue\MachineEmbroideryDesigns\` | Imported embroidery files |
| `%LOCALAPPDATA%\EmbroideryCatalogue\logs\app.log` | Application log |

This data is stored outside `Program Files` so it survives upgrades and is writable
without administrator rights.  The uninstaller offers to remove this data or leave it
in place for a future reinstall.

## Commercial Distribution

The maintainer may separately offer:

- packaged Windows installer builds
- portable USB / SD-card distributions
- paid support, installation help, or setup services
- documentation, training, or other commercial services

## No Extra Warranty

Unless explicitly agreed in writing, the software is provided **without warranty**, consistent with the licence and the repository disclaimer documents.

## Canonical Legal Text

For the binding legal terms, always refer to the root [`LICENSE`](../LICENSE) file.
