# Third-Party Notices

Embroidery Catalogue includes or depends on the following third-party components.
Each component is subject to its own licence terms, which are reproduced or summarised
below.  Where a full licence text is not included in this file, the canonical source is
linked for reference.

---

## Runtime Dependencies

### FastAPI
- **Purpose:** Web framework used to serve the application's HTTP API and HTML pages.
- **Licence:** MIT
- **Source:** <https://github.com/tiangolo/fastapi>

### Starlette
- **Purpose:** ASGI toolkit that FastAPI is built on; handles routing, middleware, and
  request/response primitives.
- **Licence:** BSD 3-Clause
- **Source:** <https://github.com/encode/starlette>

### Uvicorn
- **Purpose:** ASGI server used to run the application process.
- **Licence:** BSD 3-Clause
- **Source:** <https://github.com/encode/uvicorn>

### SQLAlchemy
- **Purpose:** ORM (Object-Relational Mapper) used to interact with the application's SQLite database.
- **Licence:** MIT
- **Source:** <https://github.com/sqlalchemy/sqlalchemy>

### SQLite
- **Purpose:** Embedded database engine used for local catalogue storage via Python's built-in `sqlite3` module.
- **Licence:** Public Domain
- **Source:** <https://www.sqlite.org/copyright.html>
- **Note:** SQLite is not installed as a separate Python package in this project; it is provided by the bundled or system CPython runtime.

### Alembic
- **Purpose:** Database migration tool used alongside SQLAlchemy to manage schema changes.
- **Licence:** MIT
- **Source:** <https://github.com/sqlalchemy/alembic>

### Jinja2
- **Purpose:** Server-side templating engine used to render HTML pages.
- **Licence:** BSD 3-Clause
- **Source:** <https://github.com/pallets/jinja>

### MarkupSafe
- **Purpose:** Dependency of Jinja2; provides safe HTML string escaping.
- **Licence:** BSD 3-Clause
- **Source:** <https://github.com/pallets/markupsafe>

### Pillow
- **Purpose:** Image processing library used to generate and manipulate design preview images.
- **Licence:** Historical Permission Notice and Disclaimer (HPND) — open-source, permissive.
- **Source:** <https://github.com/python-pillow/Pillow>
- **Note:** Pillow itself is HPND-licensed; it bundles several third-party libraries
  (zlib, libjpeg, libwebp, etc.) under their own permissive licences.  See
  <https://pillow.readthedocs.io/en/stable/about.html#license> for details.

### pyembroidery
- **Purpose:** Reads JEF, PES, and other embroidery file formats; extracts stitch data
  and dimensions.
- **Licence:** MIT
- **Source:** <https://github.com/EmbroidePy/pyembroidery>

### compoundfiles
- **Purpose:** Reads Microsoft Compound Document File Format (OLE/CFB) containers, used
  when parsing certain embroidery file types.
- **Licence:** MIT
- **Source:** <https://github.com/waveform80/compoundfiles>

### python-multipart
- **Purpose:** Parses multipart/form-data request bodies (file uploads).
- **Licence:** Apache 2.0
- **Source:** <https://github.com/Kludex/python-multipart>

### anyio / h11 / httptools / uvloop / websockets
- **Purpose:** Async I/O primitives and HTTP/WebSocket protocol implementations used
  transitively by Uvicorn and FastAPI.
- **Licences:** MIT (anyio, h11), MIT (httptools), MIT (uvloop), BSD (websockets).
- **Sources:** Available via PyPI; see individual package pages for details.

---

## Optional AI / Cloud Integration

The following packages are used **only** when Gemini-based AI tagging is enabled by
setting `GOOGLE_API_KEY` in the application environment.  They are **not** required for
normal operation and are **not** installed by default from `pyproject.toml`.

### google-genai
- **Purpose:** Google Generative AI Python SDK; used to send embroidery images to the
  Gemini API for automated tag suggestions.
- **Licence:** Apache 2.0
- **Source:** <https://github.com/googleapis/python-genai>
- **Terms of Service:** Use of this package requires acceptance of Google's
  [Generative AI Terms of Service](https://ai.google.dev/gemini-api/terms).

### google-auth and related Google authentication libraries
- **Purpose:** Authentication support used by `google-genai` and its dependency stack when
  Gemini-based AI features are enabled.
- **Licence:** Apache 2.0
- **Source:** <https://github.com/googleapis/google-auth-library-python>
- **Note:** Exact transitive Google auth packages may vary by installed version and platform.

---

## Development / Build-time Dependencies

These packages are used only for development, testing, and building the launcher
executable.  They are **not** distributed with the application.

| Package | Purpose | Licence |
|---------|---------|---------|
| pytest | Test runner | MIT |
| pytest-asyncio | Async test support for pytest | Apache 2.0 |
| httpx | HTTP client used in tests | BSD 3-Clause |
| PyInstaller | Packages the launcher script into a Windows exe | GPL 2.0 + bootloader exception |
| hatchling | Build backend for the Python package | MIT |

> **PyInstaller note:** The GPL-licensed PyInstaller toolchain is used only during the
> build process and is **not** distributed with the application binary.  The compiled
> `EmbroiderySdLauncher.exe` is subject to the PyInstaller bootloader exception, which
> permits distribution of executables produced by PyInstaller without imposing GPL
> requirements on the application code.  See
> <https://pyinstaller.org/en/stable/license.html> for the full details.

---

## Bundled Python Runtime

When the application is deployed to portable media via `prepare_portable_target.bat`, a
standalone Python 3.12 runtime is included in the `python/` folder.

- **Component:** CPython 3.12
- **Licence:** Python Software Foundation Licence Version 2 (PSF-2.0)
- **Source:** <https://www.python.org/>
- **Licence text:** The full PSF licence is included as `python/LICENSE.txt` inside the
  portable deployment package, and is also available at
  <https://docs.python.org/3/license.html>.

The bundled runtime may itself include third-party components (OpenSSL, zlib, expat,
etc.) under their own permissive licences.  See `python/LICENSE.txt` and
<https://docs.python.org/3/license.html#licenses-and-acknowledgements-for-incorporated-software>
for the full list.

---

## Wheels

Pre-built wheel files (`.whl`) may be included in the `wheels/` directory to support
offline installation on the portable deployment.  Each wheel file contains the compiled
package for the corresponding dependency listed above.  The licence for each wheel is the
same as the licence for the source package.

---

## Notes

- Licence identifiers use [SPDX](https://spdx.org/licenses/) short-form names where
  available.
- If you discover a licence inaccuracy or a missing attribution, please open a GitHub
  issue.
