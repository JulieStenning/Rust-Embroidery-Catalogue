# Per-folder source and designer assignment during import

## Status
Proposed

## Summary
The bulk import flow should support **multiple selected folders** and allow the user to set **`designer_id`** and **`source_id`** either:

1. **once for the whole import**, or
2. **individually per selected folder**.

The workflow should also allow a **new Designer** or **new Source** to be created during import if it does not already exist.

> **Important UX requirement:** the main folder picker should support **true multi-select** so the user can choose several folders in one browse action. Requiring the user to add folders one by one is not the preferred experience and should only exist as a fallback if the OS picker is unavailable.

This is intended to solve cases where the user knows metadata that the app cannot reliably infer from the files alone. Example:

- any folder containing `amazing_designs` should be treated as Designer **Amazing Designs**
- different selected folders may need different Designers or Sources in the same import session

---

## Problem statement
At the moment the import process can only assign `designer_id` and `source_id` if an **existing** designer or source name happens to appear in the path and the current path-matching logic finds it.

This has several limitations:

- many imports contain folders such as `amazing_designs`, `urban-threads`, or `etsy-bundle-1`, which do not map cleanly to existing database names
- the user may know the correct Designer or Source, but the app cannot ascertain it automatically
- the user currently has to **create the designer/source first** in admin before it can be used
- the import currently assumes a **single selected folder**, which makes per-folder assignment awkward for batch imports

---

## Goals

### Primary goals
- Support importing from **multiple folders in one workflow**
- Allow the user to set **Designer** and **Source**:
  - for **all selected folders**, or
  - **per folder**
- Allow **create-on-import** for missing Designers and Sources
- Keep the current **path inference** as a fallback, but do not rely on it exclusively
- Preserve the current single-folder experience as a valid simpler path

### Secondary goals
- Make the review screen clearer about what metadata will be applied
- Reduce post-import manual editing of designs
- Keep the implementation simple enough for a local FastAPI/Jinja app

---

## Non-goals for this phase
- No new rules-management admin screen yet
- No persistent folder-to-designer mapping table yet
- No major redesign of the underlying `Design`, `Designer`, or `Source` schema

Those can be added later if repeated imports show the need for saved reusable rules.

---

## Desired user workflow

### Step 1 — Select folders
The import start page should allow the user to:

- click **Browse…** and select **multiple folders in one action**
- review the chosen folder list before scanning
- remove any folder from the list before continuing
- optionally paste or edit folder paths manually as a fallback

Because this app is Windows-only, the preferred UX is a **true native multi-select folder picker**. A manual **Add another folder** fallback is acceptable only if the multi-select picker is unavailable on that system.

### Step 2 — Review grouped by folder
After scanning, the review page should group files by the folder they came from.

For each selected folder, show:

- the folder path
- a count of files found / selected / skipped
- a **Designer** section
- a **Source** section
- the files in that folder

Each folder section should offer these choices:

#### Designer
- **Keep inferred** — use the current path-based suggestion if possible
- **Choose existing** — select an existing Designer from a dropdown
- **Create new** — enter a new Designer name during import
- **Leave blank** — deliberately import without a designer

#### Source
- **Keep inferred**
- **Choose existing**
- **Create new**
- **Leave blank**

### Optional convenience control
At the top of the page, offer:

- **Apply same Designer/Source to all folders**

This should be optional. The user can then override specific folders if needed.

### Step 3 — Confirm import
When the user confirms the import, the selected files should be imported and the correct `designer_id` / `source_id` applied based on the chosen folder settings.

---

## Assignment precedence
The import should apply metadata in this order:

1. **Explicit per-folder choice**
2. **Global choice for all folders**
3. **Existing path inference**
4. **Blank / null**

This makes the workflow predictable and keeps the user in control.

---

## Create-on-import behavior
If the user chooses **Create new** for a folder:

- trim whitespace from the entered name
- compare case-insensitively with existing Designers / Sources
- if an equivalent item already exists, reuse it
- otherwise create it and use the new record immediately

Examples:

- `Amazing Designs`
- `amazing designs`
- `  Amazing Designs  `

should all resolve to the same entity.

---

## Matching improvements
The existing path inference should be retained, but improved slightly so these are treated more similarly:

- `Amazing Designs`
- `amazing_designs`
- `Amazing-Designs`

This is especially useful when the user leaves a folder on **Keep inferred**.

A light normalization pass is enough for now:

- lowercase the text
- treat `_`, `-`, and repeated spaces similarly
- prefer the longest matching designer/source name to avoid false positives

---

## Proposed implementation notes

### Files likely to change
- `src/routes/bulk_import.py`
- `src/services/bulk_import.py`
- `templates/import/step1_folder.html`
- `templates/import/step2_review.html`
- `src/services/designers.py`
- `src/services/sources.py`
- tests in `tests/test_services.py`, `tests/test_routes.py`, and `tests/test_bulk_import_extra.py`

### Route changes
#### `GET /import/browse-folder` (or a replacement browse endpoint)
Return **multiple selected folder paths** rather than a single path.

Implementation note for Windows:
- `tkinter.filedialog.askdirectory()` does **not** support true multi-select
- because this project is Windows-only, the preferred implementation is the native Windows folder picker via `IFileOpenDialog` with folder-picking + multi-select enabled
- return JSON shaped like `{ "paths": ["D:\\Embroidery\\Folder1", "D:\\Embroidery\\Folder2"] }`
- if the native multi-select picker is unavailable, fall back to a manual path-entry workflow rather than removing multi-folder support entirely

#### `POST /import/scan`
Accept a list such as `folder_paths[]` instead of only a single `folder_path`.

Expected behavior:
- trim blank values
- reject exact duplicates
- preserve backward compatibility for single-folder submissions
- scan all selected folders and combine the results

#### `POST /import/confirm`
Accept:
- selected files
- selected folders
- per-folder Designer choice
- per-folder Source choice
- optional global defaults

### Service changes
Extend the scanned import objects so the review page can group files safely by their selected root folder.

Useful extra fields on the scanned object:
- `source_folder`
- `folder_key`
- `folder_label`
- optional explicit override values gathered from the review form

During persistence, explicit overrides should be applied **before** fallback inference.

### UI changes
#### `step1_folder.html`
Use a multi-select browse flow that:
- opens one native picker dialog
- allows several folders to be chosen at once
- appends the selected folders into a visible review list
- allows individual folders to be removed before scanning
- optionally offers manual path entry / `Add another folder` only as a fallback

#### `step2_review.html`
Render grouped review cards or tables by folder, with:
- folder summary
- designer/source controls
- existing checkbox selection of files
- clear indication of what will be imported from each group

---

## Data model impact
No database schema change is required for the main per-folder workflow.

The existing `Design.designer_id` and `Design.source_id` fields are sufficient.

A future version may optionally add a separate table for saved path rules, but that is **out of scope for this phase**.

---

## Example scenario
The user selects these folders in one import:

1. `D:\Embroidery\amazing_designs`
2. `D:\Embroidery\urban_threads`
3. `D:\Embroidery\misc`

They then choose:

- for `amazing_designs`:
  - Designer = **Amazing Designs**
  - Source = **Purchased Downloads**
- for `urban_threads`:
  - Designer = **Urban Threads**
  - Source = **Urban Threads**
- for `misc`:
  - Designer = **Leave blank**
  - Source = **Keep inferred**

If **Amazing Designs** does not yet exist, it should be created during the import and assigned automatically.

---

## Acceptance criteria

### Functional
- The user can **multi-select multiple folders in a single browse action** before scanning
- The review screen shows files **grouped by selected folder**
- The user can assign **Designer** and **Source** per folder
- The user can choose one value for **all folders** and optionally override specific folders
- The user can **create a missing Designer or Source during import**
- Imported `Design` records receive the expected `designer_id` and `source_id`
- Existing single-folder imports still work

### UX
- The flow is understandable without external documentation
- The review page makes it obvious what metadata will be applied
- The user can still import quickly when they do not need per-folder overrides

### Technical
- No regression in existing import behavior
- Duplicate folder submissions are handled safely
- Duplicate designer/source creation is avoided

---

## Testing checklist

Add or update tests for the following:

1. **Multi-select browse** returns more than one folder path from a single browse action
2. **Multi-folder scan** returns grouped results without breaking single-folder scan
3. **Per-folder override** applies different `designer_id` / `source_id` values to different imported folders
4. **Global assignment** applies the same values to all folders when no per-folder override exists
5. **Create-on-import** creates a missing Designer / Source once and reuses it
6. **Inference fallback** still works when the user leaves a folder on `Keep inferred`
7. **Duplicate folder names** do not cross-assign metadata incorrectly
8. **Single-folder import** still behaves as before

---

## Future enhancement ideas
These are explicitly out of scope for now, but are good follow-ups:

- saved folder matching rules, e.g. `folder contains amazing_designs -> Designer: Amazing Designs`
- an admin UI to manage those rules
- a post-import bulk metadata correction screen
- reusable import presets

---

## Recommendation
Implement this in the following order:

1. multi-folder selection
2. grouped review page
3. per-folder Designer/Source controls
4. create-on-import
5. matching normalization improvements

That delivers the full workflow with the highest practical value while keeping the design straightforward for the current application architecture.
