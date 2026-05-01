# Embroidery Types Expansion Plan

## Goal
Expand the app’s import support from the current hardcoded set of `.jef`, `.pes`, `.hus`, and limited `.art` handling to the broader set of **pyembroidery-readable embroidery formats** that fit this catalogue app.

This plan is intended as a **handoff brief for a web-based coding agent**. It includes the target scope, the files likely to need changes, documentation updates, and the expected acceptance criteria.

---

## Scope

### Include these file types

#### Standard embroidery imports
- `.jef`
- `.pes`
- `.hus`
- `.dst`
- `.exp`
- `.vp3`
- `.u01`
- `.pec`
- `.xxx`
- `.tbf`
- `.10o`
- `.100`
- `.bro`
- `.dat`
- `.dsb`
- `.dsz`
- `.emd`
- `.exy`
- `.fxy`
- `.gt`
- `.inb`
- `.jpx`
- `.ksm`
- `.max`
- `.mit`
- `.new`
- `.pcd`
- `.pcm`
- `.pcq`
- `.pcs`
- `.phb`
- `.phc`
- `.sew`
- `.shv`
- `.stc`
- `.stx`
- `.tap`
- `.zhs`
- `.zxy`
- `.gcode`

#### Limited support
- `.art`

#### Included helper format
- `.pmv`

### Exclude these file types
- `.json`
- `.col`
- `.edr`
- `.inf`
- `.svg`
- `.csv`
- `.png`
- `.txt`

---

## Important context
- The current import allowlist lives in `src/services/bulk_import.py`.
- The current code already has **special-case limited handling for `.art`** because pyembroidery does not fully decode the Wilcom/Bernina object format used there.
- The generic import path already works for many formats by using:
  - `pyembroidery.read(filepath)`
  - `pattern.bounds()`
  - `select_hoop_for_dimensions()`
  - `_render_preview(pattern)`
- The goal is **not** to add export features. This is about **catalogue/import support** and making the documentation match.

---

## Current state to update
The current app messaging still says the app supports only:
- `.jef`
- `.pes`
- `.hus`
- limited `.art`

That wording appears in both code-adjacent docs and user-facing templates and must be updated as part of this work.

---

## Implementation plan

### 1. Expand the supported extension registry
Update `src/services/bulk_import.py` to:
- replace the narrow hardcoded `SUPPORTED_EXTENSIONS` set
- include the target formats listed above
- keep `.art` in the support list with its current limited behavior
- include `.pmv`
- leave the excluded helper/output formats out of the import allowlist

Also review `EXTENSION_PRIORITY` so duplicate files with the same stem are resolved sensibly across the wider format set.

Suggested approach:
- prefer richer/common home-machine formats first, such as `.jef`, `.pes`, `.vp3`, `.hus`, `.sew`
- prefer interchange/industrial formats like `.dst`, `.exp`, `.u01`, `.tbf`, `.xxx` after those
- keep `.art` near the bottom because it remains limited-support

### 2. Preserve and harden special handling for `.art`
Do **not** try to force `.art` into the generic pyembroidery path.

Keep the existing fallback approach based on:
- `_find_spider_image()`
- `_decode_art_icon()`
- existing limited metadata / preview behavior

The docs should continue to say that `.art` support is limited.

### 3. Verify the generic import path works cleanly for the new extensions
Review `_process_file()` and related helpers to ensure the app behaves safely when:
- a format reads but has weak metadata
- bounds are missing or zero
- thread colors are incomplete
- preview rendering fails
- a specific file is malformed or partially unsupported

The scan should continue on a **per-file basis** and capture errors without failing the whole import pass.

### 4. Update tests
Update or add tests covering:
- the expanded `SUPPORTED_EXTENSIONS`
- deduplication / extension priority behavior
- successful `ScannedDesign` creation for newly accepted formats
- graceful failure handling for unsupported or malformed files
- no regression to the existing `.art` branch

Likely files:
- `tests/test_services.py`
- `tests/test_bulk_import_extra.py`
- any related route tests if needed

### 5. Update documentation and in-app text
Update all documentation and UI text that currently mentions the old limited list of supported file types.

At minimum, review and update:
- `README.md`
- `docs/feature-inventory.md`
- `docs/USB_DEPLOYMENT.md`
- `docs/TROUBLESHOOTING.md`
- `templates/about.html`
- `templates/info/help.html`

Recommended wording approach:
- use a short summary on top-level pages like:  
  **“Supports a broad range of pyembroidery-readable embroidery formats, with limited `.art` support.”**
- link to or reference a canonical support list if one is added
- avoid leaving any pages that still say only `.jef`, `.pes`, and `.hus`

### 6. Optionally add a dedicated support-matrix document
If helpful, add a new doc such as `docs/SUPPORTED_FORMATS.md` with:
- included standard formats
- `.art` as limited support
- `.pmv` as the only helper import kept in scope
- excluded helper/output formats

This makes future maintenance easier and avoids repeating a long extension list everywhere.

---

## Relevant files to inspect
- `src/services/bulk_import.py`
- `tests/test_services.py`
- `tests/test_bulk_import_extra.py`
- `README.md`
- `docs/feature-inventory.md`
- `docs/USB_DEPLOYMENT.md`
- `docs/TROUBLESHOOTING.md`
- `templates/about.html`
- `templates/info/help.html`
- `pyproject.toml`
- `requirements.txt`

---

## Constraints
- Keep the work focused on **import/cataloguing support**, not export features.
- Do **not** add the excluded helper/output formats.
- `.art` must remain clearly marked as limited support.
- `.pmv` is the **only** helper-type format that should remain in scope.
- Any documentation that mentions supported file types must be updated as part of the same change.

---

## Acceptance criteria
The work is complete when:

1. The import support list includes the approved formats in this plan.
2. `.pmv` is included, but `.json`, `.col`, `.edr`, and `.inf` are not.
3. `.svg`, `.csv`, `.png`, and `.txt` are not added to the import flow.
4. `.art` still works under the current limited-support model.
5. Tests are updated to reflect the new format list.
6. All user-facing docs and in-app help/about pages that mention supported file types are updated.
7. No old wording remains that says the app only supports `.jef`, `.pes`, and `.hus`.

---

## Suggested verification steps
After implementation, verify by:

1. Running the relevant test suite.
2. Scanning a folder containing a sample mix of formats such as:
   - `.dst`
   - `.exp`
   - `.vp3`
   - `.pec`
   - `.sew`
   - `.pcs`
   - `.u01`
   - `.pmv`
   - `.art`
3. Confirming that:
   - the files are discovered by the import scan
   - dimensions/hoop suggestions appear when available
   - previews render when possible
   - failures are handled gracefully per file
4. Reviewing the updated docs/pages to confirm the new support wording is consistent.

---

## Handoff note
Please implement this as a **documentation-aligned feature expansion**:
- widen the supported import types in code
- preserve `.art` limitations
- include `.pmv`
- exclude the other helper/output formats
- update all related documentation in the same change set
